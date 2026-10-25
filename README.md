# STL Viewer Linux

Rust 编写的轻量 STL 查看器，使用 winit 原生窗口和 OpenGL GPU 渲染，编译启用 Wayland 与 X11。使用 egui 绘制窗口内菜单与 HUD，无需浏览器或 Electron。

- 二进制 / ASCII STL，支持 `solid` 开头的二进制文件。
- 只读 mmap 输入，连续顶点数组，一次上传 GPU；几何着色从屏幕导数计算，不构建拓扑或额外法线缓冲。
- 事件驱动绘制，静止时等待事件，不运行持续刷新循环。
- 文件选择、拖放、文件管理器关联、线框、旋转、平移、缩放。
- File / View / Help 菜单、六面标准视角、仅展示朝向的方位立方体、缩放百分比。
- `.deb`、`.rpm`、Arch `.pkg.tar.zst` 和通用 tar 包。

当前状态：本地 release 编译、严格 Clippy、11 个相机、解析器和方位投影单元测试及 X11 实际像素与交互检查通过，原生 Wayland 桌面及 headless Weston 首帧图像提交通过。安装包和远程 Actions 尚未验证。仓库不附带跨设备性能保证；应在目标机器上实测。

## 安装

在 GitHub Releases 下载对应 x86_64 安装包：

```sh
# Debian / Ubuntu（Ubuntu 22.04+、Debian 12+）
sudo apt install ./stl-viewer-linux-0.2.0-amd64.deb
# Fedora
sudo dnf install ./stl-viewer-linux-0.2.0-x86_64.rpm
# Arch / Manjaro
sudo pacman -U ./stl-viewer-linux-0.2.0-1-x86_64.pkg.tar.zst
```

其他发行版可解压 tar 包到 `/`，需要 glibc 2.35+、OpenGL 3.1+ 和系统图形驱动。文件选择器使用 XDG desktop portal，桌面需安装对应的 portal 后端。当前发布配置仅覆盖 x86_64；ARM 未验证。

## 使用

```sh
stl-viewer-linux model.stl
stl-viewer-linux                 # 按 O 打开，或拖入 STL
stl-viewer-linux info model.stl   # 无窗口查看三角形数量、包围盒、尺寸、表面积等
stl-viewer-linux screenshot model.stl -o model.png --width 1600 --height 1200
# 无桌面环境时可使用 xvfb-run（截图需要 OpenGL 图形上下文）
xvfb-run -a stl-viewer-linux screenshot model.stl -o model.png
```

| 操作 | 功能 |
| --- | --- |
| 左键拖动 | 旋转 |
| 右键拖动 | 平移 |
| 右键单击 | Reset / Fit 菜单 |
| 滚轮 | 缩放 |
| O / Ctrl+O | 打开文件 |
| R | 恢复视角 |
| W | 实体 / 线框 |
| 方位控件区域 | 鼠标事件穿透到场景，使用同一套拖动与缩放逻辑 |
| F1 | 操作说明 |
| Esc | 先关闭菜单或弹窗，再退出 |

STL 保存顶点坐标及相对长度，但没有可靠的物理单位信息，无法仅凭文件确认 mm 或 inch。界面显示缩放百分比：初始适配大小为 100%，R / Reset / Fit 恢复到 100%。选择标准视角保留平移与缩放。View 菜单可分别控制方位控件和缩放百分比的显示。

File → Export screenshot 导出当前视角的 PNG（仅模型视口，不包含菜单及 HUD）。View → Background color 调整背景色并自动保存，下次启动和 CLI 截图均使用该颜色。配置保存在 `$XDG_CONFIG_HOME/stl-viewer-linux/background`，未设置时使用 `~/.config/stl-viewer-linux/background`。CLI 截图默认 1600×1200，宽高范围为 1–8192；`--info` 仍兼容。

桌面文件、X11 WM_CLASS 和 Wayland app_id 统一使用 `stl-viewer-linux`；安装包包含 SVG 和 PNG 图标，X11 窗口也内嵌图标。

读取时请勿同时修改或截断源文件。模型加载目前同步执行，超大文件加载期间窗口可能短暂无响应。标题栏计时包含解析和 GPU 缓冲创建，不包含进程启动和第一帧显示。内存需求约为每三角形 36 字节 CPU 顶点数据与同等 GPU 缓冲，另加映射文件和驱动开销。

## 开发与验证

```sh
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
# 无外部依赖的解析器测试
rustc --edition 2024 --test src/mesh.rs -o /tmp/stl-mesh-tests
/tmp/stl-mesh-tests
```

Linux 构建需要 C 编译器、pkg-config、Wayland / XKB / X11 / EGL / OpenGL 开发包。CI 在 Ubuntu 22.04 构建以降低 glibc 版本要求；以软件 OpenGL 执行 X11 实际像素及窗口交互测试（需 ImageMagick、Pillow 和 xdotool），包含菜单/弹窗输入隔离、方位控件统一拖动、缩放百分比、失焦和窄窗口检查，另测 100% / 150% / 200% DPI；以 headless Weston 检查 Wayland 窗口首帧图像缓冲提交。Wayland 检查不替代真实桌面的视觉与输入验证。

Actions 会运行格式检查、Clippy、解析测试、release 编译、窗口冒烟测试和打包。推送 `vX.Y.Z` 标签后，仅当所有构建步骤成功，才将安装包及 SHA256SUMS 上传到 GitHub Release。版本标签必须与 Cargo.toml 一致。Cargo.lock 已生成，固定完整依赖树；CI 使用 `--locked` 检查和构建。

## 发布

在可联网且 `gh auth status` 正常的环境执行：

```sh
bash scripts/publish.sh
```

脚本会先生成 Cargo.lock、运行检查并以项目通用身份提交明确列出的项目文件，然后创建当前登录账号下的公开 `stl-viewer-linux` 仓库（已有 origin 则使用它）、推送 main、等待主分支构建通过，然后推送版本标签并等待发布构建。提交使用项目通用身份。不要将本机配置、凭据、用户 STL 或测试截图提交到仓库。

MIT License.
