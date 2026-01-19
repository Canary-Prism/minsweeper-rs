use crate::board::Point;
use crate::solver::Operation::{Chord, Flag, Reveal};
use crate::solver::{Action, Logic, Move, Reason, Solver};
use crate::{CellState, CellType, GameState, GameStatus};
use linked_hash_set::LinkedHashSet;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Sub;

#[derive(Copy, Clone, Debug)]
pub struct MiaSolver;

impl MiaSolver {
    const BRUTE_FORCE_LIMIT: usize = 30;
}

impl Display for MiaSolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Solver for MiaSolver {

    fn solve(&self, state: &GameState) -> Option<Move> {

        let size = state.board.size();

        if state.status != GameStatus::Playing { return None };

        for point in size.points() {
            let CellType::Safe(number) = state.board[point].cell_type else { continue };

            let mut marked_mines = HashSet::new();
            let mut empty_spaces = HashSet::new();

            for point in size.neighbours(point) {
                match state.board[point].cell_state {
                    CellState::Flagged => {
                        marked_mines.insert(point);
                        empty_spaces.insert(point);
                    }
                    CellState::Unknown => {
                        empty_spaces.insert(point);
                    }
                    _ => {}
                }
            }

            if number as usize == marked_mines.len() && empty_spaces.len() > marked_mines.len() {
                return Some(Move::single(Action::new(point, Chord), Some(Reason::new(MiaLogic::Chord, marked_mines))))
            } else if number as usize == empty_spaces.len() {
                let clicks: HashSet<_> = size.neighbours(point)
                        .filter(|e| state.board[*e].cell_state == CellState::Unknown)
                        .map(|e| Action::new(e, Flag))
                        .collect();

                if !clicks.is_empty() {
                    return Some(Move::multi(clicks, Some(Reason::new(MiaLogic::FlagChord, empty_spaces))));
                }
            } else if (number as usize) < marked_mines.len() {
                let clicks: HashSet<_> = size.neighbours(point)
                        .filter(|e| state.board[*e].cell_state == CellState::Flagged)
                        .map(|e| Action::new(e, Flag))
                        .collect();

                return Some(Move::multi(clicks, Some(Reason::new(MiaLogic::FlagChord, empty_spaces))));
            }
        }

        // hehe logical deduction
        // i hope this isn't too hateful to implement in Rust

        #[derive(Clone, Debug, Eq, PartialEq)]
        struct Flag {
            number: u8,
            points: HashSet<Point>
        }

        impl Flag {
            pub const fn new(number: u8, points: HashSet<Point>) -> Self {
                Self { number, points }
            }

            pub fn contains(&self, other: &Self) -> bool {
                self.number >= other.number
                        && self.points.is_superset(&other.points)
            }
        }

        impl PartialOrd for Flag {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                if self == other {
                    return Some(Ordering::Equal)
                }

                if self.contains(other) {
                    return Some(Ordering::Greater)
                }

                if other.contains(self) {
                    return Some(Ordering::Less)
                }

                None
            }
        }

        impl Sub for &Flag {
            type Output = Flag;

            fn sub(self, rhs: Self) -> Self::Output {
                // if !(self >= rhs) {
                //     panic!("mewo");
                // }

                let mut points = self.points.clone();

                for point in &rhs.points {
                    points.remove(point);
                }

                Flag::new(self.number - rhs.number, points)
            }
        }

