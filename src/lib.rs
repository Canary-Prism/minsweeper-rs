use crate::board::{Board, Point};
use std::fmt::{Debug, Display, Formatter};

pub mod board;
pub mod minsweeper;
pub mod solver;

pub trait Minsweeper {

    fn start(&mut self) -> &GameState;

    fn gamestate(&self) -> &GameState;

    fn reveal(&mut self, point: Point) -> Result<&GameState, &GameState>;

    fn clear_around(&mut self, point: Point) -> Result<&GameState, &GameState>;

    fn set_flagged(&mut self, point: Point, flagged: bool) -> Result<&GameState, &GameState>;

    fn toggle_flag(&mut self, point: Point) -> Result<&GameState, &GameState> {
        self.set_flagged(point, self.gamestate().board[point].cell_state != CellState::Flagged)
    }

    fn left_click(&mut self, point: Point) -> Result<&GameState, &GameState> {

        if check_interact(self, point).is_err() {
            return Err(self.gamestate())
        }

        let cell = self.gamestate().board[point];

        match cell {
            Cell { cell_type: CellType::Safe(_), cell_state: CellState::Revealed } => self.clear_around(point),
            Cell { cell_state: CellState::Unknown, .. } => self.reveal(point),
            _ => Err(self.gamestate())
        }
    }

    fn right_click(&mut self, point: Point) -> Result<&GameState, &GameState> {
        self.toggle_flag(point)
    }

}

fn check_interact(minsweeper: &(impl Minsweeper + ?Sized), point: Point) -> Result<(), ()> {
    let state = minsweeper.gamestate();
    if state.status == GameStatus::Playing
            && (0..state.board.size().width().into()).contains(&point.0)
            && (0..state.board.size().height().into()).contains(&point.1) {
        Ok(())
    } else {
        Err(())
    }
}

impl AsRef<GameState> for Result<&GameState, &GameState> {
    fn as_ref(&self) -> &GameState {
        match self {
            Ok(state) => state,
            Err(state) => state
        }
    }
}

impl<'a> From<Result<&'a GameState, &'a GameState>> for &'a GameState {
    fn from(value: Result<&'a GameState, &'a GameState>) -> Self {
        value.unwrap_or_else(|state| state)
    }
}

pub trait GameStateTrait: Clone + Debug {
    fn status(&self) -> GameStatus;
    fn board(&self) -> &Board;
    fn remaining_mines(&self) -> usize;
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub status: GameStatus,
    pub board: Board,
    pub remaining_mines: isize
}

impl GameState {
    pub const fn new(status: GameStatus, board: Board, remaining_mines: isize) -> Self {
        Self {
            status,
            board,
            remaining_mines
        }
    }

    fn hide_mines(&self) -> Self {

        Self::new(self.status, self.board.hide_mines(), self.remaining_mines)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Cell {
    pub cell_type: CellType,
    pub cell_state: CellState
}

impl Cell {
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CellType {
    Safe(u8), Mine, Unknown
}
impl CellType {
    pub const EMPTY: CellType = CellType::Safe(0);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CellState {
    Unknown, Revealed, Flagged
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GameStatus {
    Playing, Won, Lost, Never
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::board::ConventionalSize;
    use crate::minsweeper::MinsweeperGame;
    use crate::solver::mia::MiaSolver;
    use crate::solver::start::SafeStart;
    use crate::solver::GameResult::Lost;
    use crate::solver::Solver;

    #[test]
    fn it_works() {
        let mewo = const {
            size_of::<GameState>()
        };
        println!("{mewo}")
    }

    #[test]
    fn mia_solver_works_at_least() {
        println!("{:?}", ConventionalSize::Expert.size());
        let mut game = MinsweeperGame::new(ConventionalSize::Expert.size(), Box::new(|| {}), Box::new(|| {}));
        println!("starting");
        game.start_with_solver(MiaSolver);

        println!("revealing");
        game.reveal((0, 0))
                .expect("shouldn't fail i don't think???");
    }

    #[test]
    fn mia_solver_should_never_die() {
        let mut game = MinsweeperGame::new(ConventionalSize::Expert.size(), Box::new(|| {}), Box::new(|| {}));

        for _ in 0..100 {
            game.start_with_solver(SafeStart);

            game.reveal((0, 0))
                    .expect("first click shouldn't fail");

            let result = MiaSolver.solve_game(&mut game);

            if result == Lost {
                panic!("mia solver shouldn't lose\n{}", game.gamestate().board)
            }
        }
    }

    #[test]
    fn mewo() {
        println!("{:#x}", 16742399)
    }
}
