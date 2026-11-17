use crate::board::{Board, BoardSize, Point};
use crate::solver::{GameResult, Solver};
use crate::{Cell, CellState, CellType, GameStatus, Minsweeper, check_interact};
use std::collections::HashSet;


/// generates a random [MinsweeperGame] with no guarantees of any sort,
/// the mine positioning is 100% random
pub fn generate_game(board_size: BoardSize) -> MinsweeperGame {
    let mut board = Board::empty(board_size);

    let mine = Cell::new(CellType::Mine, CellState::Unknown);
    let mut points: Vec<Point> = board_size.points().collect();
    fastrand::shuffle(&mut points);
    for point in &points[..board_size.mines().get()] {
        board[*point] = mine;
    }

    generate_nmbers(&mut board);

    MinsweeperGame::new(GameStatus::Playing, board, usize::from(board_size.mines()).try_into().unwrap())
}

/// i refuse to fix the typo
fn generate_nmbers(board: &mut Board) {
    let empty_unknown = Cell::new(CellType::EMPTY, CellState::Unknown);
    for point in board.size().points() {
        let cell = &mut board[point];

        if matches!(cell.cell_type, CellType::Safe(_)) {
            *cell = empty_unknown;
        }
    }
    for point in board.size().points() {
        if board[point].cell_type == CellType::Mine {
            for point in board.size().neighbours(point) {
                if let CellType::Safe(number) = board[point].cell_type {
                    board[point] = Cell::new(CellType::Safe(number + 1), CellState::Unknown);
                }
            }
        }
    }
}

/// generate a random [MinsweeperGame] that is guaranteed to be solvable by the provided [Solver]
/// assuming the first move in the game is to reveal at the provided `point`
///
/// **this does *not* actualy hand you a game that has had the first move performed on it**
pub fn generate_solvable_game(board_size: BoardSize, solver: &dyn Solver, point: Point) -> MinsweeperGame {
    loop {
        let original_game = generate_game(board_size);

        let mut game = original_game.clone();

        Minsweeper::reveal(&mut game, point);

        let result = solver.solve_game(&mut game);

        if result == GameResult::Won {
            return original_game;
        }
    }
}

/// like [generate_solvable_game] but async
///
/// i don't like async
pub async fn generate_solvable_game_async<S: Solver + Send + Sync>(board_size: BoardSize, solver: &S, point: Point) -> MinsweeperGame {
    loop {
        let Some(state) = try_generate_solvable_game_async(board_size, solver, point).await else {
            #[cfg(feature = "tokio")]
            tokio::task::yield_now().await;
            continue
        };
        return state
    }
}
async fn try_generate_solvable_game_async<S: Solver + Send + Sync>(board_size: BoardSize, solver: &S, point: Point) -> Option<MinsweeperGame> {
    let original_game = generate_game(board_size);

    let mut game = original_game.clone();
    game.reveal(point);

    let result = solver.solve_game(&mut game);

    if result == GameResult::Won {
        Some(original_game)
    } else {
        None
    }
}


/// represents a game of minesweeper :3
///
/// this is the main implementation of [Minsweeper]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MinsweeperGame {
    pub status: GameStatus,
    pub board: Board,
    pub remaining_mines: isize
}

impl Minsweeper for MinsweeperGame {
    fn status(&self) -> GameStatus {
        self.status
    }

    fn board(&self) -> &Board {
        &self.board
    }

    fn remaining_mines(&self) -> isize {
        self.remaining_mines
    }

    fn reveal(&mut self, point: Point) {
        if check_interact(self, point).is_err() {
            return
        }


        let success = self.internal_reveal(point);

        if !success {
            self.status = GameStatus::Lost;

            return
        }

        if self.board.has_won() {
            self.status = GameStatus::Won;

            return
        }
    }

    fn clear_around(&mut self, point: Point) {
        if check_interact(self, point).is_err() {
            return
        }

        let Cell { cell_type: CellType::Safe(number), cell_state: CellState::Revealed } = self.board[point] else {
            return
        };

        let flags = self.count_flags(point);

        if flags != number as usize {
            return
        }

        let mut success = true;

        for point in self.board.size().neighbours(point) {
            success &= self.internal_reveal(point);
        }

        if !success {
            self.status = GameStatus::Lost;
            return
        }

        if self.board.has_won() {
            self.status = GameStatus::Won;

            return
        }
    }

    fn set_flagged(&mut self, point: Point, flagged: bool) {
        if check_interact(self, point).is_err() {
            return
        }

        let cell = &mut self.board[point];

        if cell.cell_state == CellState::Revealed {
            return
        }


        if flagged != (cell.cell_state == CellState::Flagged) {
            if flagged { self.remaining_mines -= 1 } else { self.remaining_mines += 1 }
        }

        cell.cell_state = if flagged { CellState::Flagged } else { CellState::Unknown };
    }
}

impl MinsweeperGame {
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

    fn internal_reveal(&mut self, point: Point) -> bool {
        // let state = state.as_mut();
        let board = &mut self.board;
        if board[point].cell_state != CellState::Unknown {
            return true
        }

        match board[point].cell_type {
            CellType::Safe(number) => {
                if number == 0 {
                    Self::reveal_empty(board, point)
                } else {
                    board[point] = Cell::new(CellType::Safe(number), CellState::Revealed)
                }
                true
            }
            CellType::Mine => {
                board[point] = Cell::new(CellType::Mine, CellState::Revealed);
                false
            }
            _ => unreachable!()
        }
    }

    fn reveal_empty(board: &mut Board, point: Point) {
        if !matches!(board[point], Cell { cell_type: CellType::EMPTY, cell_state: state } if state != CellState::Revealed) {
            return
        }

        let empty_cell = Cell::new(CellType::EMPTY, CellState::Revealed);
        board[point] = empty_cell;

        let mut flood = HashSet::new();

        flood.insert(point);

        while !flood.is_empty() {
            let point = *flood.iter().next().unwrap();
            flood.remove(&point);

            for point in board.size().neighbours(point) {
                if let Cell { cell_type: CellType::Safe(number), cell_state: state } = board[point]
                    && state != CellState::Revealed {
                    board[point] = Cell::new(CellType::Safe(number), CellState::Revealed);

                    if number == 0 {
                        flood.insert(point);
                    }
                }
            }
        }

    }

    fn count_flags(&self, point: Point) -> usize {
        self.board.size().neighbours(point)
            .filter(|e| self.board[*e].cell_state == CellState::Flagged)
            .count()
    }
}