<h1 align="center">STL Viewer</h1>

<p align="center">
  <img src="docs/screenshot.png" alt="STL Viewer" width="640">
</p>

<p align="center"><a href="README.md">English</a> | 简体中文</p>

## 简介

- **二进制体积：8.0 MiB**
- **内存占用：约 64 MiB**
- **可以在 10 ms 内启动打开（⚡ Rust + egui）**（CLI 加载耗时比 Bambu Studio 减少约 99%）

极度精简、启动飞快的原生 Linux STL viewer，支持 Wayland / X11。

三个 STL 样本的 CLI 启动及文件加载耗时：

| 样本 | STL Viewer | Bambu Studio | Blender |
| --- | ---: | ---: | ---: |
| A · 2.0 KiB | **1.5 ms** | 203.0 ms | 443.8 ms |
| B · 24.1 KiB | **1.6 ms** | 192.5 ms | 446.1 ms |
| C · 87.2 KiB | **1.4 ms** | 193.2 ms | 463.9 ms |

## 安装

从 [GitHub Releases](https://github.com/ctxinf/stl-viewer-rs/releases/latest) 下载对应的 x86_64 安装包，用系统软件安装器打开安装：Debian / Ubuntu 选 `.deb`，Fedora 选 `.rpm`，Arch / Manjaro 选 `.pkg.tar.zst`。需要 glibc 2.35+ 和 OpenGL 3.1+。

通过 **View → Colors** 设置背景色和模型色，设置会自动保存，并应用于导出的截图。标准 STL 文件只包含几何数据，不包含模型颜色。