        impl Hash for Flag {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.number.hash(state);
                for point in &self.points {
                    point.hash(state)
                }
            }
        }

        let mut flags = LinkedHashSet::new();

        for point in size.points() {
            let CellType::Safe(mut required) = state.board[point].cell_type else {
                continue
            };

            for point in size.neighbours(point) {
                if state.board[point].cell_state == CellState::Flagged {
                    required = required.saturating_sub(1)
                }
            }

            if required == 0 {
                continue
            }

            let neighbours: HashSet<_> = size.neighbours(point)
                    .filter(|e| state.board[*e].cell_state == CellState::Unknown)
                    .collect();

            if neighbours.is_empty() {
                continue
            }

            flags.insert(Flag::new(required, neighbours));
        }

        let mut changed = true;
        while changed {
            changed = false;

            let mut to_add = HashSet::new();
            for flag in &flags {
                // entirely contained stuffs
                {
                    let contained_flags: Vec<_> = flags.iter()
                            .filter(|e| flag >= e)
                            .collect();

                    for contained in contained_flags {
                        let remaining = flag - contained;

                        if remaining.points.is_empty() {
                            continue
                        }

                        if remaining.number == 0 {
                            return Some(Move::multi(
                                remaining.points
                                        .into_iter()
                                        .map(|e| Action::new(e, Reveal))
                                        .collect(),
                                Some(Reason::new(MiaLogic::RegionDeductionReveal, contained.points.clone()))
                            ))
                        } else if remaining.number as usize == remaining.points.len() {
                            return Some(Move::multi(
                                remaining.points
                                        .into_iter()
                                        .map(|e| Action::new(e, Flag))
                                        .collect(),
                                Some(Reason::new(MiaLogic::RegionDeductionFlag, contained.points.clone()))
                            ))

                        }

                        to_add.insert(remaining);
                    }
                }

                // not entirely contained stuffs
                {
                    let touching_flags = flags.iter()
                            .filter(|e| e.points.iter()
                                    .any(|e| flag.points.contains(e)));

                    for touching in touching_flags {
                        let remaining = flag - touching;

                        if remaining.points.is_empty() {
                            continue
                        }

                        if remaining.number as usize == remaining.points.len() {
                            return Some(Move::multi(
                                remaining.points
                                        .into_iter()
                                        .map(|e| Action::new(e, Flag))
                                        .collect(),
                                Some(Reason::new(MiaLogic::RegionDeductionFlag, touching.points.clone()))
                            ))
                        }
                    }
                }
            }

            changed = to_add.into_iter()
                    .map(|e| flags.insert(e))
                    .reduce(|a, b| a || b)
                    .unwrap_or(false);
        }

        if state.remaining_mines == 0 {
            let clicks: HashSet<_> = size.points()
                    .filter(|e| state.board[*e].cell_state == CellState::Unknown)
                    .map(|e| Action::new(e, Reveal))
                    .collect();

            if !clicks.is_empty() {
                return Some(Move::multi(clicks, Some(Reason::new(MiaLogic::RegionDeductionFlag, HashSet::new()))))
            }
        }

        let mut empties = HashSet::new();
        let mut adjacents = HashSet::new();

        for point in size.points() {
            if state.board[point].cell_state == CellState::Unknown {
                for neighbour in size.neighbours(point) {
                    if matches!(state.board[neighbour].cell_type, CellType::Safe(number) if number > 0) {
                        empties.insert(point);
                        adjacents.insert(neighbour);
                    }
                }
            }
        }

        if empties.len() < Self::BRUTE_FORCE_LIMIT && !adjacents.is_empty() {
            let states: Vec<GameState> = brute_force(&adjacents.into_iter().collect(), 0, state)
                    .collect();

            if !states.is_empty() {
                let mut clicks = HashSet::new();

                for point in empties.iter().copied() {
                    if states.iter().all(|e| e.board[point].cell_state != CellState::Flagged) {
                        clicks.insert(Action::new(point, Reveal));
                    }
                    if states.iter().all(|e| e.board[point].cell_state == CellState::Flagged) {
                        clicks.insert(Action::new(point, Flag));
                    }
                }

                if !clicks.is_empty() {
                    return Some(Move::multi(clicks, Some(Reason::new(MiaLogic::BruteForce, empties))))
                }

                if states.iter().all(|e| e.remaining_mines == 0) {
                    for point in size.points() {
                        if state.board[point].cell_state == CellState::Unknown
                                && states.iter().all(|e| e.board[point].cell_state != CellState::Flagged) {
                            clicks.insert(Action::new(point, Reveal));
                        }
                    }
                }

                if !clicks.is_empty() {
                    return Some(Move::multi(clicks, Some(Reason::new(MiaLogic::BruteForceExhaustion, empties))))
                }

            }

        }

        None
    }
}

