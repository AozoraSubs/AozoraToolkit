use std::path::{Path, PathBuf};

/// 应用清单：声明 Common-Controls v6 依赖。
///
/// 应用代码经 tauri 默认特性（`common-controls-v6`）间接引用
/// `TaskDialogIndirect`，该函数只有 comctl32 v6（WinSxS）导出。
/// 没有这份清单时，Windows 加载器会绑定 System32 的 v5 版本，
/// 进程启动即报 STATUS_ENTRYPOINT_NOT_FOUND。
const APP_MANIFEST: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>"#;

/// 让所有可执行文件（二进制与测试）携带 Common-Controls v6 应用清单。
///
/// tauri-build 的默认资源通过 `cargo:rustc-link-arg-bins` 只链接到
/// 二进制；测试可执行文件（`cargo test`）拿不到清单。这里改为让
/// tauri-build 不内嵌清单（`new_without_app_manifest`），由本函数按
/// 工具链把清单统一提供给所有目标：
/// - GNU（mingw）：gcc specs 的 `*endfile` 含
///   `%{!shared:%:if-exists(default-manifest.o%s)}`——链接器驱动会把搜索
///   路径中名为 `default-manifest.o` 的对象（发行版自带 asInvoker /
///   longPathAware 版本）注入每个 exe。我们生成同名的对象（v6 清单）并
///   通过 `rustc-link-search=native` 让它先被找到，每个 exe 恰好一份，
///   无合并冲突；
/// - MSVC：通过资源清单（`compile_for_everything`）实现同样效果。
fn embed_app_manifest() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR 环境变量缺失");
    let manifest_path = Path::new(&out_dir).join("app-manifest.xml");
    std::fs::write(&manifest_path, APP_MANIFEST).expect("写入应用清单失败");

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu" {
        // 用 windres 编译成 default-manifest.o，并用 -B 让 gcc specs 的
        // `if-exists(default-manifest.o)` 先找到我们的版本（-L 的优先级
        // 低于内建目录，实测无法覆盖发行版自带对象）。
        let manifest_for_rc = manifest_path.to_string_lossy().replace('\\', "/");
        let rc_path = Path::new(&out_dir).join("default-manifest.rc");
        std::fs::write(&rc_path, format!("1 24 \"{manifest_for_rc}\"\n"))
            .expect("写入清单资源文件失败");
        let output = std::process::Command::new("windres")
            .arg("-i")
            .arg(&rc_path)
            .arg("-o")
            .arg(Path::new(&out_dir).join("default-manifest.o"))
            .output();
        match output {
            Ok(output) if output.status.success() => {
                println!("cargo:rustc-link-arg=-B{out_dir}");
            }
            Ok(output) => {
                eprintln!(
                    "警告：windres 编译应用清单失败（{}），测试可执行文件可能缺少清单",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => {
                eprintln!("警告：无法启动 windres（{e}），测试可执行文件可能缺少清单");
            }
        }
    } else {
        let mut rc = String::from("1 24 {\n");
        for line in APP_MANIFEST.lines() {
            let escaped = line.replace('\\', "\\\\").replace('"', "\"\"");
            rc.push_str(&format!("\"{escaped}\"\n"));
        }
        rc.push_str("}\n");
        let rc_path = Path::new(&out_dir).join("manifest-resource.rc");
        std::fs::write(&rc_path, rc).expect("写入清单资源文件失败");
        embed_resource::compile_for_everything(&rc_path, embed_resource::NONE)
            .manifest_required()
            .expect("编译应用清单资源失败");
    }
}

/// 把 `WebView2Loader.dll` 复制到目标 profile 的 `deps/` 目录。
///
/// tauri-build 会把该 DLL 放到 `target/<profile>/WebView2Loader.dll`
/// （真实可执行文件旁）。应用一旦使用事件（`app.emit`），测试可执行文件
/// （位于 `target/<profile>/deps/`）也会导入该 DLL，而 Windows 加载器只
/// 搜索可执行文件所在目录等位置，找不到 `deps/` 旁没有的 DLL 会导致
/// `cargo test` 以 STATUS_DLL_NOT_FOUND 失败。这里在每次构建时补一份到
/// `deps/`，保证 `cargo test --workspace` 在 Windows 上可运行。
fn copy_webview2_loader_for_tests() {
    let Ok(out_dir) = std::env::var("OUT_DIR") else {
        return;
    };
    // 向上找到 target/<profile>（带 deps 子目录的那一层）。
    // OUT_DIR 布局：stable 为 target/<profile>/build/<crate>-<hash>/out，
    // 部分 nightly 为 target/<profile>/build/<crate>/<hash>/out。
    let mut profile_dir = Path::new(&out_dir).parent().map(Path::to_path_buf);
    let mut found = None;
    for _ in 0..4 {
        let Some(current) = profile_dir.as_deref() else {
            break;
        };
        if current.join("deps").is_dir() {
            found = Some(current.to_path_buf());
            break;
        }
        profile_dir = current.parent().map(Path::to_path_buf);
    }
    let Some(profile_dir) = found else {
        return;
    };
    let source: PathBuf = profile_dir.join("WebView2Loader.dll");
    if !source.is_file() {
        return;
    }
    let deps_dir = profile_dir.join("deps");
    let _ = std::fs::create_dir_all(&deps_dir);
    let _ = std::fs::copy(&source, deps_dir.join("WebView2Loader.dll"));
}

fn main() {
    // 周快照构建时注入快照版本号（如 26w41a），供 app_info 在关于页展示。
    // 不修改 Cargo.toml / 不产生提交，仅影响本次编译产物。
    if let Ok(snapshot) = std::env::var("AOZORA_SNAPSHOT_VERSION") {
        println!("cargo:rustc-env=AOZORA_SNAPSHOT_VERSION={snapshot}");
    }

    // 清单由 embed_app_manifest 统一提供（见其注释）。
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(attributes).expect("tauri-build 失败");

    if cfg!(windows) {
        embed_app_manifest();
        copy_webview2_loader_for_tests();
    }
}
