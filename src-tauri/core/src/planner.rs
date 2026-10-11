//! 编码执行计划：路径安全检查与 FFmpeg 参数数组生成。
//!
//! 所有路径以 `OsString` 逐项传入参数数组，绝不拼接 Shell 命令
//! （PLAN §4 安全执行原则）。

use crate::encode::{self, Container, EncodeConfig, EncoderFamily, QualityMode};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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
    push_video_quality_args(&mut args, spec, &config.video)?;
    if let Some(pf) = &config.video.pixel_format {
        push(&mut args, "-pix_fmt");
        push(&mut args, pf);
    }

    // 音频：复制全部音轨（M2 策略；映射细节属 M4）。
    push(&mut args, "-map");
    push(&mut args, "0:a?");
    push(&mut args, "-c:a");
    push(&mut args, "copy");

    // 字幕：按配置丢弃（-sn）、保留全部字幕轨（复制）或烧录外部字幕文件。
    // 附件流始终不映射；按轨精细映射属 M4。
    match config.subtitles.mode {
        encode::SubtitleMode::Copy => {
            push(&mut args, "-map");
            push(&mut args, "0:s?");
            push(&mut args, "-c:s");
            push(&mut args, "copy");
        }
        encode::SubtitleMode::Drop => {
            push(&mut args, "-sn");
        }
        encode::SubtitleMode::Burn => {
            // 烧录时丢弃视频内字幕轨，字幕由外部文件渲染进画面。
            push(&mut args, "-sn");
            let path = config
                .subtitles
                .external_path
                .as_ref()
                .ok_or_else(|| "烧录字幕需要选择外部字幕文件（srt / ass）".to_string())?;
            push(&mut args, "-vf");
            push(&mut args, burn_subtitle_filter(path));
        }
    }

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

/// 按编码器家族生成视频质量参数（速度/质量预设 + 质量控制）。
///
/// 各家族的实际选项名不同，且都以"实际可用的 FFmpeg 构建"为准：
/// - 软件（libx264 / libx265）：`-preset <x264 预设> -crf N`；
/// - 软件（libvpx-vp9 / libaom-av1）：`-deadline good -cpu-used N -crf N -b:v 0`
///   ——恒定质量必须显式 `-b:v 0`，否则会退化为受限 VBR；
/// - 软件（libsvtav1）：`-preset N -crf N -b:v 0`；
/// - 软件（librav1e）：`-speed N -crf N -b:v 0`；
/// - NVENC：`-preset p1…p7 -rc vbr -cq N`（`-cq` 标尺 0–51 / AV1 为 0–63）；
/// - Intel QSV：`-preset <名称> -global_quality N`（ICQ 模式）；
/// - AMD AMF：`-quality <speed|balanced|quality> -rc cqp -qp_i N -qp_p N`。
fn push_video_quality_args(
    args: &mut Vec<OsString>,
    spec: &encode::EncoderSpec,
    video: &encode::VideoConfig,
) -> Result<(), String> {
    use encode::PresetKind;

    let value = video.quality_value.to_string();
    // libvpx-vp9 / libaom-av1 的 cpu-used 越小越慢越好（档次已按字符串给出）
    let speed_value = video.preset.as_str();
    match (spec.family, video.quality_mode) {
        (EncoderFamily::Software, QualityMode::Crf) => {
            match spec.preset_kind {
                PresetKind::CpuUsed => {
                    push(args, "-deadline");
                    push(args, "good");
                    push(args, "-cpu-used");
                    push(args, speed_value);
                }
                PresetKind::Speed => {
                    push(args, "-speed");
                    push(args, speed_value);
                }
                // libx264 / libx265 / libsvtav1：统一用 -preset
                PresetKind::Preset | PresetKind::QsvPreset | PresetKind::AmfQuality => {
                    push(args, "-preset");
                    push(args, speed_value);
                }
            }
            if spec.zero_bitrate {
                // libvpx-vp9 额外开启行级多线程（纯性能选项，不影响画质）
                if spec.encoder_id == "libvpx-vp9" {
                    push(args, "-row-mt");
                    push(args, "1");
                }
                push(args, "-crf");
                push(args, value);
                push(args, "-b:v");
                push(args, "0");
            } else {
                push(args, "-crf");
                push(args, value);
            }
        }
        (EncoderFamily::Nvenc, QualityMode::Cq) => {
            push(args, "-preset");
            push(args, &video.preset);
            push(args, "-rc");
            push(args, "vbr");
            push(args, "-cq");
            push(args, value);
        }
        (EncoderFamily::Qsv, QualityMode::Cq) => {
            push(args, "-preset");
            push(args, &video.preset);
            // ICQ：global_quality 即质量等级（1–51，AV1 到 63）
            push(args, "-global_quality");
            push(args, value);
        }
        (EncoderFamily::Amf, QualityMode::Cq) => {
            push(args, "-quality");
            push(args, &video.preset);
            push(args, "-rc");
            push(args, "cqp");
            // AMF 的恒定质量按帧类型给 QP；I/P 用同一值，B 帧交给编码器默认
            push(args, "-qp_i");
            push(args, &value);
            push(args, "-qp_p");
            push(args, &value);
        }
        (family, mode) => {
            return Err(format!(
                "编码器 {}（{family:?}）与质量模式 {mode:?} 不匹配",
                spec.encoder_id
            ));
        }
    }
    Ok(())
}

