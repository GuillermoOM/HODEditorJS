# Release Process

This document describes the exact process for building and publishing a new release of HODEditorJS. 
Because the application uses Tauri, native system dependencies (GTK, WebKit) and specific cross-compilation toolchains (MinGW) are required. All builds must be performed inside the isolated `mesa-dev` distrobox container.

## Prerequisites

1. Ensure all code is committed and pushed.
2. Update the version number in:
   * `package.json`
   * `src-tauri/tauri.conf.json`
3. Ensure the distrobox container is available:
   ```bash
   distrobox enter mesa-dev
   ```

## 1. Building the Linux Release Bundles

Run the following command *inside* the `mesa-dev` distrobox:

```bash
cd /path/to/HODEditorJS
NO_STRIP=1 npm run tauri build
```

*Note: `NO_STRIP=1` is required to prevent `linuxdeploy` from crashing on modern `.relr.dyn` relocations when bundling the AppImage.*

This will generate the following artifacts:
* `src-tauri/target/release/bundle/deb/HODEditorJS_<VERSION>_amd64.deb`
* `src-tauri/target/release/bundle/rpm/HODEditorJS-<VERSION>-1.x86_64.rpm`
* `src-tauri/target/release/bundle/appimage/HODEditorJS_<VERSION>_amd64.AppImage`

## 2. Building the Windows Installer

Run the following command *inside* the `mesa-dev` distrobox:

```bash
cd /path/to/HODEditorJS
CARGO_TARGET_DIR=/tmp/cargo_target npm run tauri build -- --target x86_64-pc-windows-gnu --bundles nsis
```

*Note: `CARGO_TARGET_DIR=/tmp/cargo_target` is required because the Windows GNU `dlltool` fails when running in a path that contains spaces (like the default Steam Library path).*

This will generate the following artifact:
* `/tmp/cargo_target/x86_64-pc-windows-gnu/release/bundle/nsis/HODEditorJS_<VERSION>_x64-setup.exe`

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
       "/tmp/cargo_target/x86_64-pc-windows-gnu/release/bundle/nsis/HODEditorJS_<VERSION>_x64-setup.exe" \
       --title "v<VERSION>" \
       --notes "Release notes go here."
   ```

3. Verify the release is live on the GitHub repository's Releases page.
