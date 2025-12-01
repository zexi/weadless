# weadless - Headless Wayland Compositor

一个基于 [gst-wayland-display](https://github.com/games-on-whales/gst-wayland-display) 的 headless Wayland compositor，可以在服务器没有外接显示器的情况下远程启动桌面环境。

## 功能特性

- ✅ Headless 运行，无需物理显示器
- ✅ 支持硬件加速（DRM）和软件渲染
- ✅ 可配置的分辨率和帧率
- ✅ 支持多种像素格式（RGBx, RGBA, BGRx, BGRA）
- ✅ 自动创建 Wayland socket，方便客户端连接

## 系统要求

- Linux 系统（Wayland 是 Linux 特有的）
- Rust 1.88 或更高版本
- 如果使用硬件加速，需要 DRM 设备（如 `/dev/dri/renderD128`）
- 如果使用软件渲染，无需特殊硬件

## 构建

```bash
cd weadless
cargo build --release
```

## 使用方法

### 基本用法（软件渲染）

```bash
# 启动 compositor，使用默认配置（1920x1080@60fps）
./target/release/weadless

# 或者指定参数
./target/release/weadless --width 2560 --height 1440 --fps 60
```

### 使用硬件加速

```bash
# 使用指定的 DRM 渲染节点
./target/release/weadless --render-node /dev/dri/renderD128

# 或者让系统自动检测（默认使用 /dev/dri/renderD128）
./target/release/weadless --render-node /dev/dri/renderD128 --width 1920 --height 1080
```

### 连接 Wayland 应用

程序启动后会输出 Wayland socket 信息，例如：

```
✓ Wayland compositor 已启动
  Socket: wayland-1
  使用以下命令连接:
    export WAYLAND_DISPLAY=wayland-1
    # 然后启动你的 Wayland 应用，例如:
    # WAYLAND_DISPLAY=wayland-1 weston-terminal
```

然后你可以在另一个终端中：

```bash
export WAYLAND_DISPLAY=wayland-1
weston-terminal  # 或其他 Wayland 应用
```

### 启动完整的桌面环境

```bash
# 启动 compositor
./target/release/weadless --width 1920 --height 1080 &

# 设置环境变量
export WAYLAND_DISPLAY=wayland-1
export XDG_RUNTIME_DIR=/run/user/$(id -u)

# 启动桌面环境（例如 GNOME、KDE 或 Weston）
WAYLAND_DISPLAY=wayland-1 weston
# 或
WAYLAND_DISPLAY=wayland-1 gnome-session
```

## 命令行参数

```
Options:
  --render-node <RENDER_NODE>  渲染节点路径（例如 /dev/dri/renderD128），使用 "software" 进行软件渲染 [default: software]
  --width <WIDTH>              输出宽度（像素） [default: 1920]
  --height <HEIGHT>            输出高度（像素） [default: 1080]
  --fps <FPS>                  帧率（fps） [default: 60]
  --format <FORMAT>            视频格式（RGBx, RGBA, BGRx, BGRA） [default: RGBx]
  -h, --help                   显示帮助信息
```

## 示例场景

### 场景 1: 远程服务器运行 GUI 应用

```bash
# 在服务器上启动 compositor
ssh user@server
./weadless --width 1920 --height 1080 &

# 设置环境变量
export WAYLAND_DISPLAY=wayland-1

# 启动应用（例如 Firefox）
WAYLAND_DISPLAY=wayland-1 firefox
```

### 场景 2: Docker 容器中运行桌面

```bash
# 在容器中启动 compositor
docker run -it --device=/dev/dri/renderD128 your-image
./weadless --render-node /dev/dri/renderD128
```

### 场景 3: CI/CD 中运行 GUI 测试

```bash
# 在 CI 环境中启动 headless compositor
./weadless --width 1280 --height 720 &
export WAYLAND_DISPLAY=wayland-1

# 运行需要显示服务器的测试
WAYLAND_DISPLAY=wayland-1 your-gui-test
```

## 技术细节

- 基于 [smithay](https://github.com/Smithay/smithay) Wayland compositor 库
- 使用 `wayland-display-core` 作为核心 compositor 实现
- 支持 EGL 硬件加速和软件渲染
- 自动管理 Wayland socket 创建和客户端连接

## 故障排除

### 问题：无法创建 WaylandDisplay

**解决方案**：如果使用硬件加速，确保：
- DRM 设备存在：`ls -l /dev/dri/renderD*`
- 有权限访问设备：`sudo chmod 666 /dev/dri/renderD128`
- 如果不需要硬件加速，使用 `--render-node software`

### 问题：客户端无法连接

**解决方案**：
- 确保 `XDG_RUNTIME_DIR` 环境变量已设置
- 检查 socket 文件是否存在：`ls $XDG_RUNTIME_DIR/wayland-*`
- 确保客户端和服务器在同一用户下运行

### 问题：性能问题

**解决方案**：
- 使用硬件加速而不是软件渲染
- 降低分辨率或帧率
- 检查是否有足够的 GPU 资源

## 许可证

本项目基于 gst-wayland-display，遵循相应的许可证。

## 相关项目

- [gst-wayland-display](https://github.com/games-on-whales/gst-wayland-display) - 原始项目
- [smithay](https://github.com/Smithay/smithay) - Wayland compositor 库
- [Weston](https://gitlab.freedesktop.org/wayland/weston) - 参考 Wayland compositor 实现

