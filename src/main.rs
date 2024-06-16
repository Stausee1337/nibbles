use crossterm::event::{Event, read, poll, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode};
use crossterm::{cursor, queue, style};
use levels::{render_l1, render_l2, render_l3, render_l4, render_l5, render_l6, render_l7, render_l8, spawn_l1, spawn_l2, spawn_l3, spawn_centred};
use rand::Rng;
use rand::distributions::{Distribution, Standard};
use terminal::{Terminal, TermColor, Cell};
/*use termion::event::Event;
use termion::{
    self, raw::IntoRawMode, screen::IntoAlternateScreen, input::TermRead, event::Key, cursor, color, clear};*/
use std::io::{stdout, Write, Stdout};
use std::iter::zip;
use std::time::{Duration, Instant};
use std::fmt::Write as FmtWrite;

/*use mio::{Poll, Token, Interest, Events};
use mio::unix::SourceFd;*/

mod levels;
mod terminal;

enum GameEvent {
    Timeout,
    Up,
    Right,
    Down,
    Left,
    Action,
    Escape,
    Quit
}

impl GameEvent {
    fn map_to_direction_check_valid(&self, current_direction: Direction) -> Option<Direction> {
        let direction = match self {
            GameEvent::Up => Some(Direction::Up),
            GameEvent::Right => Some(Direction::Right),
            GameEvent::Down => Some(Direction::Down),
            GameEvent::Left => Some(Direction::Left),
            _ => None
        };

        direction.and_then(|d| Direction::get_valid_transition(current_direction, d))
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Up, Right, Down, Left
}

impl Direction {
    fn get_valid_transition(current: Direction, next: Direction) -> Option<Direction> {
        match (current, next) {
            (Direction::Up | Direction::Down, Direction::Left | Direction::Right) => Some(next),
            (Direction::Right | Direction::Left, Direction::Up | Direction::Down) => Some(next),
            _ => None
        }
    }

    fn as_integer(&self) -> i32 {
        match self {
            Direction::Up | Direction::Left => -1,
            Direction::Down | Direction::Right => 1
        }
    }
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => unreachable!()
        }
    }
}

struct GameState {
    snake_direction: Direction,
    snake_vector: Vec<SnakeCell>,
    number_pos: (usize, usize),
    current_number: u8,
    extending: u8,
    paused: bool,
    score: i32,
    lives: u8,
    level: u8,
    cached_board: Option<Board>,
    duration_since_last_update: Duration
}

#[derive(Clone, Copy, std::cmp::Eq, std::cmp::PartialEq)]
struct SnakeCell(i32, i32);

fn main() {
    let mut stdout = stdout();// .into_raw_mode().unwrap().into_alternate_screen().unwrap();
    enable_raw_mode().unwrap();

    queue!(stdout, cursor::Hide, EnterAlternateScreen).unwrap();

    stdout.flush().unwrap();

    let game_state = create_game_state();

    listen_for_events(game_state, &mut stdout); 

    queue!(stdout, cursor::Show, LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
    stdout.flush().unwrap();
}

fn spawn_number(board: &Board) -> (usize, usize) {
    loop {
        let mut thread_random = rand::thread_rng();
        
        let pos = (
            thread_random.gen_range(1..board.width - 1),
            thread_random.gen_range(1..board.height - 2)
        );

        if board.lookup(pos.0 as usize, pos.1 as usize) < 0 {
            return pos;
        }
    }
}

const UPDATE_TIME: Duration = Duration::from_millis(100);
fn listen_for_events(mut game_state: GameState, stdout: &mut Stdout) {
    let mut last_update = Instant::now();

    let mut term_state = TermState::default();
    queue!(stdout,
           cursor::MoveTo(0, 0),
           style::ResetColor).unwrap();
    stdout.flush().unwrap();

    'outer: loop {
        let duration_since_last_update = Instant::now() - last_update;
        let has_events = if duration_since_last_update < UPDATE_TIME {
            poll(UPDATE_TIME - duration_since_last_update).unwrap()
        } else { false };

        if has_events {
            let event = read();

            match event {
                Ok(event) => {
                    match translate_event(event) {
                        Some(GameEvent::Quit) => break 'outer,
                        Some(event) => handle_event(event, &mut game_state, &mut term_state, stdout),
                        None => ()
                    }
                }
                _ => ()
            }
        }

        if duration_since_last_update >= UPDATE_TIME {
            game_state.duration_since_last_update = duration_since_last_update;
            handle_event(GameEvent::Timeout, &mut game_state, &mut term_state, stdout);
            last_update = Instant::now();
        }
    }
}

