//! Aozora Encode 核心逻辑：设置模型、FFmpeg/FFprobe 检测、能力检测、
//! 媒体探测、编码配置与参数生成、进度解析、FFmpeg 运行器、预设存储、
//! 任务队列与任务历史。
//! 本 crate 不依赖 Tauri，保证其测试可在任何环境（包括 Windows-GNU）
//! 独立运行，不会因链接 WebView2 原生库而失败。

pub mod capabilities;
pub mod diagnostics;
pub mod encode;
pub mod history;
pub mod job;
pub mod media_probe;
pub mod planner;
pub mod preset;
pub mod probe;
pub mod process_flags;
pub mod progress;
pub mod queue;
pub mod settings;

/// 当前 UNIX 时间戳（毫秒）；系统时钟早于 epoch 时返回 0。
pub fn unix_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
