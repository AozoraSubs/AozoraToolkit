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

/// 能力检测结果的序列化结构必须与前端 `CapabilityReport` 对齐
/// （camelCase 字段名 + 编码器逐项可用性 + 显卡列表）。
///
/// 只构造数据、不启动 FFmpeg，因此在任何平台都可运行。
#[test]
fn capability_report_serializes_for_the_frontend() {
    use aozora_encode_lib::capability_dto_for_test;

    let json = capability_dto_for_test();
    for key in [
        "\"encoders\"",
        "\"encoderCount\"",
        "\"filterCount\"",
        "\"hasSubtitlesFilter\"",
        "\"hwAccels\"",
        "\"gpus\"",
        "\"warnings\"",
    ] {
        assert!(json.contains(key), "能力报告缺少字段 {key}：{json}");
    }
    for key in [
        "\"id\"",
        "\"label\"",
        "\"vendor\"",
        "\"codec\"",
        "\"hardware\"",
        "\"listed\"",
        "\"usable\"",
        "\"reason\"",
    ] {
        assert!(json.contains(key), "编码器条目缺少字段 {key}：{json}");
    }
}