fn translate_event(event: Event) -> Option<GameEvent> {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Up | KeyCode::Char('w') => Some(GameEvent::Up), 
            KeyCode::Right | KeyCode::Char('d') => Some(GameEvent::Right),
            KeyCode::Down | KeyCode::Char('s') => Some(GameEvent::Down),
            KeyCode::Left | KeyCode::Char('a') => Some(GameEvent::Left),
            KeyCode::Char(' ') => Some(GameEvent::Action),
            KeyCode::Char('q') => Some(GameEvent::Quit),
            KeyCode::Esc => Some(GameEvent::Escape),
            _ => None
        }
        _ => None
    }
}

static LEVELS: &'static [(fn(&Board) -> (Vec<SnakeCell>, Direction), fn(&mut Board))] = &[
    (spawn_l1, render_l1),
    (spawn_l2, render_l2),
    (spawn_l3, render_l3),
    (spawn_centred, render_l4),
    (spawn_centred, render_l5),
    (spawn_centred, render_l6),
    (spawn_centred, render_l7),
    (spawn_centred, render_l8)
];

fn handle_event(event: GameEvent, game_state: &mut GameState, term_state: &mut TermState, stdout: &mut Stdout) {
    // handle events
    match event {
        GameEvent::Up | GameEvent::Right | GameEvent::Down | GameEvent::Left => {
            if let Some(new_direction) = event.map_to_direction_check_valid(game_state.snake_direction) {
                game_state.snake_direction = new_direction;
            }
        }
        GameEvent::Action if game_over(game_state) => {
            *game_state = create_game_state();
        }
        GameEvent::Action if game_state.paused => {
            game_state.paused = false;
        }
        GameEvent::Escape if !game_state.paused => {
            game_state.paused = true;
        }
        GameEvent::Timeout => {
            let playfield = get_playfield();
            let Rect { top, left, right, bottom } = playfield;

            let width = (right - left) as usize + 1;
            let height = (bottom - top) as usize + 1;
            let mut board = match &game_state.cached_board {
                None => {
                    let mut board = Board::new(width, height);
                    LEVELS[game_state.level as usize - 1].1(&mut board);
                    game_state.cached_board = Some(board.clone());
                    board
                },
                Some(board) => board.clone()
            };


            check_unitialized_state(game_state, &board);

            if !game_over(game_state) && !game_state.paused {
                update(game_state, &board);
            }

            check_unitialized_state(game_state, &board);

            render_snake(game_state, &mut board);
            let (width, height) = crossterm::terminal::size().unwrap();
            let mut buffer = Terminal::new(width as usize, height as usize);
            draw_buffered(game_state, &board, &mut buffer);
            draw_terminal(buffer, term_state, stdout);
        }
        _ => ()
    };
}

fn cell_diff(a: Cell, b: Cell) -> bool {
    if a.data != b.data {
        return true;
    }
    if a.foreground != b.foreground {
        return true;
    }
    if a.background != b.background {
        return true;
    }
    false
}

struct TermState {
    cursor: (u16, u16),
    buffer: Option<Terminal>,
    foreground: TermColor,
    background: TermColor,
}

impl Default for TermState {
    fn default() -> Self {
        Self {
            cursor: (0, 0),
            buffer: None,
            foreground: TermColor::Default,
            background: TermColor::Default
        }
    }
}

