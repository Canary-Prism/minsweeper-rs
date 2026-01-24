use crate::board::{Board, BoardSize, Point};
use crate::solver::{GameResult, Solver};
use crate::{check_interact, Cell, CellState, CellType, GameState, GameStatus, Minsweeper};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

trait InternalMinsweeper {

    fn start(&mut self) -> &GameState;

    fn on_win(&self);
    fn on_lose(&self);

    fn player_gamestate(&self) -> &GameState;
    fn gamestate_mut(&mut self) -> impl DerefMut<Target = GameState>;

    fn reveal(&mut self, point: Point) -> Result<&GameState, &GameState> {
        if check_interact(self, point).is_err() {
            return Err(self.player_gamestate())
        }


        let success = self.internal_reveal(point);

        if !success {
            self.gamestate_mut().status = GameStatus::Lost;

            self.on_lose();

            return Ok(self.player_gamestate())
        }

        if self.gamestate_mut().board.has_won() {
            self.gamestate_mut().status = GameStatus::Won;

            self.on_win();

            return Ok(self.player_gamestate())
        }

        Ok(self.player_gamestate())

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

    fn internal_reveal(&mut self, point: Point) -> bool {
        let mut state = self.gamestate_mut();
        // let state = state.as_mut();
        let board = &mut state.board;
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

    fn clear_around(&mut self, point: Point) -> Result<&GameState, &GameState> {
        if check_interact(self, point).is_err() {
            return Err(self.player_gamestate())
        }

        let Cell { cell_type: CellType::Safe(number), cell_state: CellState::Revealed } = self.player_gamestate().board[point] else {
            return Err(self.player_gamestate())
        };

        let flags = self.count_flags(point);

        if flags != number as usize {
            return Err(self.player_gamestate())
        }

        let mut success = true;

        for point in self.player_gamestate().board.size().neighbours(point) {
            success &= self.internal_reveal(point);
        }

        if !success {
            self.gamestate_mut().status = GameStatus::Lost;

            self.on_lose();

            return Ok(self.player_gamestate())
        }

        if self.gamestate_mut().board.has_won() {
            self.gamestate_mut().status = GameStatus::Won;

            self.on_win();

            return Ok(self.player_gamestate())
        }

        Ok(self.player_gamestate())
    }

    fn set_flagged(&mut self, point: Point, flagged: bool) -> Result<&GameState, &GameState> {
        if check_interact(self, point).is_err() {
            return Err(self.player_gamestate())
        }

        let mut mewo = self.gamestate_mut();
        let state = mewo.deref_mut();
        let cell = &mut state.board[point];

        if cell.cell_state == CellState::Revealed {
            drop(mewo);
            return Err(self.player_gamestate())
        }


        if flagged != (cell.cell_state == CellState::Flagged) {
            if flagged { state.remaining_mines -= 1 } else { state.remaining_mines += 1 }
        }

        cell.cell_state = if flagged { CellState::Flagged } else { CellState::Unknown };

        drop(mewo);
        Ok(self.player_gamestate())
    }

    fn count_flags(&self, point: Point) -> usize {
        self.player_gamestate().board.size().neighbours(point)
                .filter(|e| self.player_gamestate().board[*e].cell_state == CellState::Flagged)
                .count()
    }
}

impl<T: InternalMinsweeper + ?Sized> Minsweeper for T {
    fn start(&mut self) -> &GameState {
        self.start()
    }

    fn gamestate(&self) -> &GameState {
        self.player_gamestate()
    }

    fn reveal(&mut self, point: Point) -> Result<&GameState, &GameState> {
        self.reveal(point)
    }

    fn clear_around(&mut self, point: Point) -> Result<&GameState, &GameState> {
        self.clear_around(point)
    }

    fn set_flagged(&mut self, point: Point, flagged: bool) -> Result<&GameState, &GameState> {
        self.set_flagged(point, flagged)
    }
}


pub fn generate_game(board_size: BoardSize) -> GameState {
    let mut board = Board::empty(board_size);

    let mine = Cell::new(CellType::Mine, CellState::Unknown);
    let mut mines = 0usize;
    while mines < board_size.mines().into() {
        let point = (fastrand::usize(0..board_size.width().into()),
                     fastrand::usize(0..board_size.height().into()));

        if matches!(board[point].cell_type, CellType::Safe(_)) {
            board[point] = mine;
            mines += 1;
        }
    };

    generate_nmbers(&mut board);

    GameState::new(GameStatus::Playing, board, usize::from(board_size.mines()).try_into().unwrap())
}

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

pub struct MinsweeperGame<
    S: Solver = Box<dyn Solver>,
    OnWin: Fn() = Box<dyn Fn()>,
    OnLose: Fn() = Box<dyn Fn()>,
> {
    board_size: BoardSize,
    game_state: GameState,
    player_game_state: GameState,
    on_win: OnWin,
    on_lose: OnLose,
    first: bool,
    solver: Option<S>
}

impl<S: Solver, OnWin: Fn(), OnLose: Fn()> MinsweeperGame<S, OnWin, OnLose> {

    pub fn new(board_size: BoardSize, on_win: OnWin, on_lose: OnLose) -> Self {
        Self {
            board_size,
            game_state: GameState::new(GameStatus::Never, Board::empty(board_size), 0),
            player_game_state: GameState::new(GameStatus::Never, Board::empty(board_size), 0),
            on_win,
            on_lose,
            first: true,
            solver: None
        }
    }

    fn internal_start(&mut self, solver: Option<S>) -> &GameState {
        *self.gamestate_mut() = GameState::new(GameStatus::Playing, Board::empty(self.board_size),
                                         usize::from(self.board_size.mines()).try_into().unwrap());

        self.first = true;
        self.solver = solver;

        self.player_gamestate()
    }

    pub fn start_with_solver(&mut self, solver: S) -> &GameState {
        self.internal_start(solver.into())
    }
}

impl<S: Solver, OnWin: Fn(), OnLose: Fn()> InternalMinsweeper for MinsweeperGame<S, OnWin, OnLose> {
    fn start(&mut self) -> &GameState {
        self.internal_start(None)
    }

    fn on_win(&self) {
        (self.on_win)()
    }

    fn on_lose(&self) {
        (self.on_lose)()
    }

    fn player_gamestate(&self) -> &GameState {
        if self.game_state.status == GameStatus::Playing {
            &self.player_game_state
        } else {
            &self.game_state
        }
    }

    fn gamestate_mut(&mut self) -> impl DerefMut<Target = GameState> {
        GameStateHandle {
            game_state: &mut self.game_state,
            obfuscated_game_state: &mut self.player_game_state
        }
    }

    fn reveal(&mut self, point: Point) -> Result<&GameState, &GameState> {
        if check_interact(self, point).is_err() {
            return Err(self.player_gamestate())
        }

        if self.first {
            self.first = false;

            if let Some(solver) = &self.solver {
                *self.gamestate_mut() = generate_solvable_game(self.board_size, solver, point);
            } else {
                *self.gamestate_mut() = generate_game(self.board_size);
            }
        }


        let success = self.internal_reveal(point);

        if !success {
            self.gamestate_mut().status = GameStatus::Lost;

            self.on_lose();

            return Ok(self.player_gamestate())
        }

        if self.gamestate_mut().board.has_won() {
            self.gamestate_mut().status = GameStatus::Won;

            self.on_win();

            return Ok(self.player_gamestate())
        }

        Ok(self.player_gamestate())
    }

    fn set_flagged(&mut self, point: Point, flagged: bool) -> Result<&GameState, &GameState> {
        if check_interact(self, point).is_err() || self.first {
            return Err(self.player_gamestate())
        }

        let mut mewo = self.gamestate_mut();
        let state = mewo.deref_mut();
        let cell = &mut state.board[point];

        if cell.cell_state == CellState::Revealed {
            drop(mewo);
            return Err(self.player_gamestate())
        }


        if flagged != (cell.cell_state == CellState::Flagged) {
            if flagged { state.remaining_mines -= 1 } else { state.remaining_mines += 1 }
        }

        cell.cell_state = if flagged { CellState::Flagged } else { CellState::Unknown };

        drop(mewo);
        Ok(self.player_gamestate())
    }
}

pub fn generate_solvable_game(board_size: BoardSize, solver: &dyn Solver, point: Point) -> GameState {
    loop {
        let state = generate_game(board_size);

        let mut game = SetMinsweeperGame::new(state.clone());
        Minsweeper::reveal(&mut game, point)
                .expect("should always be able to successfully reveal");

        let result = solver.solve_game(&mut game);

        if result == GameResult::Won {
            return state;
        }
    }
}

#[derive(Clone, Debug)]
pub struct SetMinsweeperGame {
    game_state: GameState,
    player_game_state: GameState
}

impl SetMinsweeperGame {
    pub fn new(game_state: GameState) -> Self {
        Self { player_game_state: game_state.hide_mines(), game_state }
    }
}

impl InternalMinsweeper for SetMinsweeperGame {
    fn start(&mut self) -> &GameState {
        unimplemented!()
    }

    fn on_win(&self) {

    }

    fn on_lose(&self) {

    }

    fn player_gamestate(&self) -> &GameState {
        &self.player_game_state
    }

    fn gamestate_mut(&mut self) -> impl DerefMut<Target = GameState> {
        GameStateHandle {
            game_state: &mut self.game_state,
            obfuscated_game_state: &mut self.player_game_state,
        }
    }
}

struct GameStateHandle<'a> {
    game_state: &'a mut GameState,
    obfuscated_game_state: &'a mut GameState
}

impl AsMut<GameState> for GameStateHandle<'_> {
    fn as_mut(&mut self) -> &mut GameState {
        self.game_state
    }
}

impl Deref for GameStateHandle<'_> {
    type Target = GameState;

    fn deref(&self) -> &Self::Target {
        self.game_state
    }
}

impl DerefMut for GameStateHandle<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.game_state
    }
}

impl Drop for GameStateHandle<'_> {
    fn drop(&mut self) {
        *self.obfuscated_game_state = self.game_state.hide_mines()
    }
}

