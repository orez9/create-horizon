// Before content loads: gives the carried kinetic blocks a template with a block entity.
fn create_horizon_declare(types: &mut Vec<String>) {
    let _ = types;
    use crate::game::create_horizon as glue;
    if let Err(e) = glue::patch_carried_blocks(std::path::Path::new(glue::CARRIED_CREATE)) {
        log::warn!("create_horizon: carried blocks not patched: {e}");
    }
}
