#!/bin/bash

# GStreamer 测试显示输出流画面的脚本
# 使用方法：
#   1. 先启动 weadless compositor（在另一个终端）
#   2. 设置 WAYLAND_DISPLAY 环境变量
#   3. 运行此脚本

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
echo "选择测试类型:"
echo "1) 测试图案 (test pattern)"
echo "2) 彩色渐变 (color gradient)"
echo "3) 球体动画 (animated ball)"
echo "4) 播放视频文件"
echo "5) 摄像头输入 (如果可用)"
echo ""
read -p "请选择 (1-5): " choice

case $choice in
    1)
        echo "生成测试图案..."
        gst-launch-1.0 \
            videotestsrc pattern=smpte ! \
            video/x-raw,width=1920,height=1080,framerate=60/1 ! \
            waylandsink sync=false
        ;;
    2)
        echo "生成彩色渐变..."
        gst-launch-1.0 \
            videotestsrc pattern=ball ! \
            video/x-raw,width=1920,height=1080,framerate=60/1 ! \
            waylandsink sync=false
        ;;
    3)
        echo "生成球体动画..."
        gst-launch-1.0 \
            videotestsrc pattern=ball is-live=true ! \
            video/x-raw,width=1920,height=1080,framerate=60/1 ! \
            waylandsink sync=false
        ;;
    4)
        read -p "请输入视频文件路径: " video_file
        if [ ! -f "$video_file" ]; then
            echo "错误: 文件不存在: $video_file"
            exit 1
        fi
        echo "播放视频文件: $video_file"
        gst-launch-1.0 \
            filesrc location="$video_file" ! \
            decodebin ! \
            videoconvert ! \
            video/x-raw,width=1920,height=1080,framerate=60/1 ! \
            waylandsink sync=false
        ;;
    5)
        echo "尝试从摄像头捕获..."
        gst-launch-1.0 \
            v4l2src device=/dev/video0 ! \
            video/x-raw,width=1920,height=1080,framerate=30/1 ! \
            videoconvert ! \
            waylandsink sync=false
        ;;
    *)
        echo "无效选择"
        exit 1
        ;;
esac

