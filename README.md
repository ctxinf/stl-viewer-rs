<h1 align="center">STL Viewer</h1>

<p align="center">
  <img src="docs/screenshot.png" alt="STL Viewer" width="640">
</p>

<p align="center">English | <a href="README.zh-CN.md">简体中文</a></p>

## Intro

- **Binary: 8.0 MiB**
- **RAM: ~64 MiB**
- **Starts and opens STL files in under 10 ms (⚡ Rust + egui)** (~99% less CLI loading time than Bambu Studio)

A minimal, fast-starting native Linux STL viewer with Wayland / X11 support.

CLI startup and file loading times for three STL samples:

| Sample | STL Viewer | Bambu Studio | Blender |
| --- | ---: | ---: | ---: |
| A · 2.0 KiB | **1.5 ms** | 203.0 ms | 443.8 ms |
| B · 24.1 KiB | **1.6 ms** | 192.5 ms | 446.1 ms |
| C · 87.2 KiB | **1.4 ms** | 193.2 ms | 463.9 ms |

## Install

Download the x86_64 package for your distribution from [GitHub Releases](https://github.com/ctxinf/stl-viewer-rs/releases/latest) and open it with your system’s software installer: `.deb` for Debian / Ubuntu, `.rpm` for Fedora, or `.pkg.tar.zst` for Arch / Manjaro. Requires glibc 2.35+ and OpenGL 3.1+.
