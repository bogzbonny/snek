#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use yeehaw::{
    Button, Context, DropdownList, DynVal, Element, EventResponses, Label, ParentPane, Slider,
};

use crate::config::Config;
use crate::game::{BoardSize, GameState, Theme};

/// Shared state between controls bar and SnekGame for bidirectional communication.
pub struct ControlState {
    pub tick_interval: Rc<RefCell<Duration>>,
    pub board_size: Rc<RefCell<BoardSize>>,
    pub theme: Rc<RefCell<Theme>>,
    pub score: Rc<RefCell<usize>>,
    pub high_score: Rc<RefCell<usize>>,
    pub state: Rc<RefCell<GameState>>,
    pub num_foods: Rc<RefCell<usize>>,
}

impl ControlState {
    pub fn new(_ctx: &Context) -> Self {
        let cfg = Config::load();
        Self::from_loaded(cfg)
    }

    /// Create a ControlState with default values. Does not read from disk.
    pub fn for_test() -> Self {
        Self::from_loaded(Config::default())
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

        let num_foods = cfg.num_foods.clamp(1, 100);

        Self {
            tick_interval: Rc::new(RefCell::new(Duration::from_millis(cfg.speed_ms.max(2)))),
            board_size: Rc::new(RefCell::new(board_size)),
            theme: Rc::new(RefCell::new(theme)),
            score: Rc::new(RefCell::new(0)),
            high_score: Rc::new(RefCell::new(cfg.high_score)),
            state: Rc::new(RefCell::new(GameState::Paused)),
            num_foods: Rc::new(RefCell::new(num_foods)),
        }
    }
}

/// Build the bottom control bar as a single ParentPane with absolute positioning.
///
/// `restart_fn` is called by the Restart button to reset the game.
pub fn build_control_bar(
    ctx: &Context,
    state: &ControlState,
    restart_fn: Rc<RefCell<dyn Fn()>>,
) -> Box<dyn Element> {
    let pane = ParentPane::new(ctx, "control_bar");

    // --- Row 0: Speed, Board size, Theme, Restart ---

    // Speed label
    pane.add_element(Box::new(Label::new(ctx, "Speed").at(0, 0)));

    // Speed slider
    let speed_slider = Slider::new_basic_line(ctx);
    *speed_slider.position.borrow_mut() = 0.5;
    {
        let mut loc = speed_slider.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(50));
        speed_slider.set_dyn_location_set(loc);
    }
    let tick_interval = state.tick_interval.clone();
    let board_size = state.board_size.clone();
    let theme = state.theme.clone();
    let high_score = state.high_score.clone();
    let num_foods = state.num_foods.clone();
    *speed_slider.adjust_fn.borrow_mut() = Box::new(move |_ctx, s| {
        let pos = *s.position.borrow();
        // Map 0.0..=1.0 → 50ms..=2.5ms
        let ms = (50.0 - pos * 47.5) as u64;
        *tick_interval.borrow_mut() = Duration::from_millis(ms);
        Config::save_values(
            ms,
            &board_size_to_str(&board_size.borrow()),
            theme_to_str(&theme.borrow()),
            *high_score.borrow(),
            *num_foods.borrow(),
        );
        EventResponses::default()
    });
    pane.add_element(Box::new(speed_slider.at(7, 0)));

    // Board size dropdown
    let board_size = state.board_size.clone();
    let tick_interval = state.tick_interval.clone();
    let theme = state.theme.clone();
    let high_score = state.high_score.clone();
    let num_foods = state.num_foods.clone();
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
            Config::save_values(
                tick_interval.borrow().as_millis() as u64,
                &board_size_to_str(&board_size.borrow()),
                theme_to_str(&theme.borrow()),
                *high_score.borrow(),
                *num_foods.borrow(),
            );
            EventResponses::default()
        }),
    );
    pane.add_element(Box::new(size_dropdown.at(59, 0)));

    // Theme dropdown
    let theme = state.theme.clone();
    let tick_interval = state.tick_interval.clone();
    let board_size = state.board_size.clone();
    let high_score = state.high_score.clone();
    let num_foods = state.num_foods.clone();
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
            Config::save_values(
                tick_interval.borrow().as_millis() as u64,
                &board_size_to_str(&board_size.borrow()),
                theme_to_str(&theme.borrow()),
                *high_score.borrow(),
                *num_foods.borrow(),
            );
            EventResponses::default()
        }),
    );
    pane.add_element(Box::new(theme_dropdown.at(74, 0)));

    // Restart button
    let restart_btn = Button::new(ctx, "Restart").with_fn(Box::new(move |_btn, _ctx| {
        let fn_ = restart_fn.borrow();
        fn_();
        drop(fn_);
        EventResponses::default()
    }));
    pane.add_element(Box::new(restart_btn.at(89, 0)));

    // --- Row 1: Food count slider ---

    pane.add_element(Box::new(Label::new(ctx, "Foods").at(0, 1)));

    let food_slider = Slider::new_basic_line(ctx);
    *food_slider.position.borrow_mut() = 0.0;
    {
        let mut loc = food_slider.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(50));
        food_slider.set_dyn_location_set(loc);
    }
    let num_foods = state.num_foods.clone();
    let tick_interval = state.tick_interval.clone();
    let board_size = state.board_size.clone();
    let theme = state.theme.clone();
    let high_score = state.high_score.clone();
    *food_slider.adjust_fn.borrow_mut() = Box::new(move |_ctx, s| {
        let pos = *s.position.borrow();
        // Map 0.0..=1.0 → 1..=100
        let n = (pos * 99.0) as usize + 1;
        *num_foods.borrow_mut() = n;
        Config::save_values(
            tick_interval.borrow().as_millis() as u64,
            &board_size_to_str(&board_size.borrow()),
            theme_to_str(&theme.borrow()),
            *high_score.borrow(),
            n,
        );
        EventResponses::default()
    });
    pane.add_element(Box::new(food_slider.at(8, 1)));

    Box::new(pane)
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
