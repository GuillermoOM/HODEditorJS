# HOD 2.0 Testing Guide

This guide documents the three parser tools that exist in `parser/src/bin/`.
Run the commands from the repository root with Rust and Cargo installed.

## Prerequisites

- Rust and Cargo must be available on `PATH`.
- The validation suite needs the external `testing/pebble_0/`,
  `testing/pebble_1/`, and `testing/pebble_2/` fixture directories. See
  `docs/testing-fixtures.md` for how to obtain the corpus.
- `verify_lossless` needs the maintainer's game installation at the absolute
  paths listed below. Those paths are machine-specific and are not included
  in this repository.

## `verify_lossless`

Purpose: run the AGENTS.md-mandated lossless round-trip gate against the
maintainer's HOD and DAE samples. It parses each input, generates a HOD, and
checks the reparsed structure.

Invocation:

```bash
cargo run --bin verify_lossless --manifest-path parser/Cargo.toml
```

Inputs: the game installation must provide all six paths hard-coded in
`parser/src/bin/verify_lossless.rs:6-11`:

```text
/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/pebble/pebble_0/pebble_0.hod
/run/media/system/Data/SteamLibrary/common/Homeworld/HWRM_FSFC/source/ship/ter_elysium/ter_elysium.hod
/run/media/system/Data/SteamLibrary/common/Homeworld/HWRM_FSFC/source/ship/ter_fenris/ter_fenris.hod
/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/freespace_remastered/resource/asteroid/asteroid_3/asteroid_3.hod
/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/effect/galaxymap/hodsunpacked/mapgalaxy/galaxymapgalaxy.dae
/run/media/system/Data/SteamLibrary/steamapps/common/Homeworld 347380/GBXTools/WorkshopTool/uncompressed_bigs/HWRM/ship/hgn_mothership/hgn_mothership.hod
```

## `validation_suite`

Purpose: validate the source-asset HOD 2.0 pipeline for `pebble_0`, `pebble_1`,
and `pebble_2`. Each case reads its vanilla HOD and OBJ, TGA, and material
inputs, generates a validation HOD, reparses it, compares structures, and
removes the generated file.

Invocation:

```bash
cargo run --bin validation_suite --manifest-path parser/Cargo.toml
```

Inputs: the external `testing/pebble_0/`, `testing/pebble_1/`, and
`testing/pebble_2/` directories, with the assets expected by the harness.
The acquisition instructions will live at `docs/testing-fixtures.md`.

## `dump_pool`

Purpose: inspect a HOD's top-level chunks, POOL header sizes, and related
material and texture chunks. This is an inspector, not a pass/fail gate.

Invocation:

```bash
cargo run --bin dump_pool --manifest-path parser/Cargo.toml -- /path/to/file.hod
```

Inputs: one HOD path argument. It needs no game installation or external
fixture corpus.

## Missing Game Installation

If the game installation required by `verify_lossless` is absent, run the
parser tests instead:

```bash
cargo test --manifest-path parser/Cargo.toml
```

State in the session record that `verify_lossless` was not run. Do not claim
that the lossless gate passed when its external inputs were unavailable.

## What the Gates Mean

The gates establish **no divergence from the known baseline**. A baseline may
contain pre-existing warnings or recompression size-difference output; those
lines are not automatically failures. Passing the commands means the current
result did not get worse than the recorded baseline, never that everything is
perfect.
