#!/bin/bash

# 使用 GStreamer 查看 weadless compositor 输出流的脚本
#
# 使用方法：
#   1. 先启动 weadless compositor（在另一个终端）
#   2. 设置 WAYLAND_DISPLAY 环境变量
#   3. 运行此脚本：./view_output.sh

set -e

# 检查 WAYLAND_DISPLAY 是否设置
if [ -z "$WAYLAND_DISPLAY" ]; then
    echo "错误: WAYLAND_DISPLAY 环境变量未设置"
    echo "请先启动 weadless compositor，然后设置:"
    echo "  export WAYLAND_DISPLAY=wayland-1"
    exit 1
fi

echo "使用 WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
echo ""

# 检查 waylandsrc 是否可用
if gst-inspect-1.0 waylandsrc > /dev/null 2>&1; then
    echo "✓ 检测到 waylandsrc 插件"
    echo "使用 waylandsrc 捕获 compositor 输出..."
    echo ""
    
    gst-launch-1.0 -v \
        waylandsrc display-name="$WAYLAND_DISPLAY" ! \
        video/x-raw,width=1920,height=1080,framerate=60/1 ! \
        videoconvert ! \
        autovideosink sync=false
else
    echo "✗ waylandsrc 插件不可用"
    echo ""
    echo "可用的替代方案："
    echo ""
    echo "方案 1: 使用 wf-recorder（如果已安装）"
    echo "  wf-recorder -m $WAYLAND_DISPLAY -f - | gst-launch-1.0 fdsrc ! decodebin ! videoconvert ! autovideosink"
    echo ""
    echo "方案 2: 修改 weadless 以通过 RTSP 或 appsrc 暴露输出流"
    echo "  查看 VIEW_OUTPUT.md 了解详细信息"
    echo ""
    echo "方案 3: 使用其他屏幕捕获工具"
    echo "  例如：grim、slurp 等"
    exit 1
fi

