pub mod mia;
pub mod start;

use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::sync::Arc;
use crate::{GameState, GameStatus, Minsweeper};
use crate::board::Point;

pub trait Solver {

    fn solve(&self, game_state: &GameState) -> Option<Move>;

    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        let mut state = minsweeper.gamestate();

        while state.status == GameStatus::Playing {
            let Some(Move { actions, ..}) = self.solve(state) else { break };

            for action in actions {
                state = minsweeper.action(action).into()
            }
        }

        match state.status {
            GameStatus::Won => GameResult::Won,
            GameStatus::Lost => GameResult::Lost,
            GameStatus::Playing => GameResult::Resigned,
            _ => unreachable!()
        }
    }
}

impl<S: Solver + ?Sized> Solver for Box<S> {
    fn solve(&self, game_state: &GameState) -> Option<Move> {
        (**self).solve(game_state)
    }
    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        (**self).solve_game(minsweeper)
    }
}
impl<S: Solver + ?Sized> Solver for Arc<S> {
    fn solve(&self, game_state: &GameState) -> Option<Move> {
        (**self).solve(game_state)
    }
    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        (**self).solve_game(minsweeper)
    }
}
impl<S: Solver + ?Sized> Solver for Rc<S> {
    fn solve(&self, game_state: &GameState) -> Option<Move> {
        (**self).solve(game_state)
    }
    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        (**self).solve_game(minsweeper)
    }
}
impl Solver for &dyn Solver {
    fn solve(&self, game_state: &GameState) -> Option<Move> {
        (**self).solve(game_state)
    }
    fn solve_game(&self, minsweeper: &mut dyn Minsweeper) -> GameResult {
        (**self).solve_game(minsweeper)
    }
}

#[derive(Debug)]
pub struct Move {
    pub actions: HashSet<Action>,
    pub reason: Option<Reason>
}

impl Move {
    // pub const fn new(actions: HashSet<Action>, reason: Option<Reason<T>>) -> Self {
    //     Self { actions, reason }
    // }

    pub fn single(action: Action, reason: Option<Reason>) -> Self {
        Self {
            actions: HashSet::from([action]),
            reason
        }
    }

    pub const fn multi(actions: HashSet<Action>, reason: Option<Reason>) -> Self {
        Self { actions, reason }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Action {
    pub point: Point,
    pub operation: Operation,
}

impl Action {
    pub const fn new(point: Point, operation: Operation) -> Self {
        Self { point, operation }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Operation {
    Reveal,
    Chord,
    Flag
}

pub trait Actionable {
    fn action(&mut self, action: Action) -> Result<&GameState, &GameState>;
}

impl<T: Minsweeper + ?Sized> Actionable for T {
    fn action(&mut self, action: Action) -> Result<&GameState, &GameState> {
        match action.operation {
            Operation::Reveal => self.reveal(action.point),
            Operation::Chord => self.clear_around(action.point),
            Operation::Flag => self.toggle_flag(action.point)
        }
    }
}

#[derive(Debug)]
pub struct Reason {
    pub logic: Box<dyn Logic>,
    pub related: HashSet<Point>
}

impl Reason {
    pub fn new<T: Logic + 'static>(logic: T, related: HashSet<Point>) -> Self {
        Self { logic: Box::new(logic), related }
    }
}

pub trait Logic: Debug + Display {

}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GameResult {
    Won, Lost, Resigned
}