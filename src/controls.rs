#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use yeehaw::{
    Button, Checkbox, Context, DynVal, Element, EventResponses, Label, ParentPane, Slider,
    SingleLineTextBox,
};

use crate::config::Config;
use crate::game::{BoardSize, GameState};

/// Shared state between controls bar and SnekGame for bidirectional communication.
pub struct ControlState {
    pub tick_interval: Rc<RefCell<Duration>>,
    pub board_size: Rc<RefCell<BoardSize>>,
    pub score: Rc<RefCell<usize>>,
    pub high_score: Rc<RefCell<usize>>,
    pub state: Rc<RefCell<GameState>>,
    pub num_foods: Rc<RefCell<usize>>,
    pub no_walls: Rc<RefCell<bool>>,
    pub emoji_double_width: Rc<RefCell<bool>>,
    pub score_display: Option<SingleLineTextBox>,
    pub best_display: Option<SingleLineTextBox>,
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

        let num_foods = cfg.num_foods.clamp(1, 100);

        Self {
            tick_interval: Rc::new(RefCell::new(Duration::from_millis(cfg.speed_ms.max(2)))),
            board_size: Rc::new(RefCell::new(board_size)),
            score: Rc::new(RefCell::new(0)),
            high_score: Rc::new(RefCell::new(cfg.high_score)),
            state: Rc::new(RefCell::new(GameState::Paused)),
            num_foods: Rc::new(RefCell::new(num_foods)),
            no_walls: Rc::new(RefCell::new(cfg.no_walls)),
            emoji_double_width: Rc::new(RefCell::new(cfg.emoji_are_double_width)),
            score_display: None,
            best_display: None,
        }
    }
}

