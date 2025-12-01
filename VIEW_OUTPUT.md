# 如何查看 weadless compositor 的输出流

本文档介绍如何使用 GStreamer 查看 weadless compositor 的输出流画面。

## 问题说明

当你启动 `weadless` compositor 并运行 `WAYLAND_DISPLAY=wayland-1 weston-terminal` 后，你希望能够在客户端使用 GStreamer 查看 compositor 的输出流（即整个 compositor 的画面）。

## 解决方案

根据 `gst-wayland-display` 的设计，compositor 的输出流需要通过 GStreamer pipeline 来访问。有几种方法可以实现：

### 方法 1: 使用 waylandsrc（如果可用）

如果系统支持 `waylandsrc` 插件，可以直接捕获 Wayland 显示：

```bash
# 设置环境变量
export WAYLAND_DISPLAY=wayland-1
export XDG_RUNTIME_DIR=/run/user/$(id -u)

# 使用 waylandsrc 捕获并显示
gst-launch-1.0 \
    waylandsrc display-name=wayland-1 ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    videoconvert ! \
    autovideosink
```

**注意**：`waylandsrc` 插件可能不是所有 GStreamer 版本都包含，需要检查是否可用：

```bash
gst-inspect-1.0 waylandsrc
```

### 方法 2: 使用 weadless 内置的输出流暴露功能（已实现）✅

`weadless` 现在支持通过 `--output appsrc` 选项暴露输出流。使用方法：

```bash
# 启动 compositor 并启用 UDP 输出流
./target/release/weadless --output appsrc --udp-address 127.0.0.1:5000

# 在客户端使用 GStreamer 接收并显示
gst-launch-1.0 \
    udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtph264depay ! \
    avdec_h264 ! \
    videoconvert ! \
    autovideosink
```

**工作原理**：
1. `weadless` 使用 `frame()` 方法获取 compositor 的输出帧
2. 通过 `appsrc` 将帧推送到 GStreamer pipeline
3. Pipeline 将帧编码为 H.264 并通过 UDP 发送
4. 客户端通过 UDP 接收并解码显示

### 方法 3: 使用共享内存或网络流

1. **使用 RTSP 服务器**：修改 `weadless` 以通过 RTSP 服务器暴露输出流
2. **使用 UDP/TCP 流**：通过 UDP 或 TCP 发送视频流
3. **使用共享内存**：通过共享内存（如 `/dev/shm`）传输帧数据

### 方法 4: 使用 screen capture 工具

如果 compositor 运行在支持屏幕捕获的环境中，可以使用：

```bash
# 使用 wf-recorder（Wayland 屏幕录制工具）
wf-recorder -m wayland-1 -f output.mp4

# 或者使用其他 Wayland 屏幕捕获工具
```

## 已实现的方案 ✅

`weadless` 现在已经实现了通过 `appsrc` 暴露输出流的功能。

### 使用方法

**启动 compositor 并启用输出流**：
```bash
./target/release/weadless --output appsrc --udp-address 127.0.0.1:5000
```

**客户端接收并显示**：
```bash
gst-launch-1.0 \
    udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtph264depay ! \
    avdec_h264 ! \
    videoconvert ! \
    autovideosink
```

### 实现细节

1. **帧获取**：在主循环中使用 `display.frame()` 方法获取 compositor 的输出帧
2. **帧推送**：通过 `appsrc` 将帧推送到 GStreamer pipeline
3. **编码和传输**：Pipeline 使用 `x264enc` 编码为 H.264，通过 `udpsink` 发送到指定地址
4. **客户端接收**：客户端使用 `udpsrc` 接收 RTP 流，解码并显示

### 完整示例

```bash
# 终端 1: 启动 compositor 并启用输出流
./target/release/weadless --output appsrc --udp-address 127.0.0.1:5000 --width 1920 --height 1080

# 终端 2: 启动客户端应用
export WAYLAND_DISPLAY=wayland-1
WAYLAND_DISPLAY=wayland-1 weston-terminal

# 终端 3: 接收并显示输出流
gst-launch-1.0 \
    udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtph264depay ! \
    avdec_h264 ! \
    videoconvert ! \
    autovideosink
```

### 未来计划

- [ ] RTSP 服务器支持（`--output rtsp`）
- [ ] TCP 传输支持
- [ ] 可配置的编码参数

## 快速测试方案（临时）

在实现完整方案之前，可以使用以下临时方案：

### 方案 A: 使用 wf-recorder

```bash
# 终端 1: 启动 compositor
./target/release/weadless --width 1920 --height 1080

# 终端 2: 启动 weston-terminal
export WAYLAND_DISPLAY=wayland-1
WAYLAND_DISPLAY=wayland-1 weston-terminal

# 终端 3: 录制并实时查看（如果支持）
wf-recorder -m wayland-1 -f - | gst-launch-1.0 fdsrc ! decodebin ! videoconvert ! autovideosink
```

### 方案 B: 使用 GStreamer 的 waylandsink 反向

如果 `waylandsrc` 不可用，可以考虑使用其他方式：

```bash
# 使用 xdg-desktop-portal 或其他屏幕捕获 API
# 这需要额外的配置
```

## 完整实现示例

查看 `src/viewer.rs` 文件，那里有一个基础的实现框架。要完整实现，需要：

1. 修改 `main.rs` 添加输出选项
2. 实现帧获取和推送逻辑
3. 创建相应的 GStreamer pipeline

## 相关资源

- [GStreamer waylandsrc 文档](https://gstreamer.freedesktop.org/documentation/wayland/waylandsrc.html)
- [GStreamer appsrc 文档](https://gstreamer.freedesktop.org/documentation/app/appsrc.html)
- [GStreamer RTSP 服务器文档](https://gstreamer.freedesktop.org/documentation/gst-rtsp-server/index.html)

