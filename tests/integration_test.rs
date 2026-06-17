#![allow(unused_must_use)]

#[path = "../src/controls.rs"]
mod controls;
#[path = "../src/game.rs"]
mod game;
#[path = "../src/config.rs"]
mod config;

use yeehaw::{
    DrawRegion, Element, Event, Keyboard, ParentPane, Size, Tui,
    VerticalStack,
};

use controls::ControlState;
use game::{BoardSize, Direction, GameState, SnakeGame};

fn make_game() -> (SnakeGame, ControlState, yeehaw::Context) {
    let (_tui, ctx) = Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    let game = SnakeGame::new(&ctx, &ctrl);
    (game, ctrl, ctx)
}

/// Create a game with an initialized Fixed board (20x10), paused and ready for input.
/// Snake head starts at (10, 5), direction Right.
fn make_initialized_game() -> (SnakeGame, ControlState, yeehaw::Context) {
    let (_tui, ctx) = Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);
    let game = SnakeGame::new(&ctx, &ctrl);
    game.restart(); // initializes board: head=(10,5), body=(9,5), tail=(8,5), dir=Right
    *ctrl.state.borrow_mut() = GameState::Paused;
    (game, ctrl, ctx)
}

/// Verify that when multiple apples are spawned, no two occupy the same cell.
#[test]
fn test_apples_never_overlap() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(40, 20);
    *ctrl.num_apples.borrow_mut() = 50;

    for _i in 0..100 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let apples = game.apples();
        let unique: std::collections::HashSet<_> = apples.iter().collect();
        assert_eq!(
            unique.len(),
            apples.len(),
            "apples must not overlap: {} total vs {} unique",
            apples.len(),
            unique.len()
        );
    }
}

#[test]
fn test_initial_state_is_paused() {
    let (game, _, _) = make_game();
    assert_eq!(game.state(), GameState::Paused, "game must start Paused");
}

#[test]
fn test_h_key_starts_game_and_sets_left() {
    // Default direction is Right. Pressing Left (opposite) from Paused should
    // start the game but keep direction at Right (opposite-direction guard).
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Right, "opposite direction should be ignored on start");
}

#[test]
fn test_j_key_starts_game_and_sets_down() {
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_J]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Down);
}

#[test]
fn test_k_key_starts_game_and_sets_up() {
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Up);
}

#[test]
fn test_l_key_starts_game_and_sets_right() {
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Right);
}

#[test]
fn test_arrow_up_starts_game_and_sets_up() {
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_UP]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Up);
}

#[test]
fn test_arrow_down_starts_game_and_sets_down() {
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_DOWN]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Down);
}

#[test]
fn test_arrow_left_starts_game_and_sets_left() {
    // Default direction is Right. Pressing Left (opposite) from Paused should
    // start the game but keep direction at Right (opposite-direction guard).
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_LEFT]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Right, "opposite direction should be ignored on start");
}

#[test]
fn test_arrow_right_starts_game_and_sets_right() {
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_RIGHT]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Right);
}

#[test]
fn test_direction_change_while_running() {
    // Direction changes are queued; direction() reflects the current direction,
    // not the queued one. The queued change is applied on the next tick.
    let (game, _, ctx) = make_initialized_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.direction(), Direction::Up);
    // Press Left while Running — queued, not applied yet.
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.direction(), Direction::Up, "queued change should not affect direction() yet");
    // Tick applies the queued Left.
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Left);
}

#[test]
fn test_cannot_reverse_direction() {
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    assert_eq!(game.direction(), Direction::Right);
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.direction(), Direction::Right, "cannot reverse into self");
}

// --- Immediate snake movement on first key press ---

#[test]
fn test_k_key_moves_snake_up_immediately() {
    let (game, _, ctx) = make_initialized_game();
    let initial_head = game.snake()[0];
    assert_eq!(initial_head, (10, 5), "head should start at center");

    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Up);

    let new_head = game.snake()[0];
    assert_eq!(new_head, (10, 4), "head should move up immediately on first key press");
}

#[test]
fn test_j_key_moves_snake_down_immediately() {
    let (game, _, ctx) = make_initialized_game();
    let initial_head = game.snake()[0];
    assert_eq!(initial_head, (10, 5));

    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_J]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Down);

    let new_head = game.snake()[0];
    assert_eq!(new_head, (10, 6), "head should move down immediately");
}

#[test]
fn test_l_key_moves_snake_right_immediately() {
    let (game, _, ctx) = make_initialized_game();
    let initial_head = game.snake()[0];
    assert_eq!(initial_head, (10, 5));

    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Right);

    let new_head = game.snake()[0];
    assert_eq!(new_head, (11, 5), "head should move right immediately");
}

#[test]
fn test_arrow_up_moves_snake_up_immediately() {
    let (game, _, ctx) = make_initialized_game();
    let initial_head = game.snake()[0];
    assert_eq!(initial_head, (10, 5));

    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_UP]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Up);

    let new_head = game.snake()[0];
    assert_eq!(new_head, (10, 4), "head should move up immediately");
}

#[test]
fn test_arrow_down_moves_snake_down_immediately() {
    let (game, _, ctx) = make_initialized_game();
    let initial_head = game.snake()[0];
    assert_eq!(initial_head, (10, 5));

    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_DOWN]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Down);

    let new_head = game.snake()[0];
    assert_eq!(new_head, (10, 6), "head should move down immediately");
}

#[test]
fn test_arrow_right_moves_snake_right_immediately() {
    let (game, _, ctx) = make_initialized_game();
    let initial_head = game.snake()[0];
    assert_eq!(initial_head, (10, 5));

    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_RIGHT]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Right);

    let new_head = game.snake()[0];
    assert_eq!(new_head, (11, 5), "head should move right immediately");
}

