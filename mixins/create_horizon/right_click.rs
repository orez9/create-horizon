// Turning a hand crank: it keeps a source for a few ticks after each click.
fn create_horizon_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    use crate::game::create_horizon as glue;
    use horizon_world::nbt::{JavaString, Tag};
    if game.native_block(pos) != glue::HAND_CRANK {
        return;
    }
    let ticks = create_horizon_engine::values::HAND_CRANK_TICKS as i32;
    game.native_with_mod_block_entity(pos, |entity| {
        entity.nbt.insert(JavaString::from(glue::CRANK_KEY), Tag::Int(ticks));
    });
    *canceled = true;
}
