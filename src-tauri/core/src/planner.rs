//! 编码执行计划：路径安全检查与 FFmpeg 参数数组生成。
//!
//! 所有路径以 `OsString` 逐项传入参数数组，绝不拼接 Shell 命令
//! （PLAN §4 安全执行原则）。

use crate::encode::{self, EncodeConfig, EncoderFamily, QualityMode};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// 输出容器扩展名（M2：MKV 为主，支持 MP4）。
pub const SUPPORTED_OUTPUT_EXTENSIONS: [&str; 2] = ["mkv", "mp4"];

/// 根据结构化配置生成 FFmpeg 参数数组（不含程序名）。
///
/// 输出路径应为任务专属的临时文件路径（见 [`temp_output_path`]），
/// 成功后由运行器重命名为最终输出。
pub fn build_args(
    config: &EncodeConfig,
    input: &Path,
    output: &Path,
) -> Result<Vec<OsString>, String> {
    encode::validate_config(config)?;
    let spec = encode::encoder_spec(&config.video.encoder_id)
        .ok_or_else(|| format!("未知编码器：{}", config.video.encoder_id))?;

    let mut args: Vec<OsString> = Vec::new();
    push(&mut args, "-hide_banner");
    // 禁止读取 stdin，避免编码过程中意外交互或阻塞。
    push(&mut args, "-nostdin");
    // -y 只作用于任务专属的临时文件：临时名含任务 ID，不存在覆盖用户数据
    // 的风险；成功后临时文件会重命名为最终输出。
    push(&mut args, "-y");
    push(&mut args, "-i");
    args.push(input.as_os_str().to_os_string());

    // 视频：仅编码第一个视频流（M2 单视频流策略）。
    push(&mut args, "-map");
    push(&mut args, "0:v:0");
    push(&mut args, "-c:v");
    push(&mut args, &config.video.encoder_id);
    push(&mut args, "-preset");
    push(&mut args, &config.video.preset);
    match (spec.family, config.video.quality_mode) {
        (EncoderFamily::Software, QualityMode::Crf) => {
            push(&mut args, "-crf");
            push(&mut args, config.video.quality_value.to_string());
        }
        (EncoderFamily::Nvenc, QualityMode::Cq) => {
            push(&mut args, "-cq");
            push(&mut args, config.video.quality_value.to_string());
        }
        (family, mode) => {
            return Err(format!(
                "编码器 {}（{:?}）与质量模式 {:?} 不匹配",
                config.video.encoder_id, family, mode
            ));
        }
    }
    if let Some(pf) = &config.video.pixel_format {
        push(&mut args, "-pix_fmt");
        push(&mut args, pf);
    }

    // 音频：复制全部音轨（M2 策略；映射细节属 M4）。
    push(&mut args, "-map");
    push(&mut args, "0:a?");
    push(&mut args, "-c:a");
    push(&mut args, "copy");

    // 字幕与附件：M2 不处理，显式丢弃（M4 实现映射与保留策略）。
    push(&mut args, "-sn");

    // 机器可读进度 + 抑制控制台统计（进度走 stdout，日志走 stderr）。
    push(&mut args, "-progress");
    push(&mut args, "pipe:1");
    push(&mut args, "-nostats");

    args.push(output.as_os_str().to_os_string());
    Ok(args)
}

fn push(args: &mut Vec<OsString>, value: impl Into<OsString>) {
    args.push(value.into());
}

// ---------- 路径安全 ----------