#[test]
fn test_space_does_not_move_snake() {
    let (game, _, ctx) = make_initialized_game();
    let initial_snake = game.snake();

    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_SPACE]));
    assert_eq!(game.state(), GameState::Running);

    let new_snake = game.snake();
    assert_eq!(initial_snake, new_snake, "SPACE should not move the snake");
}

#[test]
fn test_h_key_from_paused_causes_game_over() {
    // Snake is horizontal: head=(10,5), body=(9,5), tail=(8,5), dir=Right.
    // Pressing Left from Paused is the opposite direction — the game starts
    // but keeps direction=Right (opposite-direction guard), so no collision.
    let (game, _, ctx) = make_initialized_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.state(), GameState::Running, "pressing opposite direction should start the game");
    assert_eq!(game.direction(), Direction::Right, "direction should stay Right");
}

// ============================================================================
// Focus gating tests — verify can_receive() respects focus (yeehaw Pane integration)
// ============================================================================

#[test]
fn test_can_receive_when_focused() {
    let (game, _, _) = make_initialized_game();
    game.set_focused(true);

    for &key in &[
        Keyboard::KEY_H,
        Keyboard::KEY_J,
        Keyboard::KEY_K,
        Keyboard::KEY_L,
        Keyboard::KEY_LEFT,
        Keyboard::KEY_RIGHT,
        Keyboard::KEY_UP,
        Keyboard::KEY_DOWN,
        Keyboard::KEY_SPACE,
        Keyboard::KEY_Q,
    ] {
        assert!(
            game.can_receive(&Event::KeyCombo(vec![key])),
            "focused game must accept {key:?}",
        );
    }
}

#[test]
fn test_can_receive_rejects_unregistered_keys() {
    let (game, _, _) = make_initialized_game();
    game.set_focused(true);

    let a_key = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('a'),
        crossterm::event::KeyModifiers::NONE,
    );
    assert!(
        !game.can_receive(&Event::KeyCombo(vec![a_key])),
        "game must reject unregistered key 'a'",
    );
}

#[test]
fn test_can_receive_blocks_when_unfocused() {
    // Before the fix: can_receive ignored focus and always matched.
    // After the fix: can_receive delegates to Pane which gates on focus.
    let (game, _, _) = make_initialized_game();
    game.set_focused(false);

    for &key in &[
        Keyboard::KEY_H,
        Keyboard::KEY_J,
        Keyboard::KEY_K,
        Keyboard::KEY_L,
        Keyboard::KEY_LEFT,
        Keyboard::KEY_RIGHT,
        Keyboard::KEY_UP,
        Keyboard::KEY_DOWN,
        Keyboard::KEY_SPACE,
    ] {
        assert!(
            !game.can_receive(&Event::KeyCombo(vec![key])),
            "unfocused game must reject {key:?}",
        );
    }
}

#[test]
fn test_receivable_exposes_events_when_focused() {
    let (game, _, _) = make_initialized_game();
    game.set_focused(true);

    let rec = game.receivable();
    assert!(!rec.is_empty(), "focused game must expose receivable events");

    // At least one receivable bucket must match KEY_K
    let key_ev = Event::KeyCombo(vec![Keyboard::KEY_K]);
    let any_match = rec.iter().any(|bucket| {
        let b = bucket.borrow();
        b.contains_match(&key_ev)
    });
    assert!(any_match, "focused game's receivable must contain KEY_K");
}

#[test]
fn test_receivable_empty_when_unfocused() {
    // When unfocused, Pane::receivable() returns only "always" bucket.
    // SnakeGame registers all events as "focused" — so unfocused should be empty.
    let (game, _, _) = make_initialized_game();
    game.set_focused(false);

    let rec = game.receivable();
    // The "always" bucket is empty (we only set focused events)
    let key_ev = Event::KeyCombo(vec![Keyboard::KEY_K]);
    let any_match = rec.iter().any(|bucket| {
        let b = bucket.borrow();
        b.contains_match(&key_ev)
    });
    assert!(
        !any_match,
        "unfocused game's receivable must NOT contain direction keys",
    );
}

// ============================================================================
// Full tick cycle test — direction change + tick moves the snake
// ============================================================================

#[test]
fn test_direction_change_then_tick_moves_snake() {
    // Start from Paused, press K to start + set Up, then tick again to verify
    // the snake continues moving in the new direction.
    let (game, _, ctx) = make_initialized_game();
    // State is Paused, board is initialized.
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    // Now Running, direction=Up, snake moved once (head at 10,4).
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Up);
    let after_first_move = game.snake().clone();
    assert_eq!(after_first_move[0], (10, 4), "first move should be Up");

    // Tick again — snake should move Up again.
    game.tick(&ctx);
    let after_second_move = game.snake().clone();
    assert_eq!(
        after_second_move[0],
        (10, 3),
        "second tick should move Up again"
    );
}

#[test]
fn test_direction_change_while_running_then_tick() {
    // Start game, change direction, tick — verify the position reflects new direction.
    let (game, _, ctx) = make_initialized_game();
    // Start moving Up.
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.direction(), Direction::Up);
    assert_eq!(game.snake()[0], (10, 4));

    // Change to Left while Running — queued, not applied yet.
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.direction(), Direction::Up, "queued change should not affect direction() yet");
    // Snake hasn't moved yet.
    assert_eq!(game.snake()[0], (10, 4), "direction change alone should not move");

    // Tick applies the queued Left and moves.
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Left);
    assert_eq!(
        game.snake()[0],
        (9, 4),
        "tick after direction change should move in new direction"
    );
}

