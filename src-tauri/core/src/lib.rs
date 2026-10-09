//! Aozora Encode 核心逻辑：设置模型、FFmpeg/FFprobe 检测、能力检测与媒体探测。
//! 本 crate 不依赖 Tauri，保证其测试可在任何环境（包括 Windows-GNU）
//! 独立运行，不会因链接 WebView2 原生库而失败。

pub mod capabilities;
pub mod media_probe;
pub mod probe;
pub mod settings;
