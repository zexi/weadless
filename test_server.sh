#!/bin/bash

# 测试服务器端发送脚本
# 使用方法：./test_server.sh [host] [port]

HOST=${1:-192.168.204.165}
PORT=${2:-5000}

echo "测试服务器端发送 UDP 流到 $HOST:$PORT"
echo "按 Ctrl+C 停止"
echo ""

# 使用 videotestsrc 测试，确保 rtph264pay 配置正确
GST_PLUGIN_PATH=/usr/local/lib/gstreamer-1.0 gst-launch-1.0 -v \
    videotestsrc pattern=ball is-live=true ! \
    video/x-raw,width=1920,height=1080,framerate=30/1 ! \
    videoconvert ! \
    nvh264enc ! \
    rtph264pay config-interval=1 pt=96 ! \
    udpsink host=$HOST port=$PORT sync=false

