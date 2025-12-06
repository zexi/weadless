use clap::Parser;
use gst::prelude::*;
use gst_app::AppSrc;
use gst_video::VideoInfo;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info, warn, debug};
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

    /// 输出方式：none（默认，不输出）、appsrc（通过 appsrc 暴露到 UDP）、rtsp（RTSP 服务器）、vnc（VNC 服务器）
    #[arg(long, default_value = "none")]
    output: String,

    /// 输出地址（当 output=appsrc 时使用，格式：host:port）
    #[arg(long, default_value = "127.0.0.1:5000")]
    output_address: String,

    /// 传输协议（udp 或 tcp，当 output=appsrc 时使用）
    #[arg(long, default_value = "udp")]
    protocol: String,

    /// RTSP 服务器端口（当 output=rtsp 时使用）
    #[arg(long, default_value_t = 8554)]
    rtsp_port: u16,

    /// VNC 服务器端口（当 output=vnc 时使用）
    #[arg(long, default_value_t = 5900)]
    vnc_port: u16,

    /// VNC 密码（当 output=vnc 时使用，留空则不设置密码）
    #[arg(long)]
    vnc_password: Option<String>,
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
    display.set_video_info(GstVideoInfo::RAW(video_info.clone()));

    // 根据输出选项创建相应的输出
    let (stop_tx, stop_rx) = mpsc::channel();

    // 区分不同的输出类型
    enum OutputType {
        AppSrc(AppSrc, mpsc::Sender<()>),
        Vnc(Arc<Mutex<rustvncserver::VncServer>>, mpsc::Sender<()>),
    }

    let output_opt = match args.output.as_str() {
        "appsrc" => {
            info!("使用 appsrc 方式暴露输出流到 {}: {}", args.protocol.to_uppercase(), args.output_address);
            match start_appsrc_output(
                video_info.clone(),
                args.output_address.clone(),
                args.protocol.as_str(),
            ) {
                Ok((appsrc, tx)) => Some(OutputType::AppSrc(appsrc, tx)),
                Err(e) => {
                    error!("无法启动输出流: {}", e);
                    eprintln!("错误: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "rtsp" => {
            info!("使用 RTSP 服务器暴露输出流，端口: {}", args.rtsp_port);
            warn!("RTSP 服务器功能尚未实现，请使用 --output appsrc");
            // TODO: 实现 RTSP 服务器
            None
        }
        "vnc" => {
            info!("使用 VNC 服务器暴露输出流，端口: {}", args.vnc_port);
            match start_vnc_output(
                video_info.clone(),
                args.vnc_port,
                args.vnc_password.clone(),
            ) {
                Ok((vnc_server, tx)) => Some(OutputType::Vnc(vnc_server, tx)),
                Err(e) => {
                    error!("无法启动 VNC 服务器: {}", e);
                    eprintln!("错误: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            info!("未启用输出流暴露（使用 --output appsrc、--output rtsp 或 --output vnc 启用）");
            None
        }
    };

    info!("Wayland compositor 运行中...");
    info!("按 Ctrl+C 退出");

    // 设置 Ctrl+C 处理器
    let output_opt_clone = output_opt.clone();
    ctrlc::set_handler(move || {
        info!("收到退出信号，正在关闭...");
        let _ = stop_tx.send(());
        // 如果启用了输出流，也通知帧推送线程停止
        if let Some(OutputType::AppSrc(_, ref tx)) = output_opt_clone {
            let _ = tx.send(());
        } else if let Some(OutputType::Vnc(_, ref tx)) = output_opt_clone {
            let _ = tx.send(());
        }
    })
    .expect("无法设置 Ctrl+C 处理器");

    // 主循环：如果启用了输出流，在主循环中获取帧并推送
    match output_opt {
        Some(OutputType::AppSrc(ref appsrc, _)) => {
        let target_frame_duration = Duration::from_secs_f64(1.0 / video_info.fps().numer() as f64);
        let mut frame_count = 0u64;
        let start_time = Instant::now();

        loop {
            // 检查是否收到停止信号（带超时，避免阻塞）
            match stop_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(_) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // 超时是正常的，继续处理帧
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            // 获取帧并推送
            match display.frame() {
                Ok(buffer) => {
                    let result = appsrc.push_buffer(buffer);
                    if let Err(e) = result {
                        error!("推送 buffer 失败: {:?}", e);
                    } else {
                        frame_count += 1;
                        if frame_count % 60 == 0 {
                            let elapsed = start_time.elapsed();
                            let fps = frame_count as f64 / elapsed.as_secs_f64();
                            debug!("已推送 {} 帧，平均帧率: {:.2} fps", frame_count, fps);
                        }
                    }
                }
                Err(e) => {
                    let err_str = format!("{:?}", e);
                    if err_str.contains("Flushing") || err_str.contains("Eos") {
                        info!("Pipeline 正在关闭: {:?}", e);
                        break;
                    }
                    warn!("获取帧失败: {:?}，继续尝试...", e);
                }
            }

            // 控制帧率
            thread::sleep(target_frame_duration);
        }

            // 发送 EOS
            let _ = appsrc.end_of_stream();
        }
        Some(OutputType::Vnc(ref vnc_server, _)) => {
            let target_frame_duration = Duration::from_secs_f64(1.0 / video_info.fps().numer() as f64);
            let mut frame_count = 0u64;
            let start_time = Instant::now();

            loop {
                // 检查是否收到停止信号（带超时，避免阻塞）
                match stop_rx.recv_timeout(Duration::from_millis(10)) {
                    Ok(_) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // 超时是正常的，继续处理帧
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }

                // 获取帧并发送到 VNC 服务器
                match display.frame() {
                    Ok(buffer) => {
                        if let Err(e) = send_frame_to_vnc(vnc_server, &buffer, &video_info) {
                            error!("发送帧到 VNC 服务器失败: {:?}", e);
                        } else {
                            frame_count += 1;
                            if frame_count % 60 == 0 {
                                let elapsed = start_time.elapsed();
                                let fps = frame_count as f64 / elapsed.as_secs_f64();
                                debug!("已发送 {} 帧到 VNC，平均帧率: {:.2} fps", frame_count, fps);
                            }
                        }
                    }
                    Err(e) => {
                        let err_str = format!("{:?}", e);
                        if err_str.contains("Flushing") || err_str.contains("Eos") {
                            info!("正在关闭: {:?}", e);
                            break;
                        }
                        warn!("获取帧失败: {:?}，继续尝试...", e);
                    }
                }

                // 控制帧率
                thread::sleep(target_frame_duration);
            }
        }
        None => {
            // 未启用输出流，直接等待退出信号
            stop_rx.recv().unwrap();
        }
    }

    info!("正在清理资源...");
    // display 会在 drop 时自动清理
}

/// 使用 appsrc 方式暴露输出流
/// 返回 appsrc 和停止信号的发送端
/// 注意：frame() 必须在创建 WaylandDisplay 的线程中调用
fn start_appsrc_output(
    video_info: VideoInfo,
    output_address: String,
    protocol: &str,
) -> Result<(AppSrc, mpsc::Sender<()>), String> {
    // 创建 GStreamer pipeline
    let pipeline = gst::Pipeline::new();

    // 创建 appsrc 元素
    let appsrc = AppSrc::builder()
        .name("source")
        .caps(&gst::Caps::builder("video/x-raw")
            .field("format", video_info.format().to_string())
            .field("width", video_info.width() as i32)
            .field("height", video_info.height() as i32)
            .field("framerate", video_info.fps())
            .build())
        .format(gst::Format::Time)
        .is_live(true)
        .build();

    // 创建 videoconvert、编码器、rtph264pay 和 udpsink
    let videoconvert = gst::ElementFactory::make("videoconvert")
        .build()
        .map_err(|e| format!("无法创建 videoconvert: {:?}", e))?;
    
    // 尝试使用不同的 H.264 编码器（按优先级顺序）
    let encoder = if gst::ElementFactory::find("vaapih264enc").is_some() {
        info!("使用 vaapih264enc（硬件加速）");
        gst::ElementFactory::make("vaapih264enc")
            .property("tune", "low-power")
            .build()
            .map_err(|e| format!("无法创建 vaapih264enc: {:?}", e))?
    } else if gst::ElementFactory::find("nvh264enc").is_some() {
        info!("使用 nvh264enc（NVIDIA 硬件加速）");
        // nvh264enc 的属性需要枚举类型，这里使用默认配置
        // 默认配置已经针对低延迟进行了优化
        gst::ElementFactory::make("nvh264enc")
            .build()
            .map_err(|e| format!("无法创建 nvh264enc: {:?}", e))?
    } else if gst::ElementFactory::find("x264enc").is_some() {
        info!("使用 x264enc（软件编码）");
        gst::ElementFactory::make("x264enc")
            .property("tune", "zerolatency")
            .property("speed-preset", "ultrafast")
            .build()
            .map_err(|e| format!("无法创建 x264enc: {:?}", e))?
    } else if gst::ElementFactory::find("avenc_h264").is_some() {
        info!("使用 avenc_h264（软件编码）");
        gst::ElementFactory::make("avenc_h264")
            .property("preset", "ultrafast")
            .build()
            .map_err(|e| format!("无法创建 avenc_h264: {:?}", e))?
    } else {
        return Err(format!(
            "未找到可用的 H.264 编码器。请安装以下插件之一：\n\
            - gstreamer1.0-plugins-good (x264enc)\n\
            - gstreamer1.0-plugins-bad (avenc_h264)\n\
            - gstreamer1.0-plugins-bad (vaapih264enc, 需要硬件支持)\n\
            - gstreamer1.0-plugins-bad (nvh264enc, 需要 NVIDIA GPU)"
        ));
    };
    
    let rtph264pay = gst::ElementFactory::make("rtph264pay")
        .property("config-interval", 1i32)
        .property("pt", 96u32)
        .build()
        .map_err(|e| format!("无法创建 rtph264pay: {:?}", e))?;

    // 解析输出地址
    let (host, port) = output_address
        .split_once(':')
        .ok_or_else(|| "输出地址格式错误，应为 host:port".to_string())?;
    let port: u16 = port.parse()
        .map_err(|e| format!("端口必须是数字: {}", e))?;

    // 根据协议选择 sink
    let sink = match protocol.to_lowercase().as_str() {
        "udp" => {
            gst::ElementFactory::make("udpsink")
                .property("host", host)
                .property("port", port as i32)
                .build()
                .map_err(|e| format!("无法创建 udpsink: {:?}", e))?
        }
        "tcp" => {
            // tcpserversink 需要 sync=false 以避免阻塞
            // 默认配置会在连接断开后继续等待新连接
            gst::ElementFactory::make("tcpserversink")
                .property("host", host)
                .property("port", port as i32)
                .property("sync", false)
                .build()
                .map_err(|e| format!("无法创建 tcpserversink: {:?}", e))?
        }
        _ => {
            return Err(format!("不支持的协议: {}，支持 udp 或 tcp", protocol));
        }
    };

    // 添加元素到 pipeline
    pipeline
        .add_many(&[
            appsrc.upcast_ref(),
            &videoconvert,
            &encoder,
            &rtph264pay,
            &sink,
        ])
        .map_err(|e| format!("无法添加元素到 pipeline: {:?}", e))?;

    // 链接元素
    gst::Element::link_many(&[
        appsrc.upcast_ref(),
        &videoconvert,
        &encoder,
        &rtph264pay,
        &sink,
    ])
    .map_err(|e| format!("无法链接元素: {:?}", e))?;

    // 启动 pipeline
    pipeline
        .set_state(gst::State::Playing)
        .map_err(|e| format!("无法启动 pipeline: {:?}", e))?;

    info!("GStreamer pipeline 已启动");
    info!("输出流地址: {}://{}:{}", protocol, host, port);
    info!("客户端可以使用以下命令接收:");
    match protocol.to_lowercase().as_str() {
        "udp" => {
            info!(
                "  gst-launch-1.0 udpsrc port={} caps=\"application/x-rtp,media=video,encoding-name=H264,payload=96\" ! rtph264depay ! avdec_h264 ! videoconvert ! autovideosink",
                port
            );
        }
        "tcp" => {
            info!(
                "  gst-launch-1.0 tcpclientsrc host={} port={} ! application/x-rtp,encoding-name=H264,payload=96 ! rtph264depay ! h264parse ! avdec_h264 ! videoconvert ! autovideosink",
                host, port
            );
        }
        _ => {}
    }

    // 返回 appsrc 和停止信号发送端（暂时不使用，因为我们在主循环中处理）
    let (frame_stop_tx, _frame_stop_rx) = mpsc::channel();
    Ok((appsrc, frame_stop_tx))
}

/// 使用 VNC 服务器方式暴露输出流
/// 返回 VNC 服务器和停止信号的发送端
/// 注意：frame() 必须在创建 WaylandDisplay 的线程中调用
fn start_vnc_output(
    video_info: VideoInfo,
    vnc_port: u16,
    vnc_password: Option<String>,
) -> Result<(Arc<Mutex<rustvncserver::VncServer>>, mpsc::Sender<()>), String> {
    use rustvncserver::VncServer;

    let width = video_info.width() as u16;
    let height = video_info.height() as u16;
    let name = "weadless".to_string();
    let password = vnc_password.clone();

    // 创建 VNC 服务器（异步 API，需要在 tokio runtime 中运行）
    let (vnc_server, mut event_rx) = VncServer::new(width, height, name, password);

    // 在单独的线程中运行 VNC 服务器
    let server_clone = Arc::new(Mutex::new(vnc_server));
    let server_for_listen = server_clone.clone();
    let port = vnc_port;

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("无法创建 tokio runtime: {:?}", e))
            .unwrap();

        rt.block_on(async {
            // 处理服务器事件
            let event_handle = tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    match event {
                        rustvncserver::ServerEvent::ClientConnected { id, address } => {
                            info!("VNC 客户端 {} ({}) 已连接", id, address);
                        }
                        rustvncserver::ServerEvent::ClientDisconnected { id } => {
                            info!("VNC 客户端 {} 已断开", id);
                        }
                        _ => {}
                    }
                }
            });

            // 启动服务器
            let server = server_for_listen.lock().unwrap();
            if let Err(e) = server.listen(port).await {
                error!("VNC 服务器监听失败: {:?}", e);
            }

            event_handle.await.ok();
        });
    });

    info!("VNC 服务器已启动");
    info!("VNC 服务器地址: 0.0.0.0:{}", vnc_port);
    info!("使用 VNC 客户端连接:");
    info!("  vncviewer localhost:{}", vnc_port);
    info!("  或者: vncviewer localhost::{}", vnc_port);
    if vnc_password.is_some() {
        info!("  需要密码认证");
    }

    // 返回 VNC 服务器和停止信号发送端
    let (frame_stop_tx, _frame_stop_rx) = mpsc::channel();
    Ok((server_clone, frame_stop_tx))
}

