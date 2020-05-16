#!/bin/sh
cat bug1.txt - | ../target/release/rokumon -o HumanHuman -f --cards="jjjggjg"

# Current result:

# Running AI with duration 15s...
# AI log: steps: 483024, depth: 9, completed: false, duration: 15.00009438s
# AI recommends: move R6 from <2, 0, -2> to <1, 0, -1>

# Potentially we can rewrite it as a test, but it's not clear what to expect:
#
# #[test]
# pub fn test_unexpected_blunder() -> Fallible<()> {
#     let deck = Deck::ordered("jjjggjg")?;
#     let mut game = Game::new(Layout::Bricks7, deck, Default::default());
#     game.apply_move_str("place r2 at r2c3")?;
#     game.apply_move_str("place b3 at r2c2")?;
#     game.apply_move_str("move r2 from r2c3 to r2c2")?;
#     game.apply_move_str("fight at r2c2")?;
#     game.apply_move_str("place r6 at r2c3")?;
#     game.apply_move_str("place b1 at r1c2")?;

#     let mut bot = AlphaBetaAI::with_duration(true, 15);

#     Ok(())
# }
