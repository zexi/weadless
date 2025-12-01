#!/bin/bash

# 使用 GStreamer 通过 TCP 接收 weadless compositor 输出流的客户端脚本
#
# 使用方法：
#   ./receive_stream_tcp.sh [host] [port]
#
# 默认: localhost:8080

HOST=${1:-localhost}
PORT=${2:-8080}

echo "正在通过 TCP 接收流，地址: $HOST:$PORT"
echo "按 Ctrl+C 停止"
echo ""

gst-launch-1.0 -v \
    tcpclientsrc host=$HOST port=$PORT ! \
    application/x-rtp,encoding-name=H264,payload=96 ! \
    rtph264depay ! \
    h264parse ! \
    avdec_h264 ! \
    videoconvert ! \
    autovideosink sync=false

