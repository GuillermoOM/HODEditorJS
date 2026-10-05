# Release Process

This document describes the process for building and publishing a new release of HODEditorJS. Because the application uses Tauri, native system dependencies (GTK, WebKit) and, for Windows cross-compilation, MinGW are required. Perform release builds in your build environment; machine-specific details belong in the local-environment file.

## Prerequisites

1. Ensure all code is committed and pushed.
2. Update the version number in:
   * `package.json`
   * `src-tauri/tauri.conf.json`
3. Ensure your platform's Tauri prerequisites and any required build environment or container are available.

## 1. Building the Linux Release Bundles

Run the following command in your build environment:

```bash
cd /path/to/HODEditorJS
npm run tauri build
```

If `linuxdeploy` fails while stripping the AppImage on your distro, retry with `NO_STRIP=1 npm run tauri build`.

This will generate the following artifacts:
* `src-tauri/target/release/bundle/deb/HODEditorJS_<VERSION>_amd64.deb`
* `src-tauri/target/release/bundle/rpm/HODEditorJS-<VERSION>-1.x86_64.rpm`
* `src-tauri/target/release/bundle/appimage/HODEditorJS_<VERSION>_amd64.AppImage`

## 2. Building the Windows Installer

Run the following command in your build environment:

```bash
cd /path/to/HODEditorJS
CARGO_TARGET_DIR=<space-free-dir> npm run tauri build -- --target x86_64-pc-windows-gnu --bundles nsis
```

`CARGO_TARGET_DIR=<space-free-dir>` is optional, but recommended when the checkout path contains spaces. Windows GNU binutils `dlltool` can fail when the target directory contains spaces, for example in a Steam Library path.

This will generate the following artifact:
* `<target-dir>/x86_64-pc-windows-gnu/release/bundle/nsis/HODEditorJS_<VERSION>_x64-setup.exe` (or the equivalent path under `src-tauri/target/` when using the default target directory)

## 3. Publishing to GitHub

1. Create a new tag for the release in git:
   ```bash
   git tag v<VERSION>
   git push origin v<VERSION>
   ```

2. Use the GitHub CLI (`gh`) to create the release and upload the built artifacts:
   ```bash
   gh release create v<VERSION> \
       "src-tauri/target/release/bundle/deb/HODEditorJS_<VERSION>_amd64.deb" \
       "src-tauri/target/release/bundle/rpm/HODEditorJS-<VERSION>-1.x86_64.rpm" \
       "src-tauri/target/release/bundle/appimage/HODEditorJS_<VERSION>_amd64.AppImage" \
       "<target-dir>/x86_64-pc-windows-gnu/release/bundle/nsis/HODEditorJS_<VERSION>_x64-setup.exe" \
       --title "v<VERSION>" \
       --notes "Release notes go here."
   ```

3. Verify the release is live on the GitHub repository's Releases page.
