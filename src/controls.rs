#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use yeehaw::{
    Button, Context, DrawRegion, DrawUpdate, DropdownList, Element, ElementID, Event,
    EventResponse, EventResponses, HorizontalStackFocuser, Label, Parent, ReceivableEvents,
    Ref, Slider,
};

use crate::game::{BoardSize, GameState, Theme};

/// Wrapper that holds an Rc<Label> and delegates all Element methods to the inner Label.
/// This allows the same Label instance to be shared between the visual tree and the game,
/// enabling the game to call `set_text()` on the actual Label widget in the visual tree.
struct RcLabel(Rc<Label>);

impl Clone for RcLabel {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Element for RcLabel {
    fn kind(&self) -> &'static str {
        self.0.kind()
    }
    fn id(&self) -> ElementID {
        self.0.id()
    }
    fn can_receive(&self, ev: &Event) -> bool {
        self.0.can_receive(ev)
    }
    fn receivable(&self) -> Vec<Rc<RefCell<ReceivableEvents>>> {
        self.0.receivable()
    }
    fn receive_event(&self, ctx: &Context, ev: Event) -> (bool, EventResponses) {
        self.0.receive_event(ctx, ev)
    }
    fn set_focused(&self, focused: bool) {
        self.0.set_focused(focused)
    }
    fn get_focused(&self) -> bool {
        self.0.get_focused()
    }
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        self.0.drawing(ctx, dr, force_update)
    }
    fn get_attribute(&self, key: &str) -> Option<Vec<u8>> {
        self.0.get_attribute(key)
    }
    fn set_attribute_inner(&self, key: &str, value: Vec<u8>) {
        self.0.set_attribute_inner(key, value)
    }
    fn set_hook(&self, kind: &str, el_id: ElementID, hook: Box<dyn FnMut(&str, Box<dyn Element>)>) {
        self.0.set_hook(kind, el_id, hook)
    }
    fn remove_hook(&self, kind: &str, el_id: ElementID) {
        self.0.remove_hook(kind, el_id)
    }
    fn clear_hooks_by_id(&self, el_id: ElementID) {
        self.0.clear_hooks_by_id(el_id)
    }
    fn call_hooks_of_kind(&self, kind: &str) {
        self.0.call_hooks_of_kind(kind)
    }
    fn set_parent(&self, parent: Box<dyn Parent>) {
        self.0.set_parent(parent)
    }
    fn get_dyn_location_set(&self) -> Ref<'_, yeehaw::DynLocationSet> {
        self.0.get_dyn_location_set()
    }
    fn get_visible(&self) -> bool {
        self.0.get_visible()
    }
    fn get_ref_cell_dyn_location_set(&self) -> Rc<RefCell<yeehaw::DynLocationSet>> {
        self.0.get_ref_cell_dyn_location_set()
    }
    fn get_ref_cell_visible(&self) -> Rc<RefCell<bool>> {
        self.0.get_ref_cell_visible()
    }
    fn get_ref_cell_overflow(&self) -> Rc<RefCell<bool>> {
        self.0.get_ref_cell_overflow()
    }
    fn set_content_x_offset(&self, dr: Option<&DrawRegion>, x: usize) {
        self.0.set_content_x_offset(dr, x)
    }
    fn set_content_y_offset(&self, dr: Option<&DrawRegion>, y: usize) {
        self.0.set_content_y_offset(dr, y)
    }
    fn get_content_x_offset(&self) -> usize {
        self.0.get_content_x_offset()
    }
    fn get_content_y_offset(&self) -> usize {
        self.0.get_content_y_offset()
    }
    fn get_content_width(&self, dr: Option<&DrawRegion>) -> usize {
        self.0.get_content_width(dr)
    }
    fn get_content_height(&self, dr: Option<&DrawRegion>) -> usize {
        self.0.get_content_height(dr)
    }
}

/// Shared state between controls bar and SnakeGame for bidirectional communication.
pub struct ControlState {
    pub tick_interval: Rc<RefCell<Duration>>,
    pub board_size: Rc<RefCell<BoardSize>>,
    pub theme: Rc<RefCell<Theme>>,
    pub score: Rc<RefCell<usize>>,
    pub high_score: Rc<RefCell<usize>>,
    pub state: Rc<RefCell<GameState>>,
    /// Label widgets shared with the game for live text updates via set_text().
    pub score_label: Rc<Label>,
    pub high_score_label: Rc<Label>,
    pub status_label: Rc<Label>,
}

impl ControlState {
    pub fn new(ctx: &Context) -> Self {
        Self {
            tick_interval: Rc::new(RefCell::new(Duration::from_millis(150))),
            board_size: Rc::new(RefCell::new(BoardSize::Auto)),
            theme: Rc::new(RefCell::new(Theme::Classic)),
            score: Rc::new(RefCell::new(0)),
            high_score: Rc::new(RefCell::new(0)),
            state: Rc::new(RefCell::new(GameState::Paused)),
            score_label: Rc::new(Label::new(ctx, "Score: 0")),
            high_score_label: Rc::new(Label::new(ctx, "Best: 0")),
            status_label: Rc::new(Label::new(ctx, "Paused")),
        }
    }
}

/// Build the bottom control bar as a HorizontalStackFocuser containing all widgets.
///
/// `restart_fn` is called by the Restart button to reset the game.
pub fn build_control_bar(
    ctx: &Context,
    state: &ControlState,
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
    let diff_tick_interval = state.tick_interval.clone();
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
            // Also update tick_interval directly; setting slider_pos alone does not trigger adjust_fn.
            let ms = (500.0 - pos * 450.0) as u64;
            *diff_tick_interval.borrow_mut() = Duration::from_millis(ms);
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

    // --- Score label (shared Rc<Label> via RcLabel wrapper) ---
    stack.push(Box::new(RcLabel(state.score_label.clone())));

    // --- High score label ---
    stack.push(Box::new(RcLabel(state.high_score_label.clone())));

    // --- Status label ---
    stack.push(Box::new(RcLabel(state.status_label.clone())));

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