// ============================================================================
// Container hierarchy dispatch tests — verify events reach SnakeGame through
// focused ParentPane → VerticalStack → SnakeGame (the fix for the original bug
// where unfocused containers blocked can_receive() gates).
//
// Key detail: Pane::clone() creates a NEW focused: RefCell<bool>. The clone
// inherits the focus VALUE at clone time, not a shared reference. So the game
// must be focused BEFORE cloning for the clone in the stack to be focused.
// ============================================================================

/// Build hierarchy with explicit focus control.
/// focus_stack: whether to focus the VerticalStack before adding to root.
/// focus_game: whether to focus the game before cloning into the stack.
fn make_hierarchy(focus_stack: bool, focus_game: bool) -> (SnakeGame, ParentPane, yeehaw::Context) {
    use yeehaw::DynVal;
    use std::cell::RefCell;
    use std::rc::Rc;

    let (_tui, ctx) = Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);
    let game = SnakeGame::new(&ctx, &ctrl);
    game.restart();
    *ctrl.state.borrow_mut() = GameState::Paused;

    // Focus game BEFORE cloning so the clone inherits the correct focus state.
    // Pane::clone() copies the RefCell value, not the reference.
    // Must explicitly set focus to false when focus_game is false, because
    // SnakeGame::new sets focused=true by default.
    game.set_focused(focus_game);

    // VerticalStack with game only (no control bar to avoid widget key conflicts)
    let mut stack = VerticalStack::new(&ctx);
    {
        let mut loc = game.get_dyn_location_set().clone();
        loc.set_dyn_height(DynVal::new_flex(1.0));
        game.set_dyn_location_set(loc);
    }
    stack.push(Box::new(game.clone()));

    // Focus stack before adding to root. Must explicitly set focus to false
    // when focus_stack is false, because ParentPane::new sets focused=true.
    stack.set_focused(focus_stack);

    let root = ParentPane::new(&ctx, "root");
    root.add_element(Box::new(stack));

    (game, root, ctx)
}

#[test]
fn test_hierarchy_dispatch_all_focused_reaches_game() {
    let (game, root, ctx) = make_hierarchy(true, true);

    // Dispatch KEY_K through root — must reach the game
    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));

    assert_eq!(game.state(), GameState::Running, "game must start via hierarchy dispatch");
    assert_eq!(game.direction(), Direction::Up, "direction must be Up via hierarchy dispatch");
}

#[test]
fn test_hierarchy_dispatch_all_direction_keys() {
    let (game, root, ctx) = make_hierarchy(true, true);
    // Initial direction is Right (from restart). Start with Up to avoid
    // the opposite-direction rejection on the first key.
    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.direction(), Direction::Up);

    // Direction changes are queued; tick applies one per frame.
    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Left);

    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_J]));
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Down);

    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Right);
}

#[test]
fn test_hierarchy_dispatch_arrow_keys() {
    let (game, root, ctx) = make_hierarchy(true, true);
    // Initial direction is Right. Start with Up to avoid opposite rejection.
    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_UP]));
    assert_eq!(game.direction(), Direction::Up);

    // Direction changes are queued; tick applies one per frame.
    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_LEFT]));
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Left);

    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_DOWN]));
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Down);

    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_RIGHT]));
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Right);
}

#[test]
fn test_hierarchy_dispatch_blocks_when_stack_unfocused() {
    // VerticalStack::can_receive() → ParentPane::can_receive() has a hard
    // focus gate. When the stack is unfocused, events cannot reach children.
    let (game, root, ctx) = make_hierarchy(false, true);

    let initial_state = game.state();
    let initial_dir = game.direction();
    let captured = root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));

    assert_eq!(
        game.state(),
        initial_state,
        "unfocused stack must block event dispatch to children"
    );
    assert_eq!(game.direction(), initial_dir, "direction should not change");
}

#[test]
fn test_hierarchy_dispatch_blocks_when_game_unfocused() {
    // Even with focused stack, the target element must be focused.
    let (game, root, ctx) = make_hierarchy(true, false);

    let initial_state = game.state();
    root.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));

    assert_eq!(
        game.state(),
        initial_state,
        "unfocused game must not receive events even when stack is focused"
    );
}

// ============================================================================
// Tick logic tests — verify clone-shared state (Rc<RefCell>) correctness
// ============================================================================

/// Simulates the main.rs pattern: drawing() on original, tick() on clone.
/// Before the fix, board_initialized was NOT shared, so tick() on the clone
/// always returned early. Now clone.tick() moves the shared snake.
#[test]
fn test_tick_on_clone_after_drawing_on_original() {
    let (game, ctrl, ctx) = make_initialized_game();
    let clone = game.clone();

    // Set running and tick the clone
    *ctrl.state.borrow_mut() = GameState::Running;
    clone.tick(&ctx);

    // Snake must have moved (original direction is Right)
    let snake = game.snake();
    assert_eq!(snake[0], (11, 5), "snake head must move right after tick on clone");
}

/// Verify apple position is shared between original and clone.
#[test]
fn test_apple_shared_between_clones() {
    let (game, _, _) = make_initialized_game();
    let apple_orig = game.apple();
    let clone = game.clone();
    let apple_clone = clone.apple();

    assert_eq!(
        apple_orig, apple_clone,
        "apple position must be identical in original and clone"
    );
}

/// Multi-tick test: snake moves right correctly.
/// receive_event(KEY_L) starts the game moving Right and ticks once.
#[test]
fn test_multi_tick_movement_right() {
    let (game, _, ctx) = make_initialized_game();
    // receive_event starts game + ticks once: head (10,5) → (11,5)
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    assert_eq!(game.snake()[0], (11, 5));

    // 4 more ticks right
    for i in 2..=5 {
        game.tick(&ctx);
        assert_eq!(
            game.snake()[0],
            (10 + i, 5),
            "after {} ticks right, head should be at ({}, 5)",
            i,
            10 + i
        );
    }
}