fn draw_terminal(buffer: Terminal, term_state: &mut TermState, stdout: &mut Stdout) {
    // write!(stdout, "{}{}{}", cursor::Goto(1, 1), color::Fg(color::Reset), color::Bg(color::Reset)).unwrap();
    let TermState { 
        cursor: mut c_cpos,
        buffer: pbuffer,
        foreground: mut state_foreground,
        background: mut state_background
    } = term_state;

    // let mut state_foreground = TermColor::Default;
    // let mut state_background = TermColor::Default;

    let mut v_cpos = (0, 0);
    let mut v_flushed = false;

    for (y, row) in buffer.rows.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let change = if let Some(pbuffer) = pbuffer {
                cell_diff(*cell, *&pbuffer.rows[y][x])
            } else { true };
            if change {
                if !v_flushed {
                    if c_cpos.1 == v_cpos.1 && v_cpos.0 > c_cpos.0 { 
                        queue!(stdout, cursor::MoveRight(v_cpos.0 - c_cpos.0)).unwrap();
                    } else {
                        queue!(stdout, cursor::MoveTo(v_cpos.0, v_cpos.1)).unwrap();
                    }
                    v_flushed = true;
                }
                let new_fg = if cell.foreground != state_foreground {
                    state_foreground = cell.foreground;
                    match cell.foreground { 
                        TermColor::Default => Some(style::Color::Reset),
                        TermColor::Color(c) => Some(style::Color::AnsiValue(c))
                    }
                } else { None };
                let new_bg = if cell.background != state_background {
                    state_background = cell.background;
                    match cell.background { 
                        TermColor::Default => Some(style::Color::Reset),
                        TermColor::Color(c) => Some(style::Color::AnsiValue(c))
                    }
                } else { None };
                if new_bg == new_fg && new_bg == Some(style::Color::Reset) {
                    queue!(stdout, style::ResetColor).unwrap();
                }
                queue!(stdout, style::SetColors(style::Colors { background: new_bg, foreground: new_fg })).unwrap();
                write!(stdout, "{}", cell.data).unwrap();
                v_cpos.0 += 1;
                c_cpos = v_cpos;
            } else {
                v_cpos.0 += 1;
                v_flushed = false;
            }
        }
        // write!(result, "\r{}", cursor::Down(1)).unwrap();
        v_cpos.0 = 0;
        v_cpos.1 += 1;
        v_flushed = false;
    }

    stdout.flush().unwrap();
    term_state.buffer = Some(buffer);
    term_state.foreground = state_foreground;
    term_state.background = state_background;
    term_state.cursor = c_cpos;
}

fn check_unitialized_state(game_state: &mut GameState, board: &Board) {
    if game_state.number_pos.0 == 0 || game_state.number_pos.1 == 0 {
        game_state.number_pos = spawn_number(&board);
    }
    if game_state.snake_vector.len() == 0 {
        let (vec, dir) = LEVELS[game_state.level as usize - 1].0(board);
        game_state.snake_vector = vec;
        game_state.snake_direction = dir;
    }
}

fn render_snake(game_state: &mut GameState, board: &mut Board) {
    for &cell in game_state.snake_vector.iter() {
        board.set_pixel(cell.0 as usize, cell.1 as usize, 11);
    }
}

fn create_game_state() -> GameState {
    GameState {
        snake_direction: Direction::Right,
        snake_vector: Vec::new(),
        number_pos: (0, 0),
        current_number: 1,
        extending: 0,
        paused: true,
        score: 0,
        lives: 5,
        level: 1,
        cached_board: None,
        duration_since_last_update: Duration::from_millis(1)
    }
}

fn update(game_state: &mut GameState, board: &Board) {
    if game_state.extending != 0 {
        game_state.snake_vector.push(game_state.snake_vector.last().unwrap().clone());
        game_state.extending -= 1;
    }

    let mut head = game_state.snake_vector[0];

    // move the snake accroding to Direction
    match game_state.snake_direction {
        Direction::Left | Direction::Right => {
            head.0 += game_state.snake_direction.as_integer();
        }
        Direction::Up | Direction::Down => {
            head.1 += game_state.snake_direction.as_integer();
        }
    }

    if head.0 < 1 || head.1 < 1 || head.0 >= board.width as i32 || head.1 > board.height as i32 {
        did_make_mistake(game_state);
        return;
    }

    if board.lookup(head.0 as usize, head.1 as usize) >= 0 {
        did_make_mistake(game_state);
        return;
    }

    for idx in (1..game_state.snake_vector.len()).rev() {
        let cell = game_state.snake_vector[idx-1];
        game_state.snake_vector[idx] = cell;
        if cell == head {
            did_make_mistake(game_state);
            return;
        }
    }

    game_state.snake_vector[0] = head;
    let number = game_state.number_pos;

    if (head.0 as usize, (head.1 - head.1 % 2) as usize) == (number.0, number.1 - number.1 % 2)  {
        if game_state.current_number == 10 {
            game_state.level += 1;
            game_state.snake_vector = Vec::new();
            game_state.snake_direction = Direction::Right;
            game_state.current_number = 1;
            game_state.number_pos = (0, 0);
            game_state.score += game_state.current_number as i32 * 100;
            game_state.extending = 0;
            game_state.paused = true;
            game_state.cached_board = None;
            return;
        }
        game_state.score += game_state.current_number as i32 * 100;
        game_state.extending = game_state.current_number * 4;
        game_state.current_number += 1;
        game_state.number_pos = (0, 0);
    }
}

