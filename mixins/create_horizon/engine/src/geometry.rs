//! Block positions, axes and faces.

/// A block position: x, y, z.
pub type Pos = [i32; 3];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Axis {
    pub const ALL: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    /// This axis' component of `v`.
    pub fn of(self, v: Pos) -> i32 {
        v[self as usize]
    }

    /// The two other axes, in X, Y, Z order.
    pub fn others(self) -> [Axis; 2] {
        match self {
            Axis::X => [Axis::Y, Axis::Z],
            Axis::Y => [Axis::X, Axis::Z],
            Axis::Z => [Axis::X, Axis::Y],
        }
    }

    pub fn from_name(name: &str) -> Option<Axis> {
        match name {
            "x" => Some(Axis::X),
            "y" => Some(Axis::Y),
            "z" => Some(Axis::Z),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
        }
    }
}

/// One of a block's six faces: an axis and a side along it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Face {
    pub axis: Axis,
    pub positive: bool,
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::new(Axis::X, false),
        Face::new(Axis::X, true),
        Face::new(Axis::Y, false),
        Face::new(Axis::Y, true),
        Face::new(Axis::Z, false),
        Face::new(Axis::Z, true),
    ];

    pub const fn new(axis: Axis, positive: bool) -> Face {
        Face { axis, positive }
    }

    /// +1 or -1.
    pub fn sign(self) -> i32 {
        if self.positive { 1 } else { -1 }
    }

    pub fn opposite(self) -> Face {
        Face::new(self.axis, !self.positive)
    }

    /// The unit offset this face points along.
    pub fn offset(self) -> Pos {
        let mut v = [0; 3];
        v[self.axis as usize] = self.sign();
        v
    }

    /// The face `diff` points through, when `diff` is a unit offset.
    pub fn from_offset(diff: Pos) -> Option<Face> {
        Face::ALL.into_iter().find(|f| f.offset() == diff)
    }

    /// Index 0..6, for bit sets.
    pub fn index(self) -> usize {
        self.axis as usize * 2 + self.positive as usize
    }

    /// Minecraft's direction name (`up`, `north`...).
    pub fn from_direction_name(name: &str) -> Option<Face> {
        Some(match name {
            "down" => Face::new(Axis::Y, false),
            "up" => Face::new(Axis::Y, true),
            "north" => Face::new(Axis::Z, false),
            "south" => Face::new(Axis::Z, true),
            "west" => Face::new(Axis::X, false),
            "east" => Face::new(Axis::X, true),
            _ => return None,
        })
    }
}

pub fn add(a: Pos, b: Pos) -> Pos {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn sub(a: Pos, b: Pos) -> Pos {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// The offsets a kinetic block can reach a neighbour through: the 6 faces and
/// the 12 edges of the 3×3×3 cube around it (cogs mesh across edges).
pub const NEIGHBOUR_OFFSETS: [Pos; 18] = [
    [-1, 0, 0],
    [1, 0, 0],
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, -1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [1, 1, 0],
    [-1, 0, -1],
    [-1, 0, 1],
    [1, 0, -1],
    [1, 0, 1],
    [0, -1, -1],
    [0, -1, 1],
    [0, 1, -1],
    [0, 1, 1],
];
