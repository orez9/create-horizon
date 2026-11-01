//! How rotation passes between two neighbouring kinetic blocks.
//!
//! Derived from the gears' geometry, not from any other implementation:
//!
//! - **Coaxial shafts** touching face to face turn together: the far block's
//!   port turns as the near block's port does.
//! - **Small cogs** of the same axis side by side mesh at their rims and turn
//!   opposite ways: -1.
//! - A **large cog** (twice a small cog's radius) and a small cog of the same
//!   axis, one block off in both other axes, mesh with a 2:1 ratio, reversed:
//!   the small one turns at -2× the large one.
//! - Two **perpendicular large cogs**, one block off along both their axes,
//!   mesh like bevel gears: the rim velocities at the shared point give
//!   `ω_b = ω_a · sign(Δ_a) · sign(Δ_b)`, Δ the offset along each one's axis.

use crate::block::{Cog, KineticBlock};
use crate::geometry::{Face, Pos};

/// The speed ratio from `a` to `b`, `b` at offset `diff` from `a`:
/// `speed_b = speed_a · ratio`. Zero when they are not connected.
///
/// Symmetric by construction: `ratio(b, a, -diff) == 1 / ratio(a, b, diff)`.
pub fn ratio(a: &KineticBlock, b: &KineticBlock, diff: Pos) -> f32 {
    if let Some(face) = Face::from_offset(diff) {
        let shaft = shaft_ratio(a, b, face);
        if shaft != 0.0 {
            return shaft;
        }
        return small_cog_ratio(a, b, face);
    }
    edge_ratio(a, b, diff)
}

fn shaft_ratio(a: &KineticBlock, b: &KineticBlock, face: Face) -> f32 {
    let from = a.port_factor(face);
    let to = b.port_factor(face.opposite());
    if from == 0.0 || to == 0.0 {
        return 0.0;
    }
    // Both ports turn at the same signed speed: speed_a·from = speed_b·to.
    from / to
}

fn small_cog_ratio(a: &KineticBlock, b: &KineticBlock, face: Face) -> f32 {
    let meshing =
        a.cog == Cog::Small && b.cog == Cog::Small && a.axis == b.axis && face.axis != a.axis;
    if meshing { -1.0 } else { 0.0 }
}

fn edge_ratio(a: &KineticBlock, b: &KineticBlock, diff: Pos) -> f32 {
    match (a.cog, b.cog) {
        (Cog::Large, Cog::Small) if large_to_small(a, b, diff) => -2.0,
        (Cog::Small, Cog::Large) if large_to_small(b, a, diff) => -0.5,
        (Cog::Large, Cog::Large) => large_to_large(a, b, diff),
        _ => 0.0,
    }
}

/// A large cog and a small cog of the same axis, level along it and one
/// block off along both other axes.
fn large_to_small(large: &KineticBlock, small: &KineticBlock, diff: Pos) -> bool {
    large.axis == small.axis
        && large.axis.of(diff) == 0
        && large.axis.others().iter().all(|a| a.of(diff).abs() == 1)
}

fn large_to_large(a: &KineticBlock, b: &KineticBlock, diff: Pos) -> f32 {
    if a.axis == b.axis {
        return 0.0;
    }
    let (da, db) = (a.axis.of(diff), b.axis.of(diff));
    // `diff` is an edge offset: two components are ±1, the third is 0. The
    // non-zero ones must be along the two cogs' axes.
    if da == 0 || db == 0 {
        return 0.0;
    }
    (da.signum() * db.signum()) as f32
}
