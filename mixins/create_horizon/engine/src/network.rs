//! Solving a whole connected network of kinetic blocks at once.
//!
//! Instead of patching speeds block by block as things change, a network is
//! re-solved from scratch whenever one of its blocks changes:
//!
//! 1. Walk every connected block from the changed one, giving each a factor:
//!    its speed relative to the first block's. A loop that comes back with a
//!    different factor is a **conflict** (the gears would lock).
//! 2. Each source asks for a speed of the first block: its own speed over its
//!    factor. Sources asking for opposite directions conflict; otherwise the
//!    fastest one drives the network.
//! 3. Capacity is what every source gives at its own speed, stress what every
//!    block costs at the speed it turns. More stress than capacity stops the
//!    network (**overstressed**), though its speeds are kept for display.

use std::collections::{HashMap, VecDeque};

use crate::block::KineticBlock;
use crate::connect::ratio;
use crate::geometry::{NEIGHBOUR_OFFSETS, Pos, add, sub};

/// Read access to the kinetic blocks of a world.
pub trait KineticWorld {
    /// The kinetic block at `pos`, if there is one (and its chunk is loaded).
    fn kinetic(&self, pos: Pos) -> Option<KineticBlock>;
}

impl KineticWorld for HashMap<Pos, KineticBlock> {
    fn kinetic(&self, pos: Pos) -> Option<KineticBlock> {
        self.get(&pos).copied()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    /// The highest RPM any block may turn at.
    pub max_speed: f32,
    /// The most blocks one network may have; past it the walk stops.
    pub max_members: usize,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            max_speed: 256.0,
            max_members: 8192,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    /// No source, or every source is still.
    Still,
    Running,
    /// More stress than capacity: everything stops.
    Overstressed,
    /// Two connections disagree (a locked loop, or sources turning opposite
    /// ways): the pair of blocks where it showed. Everything stops.
    Conflict(Pos, Pos),
    /// A block would turn faster than `Limits::max_speed`. Everything stops.
    TooFast(Pos),
    /// More than `Limits::max_members` blocks. Everything stops.
    TooLarge,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Network {
    /// Every member and the speed it would turn at, in the order found.
    pub members: Vec<(Pos, f32)>,
    /// Stress capacity, SU.
    pub capacity: f32,
    /// Stress applied, SU.
    pub stress: f32,
    pub state: State,
}

impl Network {
    /// The speed `pos` actually turns at: its solved speed while running,
    /// 0 when stopped for any reason.
    pub fn speed(&self, pos: Pos) -> f32 {
        if self.state != State::Running {
            return 0.0;
        }
        self.solved_speed(pos)
    }

    /// The speed `pos` would turn at if nothing stopped the network.
    pub fn solved_speed(&self, pos: Pos) -> f32 {
        self.members
            .iter()
            .find(|(p, _)| *p == pos)
            .map_or(0.0, |(_, s)| *s)
    }

    pub fn contains(&self, pos: Pos) -> bool {
        self.members.iter().any(|(p, _)| *p == pos)
    }

    /// Every member and the speed it actually turns at.
    pub fn speeds(&self) -> impl Iterator<Item = (Pos, f32)> + '_ {
        let running = self.state == State::Running;
        self.members
            .iter()
            .map(move |&(p, s)| (p, if running { s } else { 0.0 }))
    }
}

/// Relative speeds equal within this tolerance (ratios are powers of two, so
/// real loops agree exactly; this only absorbs rounding).
fn agrees(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-4 * a.abs().max(b.abs()).max(1.0)
}

/// Solve the network holding `start`. An empty network when `start` holds no
/// kinetic block.
pub fn solve(world: &impl KineticWorld, start: Pos, limits: Limits) -> Network {
    let mut network = Network {
        members: Vec::new(),
        capacity: 0.0,
        stress: 0.0,
        state: State::Still,
    };
    if world.kinetic(start).is_none() {
        return network;
    }

    // 1. Factors relative to `start`.
    let mut factor: HashMap<Pos, f32> = HashMap::new();
    let mut blocks: Vec<(Pos, KineticBlock)> = Vec::new();
    let mut queue = VecDeque::from([start]);
    factor.insert(start, 1.0);
    let mut conflict = None;
    while let Some(pos) = queue.pop_front() {
        let Some(block) = world.kinetic(pos) else {
            continue;
        };
        blocks.push((pos, block));
        if blocks.len() > limits.max_members {
            network.state = State::TooLarge;
            network.members = blocks.iter().map(|(p, _)| (*p, 0.0)).collect();
            return network;
        }
        let here = factor[&pos];
        for offset in NEIGHBOUR_OFFSETS {
            let next = add(pos, offset);
            let Some(other) = world.kinetic(next) else {
                continue;
            };
            let r = ratio(&block, &other, sub(next, pos));
            if r == 0.0 {
                continue;
            }
            let expected = here * r;
            match factor.get(&next) {
                Some(&known) => {
                    if !agrees(known, expected) && conflict.is_none() {
                        conflict = Some(State::Conflict(pos, next));
                    }
                }
                None => {
                    factor.insert(next, expected);
                    queue.push_back(next);
                }
            }
        }
    }

    // 2. The speed of `start` the sources ask for.
    let mut drive: Option<(Pos, f32)> = None;
    for (pos, block) in &blocks {
        let Some(source) = block.source else {
            continue;
        };
        if source.speed == 0.0 {
            continue;
        }
        network.capacity += source.capacity_per_rpm * source.speed.abs();
        let wanted = source.speed / factor[pos];
        match drive {
            None => drive = Some((*pos, wanted)),
            Some((other, current)) => {
                if current.signum() != wanted.signum() {
                    conflict.get_or_insert(State::Conflict(other, *pos));
                } else if wanted.abs() > current.abs() {
                    drive = Some((*pos, wanted));
                }
            }
        }
    }
    let root = drive.map_or(0.0, |(_, s)| s);

    // 3. Speeds and stress.
    let mut fastest: Option<(Pos, f32)> = None;
    for (pos, block) in &blocks {
        let speed = root * factor[pos];
        network.members.push((*pos, speed));
        network.stress += block.impact_per_rpm * speed.abs();
        if fastest.is_none_or(|(_, s)| speed.abs() > s) {
            fastest = Some((*pos, speed.abs()));
        }
    }

    network.state = if let Some(conflict) = conflict {
        conflict
    } else if root == 0.0 {
        State::Still
    } else if let Some((pos, _)) = fastest.filter(|&(_, s)| s > limits.max_speed) {
        State::TooFast(pos)
    } else if network.stress > network.capacity {
        State::Overstressed
    } else {
        State::Running
    };
    network
}
