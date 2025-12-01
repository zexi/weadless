# 故障排除指南

## macOS 客户端显示绿色窗口但没有内容

### 问题描述
macOS 客户端连接后显示绿色窗口，但没有视频内容。

### 可能原因
1. **RTP 配置不匹配**：服务端的 `rtph264pay` 配置不正确
2. **缺少 SPS/PPS**：H.264 流缺少配置信息
3. **编码器配置问题**：编码器参数不正确

### 解决方案

**服务端命令（确保配置正确）**：
```bash
gst-launch-1.0 -v \
    videotestsrc pattern=ball is-live=true ! \
    video/x-raw,width=1920,height=1080,framerate=30/1 ! \
    videoconvert ! \
    nvh264enc ! \
    rtph264pay config-interval=1 pt=96 ! \
    udpsink host=192.168.204.165 port=5000 sync=false
```

**关键参数说明**：
- `config-interval=1`：每 1 秒发送一次 SPS/PPS（H.264 配置信息）
- `pt=96`：RTP payload type，必须与客户端匹配
- `sync=false`：禁用同步，提高性能

**客户端命令（macOS）**：
```bash
export GST_GL_API=disable
gst-launch-1.0 -v \
    udpsrc port=5000 \
        caps="application/x-rtp,media=video,clock-rate=90000,encoding-name=H264,payload=96" \
        buffer-size=524288 ! \
    rtph264depay ! \
    h264parse ! \
    avdec_h264 ! \
    videoconvert ! \
    osxvideosink sync=false
```

### 调试步骤

1. **检查是否接收到数据**：
```bash
# 使用 fakesink 测试
gst-launch-1.0 udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtph264depay ! fakesink dump=true
```

2. **检查 RTP 包**：
```bash
# 使用 rtpbin 分析
gst-launch-1.0 udpsrc port=5000 caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtpbin.recv_rtp_sink_0 rtpbin. ! rtph264depay ! h264parse ! fakesink dump=true
```

3. **使用 weadless 发送（推荐）**：
```bash
# 使用 weadless 内置的输出流功能，配置已经优化
./target/release/weadless --output appsrc --output-address 192.168.204.165:5000 --protocol udp
```

## 常见问题

### 1. 绿色窗口但没有内容
- 检查服务端 `rtph264pay` 的 `config-interval` 设置
- 确保客户端使用 `h264parse` 元素
- 检查网络连接和防火墙

### 2. 完全没有窗口
- macOS 必须使用 `osxvideosink`，不能使用 `autovideosink`
- 设置 `export GST_GL_API=disable` 禁用 OpenGL
- 检查是否接收到数据（使用 fakesink 测试）

### 3. 延迟很高
- 设置 `sync=false` 禁用同步
- 增加 UDP buffer size：`buffer-size=524288`
- 使用硬件编码器（如 nvh264enc）

### 4. 连接失败
- 检查防火墙设置
- 确保服务端和客户端在同一网络
- 检查 IP 地址和端口是否正确

