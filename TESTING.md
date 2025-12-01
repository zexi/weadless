# GStreamer 测试显示输出流画面指南

本文档介绍如何使用 GStreamer 测试 weadless compositor 的输出流画面。

## 快速开始

### 步骤 1: 启动 compositor

在一个终端中启动 weadless compositor：

```bash
./target/release/weadless --width 1920 --height 1080
```

程序会输出 Wayland socket 信息，例如：
```
✓ Wayland compositor 已启动
  Socket: wayland-1
```

### 步骤 2: 设置环境变量

在另一个终端中设置环境变量：

```bash
export WAYLAND_DISPLAY=wayland-1
export XDG_RUNTIME_DIR=/run/user/$(id -u)
```

### 步骤 3: 运行测试

#### 方法 1: 使用测试脚本

```bash
chmod +x test_display.sh
./test_display.sh
```

#### 方法 2: 直接使用 gst-launch-1.0 命令

## 常用测试命令

### 1. 测试图案 (SMPTE 测试图案)

```bash
gst-launch-1.0 \
    videotestsrc pattern=smpte ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false
```

### 2. 彩色球体动画

```bash
gst-launch-1.0 \
    videotestsrc pattern=ball is-live=true ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false
```

### 3. 彩色渐变

```bash
gst-launch-1.0 \
    videotestsrc pattern=ball ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false
```

### 4. 纯色测试

```bash
# 红色
gst-launch-1.0 \
    videotestsrc pattern=red ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false

# 绿色
gst-launch-1.0 \
    videotestsrc pattern=green ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false

# 蓝色
gst-launch-1.0 \
    videotestsrc pattern=blue ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false
```

### 5. 播放视频文件

```bash
gst-launch-1.0 \
    filesrc location=/path/to/video.mp4 ! \
    decodebin ! \
    videoconvert ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false
```

### 6. 摄像头输入（如果可用）

```bash
gst-launch-1.0 \
    v4l2src device=/dev/video0 ! \
    video/x-raw,width=1920,height=1080,framerate=30/1 ! \
    videoconvert ! \
    waylandsink sync=false
```

### 7. 网络流（RTSP/HTTP）

```bash
# RTSP 流
gst-launch-1.0 \
    rtspsrc location=rtsp://example.com/stream ! \
    rtph264depay ! \
    h264parse ! \
    avdec_h264 ! \
    videoconvert ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false

# HTTP 流
gst-launch-1.0 \
    souphttpsrc location=http://example.com/video.mp4 ! \
    decodebin ! \
    videoconvert ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    waylandsink sync=false
```

## 可用的测试图案类型

`videotestsrc` 支持以下图案类型：

- `smpte` - SMPTE 测试图案（标准电视测试图案）
- `ball` - 彩色球体动画
- `red` - 纯红色
- `green` - 纯绿色
- `blue` - 纯蓝色
- `white` - 纯白色
- `black` - 纯黑色
- `checkers-1` - 棋盘格图案
- `checkers-2` - 另一种棋盘格图案
- `checkers-4` - 更小的棋盘格图案
- `checkers-8` - 最小的棋盘格图案
- `circular` - 圆形图案
- `blink` - 闪烁图案
- `smpte75` - SMPTE 75% 测试图案
- `zone-plate` - 区域板图案
- `gamut` - 色域测试图案
- `chroma-zone-plate` - 色度区域板图案
- `solid-color` - 纯色
- `moving-ball` - 移动的球体
- `spokes` - 轮辐图案
- `gradient` - 渐变图案
- `colors` - 彩色条

## 参数说明

### waylandsink 参数

- `sync=false` - 禁用同步，提高性能（适合测试）
- `sync=true` - 启用同步，确保帧率稳定（默认）

### videotestsrc 参数

- `pattern=<type>` - 测试图案类型
- `is-live=true` - 启用实时模式
- `num-buffers=<n>` - 限制生成的帧数（用于测试）

### video/x-raw 参数

- `width=<n>` - 视频宽度（像素）
- `height=<n>` - 视频高度（像素）
- `framerate=<n>/1` - 帧率（fps）

## 故障排除

### 问题：waylandsink 无法连接

**解决方案**：
- 确保 `WAYLAND_DISPLAY` 环境变量已正确设置
- 确保 compositor 正在运行
- 检查 socket 文件是否存在：`ls $XDG_RUNTIME_DIR/wayland-*`

### 问题：视频格式不匹配

**解决方案**：
- 确保视频格式与 compositor 设置的格式匹配（默认是 RGBx）
- 使用 `videoconvert` 进行格式转换

### 问题：性能问题

**解决方案**：
- 使用 `sync=false` 禁用同步
- 降低分辨率或帧率
- 使用硬件加速（如果可用）

## 高级用法

### 录制输出流

```bash
gst-launch-1.0 \
    videotestsrc pattern=ball ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    tee name=t ! \
    queue ! waylandsink sync=false \
    t. ! queue ! videoconvert ! x264enc ! mp4mux ! filesink location=output.mp4
```

### 多路输出

```bash
gst-launch-1.0 \
    videotestsrc pattern=ball ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    tee name=t ! \
    queue ! waylandsink sync=false \
    t. ! queue ! videoconvert ! jpegenc ! multifilesink location=frame-%05d.jpg
```

### 添加文字叠加

```bash
gst-launch-1.0 \
    videotestsrc pattern=ball ! \
    video/x-raw,width=1920,height=1080,framerate=60/1 ! \
    textoverlay text="测试文字" valignment=top halignment=left ! \
    waylandsink sync=false
```

## 参考资源

- [GStreamer 官方文档](https://gstreamer.freedesktop.org/documentation/)
- [waylandsink 文档](https://gstreamer.freedesktop.org/documentation/wayland/waylandsink.html)
- [videotestsrc 文档](https://gstreamer.freedesktop.org/documentation/videotestsrc/videotestsrc.html)