fn did_make_mistake(game_state: &mut GameState) {
    game_state.lives -= 1;
    if game_state.lives > 0  {
        game_state.score -= 1000;
        game_state.current_number = 1;
        game_state.snake_vector = Vec::new();
        game_state.number_pos = (0, 0);
        game_state.snake_direction = Direction::Right;
        game_state.extending = 0;
    }
    game_state.paused = true;
}

static TEXT: &str = 
r"
   ______                        ____                     
  / ____/___ _____ ___  ___     / __ \_   _____  _____    
 / / __/ __ `/ __ `__ \/ _ \   / / / / | / / _ \/ ___/    
/ /_/ / /_/ / / / / / /  __/  / /_/ /| |/ /  __/ /        
\____/\____/_/ /_/ /_/\___/   \____/ |___/\___/_/         
";

fn draw_buffered(game_state: &mut GameState, board: &Board, buffer: &mut Terminal) {
    if game_over(game_state) {
        buffer.clear();
        buffer.set_foreground(TermColor::Color(12)); // ???
        write!(
            buffer,
            "{}",
            TEXT
        ).unwrap();

        buffer.set_foreground(TermColor::Color(15)); // ???
        write!(
            buffer,
            "\nPress SPACE to start again",
        ).unwrap();

        return;
    }
    let (width, height) = buffer.size;

    buffer.set_foreground(TermColor::Color(15));

    let state_len = if !game_state.paused {
        write!(buffer, 
               "Lives: {lives}        Level: {level}", 
               lives = game_state.lives, level = game_state.level).unwrap();
        24
    } else {
        static PAUSED_TEXT: &str = "    Paused";
        write!(buffer, "{}", PAUSED_TEXT).unwrap();
        PAUSED_TEXT.len()
    };
    let score_str = game_state.score.to_string();
    let duration_str = format!("    {}", game_state.duration_since_last_update.as_millis());
    let duration_len = duration_str.len();
    write!(buffer, "{}", duration_str).unwrap();
    buffer.right(width - state_len - duration_len - score_str.len());
    write!(buffer, "{score_str}\n").unwrap();
    buffer.set_foreground(TermColor::Color(9));

    write!(buffer, "\u{2588}{}\u{2588}\n", "\u{2580}".repeat((width - 2) as usize)).unwrap();
    for _ in 3..height {
        write!(buffer, "\u{2588}").unwrap();
        buffer.right(width - 2);
        write!(buffer, "\u{2588}").unwrap();
        buffer.down(1);
        buffer.ret();
    }
    write!(buffer, "\u{2588}{}\u{2588}", "\u{2584}".repeat((width - 2) as usize)).unwrap();

    let playfield = get_playfield();

    buffer.goto(0, 1);
    for (y, row) in board.iter().enumerate() {
        let (odd, even) = row;
        let iterator: Box<dyn Iterator<Item = (i16, i16)>> = if let Some(even) = even {
            Box::new(zip(odd, even).map(|(co, ce)| (*co, *ce)))
        } else {
            Box::new(odd.iter().map(|co| (*co, -1)))
        };
        // format!("{}", cursor::Right(1))
        for (co, ce) in iterator {
            let mut fullchar = false;
            if co == ce {
                if co < 0 { // skip
                    buffer.right(1);
                    continue;
                }
                fullchar = true;
            }

            if co >= 0 && ce >= 0 {
                buffer.set_background(TermColor::Color(co as u8));
                buffer.set_foreground(TermColor::Color(ce as u8));
            } else if check_bottom_top(playfield.bottom as usize / 2, y) {
                buffer.set_background(TermColor::Color(9));
                buffer.set_foreground(TermColor::Color(std::cmp::max(co, ce) as u8));
            } else {
                buffer.set_foreground(TermColor::Color(std::cmp::max(co, ce) as u8));
            }
            write!(buffer, 
                   "{}", if fullchar { "\u{2588}" } else if ce >= 0 {"\u{2584}"} else {"\u{2580}"}).unwrap();
            buffer.set_background(TermColor::Default);
            buffer.set_foreground(TermColor::Default);
        }
        buffer.ret();
        buffer.down(1);
    }

    buffer.set_foreground(TermColor::Color(15));
    buffer.set_background(TermColor::Default);
    buffer.goto(game_state.number_pos.0, game_state.number_pos.1 / 2 + 1);
    write!(buffer, "{}", game_state.current_number).unwrap(); 

    if game_state.paused {
        static CONTINUE_MSG: &str = "Press SPACE to continue";
        static BOX_WIDTH: usize = CONTINUE_MSG.len() + 4;
        static BOX_HEIGHT: usize = 1 + 2;

        let box_x = width / 2 - BOX_WIDTH / 2;
        let box_y = height / 2 - BOX_HEIGHT / 2;

        buffer.set_foreground(TermColor::Color(15));
        buffer.set_background(TermColor::Color(1));
        buffer.goto(box_x, box_y);
        for content in [
                "\u{2580}".repeat(BOX_WIDTH as usize - 2), 
                format!(" {} ", CONTINUE_MSG),
                "\u{2584}".repeat(BOX_WIDTH as usize - 2)] {
            write!(buffer, "\u{2588}{}\u{2588}", content).unwrap();
            buffer.left(BOX_WIDTH);
            buffer.down(1);
        }
        buffer.set_foreground(TermColor::Default);
        buffer.set_background(TermColor::Default);
    }
}

