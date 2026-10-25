![STL Viewer](docs/screenshot.png)

## Intro

- **Binary: 8.0 MiB**（x86_64 release）
- **RAM: 约 64 MiB**（本机打开小型 STL 后的进程 RSS，不含 GPU 显存）

极度精简、启动飞快的原生 Linux STL viewer，支持 Wayland / X11。

在自己电脑上，用三个 STL 样本测试启动并读取文件的耗时：

| 样本 | STL Viewer | Bambu Studio | Blender |
| --- | ---: | ---: | ---: |
| A · 2.0 KiB | **1.5 ms** | 203.0 ms | 443.8 ms |
| B · 24.1 KiB | **1.6 ms** | 192.5 ms | 446.1 ms |
| C · 87.2 KiB | **1.4 ms** | 193.2 ms | 463.9 ms |

每项重复三次取中位数，未清空文件缓存。测量范围为进程启动至 CLI 读取完成并退出：本项目 `info`、Bambu Studio 2.8.2 `--info`、Blender 5.2.1 后台导入 STL；包含各自初始化及统计开销，不代表 GUI 主窗口或首帧显示时间。

## Install

1. 从 [GitHub Releases](https://github.com/ctxinf/stl-viewer-rs/releases/latest) 下载对应的 x86_64 安装包，用系统软件安装器打开安装：Debian / Ubuntu 选 `.deb`，Fedora 选 `.rpm`，Arch / Manjaro 选 `.pkg.tar.zst`。需要 glibc 2.35+ 和 OpenGL 3.1+。
