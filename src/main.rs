use clap::Parser;
use gst_video::VideoInfo;
use std::sync::mpsc;
use std::time::Duration;
use tracing::{info, warn};
use wayland_display_core::{GstVideoInfo, WaylandDisplay};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 渲染节点路径（例如 /dev/dri/renderD128），使用 "software" 进行软件渲染
    #[arg(long, default_value = "software")]
    render_node: String,

    /// 输出宽度（像素）
    #[arg(long, default_value_t = 1920)]
    width: u32,

    /// 输出高度（像素）
    #[arg(long, default_value_t = 1080)]
    height: u32,

    /// 帧率（fps）
    #[arg(long, default_value_t = 60)]
    fps: u32,

    /// 视频格式（RGBx, RGBA, BGRx, BGRA）
    #[arg(long, default_value = "RGBx")]
    format: String,
}

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

    let args = Args::parse();

    info!("启动 headless Wayland compositor...");
    info!(
        "配置: {}x{} @ {}fps, 格式: {}",
        args.width, args.height, args.fps, args.format
    );

    // 创建 WaylandDisplay
    let mut display = match WaylandDisplay::new(Some(args.render_node.clone())) {
        Ok(display) => display,
        Err(e) => {
            eprintln!("无法创建 WaylandDisplay: {:?}", e);
            eprintln!("提示: 如果使用软件渲染，请使用 --render-node software");
            std::process::exit(1);
        }
    };

    // 等待 compositor 线程初始化完成
    std::thread::sleep(Duration::from_millis(100));

    // 获取环境变量（包括 Wayland socket）
    let env_vars: Vec<String> = display.env_vars().map(|s| s.to_string()).collect();
    for env_var in &env_vars {
        info!("环境变量: {}", env_var);
        if env_var.starts_with("WAYLAND_DISPLAY=") {
            let socket = env_var.strip_prefix("WAYLAND_DISPLAY=").unwrap();
            println!("\n✓ Wayland compositor 已启动");
            println!("  Socket: {}", socket);
            println!("  使用以下命令连接:");
            println!("    export WAYLAND_DISPLAY={}", socket);
            println!("    # 然后启动你的 Wayland 应用，例如:");
            println!("    # WAYLAND_DISPLAY={} weston-terminal", socket);
            println!();
        }
    }

    // 创建视频信息
    let video_format = match args.format.as_str() {
        "RGBx" => gst_video::VideoFormat::Rgbx,
        "RGBA" => gst_video::VideoFormat::Rgba,
        "BGRx" => gst_video::VideoFormat::Bgrx,
        "BGRA" => gst_video::VideoFormat::Bgra,
        _ => {
            warn!("未知格式 {}, 使用 RGBx", args.format);
            gst_video::VideoFormat::Rgbx
        }
    };

    let video_info = VideoInfo::builder(video_format, args.width, args.height)
        .fps(gst::Fraction::new(args.fps as i32, 1))
        .build()
        .expect("Failed to build VideoInfo");

    // 设置视频信息（这会创建输出）
    display.set_video_info(GstVideoInfo::RAW(video_info));

    info!("Wayland compositor 运行中...");
    info!("按 Ctrl+C 退出");

    // 保持程序运行
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        info!("收到退出信号，正在关闭...");
        tx.send(()).unwrap();
    })
    .expect("无法设置 Ctrl+C 处理器");

    // 等待退出信号
    rx.recv().unwrap();

    info!("正在清理资源...");
    // display 会在 drop 时自动清理
}
