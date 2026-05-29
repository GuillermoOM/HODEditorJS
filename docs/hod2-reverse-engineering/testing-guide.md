# HOD 2.0 Testing Guide

## Directory Structure

```
testing/
  pebble_0/
    pebble_0_vanilla.hod        # Original vanilla HOD 2.0
    pebble_0_roundtrip.hod      # Vanilla → parse → generate
    pebble_0_from_assets.hod    # OBJ + TGA + JSON + vanilla KDOP → generate
    Root_mesh_lod_0.obj
    Root_mesh_lod_1.obj
    Pebble_DIFF.tga
    Pebble_GLOW.tga
    Pebble_NORM.tga
    Homeworld2 Multi Mesh File_materials.json
  pebble_1/
    ...same structure...
  pebble_2/
    ...same structure...
```

## Test Commands

### Regenerate from-assets HODs

Builds from OBJ + TGA + JSON, copies KDOP/COLD/INFO from vanilla:

```bash
cd parser && cargo run --bin replicate_testing
```

### Compare vanilla, roundtrip, and from_assets

Shows chunk structure, POOL sizes, Xpress indicator/literal/match counts, KDOP hashes:

```bash
cd parser && cargo run --bin testing_diff
```

### Mandatory verification suite

Runs roundtrip, DAE fallback, and structural preservation checks:

```bash
cd parser && cargo run --bin verify_lossless
```

### HODOR Source-Asset Replication Suite

The true test of the compiler is whether it natively generates byte-accurate outputs identical to HODOR when parsing raw `.dae` files.

```bash
# Validates DAE complex materials, Xpress compression, and tangent calculation vs HODOR.exe
cd parser && cargo run --bin test_hodor_replication

# Validates empty-original save paths
cd parser && cargo run --bin test_fenris
```

### Other diagnostic binaries

```bash
# Dump recursive chunk tree for any HOD file
cd parser && cargo run --bin dump_mult -- <path_to_hod>

# Compare two HOD files byte-by-byte at chunk level
cd parser && cargo run --bin compare_hods -- <file1> <file2>

# Dump KDOP chunk contents
cd parser && cargo run --bin dump_kdop -- <path_to_hod>

# Dump POOL stream sizes
cd parser && cargo run --bin dump_pool -- <path_to_hod>

# Decompressed face pool comparison
cd parser && cargo run --bin face_pool_compare -- <vanilla_hod> <generated_hod>
```

## What the Tests Verify

| Test | What it checks |
|------|---------------|
| `replicate_testing` | Model building from OBJ/TGA/JSON, KDOP/COLD/INFO preservation from vanilla |
| `testing_diff` | Chunk layout matches vanilla, POOL stream sizes, Xpress compression stats |
| `verify_lossless` | Roundtrip fidelity (parse → generate → reparse), DAE generation |
| `test_hodor_replication` | Source-asset generation parity against HODOR.exe (Xpress logic, Tangents, Dedup) |
| `test_fenris` | Validates `.dae` generation fallback behavior when no original payload exists |
| `face_pool_compare` | Byte-identical decompressed face pool between vanilla and generated |
| `dump_mult` | Recursive chunk tree structure (HVMD → MULT → NRML BMSH) |

## Key Metrics to Watch

- **POOL chunk data size**: Must match between vanilla and roundtrip
- **KDOP hash**: Must be identical if preserved from vanilla
- **Face pool compressed/decompressed sizes**: `decomp_face == decomp_face` between vanilla and from_assets
- **LMIP chunk data size**: 87 bytes each for pebble textures (3 textures)
- **DTRM children**: Should be 2 (HIER + KDOP) for pebble HODs
- **INFO chunk data size**: 50 bytes for pebble HODs
- **First byte of POOL**: Should be `0x06` (pool_type 3518 = 0x0DB6, LE → first byte 0x06)

## Known Issues

- `cargo fmt` is unavailable: `error: 'cargo-fmt' is not installed for the toolchain 'stable-x86_64-unknown-linux-gnu'`
- Run `cargo check --lib` before committing to catch compilation errors
- The `testing_diff` binary generates roundtrip HODs as a side effect (writes `*_roundtrip.hod` files)
