// The mod's game-independent glue. This file is both the standalone Cargo library
// and the injected `crate::game::create_horizon` module; no generated copy exists.
// Everything that names a game type lives in the hook helpers beside it.

use create_horizon_engine::{Axis, Face, KineticBlock, KineticWorld, Limits, Pos, solve, values};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;

pub const SHAFT: &str = "create:shaft";
pub const COGWHEEL: &str = "create:cogwheel";
pub const LARGE_COGWHEEL: &str = "create:large_cogwheel";
pub const GEARBOX: &str = "create:gearbox";
pub const HAND_CRANK: &str = "create:hand_crank";
pub const CREATIVE_MOTOR: &str = "create:creative_motor";

/// Where Horizon's carry step leaves Create, from the game folder.
pub const CARRIED_CREATE: &str = "horizon-mods/create";

/// The state properties as declared. A block's states are its template's times its own, and
/// the registry wants properties in name order, so a declared one must sort after the
/// template's `waterlogged`: `axis` and `facing` would stop the game loading.
pub const AXIS_KEY: &str = "zaxis";
pub const FACING_KEY: &str = "zfacing";

/// The saved compound's keys.
pub const SPEED_KEY: &str = "speed";
pub const CRANK_KEY: &str = "crank";

const SIDES: &str = r#"["down", "up", "north", "south", "west", "east"]"#;
const AXES: &str = r#"["y", "x", "z"]"#;

/// Shapes can only select on a template property, and the conduit has no axis, so a
/// shaft's box is the same cross for every axis until Horizon can select on a declared one.
const CROSS: &str = "[[0.3125, 0.3125, 0, 0.6875, 0.6875, 1], \
                     [0.3125, 0, 0.3125, 0.6875, 1, 0.6875], \
                     [0, 0.3125, 0.3125, 1, 0.6875, 0.6875]]";
const COG_BOX: &str = "[[0.125, 0.125, 0.125, 0.875, 0.875, 0.875]]";
const FULL_BOX: &str = "[[0, 0, 0, 1, 1, 1]]";

fn declaration(property: &str, values: &str, boxes: &str) -> String {
    // `minecraft:conduit`: it has a block entity, opens no menu and leaves the chunk mesh
    // empty, so the rotating model drawn by the block entity is the only one seen.
    format!(
        "{{\n  \"template\": \"minecraft:conduit\",\n  \
         \"state_properties\": {{ \"{property}\": {{ \"type\": \"enum\", \"values\": {values} }} }},\n  \
         \"default_state\": {{ \"waterlogged\": \"false\" }},\n  \
         \"shapes\": {{ \"property\": \"waterlogged\", \"values\": {{ \"true\": {boxes}, \"false\": {boxes} }} }}\n}}\n"
    )
}

