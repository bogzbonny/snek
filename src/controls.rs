#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use yeehaw::{
    Button, Context, DropdownList, Element, EventResponse, EventResponses,
    HorizontalStackFocuser, Label, Slider,
};

use crate::game::{BoardSize, GameState, Theme};

/// Shared state between controls bar and SnakeGame for bidirectional communication.
pub struct ControlState {
    pub tick_interval: Rc<RefCell<Duration>>,
    pub board_size: Rc<RefCell<BoardSize>>,
    pub theme: Rc<RefCell<Theme>>,
    pub score: Rc<RefCell<usize>>,
    pub high_score: Rc<RefCell<usize>>,
    pub state: Rc<RefCell<GameState>>,
    /// Label text refs the game writes into to update the display.
    pub score_label: Rc<RefCell<String>>,
    pub high_score_label: Rc<RefCell<String>>,
    pub status_label: Rc<RefCell<String>>,
}

impl ControlState {
    pub fn new() -> Self {
        Self {
            tick_interval: Rc::new(RefCell::new(Duration::from_millis(150))),
            board_size: Rc::new(RefCell::new(BoardSize::Auto)),
            theme: Rc::new(RefCell::new(Theme::Classic)),
            score: Rc::new(RefCell::new(0)),
            high_score: Rc::new(RefCell::new(0)),
            state: Rc::new(RefCell::new(GameState::Running)),
            score_label: Rc::new(RefCell::new("Score: 0".into())),
            high_score_label: Rc::new(RefCell::new("Best: 0".into())),
            status_label: Rc::new(RefCell::new("Running".into())),
        }
    }
}

/// Build the bottom control bar as a HorizontalStackFocuser containing all widgets.
///
/// `restart_fn` is called by the Restart button to reset the game.
pub fn build_control_bar(
    ctx: &Context,
    state: ControlState,
    restart_fn: Rc<RefCell<dyn Fn()>>,
) -> Box<dyn Element> {
    let stack = HorizontalStackFocuser::new(ctx);

    // --- Speed label + slider ---
    stack.push(Box::new(Label::new(ctx, "Speed")));

    let slider = Slider::new_basic_line(ctx);
    let slider_pos = slider.position.clone();
    let tick_interval = state.tick_interval.clone();
    *slider.adjust_fn.borrow_mut() = Box::new(move |_ctx, s| {
        let pos = *s.position.borrow();
        // Map 0.0..=1.0 → 500ms..=50ms
        let ms = (500.0 - pos * 450.0) as u64;
        *tick_interval.borrow_mut() = Duration::from_millis(ms);
        EventResponses::default()
    });
    stack.push(Box::new(slider));

    // --- Difficulty dropdown ---
    let diff_dropdown = DropdownList::new(
        ctx,
        vec!["Slow", "Medium", "Fast", "Insane"],
        Box::new(move |_ctx, selected| {
            let pos = match selected.as_str() {
                "Slow" => 0.0,
                "Medium" => 0.5,
                "Fast" => 0.75,
                "Insane" => 1.0,
                _ => 0.0,
            };
            *slider_pos.borrow_mut() = pos;
            EventResponses::default()
        }),
    );
    stack.push(Box::new(diff_dropdown));

    // --- Board size dropdown ---
    let board_size = state.board_size.clone();
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
            EventResponses::default()
        }),
    );
    stack.push(Box::new(size_dropdown));

    // --- Theme dropdown ---
    let theme = state.theme.clone();
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
            EventResponses::default()
        }),
    );
    stack.push(Box::new(theme_dropdown));

    // --- Score label ---
    let score_label = Label::new(ctx, &state.score_label.borrow());
    {
        let text = score_label.text.clone();
        *state.score_label.borrow_mut() = text.borrow().clone();
    }
    stack.push(Box::new(score_label));

    // --- High score label ---
    let high_score_label = Label::new(ctx, &state.high_score_label.borrow());
    {
        let text = high_score_label.text.clone();
        *state.high_score_label.borrow_mut() = text.borrow().clone();
    }
    stack.push(Box::new(high_score_label));

    // --- Status label ---
    let status_label = Label::new(ctx, &state.status_label.borrow());
    {
        let text = status_label.text.clone();
        *state.status_label.borrow_mut() = text.borrow().clone();
    }
    stack.push(Box::new(status_label));

    // --- Restart button ---
    let restart_btn = Button::new(ctx, "Restart").with_fn(Box::new(move |_btn, _ctx| {
        let fn_ = restart_fn.borrow();
        fn_();
        drop(fn_);
        EventResponses::default()
    }));
    stack.push(Box::new(restart_btn));

    // --- Quit button ---
    let quit_btn = Button::new(ctx, "Quit").with_fn(Box::new(move |_btn, _ctx| {
        EventResponses::from(EventResponse::Quit)
    }));
    stack.push(Box::new(quit_btn));

    Box::new(stack)
}
