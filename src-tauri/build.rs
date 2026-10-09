fn main() {
    // 周快照构建时注入快照版本号（如 26w41a），供 app_info 在关于页展示。
    // 不修改 Cargo.toml / 不产生提交，仅影响本次编译产物。
    if let Ok(snapshot) = std::env::var("AOZORA_SNAPSHOT_VERSION") {
        println!("cargo:rustc-env=AOZORA_SNAPSHOT_VERSION={snapshot}");
    }
    tauri_build::build()
}
