use std::collections::HashMap;

use crate::connect::ratio;
use crate::geometry::NEIGHBOUR_OFFSETS;
use crate::*;

type World = HashMap<Pos, KineticBlock>;

fn crank(face: Face, speed: f32) -> KineticBlock {
    KineticBlock::single_shaft(face).with_source(speed, values::HAND_CRANK_CAPACITY)
}

fn run(world: &World, start: Pos) -> Network {
    solve(world, start, Limits::default())
}

#[test]
fn a_shaft_line_turns_together() {
    let mut w = World::new();
    w.insert([0, 0, 0], crank(Face::new(Axis::X, true), 32.0));
    for x in 1..=5 {
        w.insert([x, 0, 0], KineticBlock::shaft(Axis::X));
    }
    let n = run(&w, [3, 0, 0]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.members.len(), 6);
    for x in 1..=5 {
        assert_eq!(n.speed([x, 0, 0]), 32.0);
    }
    assert_eq!(n.capacity, 32.0 * 8.0);
}

#[test]
fn shafts_side_by_side_do_not_connect() {
    let mut w = World::new();
    w.insert([0, 0, 0], KineticBlock::shaft(Axis::X));
    w.insert([0, 1, 0], KineticBlock::shaft(Axis::X));
    assert_eq!(run(&w, [0, 0, 0]).members.len(), 1);
    // Nor across an end, perpendicular.
    let mut w = World::new();
    w.insert([0, 0, 0], KineticBlock::shaft(Axis::X));
    w.insert([1, 0, 0], KineticBlock::shaft(Axis::Y));
    assert_eq!(run(&w, [0, 0, 0]).members.len(), 1);
}

#[test]
fn a_crank_drives_only_through_its_shaft_end() {
    let mut w = World::new();
    w.insert([0, 0, 0], crank(Face::new(Axis::X, true), 32.0));
    w.insert([-1, 0, 0], KineticBlock::shaft(Axis::X));
    assert_eq!(run(&w, [0, 0, 0]).members.len(), 1);
}

#[test]
fn small_cogs_mesh_and_reverse() {
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        KineticBlock::cogwheel(Axis::Y).with_source(16.0, 64.0),
    );
    w.insert([1, 0, 0], KineticBlock::cogwheel(Axis::Y));
    w.insert([2, 0, 0], KineticBlock::cogwheel(Axis::Y));
    w.insert([2, 0, 1], KineticBlock::cogwheel(Axis::Y));
    let n = run(&w, [0, 0, 0]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.speed([1, 0, 0]), -16.0);
    assert_eq!(n.speed([2, 0, 0]), 16.0);
    assert_eq!(n.speed([2, 0, 1]), -16.0);
}

#[test]
fn small_cogs_on_one_axis_do_not_mesh_end_to_end_but_share_the_shaft() {
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        KineticBlock::cogwheel(Axis::Y).with_source(16.0, 64.0),
    );
    w.insert([0, 1, 0], KineticBlock::cogwheel(Axis::Y));
    assert_eq!(run(&w, [0, 0, 0]).speed([0, 1, 0]), 16.0);
}

#[test]
fn a_square_of_small_cogs_is_consistent() {
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        KineticBlock::cogwheel(Axis::Y).with_source(16.0, 64.0),
    );
    w.insert([1, 0, 0], KineticBlock::cogwheel(Axis::Y));
    w.insert([1, 0, 1], KineticBlock::cogwheel(Axis::Y));
    w.insert([0, 0, 1], KineticBlock::cogwheel(Axis::Y));
    let n = run(&w, [0, 0, 0]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.speed([1, 0, 1]), 16.0);
}

#[test]
fn large_to_small_cog_doubles_and_reverses() {
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        KineticBlock::large_cogwheel(Axis::X).with_source(8.0, 32.0),
    );
    w.insert([0, 1, 1], KineticBlock::cogwheel(Axis::X));
    w.insert([0, -1, 1], KineticBlock::cogwheel(Axis::X));
    let n = run(&w, [0, 1, 1]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.speed([0, 1, 1]), -16.0);
    assert_eq!(n.speed([0, -1, 1]), -16.0);
    // And back down: small drives large at half speed.
    let mut w = World::new();
    w.insert([0, 0, 0], KineticBlock::large_cogwheel(Axis::X));
    w.insert(
        [0, 1, 1],
        KineticBlock::cogwheel(Axis::X).with_source(16.0, 16.0),
    );
    assert_eq!(run(&w, [0, 0, 0]).speed([0, 0, 0]), -8.0);
}

