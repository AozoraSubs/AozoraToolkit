use crate::encode::{EncoderSpec, EncoderVendor, ENCODER_SPECS};
use serde::Serialize;
use std::path::Path;
use std::process::Command;

/// 一个编码器的实际可用性。
///
/// 判定分两步：
/// 1. `ffmpeg -encoders` 中是否存在（该构建是否编译进了这个编码器）；
/// 2. 硬件编码器再跑一次最小实拍（见 [`probe_encoder_usable`]），
///    排除"编译进来了但本机没有对应硬件 / 驱动不工作"的情况。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderAvailability {
    pub id: String,
    pub label: String,
    pub vendor: EncoderVendor,
    /// 输出视频编码格式（h264 / hevc / av1 / vp9）。
    pub codec: String,
    /// 是否为硬件编码器。
    pub hardware: bool,
    /// 是否出现在 `ffmpeg -encoders` 中。
    pub listed: bool,
    /// 是否可用（listed 且通过实拍探测）。
    pub usable: bool,
    /// 不可用原因（可读，用于界面与诊断）。
    pub reason: Option<String>,
}

/// 显卡 / 显示适配器信息（用于向用户解释硬件编码器从何而来）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuAdapter {
    pub name: String,
    /// 归一化后的厂商：`nvidia` / `amd` / `intel` / `other`。
    pub vendor: String,
    pub driver: Option<String>,
}

/// 硬件编码器实拍探测结果。
#[derive(Debug, Clone)]
pub struct EncoderProbe {
    pub usable: bool,
    pub error: Option<String>,
}

/// FFmpeg 能力检测结果。
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityReport {
    /// 应用内置编码器的逐个可用性（UI 的编码器列表由此生成，不再硬编码可用性）。
    pub encoders: Vec<EncoderAvailability>,
    /// 当前构建中可用的编码器总数。
    pub encoder_count: usize,
    pub filter_count: usize,
    /// 是否有 `subtitles` 滤镜（libass）——外部字幕烧录依赖它。
    pub has_subtitles_filter: bool,
    pub hw_accels: Vec<String>,
    /// 检测到的显示适配器（显卡），用于解释硬件编码器可用性。
    pub gpus: Vec<GpuAdapter>,
    /// 非致命问题（例如某子命令失败），整体检测仍可用。
    pub warnings: Vec<String>,
}

impl CapabilityReport {
    /// 可用的编码器 ID 集合（前端据此过滤编码器下拉）。
    pub fn usable_encoder_ids(&self) -> Vec<String> {
        self.encoders
            .iter()
            .filter(|item| item.usable)
            .map(|item| item.id.clone())
            .collect()
    }
}

fn run_lines(ffmpeg_path: &Path, args: &[&str]) -> Result<Vec<String>, String> {
    let mut command = Command::new(ffmpeg_path);
    crate::process_flags::hide_console_std(&mut command);
    let output = command
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

/// 用最小实拍验证一个硬件编码器是否真的能在本机工作。
///
/// 只编码 5 帧 320×240 的合成画面到 `null`，不产生任何输出文件；
/// "编译进来了但机器上跑不起来"（无对应显卡、驱动不支持、虚拟显示适配器）
/// 会在此以非零退出码暴露出来，错误信息取最后一行 stderr。
pub fn probe_encoder_usable(ffmpeg_path: &Path, encoder_id: &str) -> EncoderProbe {
    let mut command = Command::new(ffmpeg_path);
    crate::process_flags::hide_console_std(&mut command);
    let output = command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-nostdin",
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=320x240:rate=30",
            "-frames:v",
            "5",
            "-c:v",
            encoder_id,
            "-f",
            "null",
            "-",
        ])
        .output();
    match output {
        Ok(output) if output.status.success() => EncoderProbe {
            usable: true,
            error: None,
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            EncoderProbe {
                usable: false,
                error: Some(extract_encoder_error(&stderr)),
            }
        }
        Err(e) => EncoderProbe {
            usable: false,
            error: Some(format!("无法启动 FFmpeg：{e}")),
        },
    }
}

