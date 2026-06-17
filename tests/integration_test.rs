#![allow(unused_must_use)]

use yeehaw::{Element, Event, Keyboard, Tui};

use snek::controls::ControlState;
use snek::game::{BoardSize, Direction, GameState, SnakeGame};

fn make_game() -> (SnakeGame, ControlState, yeehaw::Context) {
    let (_tui, ctx) = Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::new(&ctx);
    let game = SnakeGame::new(&ctx, &ctrl);
    (game, ctrl, ctx)
}

/// Create a game with an initialized Fixed board (20x10), paused and ready for input.
/// Snake head starts at (10, 5), direction Right.
fn make_initialized_game() -> (SnakeGame, ControlState, yeehaw::Context) {
    let (_tui, ctx) = Tui::new().expect("failed to create Tui");
    let ctrl = ControlState::new(&ctx);
    *ctrl.board_size.borrow_mut() = BoardSize::Fixed(20, 10);
    let game = SnakeGame::new(&ctx, &ctrl);
    game.restart(); // initializes board: head=(10,5), body=(9,5), tail=(8,5), dir=Right
    *ctrl.state.borrow_mut() = GameState::Paused;
    (game, ctrl, ctx)
}

#[test]
fn test_initial_state_is_paused() {
    let (game, _, _) = make_game();
    assert_eq!(game.state(), GameState::Paused, "game must start Paused");
}

#[test]
fn test_h_key_starts_game_and_sets_left() {
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Left);
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
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_LEFT]));
    assert_eq!(game.state(), GameState::Running);
    assert_eq!(game.direction(), Direction::Left);
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
    let (game, _, ctx) = make_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_K]));
    assert_eq!(game.direction(), Direction::Up);
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
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
    // Pressing Left from Paused sets dir=Left, tick moves head to (9,5) which
    // collides with the body segment -> Game Over.
    let (game, _, ctx) = make_initialized_game();
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.state(), GameState::GameOver, "pressing Left should cause self-collision");
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

    // Change to Left while Running.
    game.receive_event(&ctx, Event::KeyCombo(vec![Keyboard::KEY_H]));
    assert_eq!(game.direction(), Direction::Left);
    // Snake hasn't moved yet — only direction changed.
    assert_eq!(game.snake()[0], (10, 4), "direction change alone should not move");

    // Now tick — snake should move Left.
    game.tick(&ctx);
    assert_eq!(
        game.snake()[0],
        (9, 4),
        "tick after direction change should move in new direction"
    );
}
