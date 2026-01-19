use std::fmt::{Display, Formatter};
use crate::{CellType, GameState, GameStatus, Minsweeper};
use crate::solver::{GameResult, Logic, Move, Solver};

#[derive(Copy, Clone, Debug)]
pub struct SafeStart;

impl Solver for SafeStart {

    fn solve(&self, _game_state: &GameState) -> Option<Move> {
        None
    }

    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        match minsweeper.gamestate().status {
            GameStatus::Playing | GameStatus::Won => GameResult::Won,
            GameStatus::Lost => GameResult::Lost,
            GameStatus::Never => GameResult::Resigned
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ZeroStart;

impl Solver for ZeroStart {

    fn solve(&self, _game_state: &GameState) -> Option<Move> {
        None
    }

    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        match minsweeper.gamestate().status {
            GameStatus::Playing if minsweeper.gamestate().board
                    .iter()
                    .any(|e| matches!(e.cell_type, CellType::Safe(0))) => GameResult::Won,
            GameStatus::Playing => GameResult::Lost,
            GameStatus::Won => GameResult::Won,
            GameStatus::Lost => GameResult::Lost,
            GameStatus::Never => GameResult::Resigned
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