#[test]
fn small_cog_beside_a_large_cog_does_not_mesh() {
    let mut w = World::new();
    w.insert([0, 0, 0], KineticBlock::large_cogwheel(Axis::X));
    w.insert([0, 1, 0], KineticBlock::cogwheel(Axis::X));
    assert_eq!(run(&w, [0, 0, 0]).members.len(), 1);
}

#[test]
fn perpendicular_large_cogs_mesh() {
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        KineticBlock::large_cogwheel(Axis::X).with_source(8.0, 32.0),
    );
    w.insert([1, 0, 1], KineticBlock::large_cogwheel(Axis::Z));
    w.insert([1, 0, -1], KineticBlock::large_cogwheel(Axis::Z));
    let n = run(&w, [0, 0, 0]);
    assert_eq!(n.speed([1, 0, 1]), 8.0);
    assert_eq!(n.speed([1, 0, -1]), -8.0);
}

#[test]
fn gearbox_turns_corners_and_reverses_straight_through() {
    let mut w = World::new();
    w.insert([-1, 0, 0], crank(Face::new(Axis::X, true), 32.0));
    w.insert([0, 0, 0], KineticBlock::gearbox(Axis::Y));
    w.insert([1, 0, 0], KineticBlock::shaft(Axis::X));
    w.insert([0, 0, 1], KineticBlock::shaft(Axis::Z));
    w.insert([0, 0, -1], KineticBlock::shaft(Axis::Z));
    // Its own axis has no port.
    w.insert([0, 1, 0], KineticBlock::shaft(Axis::Y));
    let n = run(&w, [-1, 0, 0]);
    assert_eq!(n.state, State::Running);
    assert!(!n.contains([0, 1, 0]));
    // Straight through: reversed.
    assert_eq!(n.speed([1, 0, 0]), -32.0);
    // Corners: the two side shafts turn opposite ways about Z.
    assert_eq!(n.speed([0, 0, 1]).abs(), 32.0);
    assert_eq!(n.speed([0, 0, 1]), -n.speed([0, 0, -1]));
}

#[test]
fn a_gear_train_through_gearboxes_matches_meshing_cogs() {
    // Two meshing cogs, also joined at their +Z ends through two gearboxes:
    // real gears agree, so this must not conflict.
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        KineticBlock::cogwheel(Axis::Z).with_source(16.0, 64.0),
    );
    w.insert([1, 0, 0], KineticBlock::cogwheel(Axis::Z));
    w.insert([0, 0, 1], KineticBlock::gearbox(Axis::Y));
    w.insert([1, 0, 1], KineticBlock::gearbox(Axis::Y));
    let n = run(&w, [0, 0, 0]);
    assert_eq!(n.state, State::Running, "{n:?}");
    assert_eq!(n.speed([1, 0, 0]), -16.0);
}

#[test]
fn a_loop_with_mismatched_ratios_conflicts() {
    // Large cog L meshes small cog S at 2:1, and their +X shafts are also
    // joined 1:1 through three gearboxes: the gears would lock.
    let mut w = World::new();
    w.insert(
        [0, 0, 0],
        KineticBlock::large_cogwheel(Axis::X).with_source(8.0, 32.0),
    );
    w.insert([0, 1, 1], KineticBlock::cogwheel(Axis::X));
    w.insert([1, 0, 0], KineticBlock::gearbox(Axis::Z)); // -X in, +Y out
    w.insert([1, 1, 0], KineticBlock::gearbox(Axis::X)); // -Y in, +Z out
    w.insert([1, 1, 1], KineticBlock::gearbox(Axis::Y)); // -Z in, -X out
    let n = run(&w, [0, 0, 0]);
    assert!(matches!(n.state, State::Conflict(..)), "{n:?}");
    assert!(n.speeds().all(|(_, s)| s == 0.0));
}

#[test]
fn opposite_sources_conflict() {
    let mut w = World::new();
    w.insert([0, 0, 0], crank(Face::new(Axis::X, true), 32.0));
    w.insert([1, 0, 0], KineticBlock::shaft(Axis::X));
    w.insert([2, 0, 0], crank(Face::new(Axis::X, false), -32.0));
    assert!(matches!(run(&w, [1, 0, 0]).state, State::Conflict(..)));
}

