use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;

use crossterm::event::KeyEvent;
use rand::Rng;
use yeehaw::{
    Attributes, Color, Context, DrawCh, DrawChPos, DrawRegion, DrawUpdate, Element, ElementID,
    Event, EventResponse, EventResponses, FgTranspSrc, Keyboard, Pane, Rc, ReceivableEvent,
    ReceivableEvents, Ref, Style,
};

use crate::config::Config;
use crate::controls::ControlState;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BoardSize {
    Auto,
    Fixed(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    Running,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Theme {
    Classic,
    Neon,
    Amber,
}

/// Kind of food item on the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FoodKind {
    RedApple,
}

impl FoodKind {
    /// Display character for this food kind.
    pub fn char_code(self) -> char {
        match self {
            FoodKind::RedApple => '🍎',
        }
    }
}

/// A food item the snek can consume.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Food {
    pub kind: FoodKind,
    pub x: usize,
    pub y: usize,
    pub consumed: bool,
}

impl Default for Food {
    fn default() -> Self {
        Self {
            kind: FoodKind::RedApple,
            x: 0,
            y: 0,
            consumed: true,
        }
    }
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

    pub fn food_color(self) -> Color {
        match self {
            Theme::Classic => Color::new(255, 0, 0),
            Theme::Neon => Color::new(255, 0, 255),
            Theme::Amber => Color::new(255, 100, 100),
        }
    }
}

#[derive(Clone)]
pub struct SnekGame {
    pane: Pane,
    snek: Rc<RefCell<Vec<(usize, usize)>>>,
    direction: Rc<RefCell<Direction>>,
    foods: Rc<RefCell<Vec<Food>>>,
    // Shared state refs — bidirectional sync with control bar
    ctrl_tick_interval: Rc<RefCell<Duration>>,
    ctrl_board_size: Rc<RefCell<BoardSize>>,
    ctrl_theme: Rc<RefCell<Theme>>,
    ctrl_score: Rc<RefCell<usize>>,
    ctrl_high_score: Rc<RefCell<usize>>,
    ctrl_state: Rc<RefCell<GameState>>,
    ctrl_num_foods: Rc<RefCell<usize>>,
    // Last-known board dimensions for Auto mode (Rc so clones share state with original)
    last_board_w: Rc<RefCell<usize>>,
    last_board_h: Rc<RefCell<usize>>,
    // True after board has been initialized with valid dimensions (Rc so clones share)
    board_initialized: Rc<RefCell<bool>>,
    // Track board size to detect mid-game changes (Rc so clones share)
    last_board_size: Rc<RefCell<BoardSize>>,
    // Queue of pending direction changes; one is dequeued per tick (max 10)
    direction_queue: Rc<RefCell<VecDeque<Direction>>>,
}

fn fg_style(color: Color) -> Style {
    Style {
        fg: Some((color, FgTranspSrc::LowerBg)),
        bg: None,
        underline_color: None,
        attr: Attributes::new(),
    }
}

#[allow(dead_code)]
impl SnekGame {
    pub fn new(ctx: &Context, ctrl: &ControlState) -> Self {
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

        let pane = Pane::new(ctx, "snek_game");
        pane.set_focused_receivable_events(rec_evs);
        pane.set_focused(true);

        let game = SnekGame {
            pane,
            snek: Rc::new(RefCell::new(Vec::new())),
            direction: Rc::new(RefCell::new(Direction::Right)),
            foods: Rc::new(RefCell::new(Vec::new())),
            // Shared state
            ctrl_tick_interval: ctrl.tick_interval.clone(),
            ctrl_board_size: ctrl.board_size.clone(),
            ctrl_theme: ctrl.theme.clone(),
            ctrl_score: ctrl.score.clone(),
            ctrl_high_score: ctrl.high_score.clone(),
            ctrl_state: ctrl.state.clone(),
            ctrl_num_foods: ctrl.num_foods.clone(),
            last_board_w: Rc::new(RefCell::new(0)),
            last_board_h: Rc::new(RefCell::new(0)),
            board_initialized: Rc::new(RefCell::new(false)),
            last_board_size: Rc::new(RefCell::new(*ctrl.board_size.borrow())),
            direction_queue: Rc::new(RefCell::new(VecDeque::new())),
        };
        game
    }

