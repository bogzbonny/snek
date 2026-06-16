use std::time::Duration;

use tokio::time::interval;
use yeehaw::{ParentPane, Tui};

mod game;
use game::SnakeGame;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let (mut tui, ctx) = Tui::new()?;

        let game = SnakeGame::new(&ctx, 40, 20);
        let root = ParentPane::new(&ctx, "root");
        root.add_element(Box::new(game.clone()));

        let tick_game = game.clone();
        let tick_ctx = ctx.clone();
        let mut iv = interval(Duration::from_millis(150));

        let ls = tokio::task::LocalSet::new();
        ls.spawn_local(async move {
            loop {
                iv.tick().await;
                tick_game.tick(&tick_ctx);
            }
        });

        ls.block_on(&rt, async { tui.run(Box::new(root)).await })?;
        Ok(())
    })
}
