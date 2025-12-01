#!/bin/bash

# macOS 专用的接收脚本
# 使用方法：./receive_stream_macos.sh [port]

PORT=${1:-5000}

echo "macOS 接收 UDP 流，端口: $PORT"
echo "按 Ctrl+C 停止"
echo ""
echo "注意：macOS 上 autovideosink 可能有 OpenGL 问题，使用 osxvideosink"
echo ""

# 禁用 OpenGL 硬件加速，使用软件渲染
export GST_GL_API=disable

# 在 macOS 上使用 osxvideosink（macOS 专用，避免 OpenGL 问题）
# 添加更多参数以确保正确接收 RTP 流
# 注意：不要指定 format=BGRA，让 videoconvert 自动转换为 osxvideosink 支持的格式
gst-launch-1.0 -v \
    udpsrc port=$PORT \
        caps="application/x-rtp,media=video,clock-rate=90000,encoding-name=H264,payload=96" \
        timeout=5000000000 \
        buffer-size=524288 ! \
    rtph264depay ! \
    h264parse ! \
    avdec_h264 ! \
    videoconvert ! \
    osxvideosink sync=false

