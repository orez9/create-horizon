fn create_horizon_server_tick(phase: crate::game::native_lifecycle::ServerTickPhase) {
    use crate::game::native_lifecycle::ServerTickPhase;
    if let ServerTickPhase::ServerTickStart { world_running: true } = phase {
        crate::game::create_horizon::with_grid(|grid| grid.begin_tick());
    }
}
