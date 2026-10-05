# Testing Fixtures

The `testing/` directory is an external, untracked input corpus. It is
gitignored and is intentionally not distributed with this repository.

## Required Inputs

The `validation_suite` tool requires these fixture directories at runtime:

- `testing/pebble_0/`
- `testing/pebble_1/`
- `testing/pebble_2/`

The history in `docs/manual-test-log.md` references these additional assets:

- `testing/ter_centaur/`
- `testing/ter_fenris/`
- `testing/ter_pharos/`
- `testing/ter_zephyrus/`
- `testing/ter_myrmidon/`
- `testing/hgn_hft.hod`

## Obtaining Fixtures

Extract the required files from your own Homeworld Remastered installation or
from your own copies of the relevant mods. Do not obtain or share them from a
third-party asset source. The locations and input requirements for the parser tools
are described in
[`docs/hod2-reverse-engineering/testing-guide.md`](hod2-reverse-engineering/testing-guide.md).
`AGENTS.md`, in its “Useful Paths for Testing” section, explains how
`verify_lossless` locates game-install data and how to map those paths to your
own installation.

Do not redistribute these assets. They are extracted Homeworld Remastered game
assets and third-party mod assets; the third-party mod used by the relevant
fixtures is **HWRM_FSFC**. The repository's [`LICENSE`](../LICENSE) is CC
BY-NC 4.0 and does not grant permission to redistribute the extracted game or
mod assets.

## Expected Layout

Place the corpus under the repository root in this layout:

```text
testing/
├── pebble_0/
├── pebble_1/
├── pebble_2/
├── ter_centaur/
├── ter_fenris/
├── ter_pharos/
├── ter_zephyrus/
├── ter_myrmidon/
└── hgn_hft.hod
```

The validation-suite directories must contain the source HOD, mesh, texture,
and material inputs expected by the harness. The historical manual-test assets
need only be restored when reproducing the corresponding entries in
`docs/manual-test-log.md`.
