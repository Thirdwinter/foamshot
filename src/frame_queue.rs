use image::codecs::gif::GifEncoder;
use image::{ImageBuffer, Rgba};
use log::debug;
use smithay_client_toolkit::shm::slot::{Buffer, SlotPool};
use wayland_client::protocol::wl_shm::Format;

#[allow(unused)]
pub struct FrameData {
    pub time: u64,
    pub canvas: Option<Vec<u8>>,
    pub is_ok: bool,
}

#[derive(Default)]
pub struct FrameQueue {
    pub f: Vec<FrameData>,
    pub pool: Option<SlotPool>,
    pub current_buffer: Option<Buffer>,
    pub is_copy: bool,
}

impl FrameQueue {
    pub fn new(pool: Option<SlotPool>) -> Self {
        Self {
            f: Vec::new(), // 初始化空队列
            pool,          // 传入的SlotPool
            current_buffer: None,
            is_copy: false,
        }
    }

    pub fn new_buffer(&mut self, w: i32, h: i32, s: i32, f: Format) {
        if let Some(pool) = self.pool.as_mut() {
            // 创建 Buffer 和 Canvas
            let (buffer, canvas) = pool.create_buffer(w, h, s, f).unwrap();

            // 填充 canvas 数据为 0
            canvas.fill(0);

            self.current_buffer = Some(buffer);
        } else {
            panic!("SlotPool is not initialized");
        }
    }
    // 从队列中取出最新的一项
    // pub fn pop_latest(&mut self) -> Option<FrameData> {
    //     // 获取队列的锁
    //     self.f.as_ref().pop() // 移除并返回队列中最后一个元素（最新的一项）
    // }

    pub fn storage_canvas(&mut self, time: u64) {
        let buffer = self.current_buffer.as_mut().unwrap();
        let pool = self.pool.as_mut().unwrap();
        let c = buffer.canvas(pool).unwrap().to_vec();

        // 创建 FrameData
        let frame_data = FrameData {
            time,
            canvas: Some(c),
            is_ok: true,
        };
        // 插入到队列中
        // let mut queue = self.f;
        self.f.push(frame_data);
        self.current_buffer = None;
        self.is_copy = true;
        // println!("len: {}", self.f.iter().len())
    }

    pub fn to_gif(&mut self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 创建输出文件
        let file = std::fs::File::create(output_path)?;
        let mut encoder = GifEncoder::new_with_speed(file, 30);
        for (i, f) in self.f.iter_mut().enumerate() {
            if i % 3 != 0 {
                continue;
            }
            let mut rgb_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(1366, 768);
            let data = f.canvas.as_mut().unwrap();

            // 复制像素数据（wl_shm_buffer使用ARGB8888格式，需要转换为RGBA）
            for y in 0..768 {
                for x in 0..1366 {
                    let offset = (y as usize * 1366 * 4) + (x as usize * 4);
                    // let a = data[offset];
                    // let r = data[offset + 1];
                    // let g = data[offset + 2];
                    // let b = data[offset + 3];
                    let r = data[offset + 1];
                    let g = data[offset + 2];
                    let b = data[offset + 3];
                    rgb_buffer.put_pixel(x as u32, y as u32, Rgba([r, g, b, 255]));
                }
            }
            // 创建帧并设置延迟
            let frame = image::Frame::new(rgb_buffer);

            // 编码当前帧
            debug!("encode_frame: {}", i);
            encoder.encode_frame(frame)?;
        }

        Ok(())
    }
}
