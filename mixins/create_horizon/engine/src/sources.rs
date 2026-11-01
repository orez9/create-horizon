//! Rotation sources' own state: what each one generates, before the network is solved.
//! The glue keeps these in the block entity and turns them into a [`KineticBlock`] each solve.

use crate::block::KineticBlock;
use crate::geometry::{Axis, Face};
use crate::values;

/// A hand crank: turns while a player keeps using it, then stops a few ticks after the last use.
///
/// Behaviour from public gameplay descriptions: holding use on a hand crank turns it at a fixed
/// speed; sneaking while using turns it the other way. The stop delay is
/// [`values::HAND_CRANK_TICKS`] (unconfirmed).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HandCrank {
    /// Ticks left before it stops; 0 when still.
    pub ticks_left: u32,
    /// Turning the reverse way (the player sneaked).
    pub reversed: bool,
}

impl HandCrank {
    /// A player used it this tick. Returns whether its generated speed changed (a network re-solve
    /// is needed).
    pub fn use_crank(&mut self, reversed: bool) -> bool {
        let before = self.speed();
        self.ticks_left = values::HAND_CRANK_TICKS;
        self.reversed = reversed;
        self.speed() != before
    }

    /// One server tick. Returns whether its generated speed changed (it just stopped).
    pub fn tick(&mut self) -> bool {
        if self.ticks_left == 0 {
            return false;
        }
        self.ticks_left -= 1;
        self.ticks_left == 0
    }

    /// Signed RPM it generates about its own axis, seen from its shaft's side.
    pub fn speed(&self) -> f32 {
        match (self.ticks_left, self.reversed) {
            (0, _) => 0.0,
            (_, false) => values::HAND_CRANK_SPEED,
            (_, true) => -values::HAND_CRANK_SPEED,
        }
    }

    /// The crank as a kinetic block: one shaft end on `shaft`, the face it is mounted on.
    pub fn block(&self, shaft: Face) -> KineticBlock {
        KineticBlock::single_shaft(shaft).with_source(self.speed(), values::HAND_CRANK_CAPACITY)
    }
}

/// A creative motor's speed as the player sets it: clamped to ±[`values::CREATIVE_MOTOR_MAX_SPEED`],
/// its sign kept (the sign picks the direction). NaN gives 0.
pub fn creative_motor_speed(wanted: f32) -> f32 {
    if wanted.is_nan() {
        return 0.0;
    }
    wanted.clamp(
        -values::CREATIVE_MOTOR_MAX_SPEED,
        values::CREATIVE_MOTOR_MAX_SPEED,
    )
}

/// A creative motor as a kinetic block: one shaft end on `shaft`, the face it outputs through,
/// with a fixed total capacity whatever its speed. A motor at 0 RPM generates nothing.
pub fn creative_motor_block(shaft: Face, speed: f32) -> KineticBlock {
    let speed = creative_motor_speed(speed);
    let per_rpm = if speed == 0.0 {
        0.0
    } else {
        values::CREATIVE_MOTOR_TOTAL_CAPACITY / speed.abs()
    };
    KineticBlock::single_shaft(shaft).with_source(speed, per_rpm)
}

/// The flowing water a water wheel touches, counted by side: sides whose flow pushes the blades
/// one way (`forward`) and the other way (`backward`). Bubble columns count as flow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WaterFlow {
    pub forward: u8,
    pub backward: u8,
}

/// The signed RPM a water wheel generates from the water around it.
///
/// Current Create (0.5 and later) wheels have a single speed level (players quote the wiki:
/// "Water Wheels now only have one speed level"): any net flow turns the wheel at
/// [`values::WATER_WHEEL_SPEED`] in the flow's direction, more water doesn't make it faster, and
/// flows that cancel out leave it still. The pre-0.5 rule ([GH-WIKI] in `values.rs`: 4 RPM base,
/// +4 per side with the blades, +2 against) is obsolete. UNCONFIRMED in game: whether opposing
/// sides cancel exactly as modelled here.
pub fn water_wheel_speed(flow: WaterFlow, speed: f32) -> f32 {
    match flow.forward.cmp(&flow.backward) {
        std::cmp::Ordering::Greater => speed,
        std::cmp::Ordering::Less => -speed,
        std::cmp::Ordering::Equal => 0.0,
    }
}

/// A water wheel as a kinetic block: shafts on both ends of `axis`.
pub fn water_wheel_block(axis: Axis, flow: WaterFlow) -> KineticBlock {
    KineticBlock::shaft(axis).with_source(
        water_wheel_speed(flow, values::WATER_WHEEL_SPEED),
        values::WATER_WHEEL_CAPACITY,
    )
}

/// A large water wheel, the same rule at its own speed and capacity.
pub fn large_water_wheel_block(axis: Axis, flow: WaterFlow) -> KineticBlock {
    KineticBlock::shaft(axis).with_source(
        water_wheel_speed(flow, values::LARGE_WATER_WHEEL_SPEED),
        values::LARGE_WATER_WHEEL_CAPACITY,
    )
}
