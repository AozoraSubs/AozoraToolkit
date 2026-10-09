//! Aozora Encode 核心逻辑：设置模型、FFmpeg/FFprobe 检测、能力检测、
//! 媒体探测、编码配置与参数生成、进度解析与 FFmpeg 运行器。
//! 本 crate 不依赖 Tauri，保证其测试可在任何环境（包括 Windows-GNU）
//! 独立运行，不会因链接 WebView2 原生库而失败。

pub mod capabilities;
pub mod encode;
pub mod job;
pub mod media_probe;
pub mod planner;
pub mod probe;
pub mod progress;
pub mod settings;