/// What the carried declarations of the kinetic blocks are replaced with: the carry step
/// picks log and stairs templates, which have no block entity and draw a still model.
pub fn declarations() -> [(&'static str, String); 6] {
    [
        ("shaft", declaration(AXIS_KEY, AXES, CROSS)),
        ("cogwheel", declaration(AXIS_KEY, AXES, COG_BOX)),
        ("large_cogwheel", declaration(AXIS_KEY, AXES, FULL_BOX)),
        ("gearbox", declaration(AXIS_KEY, AXES, FULL_BOX)),
        ("hand_crank", declaration(FACING_KEY, SIDES, COG_BOX)),
        ("creative_motor", declaration(FACING_KEY, SIDES, FULL_BOX)),
    ]
}

/// A carried blockstate file with its variants keyed on the declared property names.
fn renamed_variants(text: &str) -> String {
    let mut out = text.to_owned();
    for (from, to) in [("axis=", AXIS_KEY), ("facing=", FACING_KEY)] {
        for lead in ['"', ','] {
            out = out.replace(&format!("{lead}{from}"), &format!("{lead}{to}="));
        }
    }
    out
}

fn write_if_changed(path: &Path, text: &str) -> std::io::Result<usize> {
    if std::fs::read_to_string(path).is_ok_and(|old| old == text) {
        return Ok(0);
    }
    std::fs::write(path, text)?;
    Ok(1)
}

/// Rewrites the carried declarations and blockstates that differ. Runs before content loads; a
/// folder without Create changes nothing, and a fresh carry is patched again at the next start.
pub fn patch_carried_blocks(create: &Path) -> std::io::Result<usize> {
    let blocks = create.join("data/create/horizon/block");
    if !blocks.is_dir() {
        return Ok(0);
    }
    let blockstates = create.join("assets/create/blockstates");
    let mut changed = 0;
    for (name, text) in declarations() {
        changed += write_if_changed(&blocks.join(format!("{name}.json")), &text)?;
        let path = blockstates.join(format!("{name}.json"));
        if let Ok(old) = std::fs::read_to_string(&path) {
            changed += write_if_changed(&path, &renamed_variants(&old))?;
        }
    }
    Ok(changed)
}

pub fn is_kinetic(block: &str) -> bool {
    matches!(
        block,
        SHAFT | COGWHEEL | LARGE_COGWHEEL | GEARBOX | HAND_CRANK | CREATIVE_MOTOR
    )
}

/// The simulation's view of a placed block. `property` reads the block's state;
/// `cranked` says a hand crank is being turned.
pub fn kinetic_block(
    block: &str,
    property: impl Fn(&str) -> Option<String>,
    cranked: bool,
) -> Option<KineticBlock> {
    let axis = || Axis::from_name(&property(AXIS_KEY)?);
    let facing = || Face::from_direction_name(&property(FACING_KEY)?);
    Some(match block {
        SHAFT => KineticBlock::shaft(axis()?),
        COGWHEEL => KineticBlock::cogwheel(axis()?),
        LARGE_COGWHEEL => KineticBlock::large_cogwheel(axis()?),
        GEARBOX => KineticBlock::gearbox(axis()?),
        HAND_CRANK => {
            let crank = KineticBlock::single_shaft(facing()?.opposite());
            if cranked {
                crank.with_source(values::HAND_CRANK_SPEED, values::HAND_CRANK_CAPACITY)
            } else {
                crank
            }
        }
        CREATIVE_MOTOR => KineticBlock::single_shaft(facing()?).with_source(
            values::CREATIVE_MOTOR_SPEED,
            values::CREATIVE_MOTOR_CAPACITY,
        ),
        _ => return None,
    })
}

/// The state property a clicked face gives a placed block, as (key, value).
pub fn placement_property(block: &str, face: [i32; 3]) -> Option<(&'static str, &'static str)> {
    let direction = match face {
        [0, -1, 0] => ("down", "y"),
        [0, 1, 0] => ("up", "y"),
        [0, 0, -1] => ("north", "z"),
        [0, 0, 1] => ("south", "z"),
        [-1, 0, 0] => ("west", "x"),
        [1, 0, 0] => ("east", "x"),
        _ => return None,
    };
    match block {
        SHAFT | COGWHEEL | LARGE_COGWHEEL | GEARBOX => Some((AXIS_KEY, direction.1)),
        HAND_CRANK | CREATIVE_MOTOR => Some((FACING_KEY, direction.0)),
        _ => None,
    }
}

/// A rotation axis as a vector, for the pose stack.
pub fn axis_vector(axis: &str) -> [f32; 3] {
    match axis {
        "x" => [1.0, 0.0, 0.0],
        "z" => [0.0, 0.0, 1.0],
        _ => [0.0, 1.0, 0.0],
    }
}

/// A cogwheel's angle offset: neighbours sit half a tooth apart (eight teeth, so 22.5°)
/// so that their teeth mesh. From the gear geometry.
pub fn cog_offset_degrees(block: &str, axis: &str, pos: Pos) -> f32 {
    if block != COGWHEEL {
        return 0.0;
    }
    let along = match axis {
        "x" => pos[0],
        "z" => pos[2],
        _ => pos[1],
    };
    if (pos[0] + pos[1] + pos[2] - along).rem_euclid(2) == 1 {
        22.5
    } else {
        0.0
    }
}

#[derive(Default)]
struct Level {
    blocks: HashMap<Pos, (KineticBlock, u64)>,
    speeds: HashMap<Pos, f32>,
    dirty: bool,
}

impl KineticWorld for Level {
    fn kinetic(&self, pos: Pos) -> Option<KineticBlock> {
        self.blocks.get(&pos).map(|entry| entry.0)
    }
}

impl Level {
    fn observe(&mut self, pos: Pos, block: KineticBlock, tick: u64) -> f32 {
        let old = self.blocks.insert(pos, (block, tick));
        if old.is_none_or(|(previous, _)| previous != block) {
            self.dirty = true;
        }
        self.speeds.get(&pos).copied().unwrap_or(0.0)
    }

    /// Forgets the blocks whose block entity stopped ticking (broken, or unloaded).
    fn prune(&mut self, tick: u64) {
        let before = self.blocks.len();
        self.blocks.retain(|_, entry| entry.1 + 1 >= tick);
        self.dirty |= self.blocks.len() != before;
    }

    fn solve_all(&mut self) {
        self.dirty = false;
        let mut speeds = HashMap::with_capacity(self.blocks.len());
        let mut visited = HashSet::with_capacity(self.blocks.len());
        for &pos in self.blocks.keys() {
            if visited.contains(&pos) {
                continue;
            }
            let network = solve(self, pos, Limits::default());
            for (member, speed) in network.speeds() {
                visited.insert(member);
                speeds.insert(member, speed);
            }
        }
        self.speeds = speeds;
    }
}

/// Every level's kinetic blocks, filled by their block entities' ticks. It holds nothing
/// that isn't in the world: the blocks tell it again each tick.
#[derive(Default)]
pub struct Grid {
    levels: BTreeMap<String, Level>,
    tick: u64,
}

impl Grid {
    /// A block entity's tick: tells the grid its block and gets the speed to turn at.
    pub fn observe(&mut self, dimension: &str, pos: Pos, block: KineticBlock) -> f32 {
        let tick = self.tick;
        if !self.levels.contains_key(dimension) {
            self.levels.insert(dimension.to_owned(), Level::default());
        }
        self.levels
            .get_mut(dimension)
            .map_or(0.0, |level| level.observe(pos, block, tick))
    }

    /// The start of a server tick: re-solves the levels whose blocks changed.
    pub fn begin_tick(&mut self) {
        self.tick += 1;
        for level in self.levels.values_mut() {
            level.prune(self.tick);
            if level.dirty {
                level.solve_all();
            }
        }
    }
}

static GRID: Mutex<Option<Grid>> = Mutex::new(None);

pub fn with_grid<R>(change: impl FnOnce(&mut Grid) -> R) -> R {
    let mut grid = GRID.lock().unwrap_or_else(|e| e.into_inner());
    change(grid.get_or_insert_with(Grid::default))
}

/// The face of the last right click with an item: `native_place_block` isn't told it.
static LAST_FACE: Mutex<[i32; 3]> = Mutex::new([0, 1, 0]);

pub fn remember_face(face: [i32; 3]) {
    *LAST_FACE.lock().unwrap_or_else(|e| e.into_inner()) = face;
}

pub fn last_face() -> [i32; 3] {
    *LAST_FACE.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis_y(key: &str) -> Option<String> {
        (key == AXIS_KEY).then(|| "y".to_owned())
    }

    fn facing_up(key: &str) -> Option<String> {
        (key == FACING_KEY).then(|| "up".to_owned())
    }

    #[test]
    fn a_motor_turns_a_shaft_above_it_once_the_tick_has_solved() {
        let mut grid = Grid::default();
        let motor = kinetic_block(CREATIVE_MOTOR, facing_up, false).unwrap();
        let shaft = kinetic_block(SHAFT, axis_y, false).unwrap();
        assert_eq!(grid.observe("o", [0, 0, 0], motor), 0.0);
        assert_eq!(grid.observe("o", [0, 1, 0], shaft), 0.0);
        grid.begin_tick();
        assert_eq!(
            grid.observe("o", [0, 1, 0], shaft),
            values::CREATIVE_MOTOR_SPEED
        );
    }

    #[test]
    fn a_broken_source_stops_the_network() {
        let mut grid = Grid::default();
        let motor = kinetic_block(CREATIVE_MOTOR, facing_up, false).unwrap();
        let shaft = kinetic_block(SHAFT, axis_y, false).unwrap();
        grid.observe("o", [0, 0, 0], motor);
        grid.observe("o", [0, 1, 0], shaft);
        grid.begin_tick();
        // The motor's entity stops ticking; the shaft's goes on.
        grid.observe("o", [0, 1, 0], shaft);
        grid.begin_tick();
        grid.observe("o", [0, 1, 0], shaft);
        grid.begin_tick();
        assert_eq!(grid.observe("o", [0, 1, 0], shaft), 0.0);
    }

    #[test]
    fn levels_do_not_share_blocks() {
        let mut grid = Grid::default();
        let motor = kinetic_block(CREATIVE_MOTOR, facing_up, false).unwrap();
        let shaft = kinetic_block(SHAFT, axis_y, false).unwrap();
        grid.observe("a", [0, 0, 0], motor);
        grid.observe("b", [0, 1, 0], shaft);
        grid.begin_tick();
        assert_eq!(grid.observe("b", [0, 1, 0], shaft), 0.0);
    }

    #[test]
    fn a_hand_crank_is_a_source_only_while_turned() {
        assert!(
            kinetic_block(HAND_CRANK, facing_up, true)
                .unwrap()
                .source
                .is_some()
        );
        assert!(
            kinetic_block(HAND_CRANK, facing_up, false)
                .unwrap()
                .source
                .is_none()
        );
    }

    #[test]
    fn placing_on_a_face_sets_the_blocks_property() {
        assert_eq!(placement_property(SHAFT, [1, 0, 0]), Some((AXIS_KEY, "x")));
        assert_eq!(
            placement_property(CREATIVE_MOTOR, [0, 0, -1]),
            Some((FACING_KEY, "north"))
        );
        assert_eq!(placement_property("minecraft:stone", [0, 1, 0]), None);
    }

    #[test]
    fn neighbouring_cogwheels_sit_half_a_tooth_apart() {
        let a = cog_offset_degrees(COGWHEEL, "y", [0, 64, 0]);
        let b = cog_offset_degrees(COGWHEEL, "y", [1, 64, 0]);
        assert_eq!((a - b).abs(), 22.5);
    }

    #[test]
    fn variants_take_the_declared_names_once() {
        let once = renamed_variants(r#"{"variants": {"axis=x": {}, "half=top,facing=up": {}}}"#);
        assert_eq!(
            once,
            r#"{"variants": {"zaxis=x": {}, "half=top,zfacing=up": {}}}"#
        );
        assert_eq!(renamed_variants(&once), once);
    }

    #[test]
    fn patching_writes_each_declaration_once() {
        let dir = std::env::temp_dir().join(format!("create_horizon_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("data/create/horizon/block")).unwrap();
        assert_eq!(patch_carried_blocks(&dir).unwrap(), 6);
        assert_eq!(patch_carried_blocks(&dir).unwrap(), 0);
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(patch_carried_blocks(&dir).unwrap(), 0);
    }
}
