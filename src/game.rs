use std::time::{Duration, Instant};

use yeehaw::{Color, Context, Pane};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoardSize {
    Auto,
    Fixed(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Running,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Classic,
    Neon,
    Amber,
}

impl Theme {
    pub fn head_color(self) -> Color {
        match self {
            Theme::Classic => Color::new(0, 255, 0),
            Theme::Neon => Color::new(0, 255, 255),
            Theme::Amber => Color::new(255, 191, 0),
        }
    }

    pub fn body_color(self) -> Color {
        match self {
            Theme::Classic => Color::new(0, 128, 0),
            Theme::Neon => Color::new(0, 128, 128),
            Theme::Amber => Color::new(165, 120, 0),
        }
    }

    pub fn apple_color(self) -> Color {
        match self {
            Theme::Classic => Color::new(255, 0, 0),
            Theme::Neon => Color::new(255, 0, 255),
            Theme::Amber => Color::new(255, 100, 100),
        }
    }
}

pub struct SnakeGame {
    pane: Pane,
    snake: Vec<(usize, usize)>,
    direction: Direction,
    apple: (usize, usize),
    score: usize,
    high_score: usize,
    state: GameState,
    tick_interval: Duration,
    last_tick: Instant,
    board_size: BoardSize,
    theme: Theme,
}

impl SnakeGame {
    pub fn new(ctx: &Context, width: usize, height: usize) -> Self {
        let cx = width / 2;
        let cy = height / 2;

        // snake starts at center, length 3, pointing right
        let snake = vec![
            (cx, cy),
            (cx.saturating_sub(1), cy),
            (cx.saturating_sub(2), cy),
        ];

        SnakeGame {
            pane: Pane::new(ctx, "snake_game"),
            snake,
            direction: Direction::Right,
            apple: (0, 0),
            score: 0,
            high_score: 0,
            state: GameState::Running,
            tick_interval: Duration::from_millis(150),
            last_tick: Instant::now(),
            board_size: BoardSize::Fixed(width, height),
            theme: Theme::Classic,
        }
    }

    pub fn pane(&self) -> &Pane {
        &self.pane
    }

    pub fn snake(&self) -> &[(usize, usize)] {
        &self.snake
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn apple(&self) -> (usize, usize) {
        self.apple
    }

    pub fn score(&self) -> usize {
        self.score
    }

    pub fn high_score(&self) -> usize {
        self.high_score
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn tick_interval(&self) -> Duration {
        self.tick_interval
    }

    pub fn last_tick(&self) -> Instant {
        self.last_tick
    }

    pub fn board_size(&self) -> BoardSize {
        self.board_size
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn set_direction(&mut self, dir: Direction) {
        // prevent reversing into self
        let opposite = match dir {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
        if self.direction != opposite {
            self.direction = dir;
        }
    }

    pub fn set_state(&mut self, state: GameState) {
        self.state = state;
    }

    pub fn set_tick_interval(&mut self, interval: Duration) {
        self.tick_interval = interval;
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}
