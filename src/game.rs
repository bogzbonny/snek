use std::cell::RefCell;
use std::time::{Duration, Instant};

use yeehaw::{
    Attributes, Color, Context, DrawCh, DrawChPos, DrawRegion, DrawUpdate, Element, ElementID,
    Event, EventResponse, EventResponses, FgTranspSrc, Keyboard, Pane, ReceivableEvent,
    ReceivableEvents, Ref, Rc, Style,
};

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

#[derive(Clone)]
pub struct SnakeGame {
    pane: Pane,
    snake: RefCell<Vec<(usize, usize)>>,
    direction: RefCell<Direction>,
    apple: RefCell<(usize, usize)>,
    score: RefCell<usize>,
    high_score: RefCell<usize>,
    state: RefCell<GameState>,
    tick_interval: Duration,
    last_tick: RefCell<Instant>,
    board_size: BoardSize,
    theme: Theme,
    rec_evs: Rc<RefCell<ReceivableEvents>>,
}

fn fg_style(color: Color) -> Style {
    Style {
        fg: Some((color, FgTranspSrc::LowerBg)),
        bg: None,
        underline_color: None,
        attr: Attributes::new(),
    }
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

        let mut rec_evs = ReceivableEvents::default();
        for &key in &[
            Keyboard::KEY_H,
            Keyboard::KEY_J,
            Keyboard::KEY_K,
            Keyboard::KEY_L,
            Keyboard::KEY_LEFT,
            Keyboard::KEY_RIGHT,
            Keyboard::KEY_UP,
            Keyboard::KEY_DOWN,
            Keyboard::KEY_Q,
            Keyboard::KEY_SPACE,
        ] {
            rec_evs.push(ReceivableEvent::from(key));
        }

        SnakeGame {
            pane: Pane::new(ctx, "snake_game"),
            snake: RefCell::new(snake),
            direction: RefCell::new(Direction::Right),
            apple: RefCell::new((0, 0)),
            score: RefCell::new(0),
            high_score: RefCell::new(0),
            state: RefCell::new(GameState::Running),
            tick_interval: Duration::from_millis(150),
            last_tick: RefCell::new(Instant::now()),
            board_size: BoardSize::Fixed(width, height),
            theme: Theme::Classic,
            rec_evs: Rc::new(RefCell::new(rec_evs)),
        }
    }

    pub fn pane(&self) -> &Pane {
        &self.pane
    }

    pub fn snake(&self) -> Vec<(usize, usize)> {
        self.snake.borrow().clone()
    }

    pub fn direction(&self) -> Direction {
        *self.direction.borrow()
    }

    pub fn apple(&self) -> (usize, usize) {
        *self.apple.borrow()
    }

    pub fn score(&self) -> usize {
        *self.score.borrow()
    }

    pub fn high_score(&self) -> usize {
        *self.high_score.borrow()
    }

    pub fn state(&self) -> GameState {
        *self.state.borrow()
    }

    pub fn tick_interval(&self) -> Duration {
        self.tick_interval
    }

    pub fn last_tick(&self) -> Instant {
        *self.last_tick.borrow()
    }

    pub fn board_size(&self) -> BoardSize {
        self.board_size
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn set_direction(&mut self, dir: Direction) {
        let opposite = match dir {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
        if *self.direction.borrow() != opposite {
            *self.direction.borrow_mut() = dir;
        }
    }

    pub fn set_state(&mut self, state: GameState) {
        *self.state.borrow_mut() = state;
    }

    pub fn set_tick_interval(&mut self, interval: Duration) {
        self.tick_interval = interval;
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}

impl Element for SnakeGame {
    fn kind(&self) -> &'static str {
        "snake_game"
    }

    fn id(&self) -> ElementID {
        self.pane.id()
    }

    fn can_receive(&self, ev: &Event) -> bool {
        self.get_focused() && self.rec_evs.borrow().contains_match(ev)
    }

    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        if self.get_focused() {
            vec![self.rec_evs.clone()]
        } else {
            Vec::new()
        }
    }

    fn receive_event(&self, _ctx: &Context, _ev: Event) -> (bool, EventResponses) {
        let state = *self.state.borrow();
        let is_dir_key = |key: &crossterm::event::KeyEvent| -> bool {
            matches!(
                *key,
                Keyboard::KEY_H
                    | Keyboard::KEY_J
                    | Keyboard::KEY_K
                    | Keyboard::KEY_L
                    | Keyboard::KEY_LEFT
                    | Keyboard::KEY_RIGHT
                    | Keyboard::KEY_UP
                    | Keyboard::KEY_DOWN
            )
        };

        let key = match _ev {
            Event::KeyCombo(keys) if keys.len() == 1 => &keys[0],
            _ => return (false, EventResponses::default()),
        };

        // q always quits
        if *key == Keyboard::KEY_Q {
            return (true, EventResponses::from(EventResponse::Quit));
        }

        match state {
            GameState::Paused => {
                if *key == Keyboard::KEY_SPACE {
                    *self.state.borrow_mut() = GameState::Running;
                }
            }
            GameState::Running => {
                if *key == Keyboard::KEY_SPACE {
                    *self.state.borrow_mut() = GameState::Paused;
                } else {
                    self.handle_direction(key, is_dir_key(key));
                }
            }
            GameState::GameOver => {
                if is_dir_key(key) {
                    self.restart_game();
                }
            }
        }

        (true, EventResponses::default())
    }

    fn set_focused(&self, focused: bool) {
        self.pane.set_focused(focused);
    }

    fn get_focused(&self) -> bool {
        self.pane.get_focused()
    }

    fn drawing(&self, _ctx: &Context, dr: &DrawRegion, _force_update: bool) -> Vec<DrawUpdate> {
        let pane_w = dr.size.width as usize;
        let pane_h = dr.size.height as usize;

        let (board_w, board_h, border_x, border_y) = match self.board_size {
            BoardSize::Auto => {
                let w = pane_w.saturating_sub(2);
                let h = pane_h.saturating_sub(2);
                (w, h, 0, 0)
            }
            BoardSize::Fixed(w, h) => {
                let bw = w + 2;
                let bh = h + 2;
                let ox = (pane_w.saturating_sub(bw)) / 2;
                let oy = (pane_h.saturating_sub(bh)) / 2;
                (w, h, ox, oy)
            }
        };

        // simplification: skip rendering if pane is too small for a border
        if pane_w < 4 || pane_h < 4 {
            return Vec::new();
        }

        let mut chs = Vec::new();
        let theme = self.theme;
        let snake = self.snake.borrow();
        let apple = *self.apple.borrow();

        let head_color = fg_style(theme.head_color());
        let body_color = fg_style(theme.body_color());
        let apple_color = fg_style(theme.apple_color());
        let black = fg_style(Color::new(0, 0, 0));

        // border extents
        let bl = border_x;
        let br_ = border_x + board_w + 1;
        let bt = border_y;
        let bb = border_y + board_h + 1;

        // corners
        chs.push(DrawChPos::new(DrawCh::new('┌', black.clone()), bl as u16, bt as u16));
        chs.push(DrawChPos::new(DrawCh::new('┐', black.clone()), br_ as u16, bt as u16));
        chs.push(DrawChPos::new(DrawCh::new('└', black.clone()), bl as u16, bb as u16));
        chs.push(DrawChPos::new(DrawCh::new('┘', black.clone()), br_ as u16, bb as u16));

        // top/bottom horizontal
        for x in (bl + 1)..br_ {
            chs.push(DrawChPos::new(DrawCh::new('─', black.clone()), x as u16, bt as u16));
            chs.push(DrawChPos::new(DrawCh::new('─', black.clone()), x as u16, bb as u16));
        }

        // left/right vertical
        for y in (bt + 1)..bb {
            chs.push(DrawChPos::new(DrawCh::new('│', black.clone()), bl as u16, y as u16));
            chs.push(DrawChPos::new(DrawCh::new('│', black.clone()), br_ as u16, y as u16));
        }

        // interior cells
        let gx = border_x + 1;
        let gy = border_y + 1;
        for y in 0..board_h {
            for x in 0..board_w {
                let sx = gx + x;
                let sy = gy + y;
                let ch = if snake[0] == (x, y) {
                    DrawCh::new('◆', head_color.clone())
                } else if snake.iter().skip(1).any(|&(cx, cy)| cx == x && cy == y) {
                    DrawCh::new('■', body_color.clone())
                } else if apple == (x, y) {
                    DrawCh::new('ἴe', apple_color.clone())
                } else {
                    DrawCh::new(' ', black.clone())
                };
                chs.push(DrawChPos::new(ch, sx as u16, sy as u16));
            }
        }

        DrawUpdate::update(chs).into()
    }

    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.pane.get_attribute(key)
    }

    fn set_attribute_inner(&self, key: &str, value: Vec<u8>) {
        self.pane.set_attribute_inner(key, value);
    }

    fn set_parent(&self, parent: Box<dyn yeehaw::Parent>) {
        self.pane.set_parent(parent);
    }

    fn set_hook(&self, kind: &str, el_id: ElementID, hook: yeehaw::ElementHookFn) {
        self.pane.set_hook(kind, el_id, hook);
    }

    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.pane.remove_hook(kind, el_id);
    }

    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.pane.clear_hooks_by_id(el_id);
    }

    fn call_hooks_of_kind(&self, kind: &str) {
        self.pane.call_hooks_of_kind(kind);
    }

    fn get_dyn_location_set(&self) -> Ref<'_, yeehaw::DynLocationSet> {
        self.pane.get_dyn_location_set()
    }

    fn get_visible(&self) -> bool {
        self.pane.get_visible()
    }

    fn get_ref_cell_dyn_location_set(&self) -> Rc<RefCell<yeehaw::DynLocationSet>> {
        self.pane.get_ref_cell_dyn_location_set()
    }

    fn get_ref_cell_visible(&self) -> Rc<RefCell<bool>> {
        self.pane.get_ref_cell_visible()
    }

    fn get_ref_cell_overflow(&self) -> Rc<RefCell<bool>> {
        self.pane.get_ref_cell_overflow()
    }

    fn set_content_x_offset(&self, dr: Option<&DrawRegion>, x: usize) {
        self.pane.set_content_x_offset(dr, x);
    }

    fn set_content_y_offset(&self, dr: Option<&DrawRegion>, y: usize) {
        self.pane.set_content_y_offset(dr, y);
    }

    fn get_content_x_offset(&self) -> usize {
        self.pane.get_content_x_offset()
    }

    fn get_content_y_offset(&self) -> usize {
        self.pane.get_content_y_offset()
    }

    fn get_content_width(&self, dr: Option<&DrawRegion>) -> usize {
        self.pane.get_content_width(dr)
    }

    fn get_content_height(&self, dr: Option<&DrawRegion>) -> usize {
        self.pane.get_content_height(dr)
    }
}