/// 检查输入/输出路径：输入存在性、输出扩展名、父目录、输入输出冲突与覆盖行为。
pub fn check_paths(input: &Path, output: &Path, overwrite: bool) -> Result<(), String> {
    if !input.is_file() {
        return Err(format!("输入文件不存在或不是文件：{}", input.display()));
    }

    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    match ext {
        Some(e) if SUPPORTED_OUTPUT_EXTENSIONS.contains(&e.as_str()) => {}
        _ => {
            return Err(format!(
                "输出文件扩展名必须是 .mkv 或 .mp4：{}",
                output.display()
            ));
        }
    }

    if output.is_dir() {
        return Err(format!("输出路径是文件夹：{}", output.display()));
    }

    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| format!("输出路径缺少父目录：{}", output.display()))?;
    if !parent.is_dir() {
        return Err(format!("输出目录不存在：{}", parent.display()));
    }

    if same_file(input, output) {
        return Err(format!(
            "输出路径与输入文件相同，禁止覆盖输入：{}",
            output.display()
        ));
    }

    if output.exists() && !overwrite {
        return Err(format!(
            "输出文件已存在（未启用覆盖）：{}",
            output.display()
        ));
    }
    Ok(())
}

/// 判断输入与输出是否指向同一文件（规范化绝对路径比较；输出可能尚不存在）。
pub fn same_file(input: &Path, output: &Path) -> bool {
    match (canonicalish(input), canonicalish(output)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// 规范化为绝对路径：文件不存在时规范化其父目录后拼接文件名。
fn canonicalish(path: &Path) -> Option<PathBuf> {
    if let Ok(canon) = std::fs::canonicalize(path) {
        return Some(canon);
    }
    let name = path.file_name()?;
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty())?;
    let parent_canon = std::fs::canonicalize(parent).ok()?;
    Some(parent_canon.join(name))
}

/// 任务专属临时输出路径：与最终输出同目录、同名干 + 任务 ID 标记，
/// 扩展名保持不变（FFmpeg 依扩展名选择封装器）。
pub fn temp_output_path(final_output: &Path, job_id: &str) -> PathBuf {
    let parent = final_output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = final_output
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    let ext = final_output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mkv");
    parent.join(format!("{stem}.aozora-part-{job_id}.{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{AudioConfig, AudioMode, OutputConfig, QualityMode, VideoConfig};
    use std::fs;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-tmp")
            .join(format!("aozora-planner-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn x264_config(quality: u32, preset: &str) -> EncodeConfig {
        EncodeConfig {
            schema_version: encode::ENCODE_CONFIG_SCHEMA_VERSION,
            video: VideoConfig {
                encoder_id: "libx264".into(),
                quality_mode: QualityMode::Crf,
                quality_value: quality,
                preset: preset.into(),
                pixel_format: None,
            },
            audio: AudioConfig {
                mode: AudioMode::Copy,
            },
            output: OutputConfig {
                path: PathBuf::from("out.mkv"),
                overwrite: false,
            },
        }
    }

    #[test]
    fn x264_args_contain_expected_sequence() {
        let config = x264_config(18, "slow");
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap();
        let text: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = text.join(" ");
        assert!(
            joined.contains("-hide_banner -nostdin -y -i in.mkv"),
            "args: {joined}"
        );
        assert!(
            joined.contains("-map 0:v:0 -c:v libx264 -preset slow -crf 18"),
            "args: {joined}"
        );
        assert!(joined.contains("-map 0:a? -c:a copy"), "args: {joined}");
        assert!(joined.contains("-sn"), "args: {joined}");
        assert!(
            joined.contains("-progress pipe:1 -nostats"),
            "args: {joined}"
        );
        assert!(!joined.contains("-pix_fmt"), "args: {joined}");
        assert_eq!(args.last().unwrap(), &OsString::from("out.mkv"));
    }

    #[test]
    fn x265_args_include_pixel_format_and_crf() {
        let mut config = x264_config(16, "slow");
        config.video.encoder_id = "libx265".into();
        config.video.pixel_format = Some("yuv420p10le".into());
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap();
        let text: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = text.join(" ");
        assert!(
            joined.contains("-c:v libx265 -preset slow -crf 16 -pix_fmt yuv420p10le"),
            "args: {joined}"
        );
    }

    #[test]
    fn nvenc_args_use_cq() {
        let mut config = x264_config(21, "p7");
        config.video.encoder_id = "hevc_nvenc".into();
        config.video.quality_mode = QualityMode::Cq;
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap();
        let text: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = text.join(" ");
        assert!(
            joined.contains("-c:v hevc_nvenc -preset p7 -cq 21"),
            "args: {joined}"
        );
    }

    #[test]
    fn invalid_config_is_rejected() {
        let mut config = x264_config(18, "slow");
        config.video.quality_value = 99;
        assert!(build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).is_err());
    }

    #[test]
    fn paths_with_spaces_and_unicode_stay_intact() {
        let config = x264_config(20, "medium");
        let input = Path::new(r"C:\媒体 目录 (1)\样片 [demo].mkv");
        let output = Path::new(r"C:\媒体 目录 (1)\样片 [demo].encoded.mkv");
        let args = build_args(&config, input, output).unwrap();
        let text: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(
            text.contains(&input.to_string_lossy().into_owned()),
            "args: {text:?}"
        );
        assert!(
            text.contains(&output.to_string_lossy().into_owned()),
            "args: {text:?}"
        );
    }

    #[test]
    fn check_paths_accepts_valid_target() {
        let dir = temp_dir("valid");
        let input = dir.join("input.mkv");
        fs::write(&input, b"placeholder").unwrap();
        let output = dir.join("output.mkv");
        assert!(check_paths(&input, &output, false).is_ok());
        assert!(check_paths(&input, &output, true).is_ok());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_missing_input() {
        let dir = temp_dir("missing-input");
        let err = check_paths(&dir.join("nope.mkv"), &dir.join("out.mkv"), false).unwrap_err();
        assert!(err.contains("输入文件不存在"), "error: {err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_unsupported_extension() {
        let dir = temp_dir("bad-ext");
        let input = dir.join("input.mkv");
        fs::write(&input, b"x").unwrap();
        let err = check_paths(&input, &dir.join("out.avi"), false).unwrap_err();
        assert!(err.contains(".mkv 或 .mp4"), "error: {err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_missing_parent_dir() {
        let dir = temp_dir("missing-parent");
        let input = dir.join("input.mkv");
        fs::write(&input, b"x").unwrap();
        let err = check_paths(&input, &dir.join("no-such-dir").join("out.mkv"), false).unwrap_err();
        assert!(err.contains("输出目录不存在"), "error: {err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_existing_output_without_overwrite() {
        let dir = temp_dir("existing");
        let input = dir.join("input.mkv");
        let output = dir.join("output.mkv");
        fs::write(&input, b"x").unwrap();
        fs::write(&output, b"y").unwrap();
        let err = check_paths(&input, &output, false).unwrap_err();
        assert!(err.contains("已存在"), "error: {err}");
        assert!(check_paths(&input, &output, true).is_ok());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_input_output_conflict() {
        let dir = temp_dir("conflict");
        let input = dir.join("same.mkv");
        fs::write(&input, b"x").unwrap();
        let err = check_paths(&input, &input, true).unwrap_err();
        assert!(err.contains("禁止覆盖输入"), "error: {err}");

        // 大小写不同、路径写法不同的等价路径也应识别为同一文件
        let mut output = input.clone();
        output.pop();
        output.push("SAME.mkv");
        if cfg!(windows) {
            let err = check_paths(&input, &output, true).unwrap_err();
            assert!(err.contains("禁止覆盖输入"), "error: {err}");
        }
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn same_file_detects_case_insensitive_equality() {
        let dir = temp_dir("same-file");
        let a = dir.join("video.mkv");
        fs::write(&a, b"x").unwrap();
        let b = dir.join("VIDEO.mkv");
        let equal = same_file(&a, &b);
        if cfg!(windows) {
            assert!(equal);
        } else {
            // Unix 大小写敏感；规范化后路径不同
            assert!(!equal);
        }
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn temp_output_path_keeps_dir_and_extension() {
        let final_output = Path::new(r"D:\out\影片.mkv");
        let temp = temp_output_path(final_output, "e-123");
        assert_eq!(temp.parent(), Some(Path::new(r"D:\out")));
        let name = temp.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with("影片.aozora-part-e-123"), "name: {name}");
        assert!(name.ends_with(".mkv"), "name: {name}");
        assert_ne!(name, "影片.mkv");
    }
}
