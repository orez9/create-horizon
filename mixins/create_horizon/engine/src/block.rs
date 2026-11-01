//! What a kinetic block is, as the simulation sees it.
//!
//! Every speed here is a signed angular velocity in RPM about the block's
//! rotation axis, positive when it turns counter-clockwise seen from the
//! axis' positive end (the right-hand rule). Two coaxial shafts that touch
//! therefore always have the same signed speed.

use crate::geometry::{Axis, Face};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cog {
    None,
    /// A cogwheel: one block across, meshes with small cogs beside it.
    Small,
    /// A large cogwheel: meshes with small cogs across an edge (2:1) and
    /// with perpendicular large cogs.
    Large,
}

/// A rotation source's output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Source {
    /// Signed RPM it drives its own axis at (see the module comment).
    pub speed: f32,
    /// Stress capacity in SU per RPM of `speed`.
    pub capacity_per_rpm: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KineticBlock {
    /// The rotation axis. For a gearbox, the axis of its central gear.
    pub axis: Axis,
    /// Faces with a shaft end, as a bit set over `Face::index`.
    pub shaft_faces: u8,
    pub cog: Cog,
    /// A gearbox: its ports are the four faces around its axis, and it turns
    /// each port's rotation through 90° with bevel gears.
    pub gearbox: bool,
    pub source: Option<Source>,
    /// Stress impact in SU per RPM it turns at.
    pub impact_per_rpm: f32,
}

impl KineticBlock {
    fn base(axis: Axis) -> KineticBlock {
        KineticBlock {
            axis,
            shaft_faces: 0,
            cog: Cog::None,
            gearbox: false,
            source: None,
            impact_per_rpm: 0.0,
        }
    }

    fn with_axis_shafts(mut self) -> KineticBlock {
        self.shaft_faces |= 1 << Face::new(self.axis, false).index();
        self.shaft_faces |= 1 << Face::new(self.axis, true).index();
        self
    }

    pub fn shaft(axis: Axis) -> KineticBlock {
        Self::base(axis).with_axis_shafts()
    }

    pub fn cogwheel(axis: Axis) -> KineticBlock {
        KineticBlock {
            cog: Cog::Small,
            ..Self::shaft(axis)
        }
    }

    pub fn large_cogwheel(axis: Axis) -> KineticBlock {
        KineticBlock {
            cog: Cog::Large,
            ..Self::shaft(axis)
        }
    }

    pub fn gearbox(axis: Axis) -> KineticBlock {
        let mut block = Self::base(axis);
        block.gearbox = true;
        for face in Face::ALL {
            if face.axis != axis {
                block.shaft_faces |= 1 << face.index();
            }
        }
        block
    }

    /// A block with a single shaft end, on `face` (a hand crank, a motor).
    pub fn single_shaft(face: Face) -> KineticBlock {
        let mut block = Self::base(face.axis);
        block.shaft_faces = 1 << face.index();
        block
    }

    pub fn with_source(mut self, speed: f32, capacity_per_rpm: f32) -> KineticBlock {
        self.source = Some(Source {
            speed,
            capacity_per_rpm,
        });
        self
    }

    pub fn with_impact(mut self, impact_per_rpm: f32) -> KineticBlock {
        self.impact_per_rpm = impact_per_rpm;
        self
    }

    pub fn has_shaft(&self, face: Face) -> bool {
        self.shaft_faces & (1 << face.index()) != 0
    }

    /// The signed speed of the shaft end on `face` for a block turning at
    /// `1.0`, or 0 when there is none there. Plain blocks pass their own
    /// speed on; a gearbox's port `face` turns at `sign(face) · σ(face.axis)`,
    /// where σ is +1 for the first of the two other axes and -1 for the second:
    /// what mitre gears give (adjacent ports turn opposite ways seen from
    /// outside, so a straight-through pair reverses).
    pub fn port_factor(&self, face: Face) -> f32 {
        if !self.has_shaft(face) {
            return 0.0;
        }
        if !self.gearbox {
            return 1.0;
        }
        let sigma = if face.axis == self.axis.others()[0] {
            1.0
        } else {
            -1.0
        };
        face.sign() as f32 * sigma
    }
}