impl SnakeGame {
    fn handle_direction(&self, key: &crossterm::event::KeyEvent, _is_dir: bool) {
        let new_dir = match *key {
            Keyboard::KEY_K | Keyboard::KEY_UP => Direction::Up,
            Keyboard::KEY_J | Keyboard::KEY_DOWN => Direction::Down,
            Keyboard::KEY_H | Keyboard::KEY_LEFT => Direction::Left,
            Keyboard::KEY_L | Keyboard::KEY_RIGHT => Direction::Right,
            _ => return,
        };
        let cur = *self.direction.borrow();
        let opposite = match new_dir {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
        if cur != opposite {
            *self.direction.borrow_mut() = new_dir;
        }
    }

    fn restart_game(&self) {
        let (bw, bh) = match self.board_size {
            BoardSize::Auto => {
                // simplification: uses last known board size; unknown at restart time without DrawRegion
                (10, 10)
            }
            BoardSize::Fixed(w, h) => (w, h),
        };
        let cx = bw / 2;
        let cy = bh / 2;
        let snake = vec![(cx, cy), (cx.saturating_sub(1), cy), (cx.saturating_sub(2), cy)];
        *self.snake.borrow_mut() = snake;
        *self.direction.borrow_mut() = Direction::Right;
        *self.score.borrow_mut() = 0;
        *self.state.borrow_mut() = GameState::Running;
        *self.last_tick.borrow_mut() = Instant::now();
    }
}