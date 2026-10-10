//! 子进程启动参数辅助：GUI 应用启动控制台程序（ffmpeg / ffprobe）时，
//! Windows 会为子进程弹出控制台窗口；CREATE_NO_WINDOW 让其静默运行。

/// `CreateProcess` 的 CREATE_NO_WINDOW 标志。
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// 为 `std::process::Command` 隐藏控制台窗口（仅 Windows）。
#[cfg(windows)]
pub fn hide_console_std(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NO_WINDOW);
}

/// 非 Windows 平台 no-op。
#[cfg(not(windows))]
pub fn hide_console_std(_command: &mut std::process::Command) {}

/// 为 `tokio::process::Command` 隐藏控制台窗口（仅 Windows）。
/// tokio 的 Command 自带 `creation_flags` 平台方法，无需导入 std 扩展 trait。
#[cfg(windows)]
pub fn hide_console_tokio(command: &mut tokio::process::Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

/// 非 Windows 平台 no-op。
#[cfg(not(windows))]
pub fn hide_console_tokio(_command: &mut tokio::process::Command) {}
