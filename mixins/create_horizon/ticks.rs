fn create_horizon_ticks(block: &str, ticks: &mut bool) {
    if crate::game::create_horizon::is_kinetic(block) {
        *ticks = true;
    }
}
