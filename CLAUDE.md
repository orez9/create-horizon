# create_horizon: Create-style mechanics for Horizon

A native Horizon mod (horizon-mixins bundle) that gives the blocks of the Create mod their behaviour in
Horizon (Minecraft Java 26.3 rewritten in Rust). Goal: **gameplay 1:1 with Create 6.0.x**, so Create
players know it at once. One mod for client and server (single player, LAN, Horizon dedicated server).

## Decisions (do not re-litigate)

- **Clean-room code.** Create's maintainers ask AI tools not to read or work on their source
  (their AGENTS.md). Never open, fetch, quote or translate Create's Java source. Behaviour comes from the
  public Create wiki, in-game observation, and gear geometry. Record the source of each mechanic.
- **Assets and names are Create's own, carried at runtime.** Create's assets are All Rights Reserved: never
  commit textures, models, lang files or jars. The player installs the official Create jar through Horizon's
  mod browser; Horizon's `carry` step (docs/modpacks.md in Horizon) turns its `assets/create/` and
  `data/create/` into a content mod. This mod adds behaviour to those `create:*` block ids.
- **Horizon clients only.** Servers running this mod set `horizon-clients-only = true`.
- Pin Horizon Nightlies (`compatibility.json`). Prefer hooks via `native-hooks.json` names.

## Layout

- `mixins/create_horizon/engine/` — crate `create_horizon_engine`: the whole simulation, no game types, no
  dependencies. `geometry.rs` (Pos/Axis/Face), `block.rs` (KineticBlock), `connect.rs` (ratios between two
  blocks), `network.rs` (`solve`: whole-network re-solve), `values.rs` (SU/RPM numbers, to verify), `tests.rs`.
- `mixins/create_horizon/horizon-mixin.toml`, `*.rs` helpers — thin glue to Horizon hooks (TODO).
- `runtime.rs` — template leftover; becomes the injected glue module.

## Speed convention

Signed RPM about the block's axis, right-hand rule. Coaxial touching shafts share the signed value.
Gearbox port speed = W · sign(face) · σ(face.axis), σ = +1 for the first other axis, -1 for the second.

## Status / next steps

1. ✅ Engine: shafts, cogwheels, large cogwheels, gearboxes, sources, stress, overstress, conflicts, speed
   limit (21 tests).
2. Glue: map `create:*` blocks + state (`axis`, `facing`) → KineticBlock; block entities
   (`native_block_entity`, `_ticks`, `_tick`), re-solve on place/break/neighbour change
   (`native_place_block`, `native_destroy_block`, `native_neighbor_changed`), write speed into the BE nbt,
   sync (`native_block_entity_update_tag`). On conflict / too fast: break the placed block with drops (as
   Create does).
3. Render: `native_block_entity_submit` rotating the carried Create models (angle = `angle_degrees`).
4. Hand crank (use → spins for a few ticks), water wheel, creative motor; goggles tooltip (stress/RPM).
5. Machines: press, millstone, fan processing, then belts/depots, then contraptions.

## Validate

```sh
cargo fmt --all -- --check
cargo clippy -p create_horizon_engine --all-targets -- -D warnings
cargo test -p create_horizon_engine
```

Horizon's API reference: `docs/native-mods.md` and `crates/horizon-client/src/game/native_hooks.rs`
(API version `NATIVE_HOOKS_API`) in the Horizon checkout beside this repo. Read only the sections needed.