fn brute_force(points: &Vec<Point>, index: usize, state: &GameState) -> Box<dyn Iterator<Item = GameState>> {
    let size = state.board.size();
    let mut empties = vec![];
    let current = points[index];

    let mut flags = 0;

    let CellType::Safe(number) = state.board[current].cell_type else {
        unreachable!()
    };

    for point in size.neighbours(current) {
        match state.board[point].cell_state {
            CellState::Unknown => empties.push(point),
            CellState::Flagged => flags += 1,
            _ => {}
        }
    }

    let mines_to_flag = number - flags;

    if mines_to_flag as isize > state.remaining_mines || mines_to_flag as usize > empties.len() {
        return Box::new(std::iter::empty())
    }

    if mines_to_flag == 0 || empties.is_empty() {
        if (index + 1 == points.len()) {
            return Box::new(std::iter::once(state.clone()));
        }
        return brute_force(points, index + 1, state);
    };

    let mut stream: Vec<Box<dyn Iterator<Item = GameState>>> = vec![];

    for flag_combinations in get_flag_combinations(&empties, mines_to_flag) {
        let mut state_copy = state.clone();

        for point in &empties {
            if flag_combinations.contains(point) {
                simulate_right_click(&mut state_copy, *point)
            } else {
                simulate_reveal(&mut state_copy, *point)
            }
        }

        if index + 1 == points.len() {
            stream.push(Box::new(std::iter::once(state_copy)))
        } else {
            stream.push(Box::new(brute_force(points, index + 1, &state_copy)))
        }
    }

    Box::new(stream.into_iter()
            .flatten())
}

fn get_flag_combinations(empties: &Vec<Point>, mines_to_flag: u8) -> Vec<HashSet<Point>> {
    if empties.len() < mines_to_flag as usize {
        return Vec::new()
    }

    recursive_get_flag_combinations(HashSet::new(), empties, 0, mines_to_flag)
            .collect()
}

fn recursive_get_flag_combinations(selected: HashSet<Point>, empties: &Vec<Point>, start: usize, mines_to_flag: u8) -> Box<dyn Iterator<Item = HashSet<Point>>> {
    if mines_to_flag < 1 {
        return Box::new(std::iter::empty())
    }

    let mut stream: Vec<Box<dyn Iterator<Item = HashSet<Point>>>> = vec![];

    for i in start..empties.len() {
        let mut selected = selected.clone();
        selected.insert(empties[i]);
        if mines_to_flag == 1 {
            stream.push(Box::new(std::iter::once(selected)))
        } else {
            stream.push(recursive_get_flag_combinations(selected, empties, start + 1, mines_to_flag - 1));
        }
    }

    Box::new(stream.into_iter()
            .flatten())
}

fn simulate_right_click(state: &mut GameState, point: Point) {
    let cell = &mut state.board[point];
    match cell.cell_state {
        CellState::Unknown => {
            cell.cell_state = CellState::Flagged;
            state.remaining_mines -= 1;
        }
        CellState::Flagged => {
            cell.cell_state = CellState::Unknown;
            state.remaining_mines += 1;
        }
        CellState::Revealed => unreachable!()
    }
}

fn simulate_reveal(state: &mut GameState, point: Point) {
    // it is normally illegal to have a revealed cell still be unknown
    // but such are the circumstances we find ourselves in
    state.board[point].cell_state = CellState::Revealed;
}


#[derive(Copy, Clone, Debug)]
pub enum MiaLogic {
    Chord,
    FlagChord,
    RegionDeductionReveal,
    RegionDeductionFlag,
    ZeroMinesRemaining,
    BruteForce,
    BruteForceExhaustion,
}

impl Display for MiaLogic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MiaLogic::Chord => write!(f, "the amount of flags around the cell matches its number"),
            MiaLogic::FlagChord => write!(f, "the amount of flaggable cells around the cell matches its number"),
            MiaLogic::RegionDeductionReveal => write!(f, "the surrounding cells force the cells to be safe"),
            MiaLogic::RegionDeductionFlag => write!(f, "the surrounding cells force the cells to be a mine"),
            MiaLogic::ZeroMinesRemaining => write!(f, "0 mines remaining, all unknown cells must be safe"),
            MiaLogic::BruteForce => write!(f, "in every possible mine configuration the cells are safe/mines"),
            MiaLogic::BruteForceExhaustion => write!(f, "in every possible mine configuration every mine is determined, all unused cells must be safe")
        }
    }
}

impl Logic for MiaLogic {

}