/// Multi-tick test: snake moves up correctly.
#[test]
fn test_multi_tick_movement_up() {
    let (game, _, ctx) = make_initialized_game();
    // receive_event(KEY_K) starts game moving Up + ticks once: head (10,5) → (10,4)
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.snake()[0], (10, 4));

    for i in 2..=4 {
        game.tick(&ctx);
        assert_eq!(
            game.snake()[0],
            (10, 5 - i),
            "after {} ticks up, head should be at (10, {})",
            i,
            5 - i
        );
    }
}

/// Multi-tick test: snake moves down correctly.
#[test]
fn test_multi_tick_movement_down() {
    let (game, _, ctx) = make_initialized_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_J]));
    assert_eq!(game.snake()[0], (10, 6));

    for i in 2..=4 {
        game.tick(&ctx);
        assert_eq!(
            game.snake()[0],
            (10, 5 + i),
            "after {} ticks down, head should be at (10, {})",
            i,
            5 + i
        );
    }
}

/// Multi-tick test: snake moves left correctly.
/// Start by pressing K (Up) to enter Running, then press H (Left) to change direction.
/// (Cannot start Right then go Left — handle_direction blocks opposite.)
#[test]
fn test_multi_tick_movement_left() {
    let (game, _, ctx) = make_initialized_game();
    // Start Running by pressing Up: head (10,5) → (10,4)
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.snake()[0], (10, 4));
    // Change to Left (Left is NOT opposite of Up — allowed): queued, not applied yet.
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.direction(), Direction::Up, "queued change should not affect direction() yet");
    // Tick applies queued Left: head (10,4) → (9,4)
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Left);
    assert_eq!(game.snake()[0], (9, 4));
    // Continue left: (8,4), (7,4), (6,4)
    for i in 2..=4 {
        game.tick(&ctx);
        assert_eq!(
            game.snake()[0],
            (10 - i, 4),
            "after {} total ticks left, head should be at ({}, 4)",
            i,
            10 - i
        );
    }
}

/// Snake length must stay constant when not eating.
/// Use Up direction so the snake moves along x=10; the apple is randomly
/// placed and unlikely to be on that exact column within 9 ticks.
/// We verify by checking the apple was not consumed.
#[test]
fn test_snake_length_stays_constant_without_eating() {
    let (game, _, ctx) = make_initialized_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K])); // Up
    let initial_len = game.snake().len();
    let apple = game.apple();

    for _ in 0..9 {
        game.tick(&ctx);
    }

    // Snake moved Up 9 steps: head went from (10,5) to (10,4)..(10,0) then wraps/dies
    // The apple should not have been eaten unless it was on the path
    let ate = game.apple() != apple;
    if ate {
        // Apple was eaten; verify it respawned (the core bug check)
        assert_ne!(game.apple(), apple, "apple must respawn at new position after being eaten");
        assert_eq!(game.snake().len(), initial_len + 1, "snake must grow by 1 after eating");
    } else {
        assert_eq!(
            game.snake().len(),
            initial_len,
            "snake length must not change without eating"
        );
    }
}

/// Tick must not move snake when state is Paused.
#[test]
fn test_tick_noop_when_paused() {
    let (game, ctrl, ctx) = make_initialized_game();
    *ctrl.state.borrow_mut() = GameState::Paused;
    let snake_before = game.snake();

    game.tick(&ctx);

    assert_eq!(
        game.snake(),
        snake_before,
        "snake must not move when paused"
    );
}

/// Tick must not move snake when state is GameOver.
#[test]
fn test_tick_noop_when_game_over() {
    let (game, ctrl, ctx) = make_initialized_game();
    *ctrl.state.borrow_mut() = GameState::GameOver;
    let snake_before = game.snake();

    game.tick(&ctx);

    assert_eq!(
        game.snake(),
        snake_before,
        "snake must not move when game over"
    );
}

/// Tick on uninitialized board must be a no-op.
#[test]
fn test_tick_noop_when_board_not_initialized() {
    let (game, ctrl, ctx) = make_game();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);
    *ctrl.state.borrow_mut() = GameState::Running;

    // Do NOT call restart() — board is not initialized
    game.tick(&ctx);

    assert!(
        game.snake().is_empty(),
        "snake must be empty when board is not initialized"
    );
}

/// Hitting the left wall must trigger GameOver.
/// Start by pressing K (Up) to enter Running, then press H (Left) to change direction.
#[test]
fn test_boundary_collision_left_wall() {
    let (game, _, ctx) = make_initialized_game();
    // Start Running by pressing Up: head (10,5) → (10,4)
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    // Change to Left (allowed — not opposite of Up):
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    // Manual tick: head (10,4) → (9,4)
    game.tick(&ctx);

    // 9 more ticks: (9,4) → (0,4)
    for _ in 0..9 {
        game.tick(&ctx);
    }
    assert_eq!(game.snake()[0], (0, 4), "head should be at left edge");

    // One more tick wraps to usize::MAX → out of bounds → GameOver
    game.tick(&ctx);
    assert_eq!(
        game.state(),
        GameState::GameOver,
        "hitting left wall must trigger game over"
    );
}

/// Hitting the top wall must trigger GameOver.
#[test]
fn test_boundary_collision_top_wall() {
    let (game, _, ctx) = make_initialized_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    // head at (10, 4)

    for _ in 0..4 {
        game.tick(&ctx);
    }
    assert_eq!(game.snake()[0], (10, 0), "head should be at top edge");

    game.tick(&ctx);
    assert_eq!(
        game.state(),
        GameState::GameOver,
        "hitting top wall must trigger game over"
    );
}