/// 从 FFmpeg stderr 中提取最能说明问题的一行。
///
/// 硬件编码器失败时，最后一句话往往是
/// `Nothing was written into output file...` 这类收尾信息，真正的原因
/// （`No capable devices found` / `Error creating a MFX session` /
/// `CreateComponent(...) failed`）出现在前面，所以这里优先挑包含错误关键词的
/// 最后一行。
pub fn extract_encoder_error(stderr: &str) -> String {
    const NOISE: [&str; 2] = [
        "Nothing was written into output file",
        "at least one of its streams received no packets",
    ];
    let candidates: Vec<&str> = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !NOISE.iter().any(|noise| line.contains(noise)))
        .collect();
    const KEYWORDS: [&str; 7] = [
        "no capable",
        "error",
        "failed",
        "not supported",
        "cannot",
        "unable",
        "invalid",
    ];
    let best = candidates
        .iter()
        .rev()
        .find(|line| {
            let lower = line.to_ascii_lowercase();
            KEYWORDS.iter().any(|keyword| lower.contains(keyword))
        })
        .or_else(|| candidates.last());
    match best {
        Some(line) => line.chars().take(200).collect(),
        None => "编码器返回非零退出码（FFmpeg 未输出更多信息）".to_string(),
    }
}

/// 汇总每个内置编码器的可用性。
///
/// `listed` 来自 `ffmpeg -encoders`；硬件编码器额外实拍一次。软件编码器
/// 视为只要编译进来就可用（CPU 编码不依赖外部硬件）。
fn collect_encoders(ffmpeg_path: &Path, listed_names: &[String]) -> Vec<EncoderAvailability> {
    ENCODER_SPECS
        .iter()
        .map(|spec| availability_for(ffmpeg_path, spec, listed_names))
        .collect()
}

fn availability_for(
    ffmpeg_path: &Path,
    spec: &EncoderSpec,
    listed_names: &[String],
) -> EncoderAvailability {
    let listed = listed_names.iter().any(|name| name == spec.encoder_id);
    let hardware = spec.vendor.is_hardware();
    let (usable, reason) = if !listed {
        (
            false,
            Some(format!(
                "当前 FFmpeg 构建未包含该编码器（{}）",
                spec.encoder_id
            )),
        )
    } else if !hardware {
        (true, None)
    } else if cfg!(windows) {
        // 硬件编码器：实拍一次，排除"有编码器但没有对应硬件 / 驱动不工作"
        let probe = probe_encoder_usable(ffmpeg_path, spec.encoder_id);
        match (probe.usable, probe.error) {
            (true, _) => (true, None),
            (false, error) => (
                false,
                Some(format!(
                    "{} 硬件编码不可用：{}",
                    spec.vendor.label(),
                    error.unwrap_or_else(|| "实拍探测失败".to_string())
                )),
            ),
        }
    } else {
        // 非 Windows 平台本轮不做实拍探测，只按构建是否包含判断
        (true, None)
    };

    EncoderAvailability {
        id: spec.encoder_id.to_string(),
        label: spec.label.to_string(),
        vendor: spec.vendor,
        codec: spec.codec.to_string(),
        hardware,
        listed,
        usable,
        reason,
    }
}

/// 检测 FFmpeg 实际能力：编码器（含硬件实拍）、滤镜、硬件加速方法、显示适配器。
pub fn detect_capabilities(ffmpeg_path: &Path) -> CapabilityReport {
    let mut report = CapabilityReport::default();

    match run_lines(ffmpeg_path, &["-hide_banner", "-encoders"]) {
        Ok(lines) => {
            let names = parse_encoder_names(&lines);
            report.encoder_count = names.len();
            report.encoders = collect_encoders(ffmpeg_path, &names);
        }
        Err(e) => report.warnings.push(format!("编码器检测失败：{e}")),
    }

    match run_lines(ffmpeg_path, &["-hide_banner", "-filters"]) {
        Ok(lines) => {
            let names = parse_filter_names(&lines);
            report.filter_count = names.len();
            report.has_subtitles_filter = names.iter().any(|n| n == "subtitles");
        }
        Err(e) => report.warnings.push(format!("滤镜检测失败：{e}")),
    }

    match run_lines(ffmpeg_path, &["-hide_banner", "-hwaccels"]) {
        Ok(lines) => report.hw_accels = parse_hw_accels(&lines),
        Err(e) => report.warnings.push(format!("硬件加速检测失败：{e}")),
    }

    match detect_gpu_adapters() {
        Ok(gpus) => report.gpus = gpus,
        Err(e) => report.warnings.push(format!("显卡检测失败：{e}")),
    }

    report
}

