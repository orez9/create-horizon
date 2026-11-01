// Draws the carried model turned to this frame's angle: the chunk mesh has none of it.
fn create_horizon_submit(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: horizon_world::registry::BlockStateId,
    entity: &horizon_client::containers::ModBlockEntity,
    draw: &mut crate::game::native_block_entity_draw::BlockEntityDraw,
) {
    use crate::game::create_horizon as glue;
    use horizon_world::nbt::{JavaString, Tag};
    if !glue::is_kinetic(block) {
        return;
    }
    let speed = match entity.nbt.get(&JavaString::from(glue::SPEED_KEY)) {
        Some(Tag::Float(bits)) => f32::from_bits(*bits),
        _ => 0.0,
    };
    let ticks = draw.game_time as f64 + f64::from(draw.partial);
    let angle = create_horizon_engine::angle_degrees(speed, ticks);
    const CENTRE: [f32; 3] = [0.5, 0.5, 0.5];
    match block {
        glue::SHAFT | glue::COGWHEEL | glue::LARGE_COGWHEEL => {
            let axis = game.native_block_state_property(state, glue::AXIS_KEY).unwrap_or("y");
            let offset = glue::cog_offset_degrees(block, axis, pos);
            draw.push();
            draw.rotate_about(glue::axis_vector(axis), angle + offset, CENTRE);
            game.native_draw_block(draw, block, &[(glue::AXIS_KEY, axis)]);
            draw.pop();
        }
        glue::GEARBOX => {
            let axis = game.native_block_state_property(state, glue::AXIS_KEY).unwrap_or("y");
            game.native_draw_block(draw, block, &[(glue::AXIS_KEY, axis)]);
            // Its visible gears: a shaft through each pair of ports, either way round.
            let turning: &[(&str, f32)] = match axis {
                "x" => &[("y", 1.0), ("z", -1.0)],
                "z" => &[("x", 1.0), ("y", -1.0)],
                _ => &[("x", 1.0), ("z", -1.0)],
            };
            for &(through, sign) in turning {
                draw.push();
                draw.rotate_about(glue::axis_vector(through), angle * sign, CENTRE);
                game.native_draw_block(draw, glue::SHAFT, &[(glue::AXIS_KEY, through)]);
                draw.pop();
            }
        }
        _ => {
            let facing = game.native_block_state_property(state, glue::FACING_KEY).unwrap_or("up");
            game.native_draw_block(draw, block, &[(glue::FACING_KEY, facing)]);
        }
    }
}
