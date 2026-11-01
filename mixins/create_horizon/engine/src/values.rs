//! Gameplay numbers, from the publicly documented values of Create's default
//! configuration (the Create wiki). One table, so balancing is one edit.
//!
//! TODO: check each value against the wiki's current pages and in a Create
//! 6.0.x game before the first release; these are first estimates.

/// Hand crank: RPM while turned, and capacity in SU per RPM.
pub const HAND_CRANK_SPEED: f32 = 32.0;
pub const HAND_CRANK_CAPACITY: f32 = 8.0;
/// Ticks a hand crank keeps turning after the player's last use.
pub const HAND_CRANK_TICKS: u32 = 10;

/// Water wheel: RPM and SU per RPM.
pub const WATER_WHEEL_SPEED: f32 = 8.0;
pub const WATER_WHEEL_CAPACITY: f32 = 32.0;

/// Creative motor: default RPM, highest RPM, SU per RPM.
pub const CREATIVE_MOTOR_SPEED: f32 = 16.0;
pub const CREATIVE_MOTOR_MAX_SPEED: f32 = 256.0;
pub const CREATIVE_MOTOR_CAPACITY: f32 = 16384.0 / 256.0;

/// Stress impact of machines, SU per RPM.
pub const MECHANICAL_PRESS_IMPACT: f32 = 8.0;
pub const MILLSTONE_IMPACT: f32 = 4.0;
pub const ENCASED_FAN_IMPACT: f32 = 2.0;
