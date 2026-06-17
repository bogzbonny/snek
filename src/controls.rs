#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use yeehaw::{
    Button, Context, DropdownList, DynVal, Element, EventResponses, HorizontalStackFocuser,
    Label, Slider, VerticalStack,
};

use crate::config::Config;
use crate::game::{BoardSize, GameState, Theme};

/// Shared state between controls bar and SnakeGame for bidirectional communication.
pub struct ControlState {
    pub tick_interval: Rc<RefCell<Duration>>,
    pub board_size: Rc<RefCell<BoardSize>>,
    pub theme: Rc<RefCell<Theme>>,
    pub score: Rc<RefCell<usize>>,
    pub high_score: Rc<RefCell<usize>>,
    pub state: Rc<RefCell<GameState>>,
    pub num_apples: Rc<RefCell<usize>>,
}

impl ControlState {
    pub fn new(_ctx: &Context) -> Self {
        let cfg = Config::load();
        Self::from_loaded(cfg)
    }

    /// Create a ControlState with hardcoded defaults. Does not read from disk.
    pub fn for_test() -> Self {
        Self {
            tick_interval: Rc::new(RefCell::new(Duration::from_millis(26))),
            board_size: Rc::new(RefCell::new(BoardSize::Auto)),
            theme: Rc::new(RefCell::new(Theme::Classic)),
            score: Rc::new(RefCell::new(0)),
            high_score: Rc::new(RefCell::new(0)),
            state: Rc::new(RefCell::new(GameState::Paused)),
            num_apples: Rc::new(RefCell::new(1)),
        }
    }

    /// Create a ControlState from a Config instance.
    pub fn from_loaded(cfg: Config) -> Self {

        let board_size = match cfg.board_size.as_str() {
            "Auto" => BoardSize::Auto,
            s => {
                if let Some((w_str, h_str)) = s.split_once('x') {
                    if let (Ok(w), Ok(h)) = (w_str.parse::<usize>(), h_str.parse::<usize>()) {
                        BoardSize::Fixed(w, h)
                    } else {
                        BoardSize::Auto
                    }
                } else {
                    BoardSize::Auto
                }
            }
        };

        let theme = match cfg.theme.as_str() {
            "Neon" => Theme::Neon,
            "Amber" => Theme::Amber,
            _ => Theme::Classic,
        };

        let num_apples = cfg.num_apples.clamp(1, 100);

        Self {
            tick_interval: Rc::new(RefCell::new(Duration::from_millis(cfg.speed_ms.max(2)))),
            board_size: Rc::new(RefCell::new(board_size)),
            theme: Rc::new(RefCell::new(theme)),
            score: Rc::new(RefCell::new(0)),
            high_score: Rc::new(RefCell::new(cfg.high_score)),
            state: Rc::new(RefCell::new(GameState::Paused)),
            num_apples: Rc::new(RefCell::new(num_apples)),
        }
    }
}

/// Fixed-width spacer label for horizontal gaps.
fn spacer(ctx: &Context) -> Label {
    let label = Label::new(ctx, " ");
    {
        let mut loc = label.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(2));
        label.set_dyn_location_set(loc);
    }
    label
}

