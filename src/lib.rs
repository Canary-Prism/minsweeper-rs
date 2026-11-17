use crate::board::{Board, Point};
use std::fmt::{Debug, Display, Formatter};

pub mod board;
pub mod minsweeper;
pub mod solver;

pub trait Minsweeper {
    fn status(&self) -> GameStatus;
    fn board(&self) -> &Board;
    fn remaining_mines(&self) -> isize;

    fn reveal(&mut self, point: Point);

    fn clear_around(&mut self, point: Point);

    fn set_flagged(&mut self, point: Point, flagged: bool);

    fn toggle_flag(&mut self, point: Point) {
        self.set_flagged(point, self.board()[point].cell_state != CellState::Flagged)
    }

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

    fn right_click(&mut self, point: Point) {
        self.toggle_flag(point)
    }

}

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
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CellType {
    Safe(u8), Mine, Unknown
}
impl CellType {
    pub const EMPTY: CellType = CellType::Safe(0);
}

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
