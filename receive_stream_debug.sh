#!/bin/bash

# 使用 GStreamer 接收 weadless compositor 输出流的调试脚本
#
# 使用方法：
#   ./receive_stream_debug.sh [port]
#
# 默认端口: 8080

PORT=${1:-8080}

echo "正在接收 UDP 流，端口: $PORT"
echo "按 Ctrl+C 停止"
echo ""
echo "调试信息："
echo "- 使用 GST_DEBUG=3 显示详细日志"
echo "- 添加 fakesink 来测试是否接收到数据"
echo ""

# 首先测试是否接收到数据
echo "=== 测试 1: 检查是否接收到数据（使用 fakesink）==="
timeout 5 gst-launch-1.0 -v \
    udpsrc port=$PORT caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtph264depay ! \
    fakesink dump=true 2>&1 | head -20

echo ""
echo "=== 测试 2: 尝试显示视频（使用 osxvideosink）==="
gst-launch-1.0 -v \
    udpsrc port=$PORT caps="application/x-rtp,media=video,encoding-name=H264,payload=96" ! \
    rtph264depay ! \
    avdec_h264 ! \
    videoconvert ! \
    osxvideosink sync=false