/// 检查 ffmpeg 是否包含指定滤镜（轻量单命令检测，供任务启动前使用）。
pub fn has_filter(ffmpeg_path: &Path, filter: &str) -> bool {
    run_lines(ffmpeg_path, &["-hide_banner", "-filters"])
        .map(|lines| parse_filter_names(&lines).iter().any(|n| n == filter))
        .unwrap_or(false)
}

// ---------- 显卡检测 ----------

/// 把适配器名称 / 厂商字符串归一化为厂商标识。
pub fn normalize_gpu_vendor(name: &str, compatibility: &str) -> &'static str {
    let text = format!("{name} {compatibility}").to_ascii_lowercase();
    if text.contains("nvidia") || text.contains("geforce") || text.contains("quadro") {
        "nvidia"
    } else if text.contains("amd") || text.contains("radeon") || text.contains("advanced micro") {
        "amd"
    } else if text.contains("intel") || text.contains("arc ") || text.contains("iris") {
        "intel"
    } else {
        "other"
    }
}

/// 检测显示适配器（显卡）。
///
/// 只做只读查询，不加载任何驱动：Windows 用 PowerShell + CIM（WMI），
/// macOS 用 `system_profiler`，其他平台读 `/sys/class/drm`。检测失败不算
/// 致命错误——硬件编码器的可用性以实拍探测为准，这里的信息只用于向用户
/// 解释"为什么某个编码器不能用"。
pub fn detect_gpu_adapters() -> Result<Vec<GpuAdapter>, String> {
    #[cfg(windows)]
    {
        detect_gpu_adapters_windows()
    }
    #[cfg(target_os = "macos")]
    {
        detect_gpu_adapters_macos()
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        detect_gpu_adapters_linux()
    }
}

/// Windows：`Get-CimInstance Win32_VideoController` 取名称 / 厂商 / 驱动版本。
#[cfg(windows)]
fn detect_gpu_adapters_windows() -> Result<Vec<GpuAdapter>, String> {
    const SCRIPT: &str = "Get-CimInstance Win32_VideoController | \
         Select-Object Name,AdapterCompatibility,DriverVersion | \
         ConvertTo-Json -Compress";

    let mut command = Command::new("powershell");
    crate::process_flags::hide_console_std(&mut command);
    let output = command
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .map_err(|e| format!("无法启动 PowerShell：{e}"))?;
    if !output.status.success() {
        return Err(format!(
            "PowerShell 查询显卡失败：{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_gpu_json(&text)
}

/// 解析显卡 JSON（单条对象或数组，字段名大小写不敏感）。
pub fn parse_gpu_json(text: &str) -> Result<Vec<GpuAdapter>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| format!("显卡信息解析失败：{e}"))?;
    let items: Vec<&serde_json::Value> = match &value {
        serde_json::Value::Array(items) => items.iter().collect(),
        other => vec![other],
    };
    let mut adapters = Vec::new();
    for item in items {
        let name = json_field(item, "Name").unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let compatibility = json_field(item, "AdapterCompatibility").unwrap_or_default();
        let driver = json_field(item, "DriverVersion").filter(|value| !value.is_empty());
        adapters.push(GpuAdapter {
            vendor: normalize_gpu_vendor(&name, &compatibility).to_string(),
            name,
            driver,
        });
    }
    Ok(adapters)
}

fn json_field(value: &serde_json::Value, key: &str) -> Option<String> {
    let object = value.as_object()?;
    for (name, field) in object {
        if name.eq_ignore_ascii_case(key) {
            return match field {
                serde_json::Value::String(text) => Some(text.trim().to_string()),
                serde_json::Value::Null => None,
                other => Some(other.to_string()),
            };
        }
    }
    None
}

/// macOS：`system_profiler SPDisplaysDataType` 的纯文本输出取 "Chipset Model"。
#[cfg(target_os = "macos")]
fn detect_gpu_adapters_macos() -> Result<Vec<GpuAdapter>, String> {
    let mut command = Command::new("system_profiler");
    let output = command
        .args(["SPDisplaysDataType"])
        .output()
        .map_err(|e| format!("无法启动 system_profiler：{e}"))?;
    if !output.status.success() {
        return Err("system_profiler 查询显卡失败".to_string());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut adapters = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Chipset Model:") {
            let name = rest.trim().to_string();
            if !name.is_empty() {
                adapters.push(GpuAdapter {
                    vendor: normalize_gpu_vendor(&name, "").to_string(),
                    name,
                    driver: None,
                });
            }
        }
    }
    Ok(adapters)
}

