//! 应用级冒烟测试。
//!
//! 本文件同时作为显式测试目标存在：build.rs 借此为测试可执行文件链接
//! Windows 资源（`cargo:rustc-link-arg-tests`），其中包含
//! Common-Controls v6 应用清单——否则应用代码经 tauri 默认特性间接引用
//! `TaskDialogIndirect` 时，测试进程会绑定 comctl32 v5（System32 版本
//! 不导出该函数）并以 STATUS_ENTRYPOINT_NOT_FOUND 崩溃。

#[test]
fn windows_test_binary_has_common_controls_v6_manifest() {
    if !cfg!(windows) {
        return;
    }
    let exe = std::env::current_exe().expect("无法获取测试可执行文件路径");
    let bytes = std::fs::read(&exe).expect("无法读取测试可执行文件");
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("Microsoft.Windows.Common-Controls"),
        "测试可执行文件未内嵌 Common-Controls v6 清单（{}）",
        exe.display()
    );
}