/// Build the bottom control bar as a single ParentPane with absolute positioning.
///
/// `restart_fn` is called by the Restart button to reset the game.
pub fn build_control_bar(
    ctx: &Context,
    state: &mut ControlState,
    restart_fn: Rc<RefCell<dyn Fn()>>,
) -> Box<dyn Element> {
    let pane = ParentPane::new(ctx, "control_bar");

    // --- Row 0: Speed, Board size, Restart ---

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
    let high_score = state.high_score.clone();
    let num_foods = state.num_foods.clone();
    let no_walls = state.no_walls.clone();
    let emoji_double_width = state.emoji_double_width.clone();
    *speed_slider.adjust_fn.borrow_mut() = Box::new(move |_ctx, s| {
        let pos = *s.position.borrow();
        // Map 0.0..=1.0 → 50ms..=2.5ms
        let ms = (50.0 - pos * 47.5) as u64;
        *tick_interval.borrow_mut() = Duration::from_millis(ms);
        Config::save_values(ms, &board_size_to_str(&board_size.borrow()), *high_score.borrow(), *num_foods.borrow(), *no_walls.borrow(), *emoji_double_width.borrow());
        EventResponses::default()
    });
    pane.add_element(Box::new(speed_slider.at(7, 0)));

    // Board size textboxes
    let board_size = state.board_size.clone();
    let tick_interval = state.tick_interval.clone();
    let high_score = state.high_score.clone();
    let num_foods = state.num_foods.clone();
    let no_walls = state.no_walls.clone();
    let emoji_double_width = state.emoji_double_width.clone();

    let width_tb = SingleLineTextBox::new(ctx);
    let height_tb = SingleLineTextBox::new(ctx);

    match *board_size.borrow() {
        BoardSize::Auto => {
            width_tb.set_text("Auto".to_string());
            height_tb.set_text("Auto".to_string());
        }
        BoardSize::Fixed(w, h) => {
            width_tb.set_text(w.to_string());
            height_tb.set_text(h.to_string());
        }
    }

    {
        let mut loc = width_tb.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(4));
        width_tb.set_dyn_location_set(loc);
    }
    {
        let mut loc = height_tb.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(4));
        height_tb.set_dyn_location_set(loc);
    }

    // Width textbox hook
    {
        let board_size = board_size.clone();
        let tick_interval = tick_interval.clone();
        let high_score = high_score.clone();
        let num_foods = num_foods.clone();
        let no_walls = no_walls.clone();
        let emoji_double_width = emoji_double_width.clone();
        let height_tb_clone = height_tb.clone();
        let width_clone = width_tb.clone();
        width_tb.set_hook(Box::new(move |_ctx, is_final, text| {
            if is_final {
                let restore = match *board_size.borrow() {
                    BoardSize::Auto => "Auto".to_string(),
                    BoardSize::Fixed(w, _) => w.to_string(),
                };
                width_clone.set_text(restore);
                return EventResponses::default();
            }
            let w = parse_dim(&text);
            let h = parse_dim(&height_tb_clone.tb.get_text());
            let bs = match (w, h) {
                (Some(w), Some(h)) => BoardSize::Fixed(w, h),
                _ => BoardSize::Auto,
            };
            *board_size.borrow_mut() = bs;
            Config::save_values(
                tick_interval.borrow().as_millis() as u64,
                &board_size_to_str(&board_size.borrow()),
                *high_score.borrow(),
                *num_foods.borrow(),
                *no_walls.borrow(),
                *emoji_double_width.borrow(),
            );
            EventResponses::default()
        }));
    }

    // Height textbox hook
    {
        let board_size = board_size.clone();
        let tick_interval = tick_interval.clone();
        let high_score = high_score.clone();
        let num_foods = num_foods.clone();
        let no_walls = no_walls.clone();
        let emoji_double_width = emoji_double_width.clone();
        let width_tb_clone = width_tb.clone();
        let height_clone = height_tb.clone();
        height_tb.set_hook(Box::new(move |_ctx, is_final, text| {
            if is_final {
                let restore = match *board_size.borrow() {
                    BoardSize::Auto => "Auto".to_string(),
                    BoardSize::Fixed(_, h) => h.to_string(),
                };
                height_clone.set_text(restore);
                return EventResponses::default();
            }
            let h = parse_dim(&text);
            let w = parse_dim(&width_tb_clone.tb.get_text());
            let bs = match (w, h) {
                (Some(w), Some(h)) => BoardSize::Fixed(w, h),
                _ => BoardSize::Auto,
            };
            *board_size.borrow_mut() = bs;
            Config::save_values(
                tick_interval.borrow().as_millis() as u64,
                &board_size_to_str(&board_size.borrow()),
                *high_score.borrow(),
                *num_foods.borrow(),
                *no_walls.borrow(),
                *emoji_double_width.borrow(),
            );
            EventResponses::default()
        }));
    }

    pane.add_element(Box::new(Label::new(ctx, "W:").at(59, 0)));
    pane.add_element(Box::new(width_tb.at(62, 0)));
    pane.add_element(Box::new(Label::new(ctx, "H:").at(68, 0)));
    pane.add_element(Box::new(height_tb.at(71, 0)));

    // Restart button
    let restart_btn = Button::new(ctx, "Restart").with_fn(Box::new(move |_btn, _ctx| {
        let fn_ = restart_fn.borrow();
        fn_();
        drop(fn_);
        EventResponses::default()
    }));
    pane.add_element(Box::new(restart_btn.at(92, 0)));

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
    let high_score = state.high_score.clone();
    let no_walls = state.no_walls.clone();
    let emoji_double_width = state.emoji_double_width.clone();
    // Clones for checkbox callback (must be before food_slider closure moves originals)
    let tick_interval_cb = tick_interval.clone();
    let board_size_cb = board_size.clone();
    let high_score_cb = high_score.clone();
    let num_foods_cb = num_foods.clone();
    let no_walls_cb_ref = no_walls.clone();
    let emoji_double_width_cb = emoji_double_width.clone();
    *food_slider.adjust_fn.borrow_mut() = Box::new(move |_ctx, s| {
        let pos = *s.position.borrow();
        // Map 0.0..=1.0 → 1..=100
        let n = (pos * 99.0) as usize + 1;
        *num_foods.borrow_mut() = n;
        Config::save_values(
            tick_interval.borrow().as_millis() as u64,
            &board_size_to_str(&board_size.borrow()),
            *high_score.borrow(),
            n,
            *no_walls.borrow(),
            *emoji_double_width.borrow(),
        );
        EventResponses::default()
    });
    pane.add_element(Box::new(food_slider.at(7, 1)));

    // No walls checkbox
    pane.add_element(Box::new(Label::new(ctx, "No Walls: ").at(60, 1)));

    let no_walls_cb = Checkbox::new(ctx);
    *no_walls_cb.checked.borrow_mut() = *state.no_walls.borrow();
    no_walls_cb.set_fn(Box::new(move |_ctx, checked| {
        *no_walls_cb_ref.borrow_mut() = checked;
        Config::save_values(
            tick_interval_cb.borrow().as_millis() as u64,
            &board_size_to_str(&board_size_cb.borrow()),
            *high_score_cb.borrow(),
            *num_foods_cb.borrow(),
            checked,
            *emoji_double_width_cb.borrow(),
        );
        EventResponses::default()
    }));
    pane.add_element(Box::new(no_walls_cb.at(70, 1)));

    // --- Row 2: Score and Best ---
    pane.add_element(Box::new(Label::new(ctx, "Score:").at(0, 2)));

    let score_tb = SingleLineTextBox::new(ctx);
    score_tb.set_text((*state.score.borrow()).to_string());
    {
        let mut loc = score_tb.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(6));
        score_tb.set_dyn_location_set(loc);
    }
    state.score_display = Some(score_tb.clone());
    pane.add_element(Box::new(score_tb.at(8, 2)));

    pane.add_element(Box::new(Label::new(ctx, "Best:").at(20, 2)));

    let best_tb = SingleLineTextBox::new(ctx);
    best_tb.set_text((*state.high_score.borrow()).to_string());
    {
        let mut loc = best_tb.get_dyn_location_set().clone();
        loc.set_dyn_width(DynVal::new_fixed(6));
        best_tb.set_dyn_location_set(loc);
    }
    state.best_display = Some(best_tb.clone());
    pane.add_element(Box::new(best_tb.at(26, 2)));

    Box::new(pane)
}

fn board_size_to_str(bs: &BoardSize) -> String {
    match bs {
        BoardSize::Auto => "Auto".to_string(),
        BoardSize::Fixed(w, h) => format!("{}x{}", w, h),
    }
}

fn parse_dim(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("auto") {
        None
    } else {
        s.parse().ok()
    }
}
