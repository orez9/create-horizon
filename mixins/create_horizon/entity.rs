fn create_horizon_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    if entity.is_none() && crate::game::create_horizon::is_kinetic(block) {
        *entity = Some((block.to_owned(), Default::default()));
    }
}
