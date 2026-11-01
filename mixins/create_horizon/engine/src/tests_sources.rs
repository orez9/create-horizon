use std::collections::HashMap;

use crate::connect::ratio;
use crate::geometry::NEIGHBOUR_OFFSETS;
use crate::sources::*;
use crate::*;

type World = HashMap<Pos, KineticBlock>;

fn run(world: &World, start: Pos) -> Network {
    solve(world, start, Limits::default())
}

// Hand crank.

#[test]
fn a_crank_turns_while_used_and_stops_after_its_delay() {
    let mut crank = HandCrank::default();
    assert_eq!(crank.speed(), 0.0);
    assert!(crank.use_crank(false));
    assert_eq!(crank.speed(), 32.0);
    for _ in 1..values::HAND_CRANK_TICKS {
        assert!(!crank.tick());
        assert_eq!(crank.speed(), 32.0);
    }
    assert!(crank.tick(), "the last tick stops it");
    assert_eq!(crank.speed(), 0.0);
    assert!(!crank.tick(), "a still crank stays still");
}

#[test]
fn using_a_crank_again_restarts_its_delay_without_a_change() {
    let mut crank = HandCrank::default();
    crank.use_crank(false);
    crank.tick();
    crank.tick();
    assert!(!crank.use_crank(false), "same speed: no re-solve needed");
    assert_eq!(crank.ticks_left, values::HAND_CRANK_TICKS);
}

#[test]
fn sneaking_turns_a_crank_the_other_way() {
    let mut crank = HandCrank::default();
    crank.use_crank(false);
    assert!(crank.use_crank(true), "direction change needs a re-solve");
    assert_eq!(crank.speed(), -32.0);
}

#[test]
fn a_cranked_crank_drives_a_shaft_at_256_su() {
    let mut crank = HandCrank::default();
    crank.use_crank(false);
    let mut w = World::new();
    w.insert([0, 0, 0], crank.block(Face::new(Axis::X, true)));
    w.insert([1, 0, 0], KineticBlock::shaft(Axis::X));
    let n = run(&w, [1, 0, 0]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.speed([1, 0, 0]), 32.0);
    assert_eq!(n.capacity, 256.0);
}

// Creative motor.

#[test]
fn creative_motor_speed_is_clamped_with_its_sign_kept() {
    assert_eq!(creative_motor_speed(64.0), 64.0);
    assert_eq!(creative_motor_speed(300.0), 256.0);
    assert_eq!(creative_motor_speed(-1000.0), -256.0);
    assert_eq!(creative_motor_speed(-12.0), -12.0);
    assert_eq!(creative_motor_speed(f32::NAN), 0.0);
}

#[test]
fn a_creative_motor_gives_its_full_capacity_at_any_speed() {
    for speed in [1.0, 16.0, -256.0] {
        let mut w = World::new();
        w.insert(
            [0, 0, 0],
            creative_motor_block(Face::new(Axis::Y, true), speed),
        );
        w.insert([0, 1, 0], KineticBlock::shaft(Axis::Y));
        let n = run(&w, [0, 1, 0]);
        assert_eq!(n.state, State::Running);
        assert_eq!(n.speed([0, 1, 0]), speed);
        assert!(
            (n.capacity - 16384.0).abs() < 0.01,
            "{speed}: {}",
            n.capacity
        );
    }
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        creative_motor_block(Face::new(Axis::Y, true), 0.0),
    );
    let n = run(&w, [0, 0, 0]);
    assert_eq!((n.state, n.capacity), (State::Still, 0.0));
}

// Water wheels.

#[test]
fn a_water_wheel_has_one_speed_in_the_net_flow_direction() {
    let flow = |forward, backward| WaterFlow { forward, backward };
    assert_eq!(water_wheel_speed(flow(0, 0), 8.0), 0.0);
    assert_eq!(water_wheel_speed(flow(1, 0), 8.0), 8.0);
    assert_eq!(
        water_wheel_speed(flow(4, 0), 8.0),
        8.0,
        "more water isn't faster"
    );
    assert_eq!(water_wheel_speed(flow(1, 3), 8.0), -8.0);
    assert_eq!(
        water_wheel_speed(flow(2, 2), 8.0),
        0.0,
        "opposing flows cancel"
    );
}

