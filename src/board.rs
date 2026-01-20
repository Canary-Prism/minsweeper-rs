use crate::{Cell, CellState, CellType};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::Flatten;
use std::num::NonZeroUsize;
use std::ops::{Index, IndexMut};
use std::vec::IntoIter;

#[derive(Clone, Debug)]
pub struct Board {
    grid: Vec<Vec<Cell>>,
    size: BoardSize
}

pub type Point = (usize, usize);

impl Board {

    // pub(crate) const fn zero(board_size: BoardSize) -> Self {
    //     Self {
    //         grid: vec![],
    //         size: board_size
    //     }
    // }

    pub fn new(board_size: BoardSize, cell: Cell) -> Self {
        Self {
            grid: vec![vec![cell; board_size.height().into()]; board_size.width().into()],
            size: board_size
        }
    }

    pub fn empty(board_size: BoardSize) -> Self {
        Self::new(board_size, Cell::EMPTY)
    }

    pub fn size(&self) -> BoardSize {
        self.size
    }

    pub(crate) fn has_won(&self) -> bool {
        !self.iter()
                .any(|cell| match cell {
                    Cell { cell_type: CellType::Mine, cell_state: CellState::Revealed } => true,
                    Cell { cell_type: CellType::Safe(_), cell_state: state } if *state != CellState::Revealed => true,
                    _ => false
                })
    }

    pub(crate) fn hide_mines(&self) -> Self {
        let mut board = self.clone();

        board.grid = board.grid.into_iter()
                .map(|e| e.into_iter()
                        .map(|mut cell| {
                            if cell.cell_state != CellState::Revealed {
                                cell.cell_type = CellType::Unknown
                            }
                            cell
                        })
                        .collect())
                .collect();

        board
    }

    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        self.into_iter()
    }
}

impl Index<Point> for Board {
    type Output = Cell;

    fn index(&self, index: Point) -> &Self::Output {
        &self.grid[index.0][index.1]
    }
}

impl IndexMut<Point> for Board {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        &mut self.grid[index.0][index.1]
    }
}

impl IntoIterator for Board {
    type Item = Cell;
    type IntoIter = Flatten<IntoIter<Vec<Cell>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.grid
                .into_iter()
                .flatten()
    }
}
impl<'a> IntoIterator for &'a Board {
    type Item = &'a Cell;
    type IntoIter = Flatten<std::slice::Iter<'a, Vec<Cell>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.grid
                .iter()
                .flatten()
    }
}

impl<'a> IntoIterator for &'a mut Board {
    type Item = &'a mut Cell;
    type IntoIter = Flatten<std::slice::IterMut<'a, Vec<Cell>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.grid
                .iter_mut()
                .flatten()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.size.height.into() {
            for x in 0..self.size.width.into() {
                write!(f, "{}", self[(x, y)])?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum BoardSizeError {
    InvalidSize {
        width: usize,
        height: usize
    },
    TooManyMines {
        mines: usize,
        max_mines: usize
    },
    TooFewMines
}

impl Display for BoardSizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoardSizeError::InvalidSize { width, height } =>
                write!(f, "board size cannot be {} by {}", width, height),
            BoardSizeError::TooManyMines { mines, max_mines } =>
                write!(f, "board cannot have {} mines (max: {})", mines, max_mines),
            BoardSizeError::TooFewMines =>
                write!(f, "board cannot have 0 mines")
        }
    }
}

impl Error for BoardSizeError {}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct BoardSize {
    width: NonZeroUsize,
    height: NonZeroUsize,
    mines: NonZeroUsize
}

impl BoardSize {
    pub fn new(width: usize, height: usize, mines: usize) -> Result<Self, BoardSizeError> {

        let w = NonZeroUsize::new(width)
                .ok_or(BoardSizeError::InvalidSize { width, height })?;
        let h = NonZeroUsize::new(height)
                .ok_or(BoardSizeError::InvalidSize { width, height })?;
        let m = NonZeroUsize::new(mines)
                .ok_or(BoardSizeError::TooFewMines)?;

        if mines >= width * height {
            return Err(BoardSizeError::TooManyMines {
                mines,
                max_mines: width * height
            })
        }

        Ok(Self {
            width: w,
            height: h,
            mines: m
        })
    }

    pub fn width(&self) -> NonZeroUsize {
        self.width
    }

    pub fn height(&self) -> NonZeroUsize {
        self.height
    }

    pub fn mines(&self) -> NonZeroUsize {
        self.mines
    }

    pub fn neighbours(&self, point: Point) -> impl Iterator<Item = Point> {
        let mut neighbours = vec![];

        for y in point.1.saturating_sub(1)..=usize::min(usize::from(self.height()) - 1, point.1.saturating_add(1)) {
            for x in point.0.saturating_sub(1)..=usize::min(usize::from(self.width()) - 1, point.0.saturating_add(1)) {
                if (x, y) != point {
                    neighbours.push((x, y))
                }
            }
        }

        neighbours.into_iter()
    }

    pub fn points(&self) -> impl Iterator<Item = Point> {
        (0..self.height.into())
                .flat_map(|y| (0..self.width.into())
                        .map(move |x| (x, y)))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ConventionalSize {
    Beginner,
    Intermediate,
    Expert
}

impl ConventionalSize {
    pub fn size(self) -> BoardSize {
        match self {
            ConventionalSize::Beginner => BoardSize::new(9, 9, 10).unwrap(),
            ConventionalSize::Intermediate => BoardSize::new(16, 16, 40).unwrap(),
            ConventionalSize::Expert => BoardSize::new(30, 16, 99).unwrap()
        }
    }
}