/// Build the bottom control bar as a VerticalStack with two rows.
///
/// `restart_fn` is called by the Restart button to reset the game.
pub fn build_control_bar(
    ctx: &Context,
    state: &ControlState,
    restart_fn: Rc<RefCell<dyn Fn()>>,
) -> Box<dyn Element> {
    let vs = VerticalStack::new(ctx);

    // --- Row 1: Speed, Board size, Theme, Restart ---
    let row1 = HorizontalStackFocuser::new(ctx);

    // Speed label + slider
    row1.push(Box::new(Label::new(ctx, "Speed")));
    row1.push(Box::new(spacer(ctx)));

    let slider = Slider::new_basic_line(ctx);
    *slider.position.borrow_mut() = 0.5;
    {
        let mut loc = slider.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(50));
        slider.set_dyn_location_set(loc);
    }
    let tick_interval = state.tick_interval.clone();
    let board_size = state.board_size.clone();
    let theme = state.theme.clone();
    let high_score = state.high_score.clone();
    let num_apples = state.num_apples.clone();
    *slider.adjust_fn.borrow_mut() = Box::new(move |_ctx, s| {
        let pos = *s.position.borrow();
        // Map 0.0..=1.0 → 50ms..=2.5ms
        let ms = (50.0 - pos * 47.5) as u64;
        *tick_interval.borrow_mut() = Duration::from_millis(ms);
        Config::save_values(ms, &board_size_to_str(&board_size.borrow()), theme_to_str(&theme.borrow()), *high_score.borrow(), *num_apples.borrow());
        EventResponses::default()
    });
    row1.push(Box::new(slider));
    row1.push(Box::new(spacer(ctx)));

    // Board size dropdown
    let board_size = state.board_size.clone();
    let tick_interval = state.tick_interval.clone();
    let theme = state.theme.clone();
    let high_score = state.high_score.clone();
    let num_apples = state.num_apples.clone();
    let size_dropdown = DropdownList::new(
        ctx,
        vec!["Auto", "20x10", "30x15", "40x20", "50x25", "60x30"],
        Box::new(move |_ctx, selected| {
            let bs = match selected.as_str() {
                "Auto" => BoardSize::Auto,
                "20x10" => BoardSize::Fixed(20, 10),
                "30x15" => BoardSize::Fixed(30, 15),
                "40x20" => BoardSize::Fixed(40, 20),
                "50x25" => BoardSize::Fixed(50, 25),
                "60x30" => BoardSize::Fixed(60, 30),
                _ => BoardSize::Auto,
            };
            *board_size.borrow_mut() = bs;
            Config::save_values(tick_interval.borrow().as_millis() as u64, &board_size_to_str(&board_size.borrow()), theme_to_str(&theme.borrow()), *high_score.borrow(), *num_apples.borrow());
            EventResponses::default()
        }),
    );
    row1.push(Box::new(size_dropdown));
    row1.push(Box::new(spacer(ctx)));

    // Theme dropdown
    let theme = state.theme.clone();
    let tick_interval = state.tick_interval.clone();
    let board_size = state.board_size.clone();
    let high_score = state.high_score.clone();
    let num_apples = state.num_apples.clone();
    let theme_dropdown = DropdownList::new(
        ctx,
        vec!["Classic", "Neon", "Amber"],
        Box::new(move |_ctx, selected| {
            let t = match selected.as_str() {
                "Classic" => Theme::Classic,
                "Neon" => Theme::Neon,
                "Amber" => Theme::Amber,
                _ => Theme::Classic,
            };
            *theme.borrow_mut() = t;
            Config::save_values(tick_interval.borrow().as_millis() as u64, &board_size_to_str(&board_size.borrow()), theme_to_str(&theme.borrow()), *high_score.borrow(), *num_apples.borrow());
            EventResponses::default()
        }),
    );
    row1.push(Box::new(theme_dropdown));
    row1.push(Box::new(spacer(ctx)));

    // Restart button
    let restart_btn = Button::new(ctx, "Restart").with_fn(Box::new(move |_btn, _ctx| {
        let fn_ = restart_fn.borrow();
        fn_();
        drop(fn_);
        EventResponses::default()
    }));
    row1.push(Box::new(restart_btn));

    vs.push(Box::new(row1));

    // --- Row 2: Apple count slider ---
    let row2 = HorizontalStackFocuser::new(ctx);

    row2.push(Box::new(Label::new(ctx, "Apples")));
    row2.push(Box::new(spacer(ctx)));

    let apple_slider = Slider::new_basic_line(ctx);
    *apple_slider.position.borrow_mut() = 0.0;
    {
        let mut loc = apple_slider.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(50));
        apple_slider.set_dyn_location_set(loc);
    }
    let num_apples = state.num_apples.clone();
    let tick_interval = state.tick_interval.clone();
    let board_size = state.board_size.clone();
    let theme = state.theme.clone();
    let high_score = state.high_score.clone();
    *apple_slider.adjust_fn.borrow_mut() = Box::new(move |_ctx, s| {
        let pos = *s.position.borrow();
        // Map 0.0..=1.0 → 1..=100
        let n = (pos * 99.0) as usize + 1;
        *num_apples.borrow_mut() = n;
        Config::save_values(tick_interval.borrow().as_millis() as u64, &board_size_to_str(&board_size.borrow()), theme_to_str(&theme.borrow()), *high_score.borrow(), n);
        EventResponses::default()
    });
    row2.push(Box::new(apple_slider));

    vs.push(Box::new(row2));

    Box::new(vs)
}

fn board_size_to_str(bs: &BoardSize) -> String {
    match bs {
        BoardSize::Auto => "Auto".to_string(),
        BoardSize::Fixed(w, h) => format!("{}x{}", w, h),
    }
}

fn theme_to_str(t: &Theme) -> &'static str {
    match t {
        Theme::Classic => "Classic",
        Theme::Neon => "Neon",
        Theme::Amber => "Amber",
    }
}