/// Hitting the right wall must trigger GameOver.
#[test]
fn test_boundary_collision_right_wall() {
    let (game, _, ctx) = make_initialized_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    // head at (11, 5)

    for _ in 0..8 {
        game.tick(&ctx);
    }
    assert_eq!(game.snake()[0], (19, 5), "head should be at right edge");

    game.tick(&ctx);
    assert_eq!(
        game.state(),
        GameState::GameOver,
        "hitting right wall must trigger game over"
    );
}

/// Hitting the bottom wall must trigger GameOver.
#[test]
fn test_boundary_collision_bottom_wall() {
    let (game, _, ctx) = make_initialized_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_J]));
    // head at (10, 6)

    for _ in 0..3 {
        game.tick(&ctx);
    }
    assert_eq!(game.snake()[0], (10, 9), "head should be at bottom edge");

    game.tick(&ctx);
    assert_eq!(
        game.state(),
        GameState::GameOver,
        "hitting bottom wall must trigger game over"
    );
}

/// Self-collision prevention: handle_direction blocks 180° turns.
/// A snake moving Right cannot immediately turn Left (and vice versa).
#[test]
fn test_self_collision_u_turn_blocked() {
    let (game, _, ctx) = make_initialized_game();
    // Start moving Right: head (10,5) → (11,5)
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    assert_eq!(game.direction(), Direction::Right);
    // Try to turn Left (opposite of Right — blocked by handle_direction):
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(
        game.direction(),
        Direction::Right,
        "opposite direction change must be blocked"
    );
    // Snake continues Right: head (11,5) → (12,5)
    game.tick(&ctx);
    assert_eq!(game.snake()[0], (12, 5));
}

/// Direction cannot be reversed while running (Right → Left is blocked).
#[test]
fn test_cannot_reverse_direction_while_running() {
    let (game, _, ctx) = make_initialized_game();
    // Start moving Right
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    assert_eq!(game.direction(), Direction::Right);

    // Try to reverse to Left — should be blocked
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(
        game.direction(),
        Direction::Right,
        "direction must not reverse from Right to Left"
    );
}

/// Restart resets snake, direction, state, and score.
#[test]
fn test_restart_resets_game() {
    let (game, ctrl, ctx) = make_initialized_game();
    // Start and move the snake
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    for _ in 0..4 {
        game.tick(&ctx);
    }

    game.restart();

    assert_eq!(game.state(), GameState::Paused);
    assert_eq!(game.direction(), Direction::Right);
    assert_eq!(game.snake()[0], (10, 5), "head must reset to center");
    assert_eq!(game.snake().len(), 3, "snake length must reset to 3");
    // ctrl is a separate instance from game.ctrl_state; read score from game's ctrl_state
    assert_eq!(*ctrl.score.borrow(), 0, "score must reset to 0");
}

/// Restart from GameOver state must set state to Paused.
/// Note: ctrl returned by make_initialized_game() is a CLONE of game.ctrl_state,
/// so modifying ctrl.state does NOT affect the game. We must drive state through
/// the game itself (e.g. by hitting a wall to cause GameOver, then restarting).
#[test]
fn test_restart_from_game_over_sets_paused() {
    let (game, _, ctx) = make_initialized_game();
    // Drive the snake into the top wall to trigger GameOver
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K])); // Up, head → (10,4)
    for _ in 0..4 {
        game.tick(&ctx);
    }
    assert_eq!(game.snake()[0], (10, 0), "head at top edge");
    game.tick(&ctx); // wrap → out of bounds → GameOver
    assert_eq!(game.state(), GameState::GameOver);

    game.restart();
    assert_eq!(
        game.state(),
        GameState::Paused,
        "restart from GameOver must set state to Paused"
    );
}

/// Tick on a clone must update the shared snake visible to the original.
/// This verifies last_board_w/h sharing indirectly: if dimensions weren't
/// shared, the clone's tick would see stale dimensions and behave incorrectly.
#[test]
fn test_tick_on_clone_updates_shared_snake() {
    let (game, ctrl, ctx) = make_initialized_game();
    let clone = game.clone();
    *ctrl.state.borrow_mut() = GameState::Running;

    // Tick on clone
    clone.tick(&ctx);

    // Original must see the movement
    assert_eq!(game.snake()[0], (11, 5), "original must see clone's tick result");
    assert_eq!(clone.snake()[0], (11, 5), "clone must also see the result");
}

/// Simulate the exact main.rs pattern: create game, clone for tick, clone for restart.
#[test]
fn test_main_loop_pattern_clone_tick_works() {
    let (game, ctrl, ctx) = make_initialized_game();
    let tick_game = game.clone();

    // In main.rs, the tick loop calls tick_game.tick().
    // After the fix, board_initialized is shared, so tick should work.
    *ctrl.state.borrow_mut() = GameState::Running;

    // Simulate 5 ticks from the tick loop
    for _ in 0..5 {
        tick_game.tick(&ctx);
    }

    // Check the ORIGINAL game's snake (shared via Rc)
    let snake = game.snake();
    assert_eq!(snake[0], (15, 5), "original game must see snake movement from clone tick");
    assert_eq!(snake.len(), 3, "snake length should be unchanged without eating");
}

/// Apples must never spawn on the border (outermost row/column of the playable area).
/// Border cells for a bw×bh board: x=0, x=bw-1, y=0, y=bh-1.
fn is_on_border(apple: (usize, usize), bw: usize, bh: usize) -> bool {
    let (ax, ay) = apple;
    ax == 0 || ax == bw - 1 || ay == 0 || ay == bh - 1
}

