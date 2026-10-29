<h1 align="center">STL Viewer</h1>

<p align="center">
  <img src="docs/screenshot.png" alt="STL Viewer" width="640">
</p>

<p align="center">English | <a href="README.zh-CN.md">简体中文</a></p>

## Intro

- **Binary: 8.0 MiB**
- **RAM: ~64 MiB**
- **Starts and opens STL files in under 10 ms (⚡ Rust + egui)** (~99% less CLI loading time than Bambu Studio)

A minimal, fast-starting native STL, 3MF and OBJ viewer for Linux (Wayland / X11), Windows and macOS, with x86_64 and ARM64 support.

CLI startup and file loading times for three STL samples:

| Sample | STL Viewer | Bambu Studio | Blender |
| --- | ---: | ---: | ---: |
| A · 2.0 KiB | **1.5 ms** | 203.0 ms | 443.8 ms |
| B · 24.1 KiB | **1.6 ms** | 192.5 ms | 446.1 ms |
| C · 87.2 KiB | **1.4 ms** | 193.2 ms | 463.9 ms |

## Install

Choose the latest download for your OS and architecture from [GitHub Releases](https://github.com/ctxinf/stl-viewer-rs/releases/latest):

| Platform | Architectures | Package |
| --- | --- | --- |
| Linux | x86_64, ARM64 (aarch64 / arm64) | Debian / Ubuntu: `.deb`; Fedora: `.rpm`; Arch / Arch Linux ARM: `.pkg.tar.zst`; generic: `.tar.gz` |
| Windows | x86_64, ARM64 | Extract `.zip`, run `stl-viewer.exe` |
| macOS | Intel, Apple Silicon | Extract `.zip`, move `STL Viewer.app` to Applications |

Linux requires glibc 2.35+ and OpenGL 3.1+; macOS requires OpenGL 3.2+; Windows requires a graphics driver providing OpenGL 3.1+. ARM means 64-bit ARM; ARMv7 is not included. macOS bundles are ad-hoc signed and not notarized by Apple, so Gatekeeper may block the first launch. Windows binaries are unsigned.

## Model formats

Open, drag and drop, inspect or export STL, OBJ and 3MF models:

```sh
stl-viewer-linux model.3mf
stl-viewer-linux info model.obj
stl-viewer-linux screenshot model.3mf -o preview.png
```

All formats use the selected model color; materials, textures and slicer settings are not displayed. OBJ supports indexed faces, negative indices and triangle-fan triangulation (arbitrary concave polygons should be triangulated before export); MTL files are optional and ignored. OBJ and STL units are unspecified.

3MF loads triangle meshes from the build list, including nested components, repeated instances, transforms and Production Extension references to other model parts in the archive. Non-printable build items and modifier objects (`type="other"`) are omitted. Coordinates and dimensions are converted to millimeters. Mixed units across model parts are rejected. Slice-only, voxel, boolean and displacement geometry is not rendered. This is a geometry viewer, not a slicer project editor.

The size, memory and timing figures above are historical STL measurements, not measurements of the current multi-format build.

See [CHANGELOG.md](CHANGELOG.md) for release history.

Linux Wayland window titles use system font fallback for multilingual filenames. Install fonts for the languages you need (for example, Noto CJK). Source builds also require Fontconfig and FreeType development packages (`libfontconfig1-dev libfreetype6-dev` on Debian/Ubuntu).
