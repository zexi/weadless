//! 使用 GStreamer 查看 weadless compositor 输出流的客户端程序
//!
//! 使用方法：
//!   1. 启动 weadless compositor
//!   2. 设置 WAYLAND_DISPLAY 环境变量
//!   3. 运行此程序：cargo run --bin viewer

use gst::prelude::*;
use gst_app::AppSrc;
use std::sync::mpsc;
use tracing::info;

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // 初始化 GStreamer
    gst::init().expect("Failed to initialize GStreamer");

    info!("连接到 weadless compositor...");

    // 从环境变量获取 WAYLAND_DISPLAY
    let wayland_display = std::env::var("WAYLAND_DISPLAY")
        .expect("WAYLAND_DISPLAY 环境变量未设置");

    info!("WAYLAND_DISPLAY={}", wayland_display);

    // 创建 WaylandDisplay（连接到现有的 compositor）
    // 注意：这里我们需要连接到已运行的 compositor
    // 但实际上，我们需要从 compositor 获取输出流
    // 让我们创建一个简单的 viewer，通过 frame() 方法获取帧并显示

    // 由于 WaylandDisplay::new 会创建新的 compositor，
    // 我们需要另一种方式来获取输出流
    // 实际上，compositor 的输出应该通过 GStreamer pipeline 暴露

    // 创建 GStreamer pipeline 来显示 compositor 的输出
    // 注意：这需要 compositor 通过 appsrc 或其他方式暴露输出流
    // 由于 wayland-display-core 的设计，输出流可能已经通过内部 pipeline 暴露

    // 让我们尝试使用 waylandsrc 来捕获 compositor 的输出
    // 但实际上，我们需要的是从 compositor 获取帧数据

    // 创建一个简单的测试：使用 appsrc 来接收帧数据
    let pipeline = gst::Pipeline::new();

    // 创建 appsrc 元素
    let appsrc = AppSrc::builder()
        .name("source")
        .caps(&gst::Caps::builder("video/x-raw")
            .field("format", "RGBx")
            .field("width", 1920i32)
            .field("height", 1080i32)
            .field("framerate", gst::Fraction::new(60, 1))
            .build())
        .build();

    // 创建 videoconvert 和 autovideosink
    let videoconvert = gst::ElementFactory::make("videoconvert").build().unwrap();
    let sink = gst::ElementFactory::make("autovideosink").build().unwrap();

    // 添加元素到 pipeline
    pipeline.add_many(&[appsrc.upcast_ref(), &videoconvert, &sink]).unwrap();
    gst::Element::link_many(&[appsrc.upcast_ref(), &videoconvert, &sink]).unwrap();

    info!("GStreamer pipeline 已创建");
    info!("注意：此示例需要修改 weadless 以通过 appsrc 暴露输出流");
    info!("或者使用其他方式（如共享内存、socket）来传输帧数据");

    // 启动 pipeline
    pipeline.set_state(gst::State::Playing).unwrap();

    info!("Pipeline 正在运行...");
    info!("按 Ctrl+C 退出");

    // 等待退出信号
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        info!("收到退出信号，正在关闭...");
        tx.send(()).unwrap();
    })
    .expect("无法设置 Ctrl+C 处理器");

    rx.recv().unwrap();

    // 停止 pipeline
    pipeline.set_state(gst::State::Null).unwrap();
    info!("已退出");
}

