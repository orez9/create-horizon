fn create_horizon_update_tag(
    entity: &horizon_client::containers::ModBlockEntity,
    tag: &mut Option<horizon_world::nbt::Compound>,
) {
    use crate::game::create_horizon as glue;
    use horizon_world::nbt::{Compound, JavaString};
    if !glue::is_kinetic(&entity.id) {
        return;
    }
    let key = JavaString::from(glue::SPEED_KEY);
    let mut out = Compound::new();
    if let Some(speed) = entity.nbt.get(&key) {
        out.insert(key, speed.clone());
    }
    *tag = Some(out);
}
