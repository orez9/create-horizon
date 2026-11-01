// A placed block takes its axis or facing from the face clicked.
fn create_horizon_place(game: &mut crate::game::Game<'_>, pos: [i32; 3]) {
    use crate::game::create_horizon as glue;
    let block = game.native_block(pos);
    if let Some((key, value)) = glue::placement_property(&block, glue::last_face()) {
        game.native_set_block_property(pos, key, value);
    }
}