#[test]
fn the_fastest_source_drives_and_capacities_add() {
    let mut w = World::new();
    w.insert([0, 0, 0], crank(Face::new(Axis::X, true), 32.0));
    w.insert([1, 0, 0], KineticBlock::shaft(Axis::X));
    w.insert(
        [2, 0, 0],
        KineticBlock::shaft(Axis::X).with_source(8.0, values::WATER_WHEEL_CAPACITY),
    );
    let n = run(&w, [1, 0, 0]);
    assert_eq!(n.state, State::Running);
    assert_eq!(n.speed([2, 0, 0]), 32.0);
    assert_eq!(n.capacity, 32.0 * 8.0 + 8.0 * 32.0);
}

#[test]
fn too_much_stress_stops_everything() {
    let mut w = World::new();
    w.insert([0, 0, 0], crank(Face::new(Axis::X, true), 32.0)); // 256 SU
    w.insert([1, 0, 0], KineticBlock::shaft(Axis::X));
    w.insert(
        [2, 0, 0],
        KineticBlock::shaft(Axis::X).with_impact(values::MECHANICAL_PRESS_IMPACT),
    );
    let n = run(&w, [0, 0, 0]);
    assert_eq!(n.stress, 256.0);
    assert_eq!(n.state, State::Running); // exactly at capacity still runs
    w.insert(
        [3, 0, 0],
        KineticBlock::shaft(Axis::X).with_impact(values::MILLSTONE_IMPACT),
    );
    let n = run(&w, [0, 0, 0]);
    assert_eq!(n.state, State::Overstressed);
    assert_eq!(n.speed([2, 0, 0]), 0.0);
    assert_eq!(n.solved_speed([2, 0, 0]), 32.0);
}

#[test]
fn speeding_up_past_the_limit_is_too_fast() {
    // 128 RPM doubled by a large→small cog is 256: allowed. 256 doubled is not.
    for (speed, ok) in [(128.0, true), (256.0, false)] {
        let mut w = World::new();
        w.insert(
            [0, 0, 0],
            KineticBlock::large_cogwheel(Axis::X).with_source(speed, 64.0),
        );
        w.insert([0, 1, 1], KineticBlock::cogwheel(Axis::X));
        let n = run(&w, [0, 0, 0]);
        assert_eq!(n.state == State::Running, ok, "{speed}: {n:?}");
        if !ok {
            assert_eq!(n.state, State::TooFast([0, 1, 1]));
        }
    }
}

#[test]
fn no_source_is_still() {
    let mut w = World::new();
    w.insert([0, 0, 0], KineticBlock::shaft(Axis::X));
    w.insert([1, 0, 0], KineticBlock::shaft(Axis::X));
    let n = run(&w, [0, 0, 0]);
    assert_eq!(n.state, State::Still);
    assert_eq!(n.members.len(), 2);
}

#[test]
fn nothing_at_start_is_empty() {
    assert!(run(&World::new(), [0, 0, 0]).members.is_empty());
}

#[test]
fn oversized_networks_stop() {
    let mut w = World::new();
    for x in 0..20 {
        w.insert([x, 0, 0], KineticBlock::shaft(Axis::X));
    }
    let limits = Limits {
        max_members: 10,
        ..Limits::default()
    };
    assert_eq!(solve(&w, [0, 0, 0], limits).state, State::TooLarge);
}

#[test]
fn every_connection_is_symmetric() {
    let mut kinds = Vec::new();
    for axis in Axis::ALL {
        kinds.push(KineticBlock::shaft(axis));
        kinds.push(KineticBlock::cogwheel(axis));
        kinds.push(KineticBlock::large_cogwheel(axis));
        kinds.push(KineticBlock::gearbox(axis));
    }
    for face in Face::ALL {
        kinds.push(KineticBlock::single_shaft(face));
    }
    let mut connected = 0;
    for a in &kinds {
        for b in &kinds {
            for d in NEIGHBOUR_OFFSETS {
                let back = [-d[0], -d[1], -d[2]];
                let (ab, ba) = (ratio(a, b, d), ratio(b, a, back));
                assert_eq!(ab == 0.0, ba == 0.0, "{a:?} {b:?} {d:?}");
                if ab != 0.0 {
                    connected += 1;
                    assert_eq!(ab * ba, 1.0, "{a:?} {b:?} {d:?}");
                }
            }
        }
    }
    assert!(connected > 0);
}

#[test]
fn angles_wrap() {
    assert_eq!(angle_degrees(0.0, 1000.0), 0.0);
    assert_eq!(angle_degrees(60.0, 20.0), 360.0 * 0.0); // one turn a second
    assert!((angle_degrees(1.0, 10.0) - 3.0).abs() < 1e-4);
    assert!((angle_degrees(-1.0, 10.0) - 357.0).abs() < 1e-4);
}