#[test]
fn water_wheels_feed_a_network() {
    let flow = WaterFlow {
        forward: 1,
        backward: 0,
    };
    let mut w = World::new();
    w.insert([0, 0, 0], water_wheel_block(Axis::Z, flow));
    w.insert([0, 0, 1], KineticBlock::shaft(Axis::Z));
    w.insert([0, 0, 2], large_water_wheel_block(Axis::Z, flow));
    let n = run(&w, [0, 0, 1]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.speed([0, 0, 1]), 8.0, "the faster wheel drives");
    assert_eq!(n.capacity, 8.0 * 16.0 + 4.0 * 128.0);
}

// Transmission.

fn line(middle: KineticBlock) -> World {
    let mut w = World::new();
    w.insert(
        [-1, 0, 0],
        KineticBlock::single_shaft(Face::new(Axis::X, true)).with_source(32.0, 8.0),
    );
    w.insert([0, 0, 0], middle);
    w.insert([1, 0, 0], KineticBlock::shaft(Axis::X));
    w
}

#[test]
fn a_gearshift_reverses_only_when_powered() {
    let n = run(&line(KineticBlock::gearshift(Axis::X, false)), [1, 0, 0]);
    assert_eq!(n.speed([1, 0, 0]), 32.0);
    let n = run(&line(KineticBlock::gearshift(Axis::X, true)), [1, 0, 0]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.speed([1, 0, 0]), -32.0);
}

#[test]
fn a_powered_gearshift_beside_a_gearbox_stays_consistent() {
    // Reversing twice in a line gives the input speed back.
    let mut w = line(KineticBlock::gearshift(Axis::X, true));
    w.insert([1, 0, 0], KineticBlock::gearshift(Axis::X, true));
    w.insert([2, 0, 0], KineticBlock::shaft(Axis::X));
    let n = run(&w, [2, 0, 0]);
    assert_eq!(n.speed([2, 0, 0]), 32.0);
}

#[test]
fn a_clutch_disconnects_when_powered() {
    let n = run(&line(KineticBlock::clutch(Axis::X, false)), [1, 0, 0]);
    assert_eq!(n.speed([1, 0, 0]), 32.0);
    let w = line(KineticBlock::clutch(Axis::X, true));
    let n = run(&w, [1, 0, 0]);
    assert_eq!(n.members.len(), 1, "the far side is on its own");
    assert_eq!(n.state, State::Still);
    assert!(!run(&w, [-1, 0, 0]).contains([0, 0, 0]));
}

#[test]
fn chain_drives_link_side_by_side_without_reversing() {
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        KineticBlock::encased_chain_drive(Axis::X).with_source(16.0, 64.0),
    );
    w.insert([0, 1, 0], KineticBlock::encased_chain_drive(Axis::X));
    w.insert([0, 2, 0], KineticBlock::encased_chain_drive(Axis::X));
    w.insert([1, 2, 0], KineticBlock::shaft(Axis::X));
    let n = run(&w, [0, 0, 0]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.speed([0, 2, 0]), 16.0);
    assert_eq!(n.speed([1, 2, 0]), 16.0, "its shaft end passes it on");
}

#[test]
fn chain_drives_do_not_link_across_their_axis_or_mesh_with_cogs() {
    let mut w = World::new();
    w.insert([0, 0, 0], KineticBlock::encased_chain_drive(Axis::X));
    w.insert([0, 1, 0], KineticBlock::encased_chain_drive(Axis::Y));
    w.insert([0, 0, 1], KineticBlock::cogwheel(Axis::X));
    assert_eq!(run(&w, [0, 0, 0]).members.len(), 1);
}

#[test]
fn every_new_connection_is_symmetric() {
    let mut kinds = Vec::new();
    for axis in Axis::ALL {
        for powered in [false, true] {
            kinds.push(KineticBlock::gearshift(axis, powered));
            kinds.push(KineticBlock::clutch(axis, powered));
        }
        kinds.push(KineticBlock::encased_chain_drive(axis));
        kinds.push(KineticBlock::shaft(axis));
        kinds.push(KineticBlock::cogwheel(axis));
        kinds.push(KineticBlock::gearbox(axis));
    }
    for a in &kinds {
        for b in &kinds {
            for d in NEIGHBOUR_OFFSETS {
                let back = [-d[0], -d[1], -d[2]];
                let (ab, ba) = (ratio(a, b, d), ratio(b, a, back));
                assert_eq!(ab == 0.0, ba == 0.0, "{a:?} {b:?} {d:?}");
                if ab != 0.0 {
                    assert_eq!(ab * ba, 1.0, "{a:?} {b:?} {d:?}");
                }
            }
        }
    }
}
