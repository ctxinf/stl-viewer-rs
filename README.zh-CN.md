<h1 align="center">STL Viewer</h1>

<p align="center">
  <img src="docs/screenshot.png" alt="STL Viewer" width="640">
</p>

<p align="center"><a href="README.md">English</a> | 简体中文</p>

## 简介

- **二进制体积：8.0 MiB**
- **内存占用：约 64 MiB**
- **可以在 10 ms 内启动打开（⚡ Rust + egui）**（CLI 加载耗时比 Bambu Studio 减少约 99%）

极度精简、启动飞快的原生 STL 查看器。0.4.x 支持 Linux（Wayland / X11）、Windows 和 macOS，以及 x86_64 / ARM64 架构。

三个 STL 样本的 CLI 启动及文件加载耗时：

| 样本 | STL Viewer | Bambu Studio | Blender |
| --- | ---: | ---: | ---: |
| A · 2.0 KiB | **1.5 ms** | 203.0 ms | 443.8 ms |
| B · 24.1 KiB | **1.6 ms** | 192.5 ms | 446.1 ms |
| C · 87.2 KiB | **1.4 ms** | 193.2 ms | 463.9 ms |

## 安装

从 [GitHub Releases](https://github.com/ctxinf/stl-viewer-rs/releases/latest) 选择对应系统及架构的 0.4.x 安装包：

| 平台 | 架构 | 安装包 |
| --- | --- | --- |
| Linux | x86_64、ARM64（aarch64 / arm64） | Debian / Ubuntu：`.deb`；Fedora：`.rpm`；Arch / Arch Linux ARM：`.pkg.tar.zst`；通用：`.tar.gz` |
| Windows | x86_64、ARM64 | 解压 `.zip`，运行 `stl-viewer.exe` |
| macOS | Intel、Apple Silicon | 解压 `.zip`，将 `STL Viewer.app` 移入“应用程序” |

Linux 需要 glibc 2.35+ 和 OpenGL 3.1+；macOS 需要 OpenGL 3.2+；Windows 需要提供 OpenGL 3.1+ 的显卡驱动。ARM 指 64 位 ARM，不包含 ARMv7。macOS 包仅作本地签名，未经过 Apple 公证，首次启动可能被 Gatekeeper 拦截；Windows 包未作代码签名。

体积、内存和加载耗时为此前 Linux x86_64 测量结果，其他平台和架构可能不同。源码构建：`cargo build --release --locked`。为兼容现有用户，Cargo 包名和源码生成的可执行文件仍为 `stl-viewer-linux`（Windows 为 `.exe`）。

通过 **View → Colors** 设置背景色和模型色，设置会自动保存，并应用于导出的截图。标准 STL 文件只包含几何数据，不包含模型颜色。
