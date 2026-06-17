use std::cell::RefCell;
use std::rc::Rc;

use crossterm::style::Color as AnsiColor;
use tokio::time::interval;
use yeehaw::elements::containers::border::{BorderProperies, BorderSty};
use yeehaw::{
    Attributes, Bordered, Color, DynVal, Element, FgTranspSrc, ParentPane, Style, Tui,
    VerticalStack,
};

pub mod controls;
pub mod game;
pub mod config;

#[cfg(test)]
mod tests;

use controls::{build_control_bar, ControlState};
use game::SnekGame;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut tui, ctx) = Tui::new()?;

    let mut state = ControlState::new(&ctx);
    let tick_interval = state.tick_interval.clone();

    let game = SnekGame::new(&ctx, &state);
    game.set_focused(true);
    let tick_game = game.clone();
    let restart_game = game.clone();

    let restart_fn = Rc::new(RefCell::new(move || restart_game.restart()));
    let control_bar = build_control_bar(&ctx, &mut state, restart_fn);

    let stack = VerticalStack::new(&ctx);
    {
        let mut loc = game.get_dyn_location_set().clone();
        loc.set_dyn_height(DynVal::new_flex(1.0));
        game.set_dyn_location_set(loc);
    }
    let border_style = Style {
        fg: Some((Color::ANSI(AnsiColor::White), FgTranspSrc::LowerBg)),
        bg: None,
        underline_color: None,
        attr: Attributes::new(),
    };
    let bordered = Bordered::new(
        &ctx,
        Box::new(game),
        BorderSty::new_thin_single(border_style),
        BorderProperies::new_basic(),
    );
    stack.push(Box::new(bordered));

    {
        let mut loc = control_bar.get_dyn_location_set().clone();
        loc.set_dyn_height(DynVal::new_fixed(2));
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

    ls.run_until(tui.run(Box::new(root)))
        .await
        .map_err(Into::into)
}
