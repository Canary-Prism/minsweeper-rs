use crate::minsweeper::MinsweeperGame;
use crate::solver::{GameResult, Logic, Move, Solver};
use crate::{CellType, GameStatus, Minsweeper};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub struct SafeStart;

impl Solver for SafeStart {

    fn solve(&self, _game_state: &dyn Minsweeper) -> Option<Move> {
        None
    }

    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        match minsweeper.status() {
            GameStatus::Playing | GameStatus::Won => GameResult::Won,
            GameStatus::Lost => GameResult::Lost,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ZeroStart;

impl Solver for ZeroStart {

    fn solve(&self, _game_state: &dyn Minsweeper) -> Option<Move> {
        None
    }

    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        match minsweeper.status() {
            GameStatus::Playing if minsweeper.board()
                    .iter()
                    .any(|e| matches!(e.cell_type, CellType::Safe(0))) => GameResult::Won,
            GameStatus::Playing => GameResult::Lost,
            GameStatus::Won => GameResult::Won,
            GameStatus::Lost => GameResult::Lost,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WinStart;

impl Solver for WinStart {

    fn solve(&self, _game_state: &dyn Minsweeper) -> Option<Move> {
        None
    }

    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        match minsweeper.status() {
            GameStatus::Won => GameResult::Won,
            GameStatus::Lost => GameResult::Lost,
            GameStatus::Playing => GameResult::Resigned
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum StartLogic {}

impl Display for StartLogic {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl Logic for StartLogic {

}