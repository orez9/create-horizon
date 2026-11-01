//! The rotational power simulation of the create_horizon mod.
//!
//! An independent ("clean-room") implementation of Create-style kinetics,
//! written from the publicly documented gameplay (the Create wiki, the game's
//! behaviour) and the geometry of gears, not from the Create mod's source.
//! It knows nothing of Horizon: the mod's helpers turn world blocks into
//! [`KineticBlock`]s and write the solved speeds back.

pub mod block;
pub mod connect;
pub mod geometry;
pub mod network;
pub mod values;

pub use block::{Cog, KineticBlock, Source};
pub use geometry::{Axis, Face, Pos};
pub use network::{KineticWorld, Limits, Network, State, solve};

/// Degrees a block turning at `rpm` has turned after `ticks` game ticks
/// (20 a second), wrapped to 0..360.
pub fn angle_degrees(rpm: f32, ticks: f64) -> f32 {
    // rpm · 360° / (60 s · 20 ticks/s) = 0.3° per tick per RPM.
    ((rpm as f64 * 0.3 * ticks).rem_euclid(360.0)) as f32
}

#[cfg(test)]
mod tests;