#[test]
fn test_apple_not_on_border_after_init() {
    let (game, _, _) = make_initialized_game();
    // Board is 20×10; border cells are x=0, x=19, y=0, y=9
    let apple = game.apple();
    assert!(
        !is_on_border(apple, 20, 10),
        "apple {:?} must not be on the border of a 20×10 board",
        apple
    );
}

#[test]
fn test_apple_not_on_border_after_many_restarts() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);

    for i in 0..100 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let apple = game.apple();
        assert!(
            !is_on_border(apple, 20, 10),
            "restart {}: apple {:?} must not be on the border of a 20×10 board",
            i,
            apple
        );
    }
}

#[test]
fn test_apple_not_on_border_small_board() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(10, 8);

    for i in 0..100 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let apple = game.apple();
        assert!(
            !is_on_border(apple, 10, 8),
            "restart {}: apple {:?} must not be on the border of a 10×8 board",
            i,
            apple
        );
    }
}

/// Explicitly verify all four corners are never occupied by an apple.
#[test]
fn test_apple_not_in_any_corner() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);

    let corners = [(0, 0), (19, 0), (0, 9), (19, 9)];
    for i in 0..500 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let apple = game.apple();
        for (ci, corner) in corners.iter().enumerate() {
            assert_ne!(
                apple, *corner,
                "restart {}: apple must not be at corner {} ({:?})",
                i, ci, corner
            );
        }
    }
}

/// Verify apple never spawns on the left border (x=0) across many iterations.
#[test]
fn test_apple_not_on_left_border() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);

    for i in 0..500 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let (ax, ay) = game.apple();
        assert!(
            ax != 0,
            "restart {}: apple x={} must not be on left border (x=0)",
            i, ax
        );
        // Also verify y is valid
        assert!(ay > 0 && ay < 10, "restart {}: apple y={} out of bounds", i, ay);
    }
}

/// Verify apple never spawns on the right border (x=bw-1) across many iterations.
#[test]
fn test_apple_not_on_right_border() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);

    for i in 0..500 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let (ax, ay) = game.apple();
        assert!(
            ax != 19,
            "restart {}: apple x={} must not be on right border (x=19)",
            i, ax
        );
        assert!(ay > 0 && ay < 10, "restart {}: apple y={} out of bounds", i, ay);
    }
}

/// Verify apple never spawns on the top border (y=0) across many iterations.
#[test]
fn test_apple_not_on_top_border() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);

    for i in 0..500 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let (ax, ay) = game.apple();
        assert!(
            ay != 0,
            "restart {}: apple y={} must not be on top border (y=0)",
            i, ay
        );
        assert!(ax > 0 && ax < 20, "restart {}: apple x={} out of bounds", i, ax);
    }
}

/// Verify apple never spawns on the bottom border (y=bh-1) across many iterations.
#[test]
fn test_apple_not_on_bottom_border() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);

    for i in 0..500 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let (ax, ay) = game.apple();
        assert!(
            ay != 9,
            "restart {}: apple y={} must not be on bottom border (y=9)",
            i, ay
        );
        assert!(ax > 0 && ax < 20, "restart {}: apple x={} out of bounds", i, ax);
    }
}

/// Test minimum viable board (4×4) — inner area 2×2, enough for snake + apple.
#[test]
fn test_apple_on_minimum_board() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(4, 4);

    for i in 0..100 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let apple = game.apple();
        assert!(
            !is_on_border(apple, 4, 4),
            "restart {}: apple {:?} must not be on border of 4×4 board",
            i, apple
        );
    }
}

/// Test square board with even dimensions.
#[test]
fn test_apple_not_on_border_square_board() {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(16, 16);

    for i in 0..200 {
        let game = game::SnakeGame::new(&ctx, &ctrl);
        game.restart();
        let apple = game.apple();
        assert!(
            !is_on_border(apple, 16, 16),
            "restart {}: apple {:?} must not be on border of 16×16 board",
            i, apple
        );
    }
}

// ============================================================================
// Direction queue tests — verify one direction change is applied per tick,
// preventing rapid keypresses from causing the snake to reverse into itself.
// ============================================================================

/// Rapid keypresses are queued: Right -> Down -> Left processes Down first,
/// then Left on the next tick. The snake never reverses into itself.
#[test]
fn test_rapid_keypress_queue_processes_one_per_tick() {
    let (game, _, ctx) = make_initialized_game();
    // Start moving Right
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    assert_eq!(game.direction(), Direction::Right);
    assert_eq!(game.snake()[0], (11, 5));

    // Rapidly press Down then Left before the next tick
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_J])); // Down
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H])); // Left

    // Direction should still be Right — changes are queued, not applied yet
    assert_eq!(game.direction(), Direction::Right);

    // First tick: apply Down (first in queue)
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Down);
    assert_eq!(game.snake()[0], (11, 6), "snake moves Down");

    // Second tick: apply Left (second in queue)
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Left);
    assert_eq!(game.snake()[0], (10, 6), "snake moves Left");
}

/// Queue rejects new entries when it reaches maximum length of 10.
#[test]
fn test_direction_queue_max_length() {
    let (game, _, ctx) = make_initialized_game();
    // Start moving Up
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.direction(), Direction::Up);

    // Enqueue 10 direction changes
    for _ in 0..10 {
        game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L])); // Right
    }

    // 11th press should be rejected (queue already at 10)
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H])); // Left — rejected

    // Process all 10 queued Right directions
    for _ in 0..10 {
        game.tick(&ctx);
        assert_eq!(game.direction(), Direction::Right);
    }

    // Queue should be empty now — Left was never enqueued
    game.tick(&ctx);
    assert_eq!(
        game.direction(),
        Direction::Right,
        "direction should stay Right — Left was rejected due to queue limit"
    );
}