    pub fn snek(&self) -> Vec<(usize, usize)> {
        self.snek.borrow().clone()
    }

    pub fn direction(&self) -> Direction {
        *self.direction.borrow()
    }

    /// Return the first non-consumed food, or a sentinel if none exist.
    pub fn food(&self) -> Food {
        *self.foods.borrow()
            .iter()
            .find(|f| !f.consumed)
            .unwrap_or(&Food::default())
    }

    pub fn foods(&self) -> Vec<Food> {
        self.foods.borrow().clone()
    }

    pub fn score(&self) -> usize {
        *self.ctrl_score.borrow()
    }

    pub fn high_score(&self) -> usize {
        *self.ctrl_high_score.borrow()
    }

    pub fn state(&self) -> GameState {
        *self.ctrl_state.borrow()
    }

    pub fn tick_interval(&self) -> Duration {
        *self.ctrl_tick_interval.borrow()
    }

    pub fn board_size(&self) -> BoardSize {
        *self.ctrl_board_size.borrow()
    }

    pub fn theme(&self) -> Theme {
        *self.ctrl_theme.borrow()
    }

    pub fn set_direction(&self, dir: Direction) {
        let opposite = match dir {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        };
        if *self.direction.borrow() != opposite {
            *self.direction.borrow_mut() = dir;
        }
        self.direction_queue.borrow_mut().clear();
    }

    /// Initialize snek, food and score for the given board dimensions.
    fn init_board(&self, bw: usize, bh: usize) {
        // Guard: playable area must be large enough for food.
        if bw.saturating_mul(bh) < 4 {
            return;
        }

        let cx = bw / 2;
        let cy = bh / 2;
        let snek = vec![
            (cx, cy),
            (cx.saturating_sub(1), cy),
            (cx.saturating_sub(2), cy),
        ];
        *self.snek.borrow_mut() = snek;
        *self.direction.borrow_mut() = Direction::Right;
        *self.ctrl_score.borrow_mut() = 0;
        self.foods.borrow_mut().clear();
        self.spawn_food(bw, bh);
        *self.board_initialized.borrow_mut() = true;
    }
}