/// 生成烧录外部字幕的滤镜图字符串。
///
/// libass 的 `subtitles` 滤镜同时支持 srt 与 ass。滤镜参数按
/// `av_get_token` 规则转义：反斜杠统一为 `/`，`:`、`'` 用 `\` 转义，
/// 并用单引号包裹整个文件名（避免 `,` `;` 被当作滤镜链分隔符）。
pub fn burn_subtitle_filter(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    let mut escaped = String::with_capacity(raw.len() + 4);
    for ch in raw.chars() {
        if ch == ':' || ch == '\'' {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    format!("subtitles=filename='{escaped}'")
}

// ---------- 路径安全 ----------

/// 检查输入/输出路径：输入存在性、输出扩展名与所选容器一致、父目录、
/// 输入输出冲突与覆盖行为。
pub fn check_paths(
    input: &Path,
    output: &Path,
    container: Container,
    overwrite: bool,
) -> Result<(), String> {
    if !input.is_file() {
        return Err(format!("输入文件不存在或不是文件：{}", input.display()));
    }

    let expected = container.extension();
    let ext = output
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    match ext {
        Some(e) if e == expected => {}
        // 扩展名属于其他受支持容器：提示用户是容器选择与文件名不一致，
        // 而不是把 .webm 误报成"不支持的格式"。
        Some(e) if Container::extensions().contains(&e.as_str()) => {
            return Err(format!(
                "输出文件扩展名 .{e} 与所选封装格式 {}（.{expected}）不一致：{}",
                container_label(container),
                output.display()
            ));
        }
        _ => {
            return Err(format!(
                "输出文件扩展名必须是 .mkv、.mp4 或 .webm：{}",
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

/// 容器的用户可见名称（用于错误信息与提示）。
pub fn container_label(container: Container) -> &'static str {
    match container {
        Container::Mkv => "MKV",
        Container::Mp4 => "MP4",
        Container::Webm => "WebM",
    }
}

/// 判断文件名是否属于本应用的临时输出（`*.aozora-part-*`），
/// 这类文件是崩溃/强杀遗留的垃圾，可以安全清理。
pub fn is_temp_output_name(name: &str) -> bool {
    name.contains(".aozora-part-")
}

/// 清除文件的只读属性（Windows 下只读属性会使打开写入与删除都失败）。
/// 非 Windows 平台为 no-op。
#[allow(clippy::permissions_set_readonly_false)]
pub fn clear_readonly(path: &Path) {
    if let Ok(metadata) = std::fs::metadata(path) {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            // 现代 std 在 Windows 上会正确清除 FILE_ATTRIBUTE_READONLY
            // （由 verify_writable_cleans_readonly_stale_temp 用例实测覆盖）。
            permissions.set_readonly(false);
            let _ = std::fs::set_permissions(path, permissions);
        }
    }
}

fn try_open_writable(target: &Path) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(target)
}

/// 启动前探测输出位置是否可写：以写模式创建（或截断）目标文件后立即删除。
///
/// FFmpeg 遇到权限问题时只报 `Permission denied`（退出码 -13），无法区分
/// "目录不可写"与"同名文件被占用/只读"；这里提前探测并给出可读原因，
/// 让用户在任务开始前就知道问题所在。若是本应用遗留的临时输出
/// （`*.aozora-part-*`，可能带只读属性），先自动清理后重试。
pub fn verify_writable(target: &Path) -> Result<(), String> {
    let existed = target.exists();
    match try_open_writable(target) {
        Ok(file) => {
            drop(file);
            // 探测文件用完即删，不留下垃圾（FFmpeg 稍后会重新创建）。
            let _ = std::fs::remove_file(target);
            Ok(())
        }
        Err(first_error) => {
            let is_our_temp = target
                .file_name()
                .and_then(|n| n.to_str())
                .map(is_temp_output_name)
                .unwrap_or(false);
            // 崩溃遗留的临时输出：清只读属性后删除并重试一次。
            if is_our_temp && target.exists() {
                clear_readonly(target);
                if std::fs::remove_file(target).is_ok() {
                    if let Ok(file) = try_open_writable(target) {
                        drop(file);
                        let _ = std::fs::remove_file(target);
                        return Ok(());
                    }
                }
                return Err(format!(
                    "无法清理旧的临时输出文件（{}）：{first_error}。文件可能被其他程序占用，请关闭占用该文件的程序后重试",
                    target.display()
                ));
            }
            let parent = target
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "（未知目录）".to_string());
            if existed {
                Err(format!(
                    "无法覆盖已存在的文件（{}）：{first_error}。文件可能被其他程序占用、为只读属性，或目录权限不足",
                    target.display()
                ))
            } else {
                Err(format!(
                    "输出目录不可写（{parent}）：{first_error}。请检查目录权限、安全软件拦截或\"受控文件夹访问\"设置",
                ))
            }
        }
    }
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

/// 任务专属临时输出路径：位于应用自己的临时目录（由调用方提供，
/// 保证可写），文件名沿用最终输出的主干 + 任务 ID 标记，扩展名保持
/// 不变（FFmpeg 依扩展名选择封装器）。
pub fn temp_output_path(temp_dir: &Path, final_output: &Path, job_id: &str) -> PathBuf {
    let stem = final_output
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    let ext = final_output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mkv");
    temp_dir.join(format!("{stem}.aozora-part-{job_id}.{ext}"))
}

/// 清理临时目录中本应用遗留的临时输出（`*.aozora-part-*`，
/// 崩溃/强杀残留，可能带只读属性）。
pub fn clean_stale_temp_files(temp_dir: &Path) {
    let Ok(entries) = std::fs::read_dir(temp_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_ours = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(is_temp_output_name)
            .unwrap_or(false);
        if is_ours && path.is_file() {
            clear_readonly(&path);
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// 探测最终输出所在目录可写：在目标目录创建探针文件后删除，
/// 不触碰最终输出文件本身。用于在开始压制前发现"目标目录不可写"，
/// 避免 FFmpeg 编码完成后才发现无法收尾。
pub fn verify_destination_writable(final_output: &Path, job_id: &str) -> Result<(), String> {
    let parent = final_output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| format!("输出路径缺少父目录：{}", final_output.display()))?;
    let stem = final_output
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    let probe = parent.join(format!("{stem}.aozora-probe-{job_id}.tmp"));
    verify_writable(&probe)
}

/// 目标目录不可写时的回退成品路径：应用临时目录内、沿用最终输出的
/// 文件名主干，并附加任务 ID 保证唯一——应用临时目录是所有回退成品
/// 共用的目录，若只按文件名命名，不同目录下的同名输入（或重复入队的
/// 同一配置）会互相覆盖成品。
pub fn fallback_output_path(temp_dir: &Path, final_output: &Path, job_id: &str) -> PathBuf {
    let stem = final_output
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    let ext = final_output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mkv");
    temp_dir.join(format!("{stem}.{job_id}.{ext}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{
        AudioConfig, AudioMode, OutputConfig, QualityMode, SubtitleConfig, SubtitleMode,
        VideoConfig,
    };
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
            subtitles: SubtitleConfig::default(),
            output: OutputConfig {
                path: PathBuf::from("out.mkv"),
                container: Container::Mkv,
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
    fn subtitle_copy_mode_maps_and_copies_subtitles() {
        let mut config = x264_config(18, "slow");
        config.subtitles = SubtitleConfig {
            mode: SubtitleMode::Copy,
            external_path: None,
        };
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap();
        let text: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = text.join(" ");
        assert!(joined.contains("-map 0:s? -c:s copy"), "args: {joined}");
        assert!(!joined.contains("-sn"), "args: {joined}");
    }

    #[test]
    fn subtitle_drop_mode_uses_sn() {
        let mut config = x264_config(18, "slow");
        config.subtitles = SubtitleConfig {
            mode: SubtitleMode::Drop,
            external_path: None,
        };
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap();
        let text: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = text.join(" ");
        assert!(joined.contains("-sn"), "args: {joined}");
        assert!(!joined.contains("-c:s"), "args: {joined}");
    }

    #[test]
    fn subtitle_burn_mode_adds_escaped_filter_and_drops_packaged() {
        let mut config = x264_config(18, "slow");
        config.subtitles = SubtitleConfig {
            mode: SubtitleMode::Burn,
            external_path: Some(PathBuf::from(r"C:\字幕 目录\样片 [01].ass")),
        };
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap();
        let text: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = text.join(" ");
        assert!(joined.contains("-sn"), "args: {joined}");
        assert!(!joined.contains("-map 0:s?"), "args: {joined}");
        let vf_index = text.iter().position(|a| a == "-vf").expect("应有 -vf");
        let filter = &text[vf_index + 1];
        assert!(
            filter.starts_with("subtitles=filename='"),
            "filter: {filter}"
        );
        // 冒号被转义、反斜杠统一为斜杠、空格/括号/中文保留
        assert!(
            filter.contains(r"C\:/字幕 目录/样片 [01].ass"),
            "filter: {filter}"
        );
    }

    #[test]
    fn burn_subtitle_filter_escapes_special_chars() {
        let filter = burn_subtitle_filter(Path::new(r"C:\dir\sub's title.ass"));
        assert_eq!(filter, r"subtitles=filename='C\:/dir/sub\'s title.ass'");
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
            joined.contains("-c:v hevc_nvenc -preset p7 -rc vbr -cq 21"),
            "args: {joined}"
        );
    }

    #[test]
    fn qsv_args_use_global_quality() {
        let mut config = x264_config(23, "medium");
        config.video.encoder_id = "hevc_qsv".into();
        config.video.quality_mode = QualityMode::Cq;
        config.video.pixel_format = Some("nv12".into());
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap();
        let joined: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = joined.join(" ");
        assert!(
            joined.contains("-c:v hevc_qsv -preset medium -global_quality 23 -pix_fmt nv12"),
            "args: {joined}"
        );
        assert!(!joined.contains("-crf"), "QSV 不应使用 -crf：{joined}");
    }

    #[test]
    fn amf_args_use_cqp() {
        let mut config = x264_config(24, "balanced");
        config.video.encoder_id = "h264_amf".into();
        config.video.quality_mode = QualityMode::Cq;
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap();
        let joined: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = joined.join(" ");
        assert!(
            joined.contains("-c:v h264_amf -quality balanced -rc cqp -qp_i 24 -qp_p 24"),
            "args: {joined}"
        );
    }

    #[test]
    fn av1_software_encoders_use_their_own_speed_option() {
        // libsvtav1：-preset N（数字档位）
        let mut config = x264_config(35, "8");
        config.video.encoder_id = "libsvtav1".into();
        let joined =
            join_args(&build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap());
        assert!(
            joined.contains("-c:v libsvtav1 -preset 8 -crf 35 -b:v 0"),
            "args: {joined}"
        );

        // libaom-av1：-deadline good -cpu-used N
        let mut config = x264_config(32, "6");
        config.video.encoder_id = "libaom-av1".into();
        let joined =
            join_args(&build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap());
        assert!(
            joined.contains("-c:v libaom-av1 -deadline good -cpu-used 6 -crf 32 -b:v 0"),
            "args: {joined}"
        );
        assert!(
            !joined.contains("-row-mt"),
            "row-mt 只用于 libvpx-vp9：{joined}"
        );

        // librav1e：-speed N
        let mut config = x264_config(32, "6");
        config.video.encoder_id = "librav1e".into();
        let joined =
            join_args(&build_args(&config, Path::new("in.mkv"), Path::new("out.mkv")).unwrap());
        assert!(
            joined.contains("-c:v librav1e -speed 6 -crf 32 -b:v 0"),
            "args: {joined}"
        );
    }

    /// 参数数组转成一行文本，便于断言。
    fn join_args(args: &[OsString]) -> String {
        args.iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn vp9_args_force_constant_quality() {
        let mut config = x264_config(32, "2");
        config.video.encoder_id = "libvpx-vp9".into();
        let args = build_args(&config, Path::new("in.mkv"), Path::new("out.webm")).unwrap();
        let joined: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = joined.join(" ");
        // VP9 恒定质量必须带 -b:v 0，且用 cpu-used 而非 -preset
        assert!(
            joined.contains("-c:v libvpx-vp9 -deadline good -cpu-used 2 -row-mt 1 -crf 32 -b:v 0"),
            "args: {joined}"
        );
        assert!(
            !joined.contains("-preset"),
            "libvpx-vp9 不应出现 -preset：{joined}"
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
        assert!(check_paths(&input, &output, Container::Mkv, false).is_ok());
        assert!(check_paths(&input, &output, Container::Mkv, true).is_ok());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn verify_writable_creates_and_removes_probe_file() {
        let dir = temp_dir("writable");
        let target = dir.join("probe.mkv");
        assert!(verify_writable(&target).is_ok());
        assert!(!target.exists(), "探测文件应被删除");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn verify_writable_reports_missing_directory() {
        let dir = temp_dir("unwritable");
        let err = verify_writable(&dir.join("no-such-dir").join("probe.mkv")).unwrap_err();
        assert!(err.contains("输出目录不可写"), "error: {err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(windows)]
    #[test]
    fn verify_writable_reports_readonly_existing_file() {
        let dir = temp_dir("readonly");
        let target = dir.join("probe.mkv");
        fs::write(&target, "旧内容").unwrap();
        // 只读属性：Windows 下写入打开失败（EACCES）
        let mut permissions = fs::metadata(&target).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&target, permissions).unwrap();
        let err = verify_writable(&target).unwrap_err();
        assert!(err.contains("无法覆盖已存在的文件"), "error: {err}");
        fs::remove_file(&target).ok();
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(windows)]
    #[test]
    fn verify_writable_cleans_readonly_stale_temp() {
        let dir = temp_dir("stale-temp");
        // 模拟崩溃遗留的只读临时输出文件
        let target = dir.join("out.aozora-part-e-123.mkv");
        fs::write(&target, "残留").unwrap();
        let mut permissions = fs::metadata(&target).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&target, permissions).unwrap();

        assert!(verify_writable(&target).is_ok());
        assert!(!target.exists(), "残留临时文件应被自动清理");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_missing_input() {
        let dir = temp_dir("missing-input");
        let err = check_paths(
            &dir.join("nope.mkv"),
            &dir.join("out.mkv"),
            Container::Mkv,
            false,
        )
        .unwrap_err();
        assert!(err.contains("输入文件不存在"), "error: {err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_unsupported_extension() {
        let dir = temp_dir("bad-ext");
        let input = dir.join("input.mkv");
        fs::write(&input, b"x").unwrap();
        let err = check_paths(&input, &dir.join("out.avi"), Container::Mkv, false).unwrap_err();
        assert!(err.contains(".mkv、.mp4 或 .webm"), "error: {err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_accepts_each_supported_container() {
        let dir = temp_dir("containers");
        let input = dir.join("input.mkv");
        fs::write(&input, b"x").unwrap();
        for (container, ext) in [
            (Container::Mkv, "mkv"),
            (Container::Mp4, "mp4"),
            (Container::Webm, "webm"),
        ] {
            let output = dir.join(format!("out.{ext}"));
            assert!(
                check_paths(&input, &output, container, false).is_ok(),
                "container {container:?} 应接受 .{ext}"
            );
        }
        // 扩展名大小写不敏感
        assert!(check_paths(&input, &dir.join("OUT.MP4"), Container::Mp4, false).is_ok());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_extension_container_mismatch() {
        let dir = temp_dir("mismatch");
        let input = dir.join("input.mkv");
        fs::write(&input, b"x").unwrap();
        // 选了 MP4 却写 .mkv：提示两者不一致，而不是"不支持的扩展名"
        let err = check_paths(&input, &dir.join("out.mkv"), Container::Mp4, false).unwrap_err();
        assert!(err.contains("与所选封装格式 MP4"), "error: {err}");
        let err = check_paths(&input, &dir.join("out.webm"), Container::Mkv, false).unwrap_err();
        assert!(err.contains("与所选封装格式 MKV"), "error: {err}");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_missing_parent_dir() {
        let dir = temp_dir("missing-parent");
        let input = dir.join("input.mkv");
        fs::write(&input, b"x").unwrap();
        let err = check_paths(
            &input,
            &dir.join("no-such-dir").join("out.mkv"),
            Container::Mkv,
            false,
        )
        .unwrap_err();
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
        let err = check_paths(&input, &output, Container::Mkv, false).unwrap_err();
        assert!(err.contains("已存在"), "error: {err}");
        assert!(check_paths(&input, &output, Container::Mkv, true).is_ok());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn check_paths_rejects_input_output_conflict() {
        let dir = temp_dir("conflict");
        let input = dir.join("same.mkv");
        fs::write(&input, b"x").unwrap();
        let err = check_paths(&input, &input, Container::Mkv, true).unwrap_err();
        assert!(err.contains("禁止覆盖输入"), "error: {err}");

        // 大小写不同、路径写法不同的等价路径也应识别为同一文件
        let mut output = input.clone();
        output.pop();
        output.push("SAME.mkv");
        if cfg!(windows) {
            let err = check_paths(&input, &output, Container::Mkv, true).unwrap_err();
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
    fn temp_output_path_uses_app_temp_dir_and_keeps_extension() {
        let temp_dir = Path::new(r"C:\app\encode-tmp");
        let final_output = Path::new(r"D:\out\影片.mkv");
        let temp = temp_output_path(temp_dir, final_output, "e-123");
        assert_eq!(temp.parent(), Some(temp_dir));
        assert_ne!(temp, Path::new(r"D:\out\影片.mkv"));
        let name = temp.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with("影片.aozora-part-e-123"), "name: {name}");
        assert!(name.ends_with(".mkv"), "name: {name}");
    }

    #[test]
    fn clean_stale_temp_files_removes_only_our_temps() {
        let dir = temp_dir("stale-clean");
        let ours = dir.join("out.aozora-part-e-9.mkv");
        let other = dir.join("keep.mp4");
        fs::write(&ours, "x").unwrap();
        fs::write(&other, "y").unwrap();
        clean_stale_temp_files(&dir);
        assert!(!ours.exists(), "本应用临时文件应被清理");
        assert!(other.exists(), "无关文件不应被删除");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn verify_destination_writable_probes_and_cleans() {
        let dir = temp_dir("dest-probe");
        let final_output = dir.join("out.mkv");
        assert!(verify_destination_writable(&final_output, "e-1").is_ok());
        let leftovers = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".aozora-probe-"))
            .count();
        assert_eq!(leftovers, 0, "探针文件应被删除");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn fallback_output_path_is_unique_per_job() {
        let temp_dir = Path::new(r"C:\app\encode-tmp");
        let final_output = Path::new(r"D:\影片\样片.encoded.mkv");
        let first = fallback_output_path(temp_dir, final_output, "e-1");
        assert_eq!(first, Path::new(r"C:\app\encode-tmp\样片.encoded.e-1.mkv"));
        // 不同任务（即使最终文件名相同）的回退路径必须不同，避免互相覆盖成品
        let second = fallback_output_path(temp_dir, Path::new(r"E:\其他\样片.encoded.mkv"), "e-2");
        assert_ne!(first, second);
    }
}
