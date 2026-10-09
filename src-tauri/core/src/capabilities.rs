use serde::Serialize;
use std::path::Path;
use std::process::Command;

/// 应用关注的编码器是否可用。
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownEncoders {
    pub libx264: bool,
    pub libx265: bool,
    pub h264_nvenc: bool,
    pub hevc_nvenc: bool,
    pub av1_nvenc: bool,
}

/// FFmpeg 能力检测结果。
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityReport {
    pub encoders: KnownEncoders,
    pub encoder_count: usize,
    pub filter_count: usize,
    pub hw_accels: Vec<String>,
    /// 非致命问题（例如某子命令失败），整体检测仍可用。
    pub warnings: Vec<String>,
}

fn run_lines(ffmpeg_path: &Path, args: &[&str]) -> Result<Vec<String>, String> {
    let output = Command::new(ffmpeg_path)
        .args(args)
        .output()
        .map_err(|e| format!("无法启动 {}：{e}", ffmpeg_path.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail: String = stderr
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" | ");
        return Err(format!(
            "{} 执行失败（退出码：{}）：{}",
            ffmpeg_path.display(),
            output.status,
            tail
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// 解析 `ffmpeg -encoders` 输出中的编码器名称集合。
/// 行格式：` V....D libx264  libx264 H.264 / ...`——第一列为标志，第二列为名称。
pub fn parse_encoder_names(lines: &[String]) -> Vec<String> {
    parse_flag_lines(lines)
}

/// 解析 `ffmpeg -filters` 输出中的滤镜名称集合。
pub fn parse_filter_names(lines: &[String]) -> Vec<String> {
    parse_flag_lines(lines)
}

fn parse_flag_lines(lines: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    for line in lines {
        let trimmed = line.trim_start();
        let mut tokens = trimmed.split_whitespace();
        let Some(flags) = tokens.next() else { continue };
        // 标志列仅由能力字母与 '.' 组成
        if flags.is_empty()
            || !flags
                .chars()
                .all(|c| c == '.' || "VASFEDXBILTSC".contains(c))
        {
            continue;
        }
        if let Some(name) = tokens.next() {
            // 跳过 " V..... = Video" 这类分组标题行
            if !name.starts_with('=') {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// 解析 `ffmpeg -hwaccels` 输出（首行 "Hardware acceleration methods:" 之后每行一个名称）。
pub fn parse_hw_accels(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .skip_while(|l| !l.contains("Hardware acceleration methods"))
        .skip(1)
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// 检测 FFmpeg 实际能力：编码器、滤镜、硬件加速方法。
pub fn detect_capabilities(ffmpeg_path: &Path) -> CapabilityReport {
    let mut report = CapabilityReport::default();

    match run_lines(ffmpeg_path, &["-hide_banner", "-encoders"]) {
        Ok(lines) => {
            let names = parse_encoder_names(&lines);
            report.encoder_count = names.len();
            for name in &names {
                match name.as_str() {
                    "libx264" => report.encoders.libx264 = true,
                    "libx265" => report.encoders.libx265 = true,
                    "h264_nvenc" => report.encoders.h264_nvenc = true,
                    "hevc_nvenc" => report.encoders.hevc_nvenc = true,
                    "av1_nvenc" => report.encoders.av1_nvenc = true,
                    _ => {}
                }
            }
        }
        Err(e) => report.warnings.push(format!("编码器检测失败：{e}")),
    }

    match run_lines(ffmpeg_path, &["-hide_banner", "-filters"]) {
        Ok(lines) => report.filter_count = parse_filter_names(&lines).len(),
        Err(e) => report.warnings.push(format!("滤镜检测失败：{e}")),
    }

    match run_lines(ffmpeg_path, &["-hide_banner", "-hwaccels"]) {
        Ok(lines) => report.hw_accels = parse_hw_accels(&lines),
        Err(e) => report.warnings.push(format!("硬件加速检测失败：{e}")),
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_encoder_names_extracts_codec_ids() {
        let lines = vec![
            "Encoders:".to_string(),
            " V..... = Video".to_string(),
            " V....D libx264              libx264 H.264 / AVC".to_string(),
            " V....D libx265              libx265 H.265 / HEVC".to_string(),
            " A..... aac                  AAC (Advanced Audio Coding)".to_string(),
        ];
        let names = parse_encoder_names(&lines);
        assert!(names.contains(&"libx264".to_string()));
        assert!(names.contains(&"libx265".to_string()));
        assert!(names.contains(&"aac".to_string()));
        assert_eq!(names.len(), 3, "names: {names:?}");
    }

    #[test]
    fn parse_filter_names_extracts_filter_ids() {
        let lines = vec![
            "Filters:".to_string(),
            " TSC scale  VV->V  Scale the input video.".to_string(),
            " ... libass   V->V   Render subtitles".to_string(),
        ];
        let names = parse_filter_names(&lines);
        assert_eq!(names, vec!["scale".to_string(), "libass".to_string()]);
    }

    #[test]
    fn parse_hw_accels_extracts_method_names() {
        let lines = vec![
            "Hardware acceleration methods:".to_string(),
            "cuda".to_string(),
            "dxva2".to_string(),
            "qsv".to_string(),
        ];
        assert_eq!(
            parse_hw_accels(&lines),
            vec!["cuda".to_string(), "dxva2".to_string(), "qsv".to_string()]
        );
    }

    /// 实机验证：仅在显式 `--ignored` 时运行，要求本机 PATH 中已安装 ffmpeg。
    #[test]
    #[ignore = "requires ffmpeg installed on PATH"]
    fn detect_capabilities_real_ffmpeg() {
        use crate::media_probe::find_in_path;
        let path_entries: Vec<std::path::PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        let ffmpeg = find_in_path("ffmpeg", &path_entries).expect("ffmpeg 未在 PATH 中检测到");
        let report = detect_capabilities(&ffmpeg);
        assert!(report.encoder_count > 0, "编码器列表为空");
        assert!(report.filter_count > 0, "滤镜列表为空");
        assert!(
            report.encoders.libx264,
            "本机 ffmpeg 应包含 libx264（gyan 完整构建）"
        );
    }
}