impl Element for SnekGame {
    fn kind(&self) -> &'static str {
        "snek_game"
    }

    fn id(&self) -> ElementID {
        self.pane.id()
    }

    fn can_receive(&self, ev: &Event) -> bool {
        self.pane.can_receive(ev)
    }

    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        self.pane.receivable()
    }

    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        let state = *self.ctrl_state.borrow();

        let is_dir_key = |k: KeyEvent| -> bool {
            Keyboard::is_key_one_of(
                k,
                vec![
                    Keyboard::KEY_H,
                    Keyboard::KEY_J,
                    Keyboard::KEY_K,
                    Keyboard::KEY_L,
                    Keyboard::KEY_LEFT,
                    Keyboard::KEY_RIGHT,
                    Keyboard::KEY_UP,
                    Keyboard::KEY_DOWN,
                ],
            )
        };

        // Map key to direction without reversal protection.
        // Used in Paused state where no movement has occurred yet.
        let key_to_direction = |k: KeyEvent| -> Direction {
            if k == Keyboard::KEY_K || k == Keyboard::KEY_UP {
                Direction::Up
            } else if k == Keyboard::KEY_J || k == Keyboard::KEY_DOWN {
                Direction::Down
            } else if k == Keyboard::KEY_H || k == Keyboard::KEY_LEFT {
                Direction::Left
            } else {
                Direction::Right
            }
        };

        let key = match ev {
            Event::KeyCombo(keys) if keys.len() == 1 => keys[0],
            _ => return (false, EventResponses::default()),
        };

        if key == Keyboard::KEY_Q {
            return (true, EventResponses::from(EventResponse::Quit));
        }

        match state {
            GameState::Paused => {
                if key == Keyboard::KEY_SPACE || is_dir_key(key) {
                    if is_dir_key(key) {
                        let new_dir = key_to_direction(key);
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
                    *self.ctrl_state.borrow_mut() = GameState::Running;
                    // Move snek immediately on first direction key press.
                    if is_dir_key(key) {
                        self.tick(ctx);
                    }
                }
            }
            GameState::Running => {
                if key == Keyboard::KEY_SPACE {
                    *self.ctrl_state.borrow_mut() = GameState::Paused;
                } else if is_dir_key(key) {
                    let new_dir = key_to_direction(key);
                    let mut q = self.direction_queue.borrow_mut();
                    if q.len() < 10 {
                        q.push_back(new_dir);
                    }
                }
            }
            GameState::GameOver => {
                if key == Keyboard::KEY_SPACE || is_dir_key(key) {
                    self.restart();
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

        if pane_w < 4 || pane_h < 4 {
            return Vec::new();
        }

        let board_size = *self.ctrl_board_size.borrow();
        let (board_w, board_h, content_x, content_y) = match board_size {
            BoardSize::Auto => {
                // Playable area: full pane minus 1 row for status line.
                let w = pane_w;
                let h = pane_h.saturating_sub(1);
                // Only cache when playable area is large enough. Only grow, never shrink.
                if w.saturating_mul(h) >= 4 {
                    let cur_w = *self.last_board_w.borrow();
                    let cur_h = *self.last_board_h.borrow();
                    if w >= cur_w && h >= cur_h {
                        *self.last_board_w.borrow_mut() = w;
                        *self.last_board_h.borrow_mut() = h;
                    }
                }
                let cached_w = *self.last_board_w.borrow();
                let cached_h = *self.last_board_h.borrow();
                let bw = if cached_w > 0 { cached_w } else { w };
                let bh = if cached_h > 0 { cached_h } else { h };
                // Content starts at (0, 0) — Bordered wrapper handles the border outside.
                (bw, bh, 0, 0)
            }
            BoardSize::Fixed(w, h) => {
                // Center the board within the pane.
                let inner_w = pane_w;
                let inner_h = pane_h;
                let ox = inner_w.saturating_sub(w) / 2;
                let oy = inner_h.saturating_sub(h) / 2;
                (w, h, ox, oy)
            }
        };

        // Initialize board on first draw with actual pane dimensions
        if !*self.board_initialized.borrow() {
            self.init_board(board_w, board_h);
        }

        let mut updates = Vec::new();

        let mut chs = Vec::new();
        let theme = *self.ctrl_theme.borrow();
        let snek = self.snek.borrow();
        let foods = self.foods.borrow();

        let head_color = fg_style(theme.head_color());
        let body_color = fg_style(theme.body_color());
        let food_color = fg_style(theme.food_color());
        let default_style = Style {
            fg: None,
            bg: None,
            underline_color: None,
            attr: Attributes::new(),
        };

        let state = *self.ctrl_state.borrow();
        let overlay_msgs: Option<Vec<String>> = match state {
            GameState::Paused => Some(vec![
                "- snek -".into(),
                "(press an arrow key to start)".into(),
            ]),
            GameState::GameOver => {
                let score = *self.ctrl_score.borrow();
                Some(vec![
                    "- game over -".into(),
                    format!("your score: {}", score),
                ])
            }
            _ => None,
        };

        for y in 0..board_h {
            for x in 0..board_w {
                let sx = content_x + x;
                let sy = content_y + y;
                let ch = if let Some(ref msgs) = overlay_msgs {
                    if let Some((line_idx, char_idx)) =
                        msgs.iter().enumerate().find_map(|(i, m)| {
                            let line_y = board_h / 2 - 1 + i;
                            let start_x = board_w.saturating_sub(m.len()) / 2;
                            if y == line_y && x >= start_x && x < start_x + m.len() {
                                Some((i, x - start_x))
                            } else {
                                None
                            }
                        })
                    {
                        DrawCh::new(
                            msgs[line_idx].as_bytes()[char_idx] as char,
                            default_style.clone(),
                        )
                    } else {
                        DrawCh::new(' ', default_style.clone())
                    }
                } else if snek[0] == (x, y) {
                    DrawCh::new('◆', head_color.clone())
                } else if snek.iter().skip(1).any(|&(cx, cy)| cx == x && cy == y) {
                    DrawCh::new('■', body_color.clone())
                } else if let Some(f) = foods.iter().find(|f| !f.consumed && f.x == x && f.y == y) {
                    DrawCh::new(f.kind.char_code(), food_color.clone())
                } else {
                    DrawCh::new(' ', default_style.clone())
                };
                chs.push(DrawChPos::new(ch, sx as u16, sy as u16));
            }
        }

        // Render score line below the board (last row of the pane).
        let status_y = pane_h - 1;
        if status_y > 0 {
            let score = *self.ctrl_score.borrow();
            let high = *self.ctrl_high_score.borrow();
            let status_str = format!("Score: {}  Best: {}", score, high);
            let start_x = pane_w.saturating_sub(status_str.len()) / 2;
            for (i, ch) in status_str.chars().enumerate() {
                chs.push(DrawChPos::new(
                    DrawCh::new(ch, default_style.clone()),
                    (start_x + i) as u16,
                    status_y as u16,
                ));
            }
        }

        updates.push(DrawUpdate::update(chs));
        updates
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

#[allow(dead_code)]
impl SnekGame {
    /// Test helper: replace all food with a single RedApple at the given position.
    pub fn spawn_food_at(&self, x: usize, y: usize) {
        self.foods.borrow_mut().clear();
        self.foods.borrow_mut().push(Food {
            kind: FoodKind::RedApple,
            x,
            y,
            consumed: false,
        });
    }

    /// Remove consumed food and spawn new food to reach target count.
    fn spawn_food(&self, bw: usize, bh: usize) {
        if bw.saturating_mul(bh) < 4 {
            return;
        }
        let target = *self.ctrl_num_foods.borrow();
        let mut foods = self.foods.borrow_mut();
        if target == 0 {
            foods.clear();
            return;
        }

        // Remove consumed food
        foods.retain(|f| !f.consumed);

        let snek = self.snek.borrow();
        let occupied: std::collections::HashSet<_> = snek
            .iter()
            .copied()
            .chain(foods.iter().map(|f| (f.x, f.y)))
            .collect();
        let free: Vec<_> = (0..bw)
            .flat_map(|x| (0..bh).map(move |y| (x, y)))
            .filter(|p| !occupied.contains(p))
            .collect();
        drop(snek);
        let mut rng = rand::thread_rng();
        let needed = target.saturating_sub(foods.len());
        let mut available = free;
        for _ in 0..needed {
            if available.is_empty() {
                break;
            }
            let idx = rng.gen_range(0..available.len());
            let pos = available.remove(idx);
            foods.push(Food {
                kind: FoodKind::RedApple,
                x: pos.0,
                y: pos.1,
                consumed: false,
            });
        }
    }

    pub fn restart(&self) {
        let (bw, bh) = match *self.ctrl_board_size.borrow() {
            BoardSize::Auto => (*self.last_board_w.borrow(), *self.last_board_h.borrow()),
            BoardSize::Fixed(w, h) => (w, h),
        };
        if bw > 0 && bh > 0 {
            self.init_board(bw, bh);
        } else {
            // Board not yet drawn; defer initialization to next draw
            *self.board_initialized.borrow_mut() = false;
            *self.ctrl_score.borrow_mut() = 0;
        }
        *self.ctrl_state.borrow_mut() = GameState::Paused;
        self.direction_queue.borrow_mut().clear();
    }

    pub fn tick(&self, _ctx: &Context) {
        if *self.ctrl_state.borrow() != GameState::Running {
            return;
        }

        // Skip if board not yet initialized by drawing()
        if !*self.board_initialized.borrow() {
            return;
        }

        // Process one queued direction change per tick
        {
            if let Some(next_dir) = self.direction_queue.borrow_mut().pop_front() {
                let cur = *self.direction.borrow();
                let opposite = match next_dir {
                    Direction::Up => Direction::Down,
                    Direction::Down => Direction::Up,
                    Direction::Left => Direction::Right,
                    Direction::Right => Direction::Left,
                };
                if cur != opposite {
                    *self.direction.borrow_mut() = next_dir;
                }
            }
        }

        // Detect board size change mid-game; restart to reposition snek/food
        let new_board_size = *self.ctrl_board_size.borrow();
        if new_board_size != *self.last_board_size.borrow() {
            *self.last_board_size.borrow_mut() = new_board_size;
            self.restart();
            return;
        }

        let dir = *self.direction.borrow();
        let (bw, bh) = match *self.ctrl_board_size.borrow() {
            BoardSize::Fixed(w, h) => (w, h),
            BoardSize::Auto => (*self.last_board_w.borrow(), *self.last_board_h.borrow()),
        };
        let mut snek = self.snek.borrow_mut();
        let (hx, hy) = snek[0];

        // Consistent wrapping arithmetic for all directions; bounds check catches OOB.
        let (nx, ny) = match dir {
            Direction::Up => (hx, hy.wrapping_sub(1)),
            Direction::Down => (hx, hy.wrapping_add(1)),
            Direction::Left => (hx.wrapping_sub(1), hy),
            Direction::Right => (hx.wrapping_add(1), hy),
        };

        let mut eating = false;
        {
            let mut foods = self.foods.borrow_mut();
            for f in foods.iter_mut() {
                if f.x == nx && f.y == ny && !f.consumed {
                    f.consumed = true;
                    eating = true;
                    break;
                }
            }
        }

        if nx >= bw || ny >= bh {
            drop(snek);
            *self.ctrl_state.borrow_mut() = GameState::GameOver;
            return;
        }

        // Exclude tail from collision check when not eating: the tail will move away.
        let segments_to_check = if eating {
            snek.len()
        } else {
            snek.len().saturating_sub(1)
        };
        if snek
            .iter()
            .take(segments_to_check)
            .any(|&(sx, sy)| sx == nx && sy == ny)
        {
            drop(snek);
            *self.ctrl_state.borrow_mut() = GameState::GameOver;
            return;
        }

        if eating {
            snek.insert(0, (nx, ny));

            let new_score = {
                *self.ctrl_score.borrow_mut() += 1;
                *self.ctrl_score.borrow()
            };
            if new_score > *self.ctrl_high_score.borrow() {
                *self.ctrl_high_score.borrow_mut() = new_score;
                let speed_ms = self.ctrl_tick_interval.borrow().as_millis() as u64;
                let board_size = match *self.ctrl_board_size.borrow() {
                    BoardSize::Auto => "Auto".to_string(),
                    BoardSize::Fixed(w, h) => format!("{}x{}", w, h),
                };
                let theme = match *self.ctrl_theme.borrow() {
                    Theme::Classic => "Classic",
                    Theme::Neon => "Neon",
                    Theme::Amber => "Amber",
                };
                let num_foods = *self.ctrl_num_foods.borrow();
                Config::save_values(speed_ms, &board_size, theme, new_score, num_foods);
            }

            drop(snek);
            self.spawn_food(bw, bh);
        } else {
            snek.pop();
            snek.insert(0, (nx, ny));
        }
    }
}
