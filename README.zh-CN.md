<h1 align="center">STL Viewer</h1>

<p align="center">
  <img src="docs/screenshot.png" alt="STL Viewer" width="640">
</p>

<p align="center"><a href="README.md">English</a> | 简体中文</p>

## 简介

- **二进制体积：8.0 MiB**
- **内存占用：约 64 MiB**
- **可以在 10 ms 内启动打开（⚡ Rust + egui）**（CLI 加载耗时比 Bambu Studio 减少约 99%）

极度精简、启动飞快的原生 STL、3MF 和 OBJ 查看器。支持 Linux（Wayland / X11）、Windows 和 macOS，以及 x86_64 / ARM64 架构。

三个 STL 样本的 CLI 启动及文件加载耗时：

| 样本 | STL Viewer | Bambu Studio | Blender |
| --- | ---: | ---: | ---: |
| A · 2.0 KiB | **1.5 ms** | 203.0 ms | 443.8 ms |
| B · 24.1 KiB | **1.6 ms** | 192.5 ms | 446.1 ms |
| C · 87.2 KiB | **1.4 ms** | 193.2 ms | 463.9 ms |

## 安装

从 [GitHub Releases](https://github.com/ctxinf/stl-viewer-rs/releases/latest) 选择对应系统及架构的最新安装包：

| 平台 | 架构 | 安装包 |
| --- | --- | --- |
| Linux | x86_64、ARM64（aarch64 / arm64） | Debian / Ubuntu：`.deb`；Fedora：`.rpm`；Arch / Arch Linux ARM：`.pkg.tar.zst`；通用：`.tar.gz` |
| Windows | x86_64、ARM64 | 解压 `.zip`，运行 `stl-viewer.exe` |
| macOS | Intel、Apple Silicon | 解压 `.zip`，将 `STL Viewer.app` 移入“应用程序” |

Linux 需要 glibc 2.35+ 和 OpenGL 3.1+；macOS 需要 OpenGL 3.2+；Windows 需要提供 OpenGL 3.1+ 的显卡驱动。ARM 指 64 位 ARM，不包含 ARMv7。macOS 包仅作本地签名，未经过 Apple 公证，首次启动可能被 Gatekeeper 拦截；Windows 包未作代码签名。

体积、内存和加载耗时为此前 Linux x86_64 测量结果，其他平台和架构可能不同。源码构建：`cargo build --release --locked`。为兼容现有用户，Cargo 包名和源码生成的可执行文件仍为 `stl-viewer-linux`（Windows 为 `.exe`）。

通过 **View → Colors** 设置背景色和模型色，设置会自动保存，并应用于导出的截图。标准 STL 文件只包含几何数据，不包含模型颜色。


## 模型格式

打开文件、拖拽、CLI 信息查询和截图导出均支持 STL、OBJ 和 3MF：

```sh
stl-viewer-linux model.3mf
stl-viewer-linux info model.obj
stl-viewer-linux screenshot model.3mf -o preview.png
```

所有格式均使用设置中的统一模型色，不显示材质、纹理和切片参数。OBJ 支持面索引、负索引和三角扇三角化；任意凹多边形建议先在建模软件中三角化。无需配套 MTL 文件，材质引用会忽略。STL 和 OBJ 的单位不确定。

3MF 读取构建列表中的三角网格，支持嵌套组件、重复实例、摆放变换，以及 Production 扩展引用的包内其他模型文件。跳过不可打印的构建项和修饰对象（`type="other"`）。坐标与尺寸统一转换为毫米；跨模型文件混用单位时会报错。纯切片、体素、布尔和位移几何不渲染。本程序用于查看几何，不编辑切片工程。

上方体积、内存和耗时数据为历史 STL 测量结果，不代表当前多格式版本的实测值。

版本变更记录见 [CHANGELOG.md](CHANGELOG.md)。

Linux Wayland 标题栏使用系统字体回退显示多语言文件名，请安装对应语言字体（例如 Noto CJK）。源码构建还需要 Fontconfig 和 FreeType 开发包（Debian/Ubuntu：`libfontconfig1-dev libfreetype6-dev`）。