/// Opposite direction is rejected at dequeue time, not at enqueue time.
/// The rejected item is discarded and the next tick processes the next item.
#[test]
fn test_opposite_direction_rejected_at_dequeue() {
    let (game, _, ctx) = make_initialized_game();
    // Start moving Right
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));
    assert_eq!(game.direction(), Direction::Right);

    // Queue: Left (opposite of Right, will be rejected), then Down
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H])); // Left
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_J])); // Down

    // First tick: dequeue Left, reject (opposite of Right), direction stays Right
    game.tick(&ctx);
    assert_eq!(
        game.direction(),
        Direction::Right,
        "opposite direction rejected at dequeue, direction unchanged"
    );
    assert_eq!(game.snake()[0], (12, 5), "snake continues Right");

    // Second tick: dequeue Down, apply
    game.tick(&ctx);
    assert_eq!(game.direction(), Direction::Down);
    assert_eq!(game.snake()[0], (12, 6), "snake moves Down");
}

/// Restart clears the direction queue.
#[test]
fn test_restart_clears_direction_queue() {
    let (game, _, ctx) = make_initialized_game();
    // Start moving Right
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_L]));

    // Queue some direction changes
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_J])); // Down
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H])); // Left

    // Restart should clear the queue
    game.restart();
    assert_eq!(game.state(), GameState::Paused);
    assert_eq!(game.direction(), Direction::Right);

    // Start again and tick — no queued directions should apply
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K])); // Up
    assert_eq!(game.direction(), Direction::Up);
    game.tick(&ctx);
    // If queue wasn't cleared, Down would be applied here instead of staying Up
    assert_eq!(game.direction(), Direction::Up, "no stale queue entries after restart");
}

/// Regression test: in Auto mode a smaller DrawRegion must NOT shrink the cached
/// board dimensions.  Previously a layout-probe with a medium-sized DrawRegion
/// would overwrite `last_board_w`/`last_board_h`, leaving the apple outside the
/// rendering range — the apple would "disappear" until the DrawRegion grew back.
#[test]
fn auto_mode_cache_must_not_shrink_on_small_drawregion() {
    let (_tui, ctx) = Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Auto;
    let game = SnakeGame::new(&ctx, &ctrl);

    // 1) Initialise board with a large DrawRegion (80x40 pane → 78x38 board)
    let large_dr = DrawRegion::new_large().with_size(Size::new(80, 40));
    game.drawing(&ctx, &large_dr, false);

    // Board should be initialised — apple position should not be (0,0)
    let apple_before = game.apple();
    assert_ne!(apple_before, (0, 0), "board should be initialised");

    // 2) Draw with a smaller DrawRegion (40x20 pane → 38x18 board).
    //    Without the fix the cache would shrink and tick() would use 38x18
    //    dimensions. The apple (at its original position in 78x38 space)
    //    would be unreachable.
    let small_dr = DrawRegion::new_large().with_size(Size::new(40, 20));
    game.drawing(&ctx, &small_dr, false);

    // 3) Apple position must NOT have changed — it was never eaten.
    assert_eq!(
        game.apple(),
        apple_before,
        "apple position must not change after small DrawRegion"
    );

    // 4) Verify tick() still uses the original (large) dimensions — the snake
    //    should be able to reach and eat the apple.
    *ctrl.state.borrow_mut() = GameState::Running;
    let result = steer_to_apple(&game, &ctrl, &ctx);
    assert!(
        result,
        "snake should be able to reach and eat the apple after small DrawRegion"
    );
}

/// Helper: make a game on a 6×4 fixed board. Snake head starts at (3,2),
/// direction Right. Inner spawn area is (1..5)×(1..3) = 8 cells.
fn make_tiny_game() -> (SnakeGame, ControlState, yeehaw::Context) {
    let (_tui, ctx) = yeehaw::Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::for_test();
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(6, 4);
    let game = SnakeGame::new(&ctx, &ctrl);
    game.restart(); // head=(3,2), body=(2,2), tail=(1,2), dir=Right
    *ctrl.state.borrow_mut() = GameState::Paused;
    (game, ctrl, ctx)
}

/// Steer the snake toward the apple using greedy shortest-path heuristics.
/// Returns true if the snake ate the apple, false if game ended first.
fn steer_to_apple(game: &SnakeGame, ctrl: &ControlState, ctx: &yeehaw::Context) -> bool {
    *ctrl.state.borrow_mut() = GameState::Running;

    for _ in 0..200 {
        if game.state() != GameState::Running {
            return false;
        }
        let apple = game.apple();
        let head = game.snake()[0];
        let dir = game.direction();

        // Greedy: move toward apple
        let mut next_dir = dir;
        if apple.0 > head.0 && dir != Direction::Left {
            next_dir = Direction::Right;
        } else if apple.0 < head.0 && dir != Direction::Right {
            next_dir = Direction::Left;
        } else if apple.1 > head.1 && dir != Direction::Up {
            next_dir = Direction::Down;
        } else if apple.1 < head.1 && dir != Direction::Down {
            next_dir = Direction::Up;
        } else {
            // Cannot move toward apple directly (would need to reverse).
            // Turn 90° to reposition, then approach from a different axis.
            next_dir = match dir {
                Direction::Up | Direction::Down => Direction::Right,
                Direction::Left | Direction::Right => Direction::Up,
            };
        }

        // Send direction change if different from current
        if next_dir != dir {
            let key = match next_dir {
                Direction::Up => Keyboard::KEY_K,
                Direction::Down => Keyboard::KEY_J,
                Direction::Left => Keyboard::KEY_H,
                Direction::Right => Keyboard::KEY_L,
            };
            game.receive_event(ctx, Event::KeyCombo(vec![key]));
        }

        let apple_before = game.apple();
        game.tick(ctx);
        let apple_after = game.apple();

        if apple_after != apple_before {
            // Apple was eaten — verify it respawned at a NEW position
            assert_ne!(
                apple_after, apple_before,
                "BUG: apple did not respawn after being eaten; still at {:?}",
                apple_before
            );
            // Verify new apple is not on the snake
            let snake = game.snake();
            assert!(
                !snake.iter().any(|s| *s == apple_after),
                "respawned apple ({:?}) must not overlap snake",
                apple_after
            );
            return true;
        }
    }
    false
}

