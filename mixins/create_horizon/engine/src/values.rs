//! Gameplay numbers, from Create's publicly documented default configuration.
//! One table, so balancing is one edit.
//!
//! Sources (documentation only, never Create's code):
//! - [GH-WIKI] Create's GitHub wiki, "Stress Units, Capacity and Impact" (last edited
//!   2021-03-14): <https://github.com/Creators-of-Create/Create/wiki/Stress-Units,-Capacity-and-Impact>
//! - [FANDOM] the Create community wiki, "Kinetic Stress": <https://create.fandom.com/wiki/Kinetic_Stress>
//!   (read through search results only: the page was not reachable from the session that wrote this).
//!
//! **Confirmed** means two sources agree, one of them describing current Create. Everything marked
//! **UNCONFIRMED** still needs a check in a Create 6.0.10 game (Engineer's Goggles / Stressometer)
//! or on the current wiki before the first release.

/// Hand crank: RPM while turned, and capacity in SU per RPM (256 SU in all).
/// Confirmed: [GH-WIKI] table "Hand Crank 8 SU/RPM, 256 SU, 32 RPM"; [FANDOM] "256 SU, 32 RPM".
pub const HAND_CRANK_SPEED: f32 = 32.0;
pub const HAND_CRANK_CAPACITY: f32 = 8.0;
/// Ticks a hand crank keeps turning after the player's last use.
/// UNCONFIRMED: no source gives it; measure in game (hold use, release, count ticks until 0 RPM).
pub const HAND_CRANK_TICKS: u32 = 10;

/// Water wheel: RPM and SU per RPM.
/// UNCONFIRMED: sources disagree. [FANDOM] via search: "Water Wheel 128 SU, 8 RPM, 16 SU/RPM";
/// a third-party guide says 256 SU at 8 RPM; [GH-WIKI] (pre-0.5, multi-speed wheels) says
/// 16 SU/RPM at 6 to 20 RPM. 8 RPM is common to the current sources; capacity kept at 16 SU/RPM
/// (128 SU) from [FANDOM] until checked in game.
pub const WATER_WHEEL_SPEED: f32 = 8.0;
pub const WATER_WHEEL_CAPACITY: f32 = 16.0;

/// Large water wheel. UNCONFIRMED: [FANDOM] via search: "512 SU at 4 RPM" (128 SU/RPM).
pub const LARGE_WATER_WHEEL_SPEED: f32 = 4.0;
pub const LARGE_WATER_WHEEL_CAPACITY: f32 = 128.0;

/// Creative motor: default RPM, highest RPM, and its total capacity in SU.
/// Speed range confirmed: [GH-WIKI] "-256 to 256", and 256 is the network speed limit
/// ([`crate::Limits`]). UNCONFIRMED: the default speed (16) and the capacity: [GH-WIKI] lists
/// "16384" in its SU-per-RPM column with no total; current Create shows a fixed total, taken here
/// as 16384 SU at any speed.
pub const CREATIVE_MOTOR_SPEED: f32 = 16.0;
pub const CREATIVE_MOTOR_MAX_SPEED: f32 = 256.0;
pub const CREATIVE_MOTOR_TOTAL_CAPACITY: f32 = 16384.0;
/// Kept for the glue written against the first engine: SU per RPM at the highest speed.
pub const CREATIVE_MOTOR_CAPACITY: f32 = CREATIVE_MOTOR_TOTAL_CAPACITY / CREATIVE_MOTOR_MAX_SPEED;

/// Stress impact of machines, SU per RPM.
/// UNCONFIRMED for 6.0: [GH-WIKI] (2021) lists Mechanical Press 8, Millstone 4, Encased Fan 2;
/// not yet checked against the current wiki or in game.
pub const MECHANICAL_PRESS_IMPACT: f32 = 8.0;
pub const MILLSTONE_IMPACT: f32 = 4.0;
pub const ENCASED_FAN_IMPACT: f32 = 2.0;
