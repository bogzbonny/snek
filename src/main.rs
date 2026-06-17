use std::cell::RefCell;
use std::rc::Rc;

use tokio::time::interval;
use yeehaw::{DynVal, Element, ParentPane, Tui, VerticalStack};

mod controls;
mod game;

use controls::{build_control_bar, ControlState};
use game::SnakeGame;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut tui, ctx) = Tui::new()?;

    let state = ControlState::new(&ctx);
    let tick_interval = state.tick_interval.clone();

    let game = SnakeGame::new(&ctx, &state);
    game.set_focused(true);
    let tick_game = game.clone();
    let restart_game = game.clone();

    let restart_fn = Rc::new(RefCell::new(move || restart_game.restart()));
    let control_bar = build_control_bar(&ctx, &state, restart_fn);

    // Layout: VerticalStack — game (~80%) then control bar (~20%)
    let stack = VerticalStack::new(&ctx);

    {
        let mut loc = game.get_dyn_location_set().clone();
        loc.set_dyn_height(DynVal::new_flex(0.8));
        game.set_dyn_location_set(loc);
    }
    stack.push(Box::new(game));

    {
        let mut loc = control_bar.get_dyn_location_set().clone();
        loc.set_dyn_height(DynVal::new_flex(0.2));
        control_bar.set_dyn_location_set(loc);
    }
    stack.push(control_bar);

    // Focus containers so can_receive() gates pass during Tui event dispatch.
    // ParentPane::can_receive() requires self.get_focused() == true before
    // checking children, and VerticalStack delegates to its inner ParentPane.
    stack.set_focused(true);

    let root = ParentPane::new(&ctx, "root");
    root.add_element(Box::new(stack));
    root.set_focused(true);

    // Tick loop — interval resets when the shared duration changes
    let tick_ctx = ctx.clone();
    let ls = tokio::task::LocalSet::new();
    ls.spawn_local(async move {
        let mut iv = interval(*tick_interval.borrow());
        loop {
            let new_dur = *tick_interval.borrow();
            if new_dur != iv.period() {
                iv = interval(new_dur);
            }
            iv.tick().await;
            tick_game.tick(&tick_ctx);
        }
    });

    ls.run_until(tui.run(Box::new(root))).await.map_err(Into::into)
}