/// Core reproduction: after the snake eats an apple, a NEW apple must
/// appear at a different position. This is the exact bug the user reports.
#[test]
fn test_apple_respawns_at_new_position_after_eating() {
    let (game, ctrl, ctx) = make_tiny_game();
    let ate = steer_to_apple(&game, &ctrl, &ctx);
    assert!(
        ate,
        "snake should have eaten the apple on a 6x4 board with steering"
    );
}

/// Multiple eats: verify every eat results in a valid respawn.
#[test]
fn test_apple_always_respawns_after_multiple_eats() {
    let (game, ctrl, ctx) = make_tiny_game();
    *ctrl.state.borrow_mut() = GameState::Running;

    let mut eats = 0u32;
    let mut prev_apple = game.apple();

    for _ in 0..300 {
        if game.state() != GameState::Running {
            break;
        }
        // Steer toward apple each tick
        let apple = game.apple();
        let head = game.snake()[0];
        let dir = game.direction();
        let mut next_dir = dir;
        if apple.0 > head.0 && dir != Direction::Left {
            next_dir = Direction::Right;
        } else if apple.0 < head.0 && dir != Direction::Right {
            next_dir = Direction::Left;
        } else if apple.1 > head.1 && dir != Direction::Up {
            next_dir = Direction::Down;
        } else if apple.1 < head.1 && dir != Direction::Down {
            next_dir = Direction::Up;
        } else {
            // Cannot move toward apple directly (would need to reverse).
            // Turn 90° to reposition, then approach from a different axis.
            next_dir = match dir {
                Direction::Up | Direction::Down => Direction::Right,
                Direction::Left | Direction::Right => Direction::Up,
            };
        }
        if next_dir != dir {
            let key = match next_dir {
                Direction::Up => Keyboard::KEY_K,
                Direction::Down => Keyboard::KEY_J,
                Direction::Left => Keyboard::KEY_H,
                Direction::Right => Keyboard::KEY_L,
            };
            game.receive_event(&ctx, Event::KeyCombo(vec![key]));
        }

        game.tick(&ctx);
        let current = game.apple();
        if current != prev_apple {
            assert_ne!(current, prev_apple, "new apple must differ from old");
            // Verify new apple is valid
            let snake = game.snake();
            assert!(!snake.iter().any(|s| *s == current), "apple must not overlap snake");
            eats += 1;
            prev_apple = current;
        }
    }
    assert!(
        eats >= 2,
        "expected at least 2 apple eats on a 6x4 board with steering, got {}",
        eats
    );
}

/// Verify that after eating an apple, the new apple is never at the
/// exact same coordinates as the one that was just eaten.
#[test]
fn test_apple_never_respawns_at_eaten_position() {
    let (game, ctrl, ctx) = make_tiny_game();
    let ate = steer_to_apple(&game, &ctrl, &ctx);
    assert!(ate, "snake should have eaten the apple");
    // If we got here, steer_to_apple already asserted apple_after != apple_before
}

/// Regression: verify that the apple position is always valid after respawn.
#[test]
fn test_apple_position_valid_after_respawn() {
    let (game, ctrl, ctx) = make_tiny_game();
    *ctrl.state.borrow_mut() = GameState::Running;

    for _ in 0..300 {
        if game.state() != GameState::Running {
            break;
        }
        // Steer toward apple
        let apple = game.apple();
        let head = game.snake()[0];
        let dir = game.direction();
        let mut next_dir = dir;
        if apple.0 > head.0 && dir != Direction::Left {
            next_dir = Direction::Right;
        } else if apple.0 < head.0 && dir != Direction::Right {
            next_dir = Direction::Left;
        } else if apple.1 > head.1 && dir != Direction::Up {
            next_dir = Direction::Down;
        } else if apple.1 < head.1 && dir != Direction::Down {
            next_dir = Direction::Up;
        }
        if next_dir != dir {
            let key = match next_dir {
                Direction::Up => Keyboard::KEY_K,
                Direction::Down => Keyboard::KEY_J,
                Direction::Left => Keyboard::KEY_H,
                Direction::Right => Keyboard::KEY_L,
            };
            game.receive_event(&ctx, Event::KeyCombo(vec![key]));
        }

        game.tick(&ctx);
        let apple = game.apple();
        // For Fixed(6, 4): valid inner area is x in [1..5], y in [1..3]
        assert!(apple.0 > 0 && apple.0 < 5, "apple x={} must be in (0, 5)", apple.0);
        assert!(apple.1 > 0 && apple.1 < 3, "apple y={} must be in (0, 3)", apple.1);
        let snake = game.snake();
        // When the snake fills the entire inner area there is nowhere to
        // respawn — the apple stays at the cell just eaten (on the head).
        let inner_area = 4 * 2; // inner_w * inner_h for Fixed(6, 4)
        if snake.len() < inner_area {
            assert!(
                !snake.iter().any(|s| *s == apple),
                "apple ({:?}) must not overlap snake",
                apple
            );
        }
    }
}
