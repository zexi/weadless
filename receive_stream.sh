#!/bin/bash

# 使用 GStreamer 接收 weadless compositor 输出流的客户端脚本
#
# 使用方法：
#   ./receive_stream.sh [port]
#
# 默认端口: 5000

PORT=${1:-5000}

echo "正在接收 UDP 流，端口: $PORT"
echo "按 Ctrl+C 停止"
echo ""

# 检测操作系统，macOS 使用 osxvideosink，Linux 使用 autovideosink
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "检测到 macOS，使用 osxvideosink"
    export GST_GL_API=disable
    SINK="osxvideosink"
else
    echo "检测到 Linux，使用 autovideosink"
    SINK="autovideosink"
fi

gst-launch-1.0 -v \
    udpsrc port=$PORT caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtph264depay ! \
    h264parse ! \
    avdec_h264 ! \
    videoconvert ! \
    $SINK sync=false