/// Linux：读 `/sys/class/drm/card*/device/{vendor,uevent}`。
#[cfg(all(not(windows), not(target_os = "macos")))]
fn detect_gpu_adapters_linux() -> Result<Vec<GpuAdapter>, String> {
    let mut adapters = Vec::new();
    let entries =
        std::fs::read_dir("/sys/class/drm").map_err(|e| format!("/sys/class/drm：{e}"))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let device = entry.path().join("device");
        let driver = std::fs::read_to_string(device.join("uevent"))
            .ok()
            .and_then(|text| {
                text.lines()
                    .find_map(|line| line.strip_prefix("DRIVER=").map(str::to_string))
            });
        // 供应商 ID 十六进制：0x10de NVIDIA、0x1002 AMD、0x8086 Intel
        let vendor_id = std::fs::read_to_string(device.join("vendor"))
            .ok()
            .map(|text| text.trim().to_ascii_lowercase());
        let vendor = match vendor_id.as_deref() {
            Some("0x10de") => "nvidia",
            Some("0x1002") | Some("0x1022") => "amd",
            Some("0x8086") => "intel",
            _ => "other",
        };
        adapters.push(GpuAdapter {
            name: driver.clone().unwrap_or_else(|| name.clone()),
            vendor: vendor.to_string(),
            driver,
        });
    }
    Ok(adapters)
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

    #[test]
    fn normalize_gpu_vendor_recognises_the_three_main_vendors() {
        assert_eq!(
            normalize_gpu_vendor("NVIDIA GeForce RTX 4080 Laptop GPU", "NVIDIA"),
            "nvidia"
        );
        assert_eq!(
            normalize_gpu_vendor("AMD Radeon(TM) 610M", "Advanced Micro Devices, Inc."),
            "amd"
        );
        assert_eq!(
            normalize_gpu_vendor("Intel(R) Iris(R) Xe Graphics", "Intel Corporation"),
            "intel"
        );
        assert_eq!(
            normalize_gpu_vendor("GameViewer Virtual Display Adapter", "GameViewer"),
            "other"
        );
    }

    #[test]
    fn parse_gpu_json_handles_single_object_and_array() {
        // ConvertTo-Json 在只有一块显卡时返回对象而非数组
        let single = r#"{"Name":"NVIDIA GeForce RTX 4080","AdapterCompatibility":"NVIDIA","DriverVersion":"32.0.16.1088"}"#;
        let parsed = parse_gpu_json(single).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].vendor, "nvidia");
        assert_eq!(parsed[0].driver.as_deref(), Some("32.0.16.1088"));

        let array = r#"[
          {"Name":"AMD Radeon 610M","AdapterCompatibility":"Advanced Micro Devices, Inc.","DriverVersion":null},
          {"Name":"MuMu Virtual Display Adapter","AdapterCompatibility":"MuMu"}
        ]"#;
        let parsed = parse_gpu_json(array).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].vendor, "amd");
        assert_eq!(parsed[0].driver, None);
        assert_eq!(parsed[1].vendor, "other");

        assert!(parse_gpu_json("").unwrap().is_empty());
        assert!(parse_gpu_json("not json").is_err());
    }

    #[test]
    fn availability_marks_unlisted_and_hardware_encoders() {
        // 直接给已解析出的编码器名（模拟 `ffmpeg -encoders` 的结果）
        let listed = vec!["libx264".to_string(), "av1_nvenc".to_string()];
        let report = CapabilityReport {
            encoders: collect_encoders(Path::new("ffmpeg"), &listed),
            ..CapabilityReport::default()
        };
        let by_id = |id: &str| {
            report
                .encoders
                .iter()
                .find(|item| item.id == id)
                .unwrap_or_else(|| panic!("缺少编码器 {id}"))
        };
        // 编译进来的软件编码器直接可用（不依赖硬件）
        assert!(by_id("libx264").usable);
        assert!(by_id("libx264").reason.is_none());
        // 未编译进来的编码器不可用，并给出原因
        let x265 = by_id("libx265");
        assert!(!x265.listed && !x265.usable);
        assert!(x265.reason.as_deref().unwrap_or("").contains("未包含"));
        // 硬件编码器带厂商标记（实拍结果取决于本机是否有对应硬件）
        assert!(by_id("av1_nvenc").hardware);
        assert_eq!(by_id("av1_nvenc").vendor.label(), "NVIDIA");
        assert!(!by_id("libx265").hardware);
    }

    #[test]
    fn extract_encoder_error_picks_the_meaningful_line() {
        // 真实的 av1_nvenc 失败输出：原因在前，收尾噪声在后
        let stderr = "[av1_nvenc @ 0x123] Cannot load nvcuda.dll.\n\
                      [av1_nvenc @ 0x123] No capable devices found\n\
                      [out#0/null @ 0x456] Nothing was written into output file, because at least one of its streams received no packets.\n";
        let message = extract_encoder_error(stderr);
        assert!(
            message.contains("No capable devices found"),
            "message: {message}"
        );
        assert!(
            !message.contains("Nothing was written"),
            "message: {message}"
        );

        // QSV 的 MFX 会话错误
        let stderr = "[h264_qsv @ 0x1] Error creating a MFX session: -9.\n\
                      [out#0/null @ 0x2] Nothing was written into output file, because at least one of its streams received no packets.\n";
        assert!(extract_encoder_error(stderr).contains("MFX session"));

        // 完全没有可用信息时给出兜底文案
        assert!(extract_encoder_error("").contains("未输出更多信息"));
        assert!(extract_encoder_error(
            "[out#0/null @ 0x1] Nothing was written into output file, because at least one of its streams received no packets.\n"
        )
        .contains("未输出更多信息"));
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
        assert!(!report.encoders.is_empty(), "内置编码器列表为空");
        let usable = report.usable_encoder_ids();
        assert!(
            usable.iter().any(|id| id == "libx264"),
            "本机 ffmpeg 应包含 libx264（gyan 完整构建），实际可用：{usable:?}"
        );
        for item in &report.encoders {
            println!(
                "{:<14} vendor={} hardware={} listed={} usable={} {}",
                item.id,
                item.vendor.label(),
                item.hardware,
                item.listed,
                item.usable,
                item.reason.clone().unwrap_or_default()
            );
        }
    }

    /// 实机验证：硬件编码器实拍探测（本机有对应硬件时应通过）。
    #[test]
    #[ignore = "requires ffmpeg with the probed hardware encoder"]
    fn probe_hardware_encoder_real() {
        use crate::media_probe::find_in_path;
        let path_entries: Vec<std::path::PathBuf> = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        let ffmpeg = find_in_path("ffmpeg", &path_entries).expect("ffmpeg 未在 PATH 中检测到");
        for encoder in ["h264_nvenc", "h264_amf"] {
            let probe = probe_encoder_usable(&ffmpeg, encoder);
            println!("{encoder}: usable={} {:?}", probe.usable, probe.error);
        }
    }
}
