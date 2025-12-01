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
- **如果使用输出流功能（`--output appsrc`），需要安装 GStreamer 插件**：
  ```bash
  # Ubuntu/Debian
  sudo apt-get install gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly
  
  # Arch Linux
  sudo pacman -S gstreamer gstreamer-vaapi gstreamer-plugins-good gstreamer-plugins-bad gstreamer-plugins-ugly
  
  # Fedora
  sudo dnf install gstreamer1-plugins-good gstreamer1-plugins-bad gstreamer1-plugins-ugly
  ```

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

### 查看 compositor 输出流

启动 compositor 时启用输出流暴露功能：

**UDP 模式（默认）**：
```bash
# 启动 compositor 并启用 UDP 输出流
./target/release/weadless --output appsrc --output-address 127.0.0.1:5000 --protocol udp

# 在另一个终端中，使用 GStreamer 接收并显示
# Linux:
gst-launch-1.0 \
    udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtph264depay ! \
    avdec_h264 ! \
    videoconvert ! \
    autovideosink

# macOS（必须使用 osxvideosink，autovideosink 有 OpenGL 问题）:
export GST_GL_API=disable
gst-launch-1.0 \
    udpsrc port=5000 caps="application/x-rtp,media=video,clock-rate=90000,encoding-name=H264,payload=96" buffer-size=524288 ! \
    rtph264depay ! \
    h264parse ! \
    avdec_h264 ! \
    videoconvert ! \
    osxvideosink sync=false

# 或者直接使用脚本（自动检测操作系统）:
./receive_stream.sh 5000
```

**TCP 模式（适合跨网络或 macOS）**：
```bash
# 启动 compositor 并启用 TCP 输出流
./target/release/weadless --output appsrc --output-address 192.168.6.60:8080 --protocol tcp

# 在客户端（如 macOS）使用 GStreamer 接收并显示
gst-launch-1.0 \
    tcpclientsrc host=192.168.6.60 port=8080 ! \
    application/x-rtp,encoding-name=H264,payload=96 ! \
    rtph264depay ! \
    h264parse ! \
    avdec_h264 ! \
    videoconvert ! \
    autovideosink
```

**其他方法**：
- 如果 `waylandsrc` 插件可用，也可以使用 `view_output.sh` 脚本
- 查看 `VIEW_OUTPUT.md` 了解详细信息和更多选项

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
  --output <OUTPUT>            输出方式：none（默认，不输出）、appsrc（通过 appsrc 暴露）、rtsp（RTSP 服务器） [default: none]
  --output-address <ADDRESS>   输出地址（当 output=appsrc 时使用，格式：host:port） [default: 127.0.0.1:5000]
  --protocol <PROTOCOL>        传输协议（udp 或 tcp，当 output=appsrc 时使用） [default: udp]
  --rtsp-port <RTSP_PORT>      RTSP 服务器端口（当 output=rtsp 时使用） [default: 8554]
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

### 问题：无法启动输出流（找不到编码器）

**错误信息**：`Failed to find element factory with name 'x264enc'`

**解决方案**：
- 安装 GStreamer 插件（见系统要求部分）
- 程序会自动尝试使用可用的编码器（按优先级）：
  1. `vaapih264enc`（Intel/AMD 硬件加速）
  2. `nvh264enc`（NVIDIA 硬件加速）
  3. `x264enc`（软件编码，需要 gstreamer1.0-plugins-good）
  4. `avenc_h264`（软件编码，需要 gstreamer1.0-plugins-bad）
- 如果所有编码器都不可用，程序会显示详细的错误信息和安装建议

## 许可证

本项目基于 gst-wayland-display，遵循相应的许可证。

## 相关项目

- [gst-wayland-display](https://github.com/games-on-whales/gst-wayland-display) - 原始项目
- [smithay](https://github.com/Smithay/smithay) - Wayland compositor 库
- [Weston](https://gitlab.freedesktop.org/wayland/weston) - 参考 Wayland compositor 实现

