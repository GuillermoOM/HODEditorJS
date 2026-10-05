# Local Environment Convention

Every developer or machine should keep its own **untracked** local-environment file for machine-specific paths, container names, toolchains, and build quirks. This machine's profile is `.omo/local-environment.md`, which is gitignored. Agents and AIs should read that file if it exists before assuming paths or command variants.

Use this copy-paste template for another environment:

```markdown
# Machine-Local Development Profile

GAME_INSTALL_ROOT: <GAME_INSTALL_ROOT>

## Fixture directories
- HOD 1.0: <HOD_1_0_FIXTURE_DIR>
- HOD 1.3: <HOD_1_3_FIXTURE_DIR>
- HOD 2.0: <HOD_2_0_FIXTURE_DIR>
- DAE: <DAE_FIXTURE_DIR>
- Pebbles: <PEBBLES_FIXTURE_DIR>

## verify_lossless fixtures
- pebble_0.hod: <PEBBLE_0_HOD_PATH>
- ter_elysium.hod: <TER_ELYSIUM_HOD_PATH>
- ter_fenris.hod: <TER_FENRIS_HOD_PATH>
- asteroid_3.hod: <ASTEROID_3_HOD_PATH>
- galaxymapgalaxy.dae: <GALAXYMAPGALAXY_DAE_PATH>
- hgn_mothership.hod: <HGN_MOTHERSHIP_HOD_PATH>

These six keys are read directly by `verify_lossless` from this file (override the file location with `HODE_LOCAL_ENV`). Keep the `key: path` format.

## Container and toolchain notes
- Container/image name (if used): <CONTAINER_OR_IMAGE>
- MinGW toolchain notes for Windows cross-compile: <MINGW_NOTES>
- Space-free target directory if the checkout path contains spaces: <SPACE_FREE_TARGET_DIR>
- AppImage strip quirks: <APPIMAGE_STRIP_NOTES>
```

## Agentic AI Development

Start at `AGENTS.md`, then read `docs/agent-handbook/README.md`. Parser changes require `verify_lossless`; `cargo run --bin verify_lossless` reads fixture paths from the local-environment file. When the game installation is absent, use the documented `cargo test --manifest-path parser/Cargo.toml` fallback and record that `verify_lossless` was not run. Obtain external fixtures according to `docs/testing-fixtures.md`; fixtures are extracted from the developer's own installation and are never redistributed.
