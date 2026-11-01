// Tells the grid this block and takes the speed it solved; clients hear when it changes.
fn create_horizon_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    use crate::game::create_horizon as glue;
    use horizon_world::nbt::{JavaString, Tag};
    if !glue::is_kinetic(&entity.id) {
        return;
    }
    let crank_key = JavaString::from(glue::CRANK_KEY);
    let crank = match entity.nbt.get(&crank_key) {
        Some(Tag::Int(left)) => *left,
        _ => 0,
    };
    if crank > 0 {
        entity.nbt.insert(crank_key, Tag::Int(crank - 1));
    }
    let Some(block) =
        glue::kinetic_block(&entity.id, |key| game.native_block_property(pos, key), crank > 0)
    else {
        return;
    };
    let dimension = game.native_dimension();
    let speed = glue::with_grid(|grid| grid.observe(dimension, pos, block));
    let speed_key = JavaString::from(glue::SPEED_KEY);
    if entity.nbt.get(&speed_key) != Some(&Tag::Float(speed.to_bits())) {
        entity.nbt.insert(speed_key, Tag::Float(speed.to_bits()));
        game.native_block_entity_sync(pos);
    }
}
