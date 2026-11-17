use crate::board::{Board, Point};
use std::fmt::{Debug, Display, Formatter};

pub mod board;
pub mod minsweeper;
pub mod solver;

/// Main trait for Minsweeper
///
/// all of the things you can do to a game of minesweeper idk what else you want me to say
///
/// the main implementation is [minsweeper::MinsweeperGame] so
pub trait Minsweeper {

    /// gets the status of this game
    ///
    /// the status is either [GameStatus::Playing], [GameStatus::Playing], or [GameStatus::Lost]
    fn status(&self) -> GameStatus;

    /// gets the board of this game
    fn board(&self) -> &Board;

    /// gets the remaining mines
    ///
    /// the remaining mines is calculated from
    /// the total amount of mines in the game - the amount of flags
    fn remaining_mines(&self) -> isize;

    /// reveals a cell on the board at the specified location
    ///
    /// this is essentially what left clicking on an [Unknown][CellState::Unknown] [Cell] conventionally does
    fn reveal(&mut self, point: Point);

    /// reveals all non [Flagged][CellState::Flagged] neighbours of the cell at the specified location
    /// assuming that location is a [Revealed][CellState::Revealed] [Cell] and its number
    /// matches the amount of flags
    ///
    /// this is conventionally known as Chording
    /// and is essentially what left clicking on a [Revealed][CellState::Revealed] [Cell] conventionally does
    fn clear_around(&mut self, point: Point);

    /// flags or unflags a cell
    ///
    /// internally this changes the [Cell]'s [CellState] to be either
    /// [CellState::Unknown] if `flagged` is false or [CellState::Flagged] if it's true
    ///
    /// does nothing if the cell's state is [CellState::Revealed]
    fn set_flagged(&mut self, point: Point, flagged: bool);

    /// toggles the flagging of a cell
    ///
    /// similar to [Self::set_flagged] but toggles between the two states instead
    ///
    /// conventionally this is what right clicking does to a non [Revealed][CellState::Revealed] cell
    fn toggle_flag(&mut self, point: Point) {
        self.set_flagged(point, self.board()[point].cell_state != CellState::Flagged)
    }

    /// convenience method for doing either [Self::reveal] or [Self::clear_around]
    /// depending on the cell clicked on
    fn left_click(&mut self, point: Point) {

        if check_interact(self, point).is_err() {
            return
        }

        let cell = self.board()[point];

        match cell {
            Cell { cell_type: CellType::Safe(_), cell_state: CellState::Revealed } => self.clear_around(point),
            Cell { cell_state: CellState::Unknown, .. } => self.reveal(point),
            _ => ()
        }
    }

    /// convenience method for literally the same thing as [Self::toggle_flag]
    fn right_click(&mut self, point: Point) {
        self.toggle_flag(point)
    }

}

/// check whether a given point is even like valid to interact with
fn check_interact(minsweeper: &(impl Minsweeper + ?Sized), point: Point) -> Result<(), ()> {
    let state = minsweeper;
    if state.status() == GameStatus::Playing
            && (0..state.board().size().width().into()).contains(&point.0)
            && (0..state.board().size().height().into()).contains(&point.1) {
        Ok(())
    } else {
        Err(())
    }
}

/// represents a single cell in a minesweeper game
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cell {
    pub cell_type: CellType,
    pub cell_state: CellState
}

impl Cell {

    /// an empty and unknown cell
    pub const EMPTY: Cell = Cell::new(CellType::EMPTY, CellState::Unknown);

    pub const fn new(cell_type: CellType, cell_state: CellState) -> Self {
        Self {
            cell_type,
            cell_state
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.cell_type, self.cell_state) {
            (CellType::Safe(0), _) => write!(f, " "),
            (CellType::Safe(number), _) => write!(f, "{number}"),
            (CellType::Mine, _) => write!(f, "*"),
            (CellType::Unknown, CellState::Revealed) => write!(f, "?"),
            (CellType::Unknown, CellState::Flagged) => write!(f, "!"),
            (CellType::Unknown, CellState::Unknown) => write!(f, "▩")
        }
    }
}

/// represents the type of cell
///
/// this is its actual data
/// [CellType::Unknown] is currently unused
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CellType {

    /// a safe cell is any cell that is not a [Mine][CellType::Mine]
    ///
    /// contains a `u8` that represents the amount of mines in its neighbour,
    /// ranges from 0 to 8 inclusive
    Safe(u8),

    /// a mine :3
    ///
    /// kaboom
    Mine,

    /// unused, should i get rid of it ,,?
    Unknown
}
impl CellType {
    pub const EMPTY: CellType = CellType::Safe(0);
}

/// the state of a cell
/// 
/// like whether it's unknown or revealed or flagged you get it smh
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CellState {
    Unknown, Revealed, Flagged
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GameStatus {
    Playing, Won, Lost
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::ConventionalSize;
    use crate::minsweeper::MinsweeperGame;
    use crate::solver::GameResult::Lost;
    use crate::solver::Solver;
    use crate::solver::mia::MiaSolver;
    use crate::solver::start::SafeStart;

    #[test]
    fn it_works() {
        let mewo = const {
            size_of::<MinsweeperGame>()
        };
        println!("{mewo}")
    }

    #[test]
    fn mia_solver_works_at_least() {
        minsweeper::generate_solvable_game(ConventionalSize::Expert.size(), &MiaSolver::default(), (0, 0));
    }

    #[test]
    fn solvers_should_still_generate_things_waaaaaaaa() {
        for i in 0..100 {
            minsweeper::generate_solvable_game(ConventionalSize::Expert.size(), &MiaSolver::default(), (0, 0));
            println!("{}", i);
        }
    }

    #[test]
    fn mia_solver_should_never_die() {

        for _ in 0..100 {
            let mut game = minsweeper::generate_solvable_game(ConventionalSize::Expert.size(), &SafeStart, (0, 0));

            game.reveal((0, 0));

            let result = MiaSolver::default().solve_game(&mut game);

            if result == Lost {
                panic!("mia solver shouldn't lose\n{}", game.board())
            }
        }
    }

    #[test]
    fn mewo() {
        println!("{:#x}", 16742399)
    }
}