/// 将 GStreamer buffer 发送到 VNC 服务器
fn send_frame_to_vnc(
    vnc_server: &Arc<Mutex<rustvncserver::VncServer>>,
    buffer: &gst::Buffer,
    video_info: &VideoInfo,
) -> Result<(), String> {
    // 获取 buffer 的数据
    let map = buffer
        .map_readable()
        .map_err(|e| format!("无法映射 buffer: {:?}", e))?;

    let data = map.as_slice();
    let width = video_info.width() as usize;
    let height = video_info.height() as usize;

    // 根据视频格式处理数据
    // VNC 需要 RGB888 格式（每个像素 3 字节）
    let rgb_data = match video_info.format() {
        gst_video::VideoFormat::Rgbx => {
            // RGBx -> RGB (去掉 alpha 通道)
            let mut rgb = Vec::with_capacity(width * height * 3);
            for chunk in data.chunks_exact(4) {
                rgb.push(chunk[0]); // R
                rgb.push(chunk[1]); // G
                rgb.push(chunk[2]); // B
            }
            rgb
        }
        gst_video::VideoFormat::Rgba => {
            // RGBA -> RGB (去掉 alpha 通道)
            let mut rgb = Vec::with_capacity(width * height * 3);
            for chunk in data.chunks_exact(4) {
                rgb.push(chunk[0]); // R
                rgb.push(chunk[1]); // G
                rgb.push(chunk[2]); // B
            }
            rgb
        }
        gst_video::VideoFormat::Bgrx => {
            // BGRx -> RGB (交换 R 和 B，去掉 alpha)
            let mut rgb = Vec::with_capacity(width * height * 3);
            for chunk in data.chunks_exact(4) {
                rgb.push(chunk[2]); // R
                rgb.push(chunk[1]); // G
                rgb.push(chunk[0]); // B
            }
            rgb
        }
        gst_video::VideoFormat::Bgra => {
            // BGRA -> RGB (交换 R 和 B，去掉 alpha)
            let mut rgb = Vec::with_capacity(width * height * 3);
            for chunk in data.chunks_exact(4) {
                rgb.push(chunk[2]); // R
                rgb.push(chunk[1]); // G
                rgb.push(chunk[0]); // B
            }
            rgb
        }
        _ => {
            return Err(format!(
                "不支持的视频格式: {:?}，VNC 需要 RGB 格式",
                video_info.format()
            ));
        }
    };

    // 发送帧到 VNC 服务器
    let mut server = vnc_server
        .lock()
        .map_err(|e| format!("无法锁定 VNC 服务器: {:?}", e))?;

    server
        .update_framebuffer(0, 0, width as u16, height as u16, &rgb_data)
        .map_err(|e| format!("无法更新 VNC 帧缓冲区: {:?}", e))?;

    Ok(())
}
