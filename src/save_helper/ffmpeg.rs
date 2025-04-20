use ffmpeg::{Packet, Rational, codec, format, frame, software, util::format::Pixel};

use crate::frame_queue::FrameQueue;

pub fn save_frames_to_video(
    frame_queue: &FrameQueue,
    output_path: &str,
    width: u32,
    height: u32,
    fps: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化 FFmpeg
    ffmpeg::init()?;

    // 创建输出格式上下文
    let mut output_context = format::output(&output_path)?;

    // 添加视频流
    let mut stream = output_context.add_stream(ffmpeg::codec::Id::H264)?;
    let time_base = Rational::new(1, fps as i32);

    // 配置编码器
    {
        let mut encoder = stream.codec_mut().encoder().video()?;
        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_format(Pixel::YUV420P); // 视频编码一般使用 YUV420P
        encoder.set_time_base(time_base);
        encoder.set_frame_rate(Rational::new(fps as i32, 1));
        encoder.set_bit_rate(400_000); // 设置比特率
        encoder.open_as(ffmpeg::codec::Id::H264)?;
    }

    output_context.write_header()?;

    // 用于像素格式转换的缩放器
    let mut scaler = software::scaling::Context::get(
        Pixel::RGB32, // 假设你的帧数据是 RGB32 格式
        width,
        height,
        Pixel::YUV420P,
        width,
        height,
        software::scaling::Flags::BILINEAR,
    )?;

    let mut pts = 0;

    // 遍历帧队列
    for frame_data in &frame_queue.f {
        if let Some(canvas) = &frame_data.canvas {
            let mut input_frame = frame::Video::new(Pixel::RGB32, width, height);
            input_frame.data_mut(0).copy_from_slice(canvas);

            let mut scaled_frame = frame::Video::new(Pixel::YUV420P, width, height);
            scaler.run(&input_frame, &mut scaled_frame)?;

            // 设置时间戳
            scaled_frame.set_pts(Some(pts));
            pts += 1;

            // 编码帧
            let mut packet = Packet::empty();
            unsafe {
                stream
                    .codec()
                    .encoder()
                    .video()?
                    .encode(&scaled_frame, &mut packet)?;
                if !packet.is_empty() {
                    output_context.write(&packet)?;
                }
            }
        }
    }

    // 刷新编码器
    let mut packet = Packet::empty();
    unsafe {
        while stream.codec().encoder().video()?.flush(&mut packet)? {
            if !packet.is_empty() {
                output_context.write(&packet)?;
            }
        }
    }

    // 写入尾部
    output_context.write_trailer()?;
    Ok(())
}