fn check_bottom_top(bottom: usize, y: usize) -> bool {
    y == 0 || y == bottom - 1
}

#[derive(Clone)]
struct Board {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) rows: Vec<Vec<i16>>
}

impl Board {
    fn new(width: usize, height: usize) -> Self {
        Self::new_init(width, height, -1)
    }

    fn new_init(width: usize, height: usize, init: i16) -> Board {
        Board {
            width,
            height,
            rows: vec![vec![init; width]; height + 1]
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, val: i16) {
        self.rows[y][x] = val;
    }

    fn iter(&self) -> BoardIterator<'_> {
        BoardIterator::new(&self)
    }

    fn lookup(&self, x: usize, y: usize) -> i16 {
        self.rows[y][x]
    }
}

struct BoardIterator<'a>(&'a Board, usize);

impl<'a> BoardIterator<'a> {
    fn new(board: &'a Board) -> BoardIterator<'a> {
        BoardIterator(board, 0)
    }
}

impl<'a> Iterator for BoardIterator<'a> {
    type Item = (&'a [i16], Option<&'a [i16]>);

    fn next(&mut self) -> Option<Self::Item> {
        let max = self.0.rows.len();
        if self.1 >= max {
            return None;
        }
        
        let end = max % 2 != 0 && self.1 + 1 >= max;
        let res = ( 
            self.0.rows[self.1].as_ref(),
            if !end { Some(self.0.rows[self.1 + 1].as_ref()) } else { None },
        );
        self.1 += 2;
        Some(res)
    }
}

fn game_over(game_state: &GameState) -> bool {
    game_state.lives == 0
}

struct Rect {
    top: i32,
    left: i32,
    right: i32,
    bottom: i32,
}

fn get_playfield() -> Rect {
    let (width, height) = crossterm::terminal::size().unwrap();
    let (width, height) = (width as i32, height as i32 * 2);
    Rect {
        top: 4,
        left: 2,
        right: width,
        bottom: height - 1,
    }
}
