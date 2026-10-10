use crate::settings::Settings;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

const FFMPEG: &str = "ffmpeg";
const FFPROBE: &str = "ffprobe";

/// 检测到的单个工具实例。
#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub path: String,
    /// `tool -version` 输出的首行（含版本号的完整横幅）。
    pub version: Option<String>,
    /// 来源："path"（运行环境 PATH）| "app_bin"（安装目录 bin）| "custom"（自定义路径）。
    pub source: String,
}

/// FFmpeg / FFprobe 安装检测结果。
#[derive(Debug, Default, Serialize)]
pub struct ToolDetectionResult {
    pub ffmpeg: Option<ToolInfo>,
    pub ffprobe: Option<ToolInfo>,
    #[serde(rename = "customToolDir")]
    pub custom_tool_dir: Option<String>,
    /// 面向用户的可读问题说明（未找到、二进制无法执行等）。
    pub errors: Vec<String>,
}

/// 当前平台上某工具的可执行文件名候选。
fn candidate_names(tool: &str) -> Vec<String> {
    if cfg!(windows) {
        vec![format!("{tool}.exe"), tool.to_string()]
    } else {
        vec![tool.to_string()]
    }
}

/// 在给定 PATH 条目中按顺序查找工具（只做文件存在性检查，不验证可执行）。
pub fn find_in_path(tool: &str, path_entries: &[PathBuf]) -> Option<PathBuf> {
    for dir in path_entries {
        for name in candidate_names(tool) {
            let candidate = dir.join(&name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// 工具箱安装目录下的 `bin` 文件夹（当前可执行文件所在目录旁的 bin），不存在时返回 None。
pub fn app_bin_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let bin = exe_dir.join("bin");
    bin.is_dir().then_some(bin)
}

/// 运行 `tool -version` 验证工具可用并返回首行输出（版本横幅）。
pub fn validate_tool(path: &Path) -> Result<String, String> {
    let mut command = Command::new(path);
    crate::process_flags::hide_console_std(&mut command);
    let output = command
        .arg("-version")
        .output()
        .map_err(|e| format!("无法启动 {}：{e}", path.display()))?;
    if !output.status.success() {
        return Err(format!(
            "{} 执行失败（退出码：{}）",
            path.display(),
            output.status
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .map(str::to_string)
        .ok_or_else(|| format!("{} 没有输出版本信息", path.display()))
}

/// 按 PLAN M1 的检测顺序解析每个工具：
/// 1. 运行环境 PATH；
/// 2. 安装目录 `bin`（仅 Windows）；
/// 3. 自定义路径（仅 Windows）。
///
/// 每个工具取第一个验证通过的实例；失败的候选和未找到的情况记录为可读错误。
pub fn detect_tools(settings: &Settings) -> ToolDetectionResult {
    let path_entries: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    detect_tools_with(settings, &path_entries, app_bin_dir().as_deref())
}

/// 与 [`detect_tools`] 相同，但 PATH 条目与 bin 目录可注入，便于测试各来源分支。
pub fn detect_tools_with(
    settings: &Settings,
    path_entries: &[PathBuf],
    bin_dir: Option<&Path>,
) -> ToolDetectionResult {
    let mut result = ToolDetectionResult {
        custom_tool_dir: settings
            .custom_tool_dir
            .as_ref()
            .map(|p| p.display().to_string()),
        ..Default::default()
    };
    let mut errors: Vec<String> = Vec::new();

    let custom_dir: Option<PathBuf> = settings.custom_tool_dir.as_ref().map(PathBuf::from);
    if let Some(dir) = &custom_dir {
        if !dir.is_dir() {
            errors.push(format!("自定义路径不存在或不是文件夹：{}", dir.display()));
        }
    }

    for (tool, slot) in [(FFMPEG, &mut result.ffmpeg), (FFPROBE, &mut result.ffprobe)] {
        let mut found = false;

        // 1. 运行环境 PATH（所有平台）
        if let Some(path) = find_in_path(tool, path_entries) {
            match validate_tool(&path) {
                Ok(version) => {
                    *slot = Some(ToolInfo {
                        path: path.display().to_string(),
                        version: Some(version),
                        source: "path".to_string(),
                    });
                    found = true;
                }
                Err(e) => errors.push(format!("PATH 中的 {tool} 无法使用：{e}")),
            }
        }

        // 2. 安装目录 bin（仅 Windows）
        if !found && cfg!(windows) {
            if let Some(bin) = bin_dir {
                for name in candidate_names(tool) {
                    let candidate = bin.join(&name);
                    if candidate.is_file() {
                        match validate_tool(&candidate) {
                            Ok(version) => {
                                *slot = Some(ToolInfo {
                                    path: candidate.display().to_string(),
                                    version: Some(version),
                                    source: "app_bin".to_string(),
                                });
                                found = true;
                            }
                            Err(e) => {
                                errors.push(format!("安装目录 bin 中的 {tool} 无法使用：{e}"));
                            }
                        }
                        break;
                    }
                }
            }
        }

        // 3. 自定义路径（仅 Windows）
        if !found && cfg!(windows) {
            if let Some(dir) = custom_dir.as_ref().filter(|d| d.is_dir()) {
                for name in candidate_names(tool) {
                    let candidate = dir.join(&name);
                    if candidate.is_file() {
                        match validate_tool(&candidate) {
                            Ok(version) => {
                                *slot = Some(ToolInfo {
                                    path: candidate.display().to_string(),
                                    version: Some(version),
                                    source: "custom".to_string(),
                                });
                                found = true;
                            }
                            Err(e) => {
                                errors.push(format!("自定义路径中的 {tool} 无法使用：{e}"));
                            }
                        }
                        break;
                    }
                }
            }
        }

        if !found {
            let hint = if cfg!(windows) {
                "请安装 FFmpeg，或在设置中指定包含 ffmpeg / ffprobe 的文件夹"
            } else {
                "请安装 FFmpeg 并确保其在 PATH 中"
            };
            errors.push(format!("未检测到可用的 {tool}。{hint}。"));
        }
    }

    errors.sort();
    errors.dedup();
    result.errors = errors;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        // 放在 crate 目录内的 test-tmp（位于 workspace 内），
        // 避免依赖可能受限的系统 %TEMP%。
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-tmp")
            .join(format!("aozora-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn candidate_names_include_exe_on_windows() {
        let names = candidate_names("ffmpeg");
        if cfg!(windows) {
            assert!(names.contains(&"ffmpeg.exe".to_string()));
        } else {
            assert_eq!(names, vec!["ffmpeg".to_string()]);
        }
    }

    #[test]
    fn find_in_path_locates_candidate_file() {
        let dir = temp_dir("find-test");
        let name = if cfg!(windows) {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        };
        let fake = dir.join(name);
        std::fs::write(&fake, b"placeholder").unwrap();
        assert_eq!(
            find_in_path("ffmpeg", std::slice::from_ref(&dir)),
            Some(fake)
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_in_path_returns_none_when_missing() {
        let dir = temp_dir("empty-test");
        assert_eq!(find_in_path("ffmpeg", std::slice::from_ref(&dir)), None);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn validate_tool_reports_missing_binary() {
        let err = validate_tool(Path::new("__definitely_missing_ffmpeg__")).unwrap_err();
        assert!(err.contains("无法启动"), "unexpected error: {err}");
    }

    #[test]
    fn detect_tools_reports_invalid_custom_dir() {
        let settings = Settings {
            custom_tool_dir: Some(PathBuf::from("__no_such_dir__")),
            ..Default::default()
        };
        let result = detect_tools_with(&settings, &[], None);
        assert!(
            result.errors.iter().any(|e| e.contains("自定义路径不存在")),
            "errors: {:?}",
            result.errors
        );
    }

    /// 实机验证：仅在显式 `--ignored` 时运行，要求本机 PATH 中已安装 ffmpeg。
    #[test]
    #[ignore = "requires ffmpeg installed on PATH"]
    fn detect_tools_finds_path_ffmpeg_and_version() {
        let result = detect_tools(&Settings::default());
        let ffmpeg = result.ffmpeg.expect("ffmpeg 未在 PATH 中检测到");
        assert_eq!(ffmpeg.source, "path");
        assert!(ffmpeg.version.as_deref().unwrap_or("").contains("ffmpeg"));
        let ffprobe = result.ffprobe.expect("ffprobe 未在 PATH 中检测到");
        assert_eq!(ffprobe.source, "path");
        assert!(ffprobe.version.as_deref().unwrap_or("").contains("ffprobe"));
    }

    /// 实机验证：仅在显式 `--ignored` 时运行。以真实 ffmpeg 目录注入
    /// bin 分支与自定义路径分支，确认两者都能解析并标记正确来源。
    #[test]
    #[ignore = "requires ffmpeg installed on PATH"]
    fn detect_tools_resolves_app_bin_and_custom_branches() {
        let path_entries: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        let ffmpeg_dir = find_in_path("ffmpeg", &path_entries)
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .expect("ffmpeg 未在 PATH 中检测到");
        let empty: &[PathBuf] = &[];

        let bin_result = detect_tools_with(&Settings::default(), empty, Some(&ffmpeg_dir));
        if cfg!(windows) {
            let bin_ffmpeg = bin_result.ffmpeg.as_ref().expect("bin 分支未解析到 ffmpeg");
            assert_eq!(bin_ffmpeg.source, "app_bin");
        } else {
            // 非 Windows 不检查 bin 目录，未找到属于预期行为
            assert!(bin_result.ffmpeg.is_none());
        }

        let custom_settings = Settings {
            custom_tool_dir: Some(ffmpeg_dir),
            ..Default::default()
        };
        let custom_result = detect_tools_with(&custom_settings, empty, None);
        if cfg!(windows) {
            let custom_ffmpeg = custom_result
                .ffmpeg
                .as_ref()
                .expect("自定义路径分支未解析到 ffmpeg");
            assert_eq!(custom_ffmpeg.source, "custom");
        } else {
            // 非 Windows 不检查自定义路径，未找到属于预期行为
            assert!(custom_result.ffmpeg.is_none());
        }
    }
}
