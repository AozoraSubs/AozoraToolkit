//! 预设模型与本地预设存储（PLAN §4.4 / §8）。
//!
//! 预设只保存编码参数（视频 / 音频 / 字幕 / 覆盖策略），**不保存任何
//! 机器相关的绝对路径**（输入文件、输出文件、外部字幕文件）：应用预设后由
//! 用户在压制页选择具体路径。内置示例预设在程序中编译提供（只读），
//! 用户预设以 `<id>.json` 存放在配置目录的 `presets/` 下。

use crate::encode::{
    self, AudioConfig, Container, EncodeConfig, OutputConfig, SubtitleConfig, SubtitleMode,
    VideoConfig, ENCODE_CONFIG_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// 预设 JSON 格式版本。
pub const PRESET_SCHEMA_VERSION: u32 = 1;
/// 预设名称最大字符数。
pub const MAX_PRESET_NAME_CHARS: usize = 60;
/// 预设描述最大字符数。
pub const MAX_PRESET_DESCRIPTION_CHARS: usize = 200;
/// 导入预设文件的大小上限（1 MiB），避免误选巨大文件。
pub const MAX_PRESET_FILE_BYTES: u64 = 1024 * 1024;

/// 内置预设 ID 前缀（用户文件不会使用该前缀）。
pub const BUILTIN_PRESET_PREFIX: &str = "builtin-";

static PRESET_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

// ---------- 数据模型 ----------

/// 预设中的输出选项：只保留与机器无关的输出意图（封装格式与覆盖策略），
/// 不含输出路径。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetOutput {
    /// 封装格式；旧预设无此字段时默认 MKV。
    #[serde(default)]
    pub container: Container,
    /// 应用预设时是否允许覆盖已存在的输出文件。
    #[serde(default)]
    pub overwrite: bool,
}

/// 一条编码预设。字段未知时解析失败（`deny_unknown_fields`），
/// 避免导入携带额外字段的文件时静默丢弃内容。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preset {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// 内置示例预设标记：内置预设由程序提供，不可重命名或删除。
    #[serde(default)]
    pub builtin: bool,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    #[serde(default)]
    pub subtitles: SubtitleConfig,
    #[serde(default)]
    pub output: PresetOutput,
    #[serde(default)]
    pub created_at_ms: u64,
    #[serde(default)]
    pub updated_at_ms: u64,
}

/// 新建 / 复制预设时由前端提交的输入（ID 与时间戳由存储层生成）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetInput {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub video: VideoConfig,
    pub audio: AudioConfig,
    #[serde(default)]
    pub subtitles: SubtitleConfig,
    #[serde(default)]
    pub output: PresetOutput,
}

/// 预设列表与逐文件加载错误（单个文件损坏不影响其他预设）。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetList {
    pub presets: Vec<Preset>,
    pub errors: Vec<String>,
}

// ---------- 内置示例预设 ----------

/// 内置示例预设（PLAN §4.4：H.264 兼容、HEVC 高画质、NVENC 快速编码）。
///
/// 这些预设是起点建议而非"最佳参数"：不同编码器的质量数值不可直接比较。
pub fn builtin_presets() -> Vec<Preset> {
    vec![
        Preset {
            schema_version: PRESET_SCHEMA_VERSION,
            id: format!("{BUILTIN_PRESET_PREFIX}h264-compatible"),
            name: "H.264 兼容（CRF 18 / slow）".to_string(),
            description: Some(
                "兼容性优先的 H.264 软件编码起点；同画质下体积通常大于 HEVC。".to_string(),
            ),
            builtin: true,
            video: builtin_video("libx264", 18, "slow", None),
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            output: PresetOutput::default(),
            created_at_ms: 0,
            updated_at_ms: 0,
        },
        Preset {
            schema_version: PRESET_SCHEMA_VERSION,
            id: format!("{BUILTIN_PRESET_PREFIX}hevc-quality"),
            name: "HEVC 高画质（CRF 16 / slow / 10-bit）".to_string(),
            description: Some(
                "离线压制的高画质起点（libx265，10-bit）；建议与源片视觉对比后再调整 CRF。"
                    .to_string(),
            ),
            builtin: true,
            video: builtin_video("libx265", 16, "slow", Some("yuv420p10le")),
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            output: PresetOutput::default(),
            created_at_ms: 0,
            updated_at_ms: 0,
        },
        Preset {
            schema_version: PRESET_SCHEMA_VERSION,
            id: format!("{BUILTIN_PRESET_PREFIX}nvenc-fast"),
            name: "NVENC 快速编码（HEVC CQ 21 / p5）".to_string(),
            description: Some(
                "NVIDIA 硬件编码，速度优先于压缩效率；需要支持 HEVC 的 NVIDIA GPU。".to_string(),
            ),
            builtin: true,
            video: builtin_video("hevc_nvenc", 21, "p5", None),
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            output: PresetOutput::default(),
            created_at_ms: 0,
            updated_at_ms: 0,
        },
    ]
}

fn builtin_video(
    encoder_id: &str,
    quality_value: u32,
    preset: &str,
    pixel_format: Option<&str>,
) -> VideoConfig {
    let spec = encode::encoder_spec(encoder_id).expect("内置预设的编码器必须在规格表中");
    VideoConfig {
        encoder_id: spec.encoder_id.to_string(),
        quality_mode: encode::quality_mode_for(spec.encoder_id).expect("编码器规格完整"),
        quality_value,
        preset: preset.to_string(),
        pixel_format: pixel_format.map(str::to_string),
    }
}

// ---------- 校验 ----------

/// 预设 ID 必须是安全的文件名主干（拒绝路径分隔符与 `..`，
/// 防止删除 / 读取越出预设目录）。
pub fn is_safe_preset_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && !id.starts_with('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// 校验预设版本、ID、名称与编码参数范围。
///
/// 预设在 `burn` 字幕模式下不携带外部字幕文件路径（该路径与机器相关），
/// 因此这里只校验视频 / 音频参数，字幕文件在开始压制时校验。
pub fn validate_preset(preset: &Preset) -> Result<(), String> {
    if preset.schema_version != PRESET_SCHEMA_VERSION {
        return Err(format!(
            "预设版本不受支持：{}（当前支持版本 {}）",
            preset.schema_version, PRESET_SCHEMA_VERSION
        ));
    }
    if !is_safe_preset_id(&preset.id) {
        return Err(format!("预设 ID 无效：{}", preset.id));
    }
    validate_name(&preset.name)?;
    validate_description(preset.description.as_deref())?;
    if preset.subtitles.external_path.is_some() {
        return Err("预设不应包含外部字幕文件路径（该路径与机器相关），请移除后重试".to_string());
    }
    let mut config = preset.to_encode_config();
    if config.subtitles.mode == SubtitleMode::Burn {
        // 烧录文件在应用预设后选择；这里只校验其余参数。
        config.subtitles.mode = SubtitleMode::Drop;
    }
    encode::validate_config(&config)
}

/// 校验预设名称（非空、长度上限）。
pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("预设名称不能为空".to_string());
    }
    if trimmed.chars().count() > MAX_PRESET_NAME_CHARS {
        return Err(format!(
            "预设名称过长（最多 {MAX_PRESET_NAME_CHARS} 个字符）"
        ));
    }
    Ok(())
}

fn validate_description(description: Option<&str>) -> Result<(), String> {
    if let Some(text) = description {
        if text.chars().count() > MAX_PRESET_DESCRIPTION_CHARS {
            return Err(format!(
                "预设描述过长（最多 {MAX_PRESET_DESCRIPTION_CHARS} 个字符）"
            ));
        }
    }
    Ok(())
}

fn normalized_description(description: Option<String>) -> Result<Option<String>, String> {
    let trimmed = description
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    validate_description(trimmed.as_deref())?;
    Ok(trimmed)
}

impl Preset {
    /// 转换为编码配置（输出路径留空，由压制页补充）。
    pub fn to_encode_config(&self) -> EncodeConfig {
        EncodeConfig {
            schema_version: ENCODE_CONFIG_SCHEMA_VERSION,
            video: self.video.clone(),
            audio: self.audio,
            subtitles: self.subtitles.clone(),
            output: OutputConfig {
                path: PathBuf::new(),
                container: self.output.container,
                overwrite: self.output.overwrite,
            },
        }
    }
}

/// 生成新的预设 ID（时间戳 + 进程内计数器，仅含文件名安全字符）。
pub fn new_preset_id(now_ms: u64) -> String {
    let counter = PRESET_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("p-{now_ms:x}-{counter}")
}

/// 追加后缀并保证总长度不超过名称上限（超长部分先截断，
/// 避免"导入时因加后缀而报名称过长"这种误导性错误）。
fn with_suffix_within_limit(base: &str, suffix: &str) -> String {
    let suffix_len = suffix.chars().count();
    let keep = MAX_PRESET_NAME_CHARS.saturating_sub(suffix_len);
    let truncated: String = base.chars().take(keep).collect();
    format!("{}{}", truncated.trim_end(), suffix)
}

// ---------- 存储 ----------

/// 预设存储：内置预设在内存中，用户预设为 `dir/<id>.json`。
#[derive(Debug, Clone)]
pub struct PresetStore {
    dir: PathBuf,
}

impl PresetStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// 列出内置预设与用户预设；损坏的文件只记录错误，不阻止其他预设。
    pub fn list(&self) -> PresetList {
        let mut presets = builtin_presets();
        let mut errors = Vec::new();
        let mut user = self.load_user_presets(&mut errors);
        // 最近更新的用户预设排在前面，便于在界面上找到刚保存的项。
        user.sort_by(|a, b| {
            b.updated_at_ms
                .cmp(&a.updated_at_ms)
                .then_with(|| a.name.cmp(&b.name))
        });
        presets.extend(user);
        PresetList { presets, errors }
    }

    /// 按 ID 查找预设（内置优先）。
    pub fn find(&self, id: &str) -> Option<Preset> {
        if let Some(preset) = builtin_presets().into_iter().find(|p| p.id == id) {
            return Some(preset);
        }
        self.load_user(id).ok().flatten()
    }

    /// 由前端提交的输入创建用户预设。
    pub fn create(&self, input: PresetInput, now_ms: u64) -> Result<Preset, String> {
        let preset = Preset {
            schema_version: PRESET_SCHEMA_VERSION,
            id: new_preset_id(now_ms),
            name: input.name.trim().to_string(),
            description: normalized_description(input.description)?,
            builtin: false,
            video: input.video,
            audio: input.audio,
            // 预设不保存机器相关路径
            subtitles: SubtitleConfig {
                mode: input.subtitles.mode,
                external_path: None,
            },
            output: input.output,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        };
        validate_preset(&preset)?;
        self.ensure_name_available(&preset.name, None)?;
        self.write(&preset)?;
        Ok(preset)
    }

    /// 重命名用户预设（内置预设不可修改）。
    pub fn rename(&self, id: &str, name: &str, now_ms: u64) -> Result<Preset, String> {
        let mut preset = self.user_preset(id)?;
        preset.name = name.trim().to_string();
        preset.updated_at_ms = now_ms;
        validate_preset(&preset)?;
        self.ensure_name_available(&preset.name, Some(id))?;
        self.write(&preset)?;
        Ok(preset)
    }

    /// 删除用户预设（内置预设不可删除）。
    pub fn delete(&self, id: &str) -> Result<(), String> {
        self.user_preset(id)?;
        let path = self.preset_path(id)?;
        std::fs::remove_file(&path)
            .map_err(|e| format!("删除预设文件失败（{}）：{e}", path.display()))
    }

    /// 导入预设 JSON 文本：校验格式与范围、重新分配 ID，
    /// 名称重复时自动追加“（导入）”。
    pub fn import_text(&self, text: &str, now_ms: u64) -> Result<Preset, String> {
        let mut preset: Preset = serde_json::from_str(text)
            .map_err(|e| format!("预设 JSON 解析失败（字段缺失、类型错误或存在未知字段）：{e}"))?;
        // 导入的预设一律视为普通用户预设
        preset.builtin = false;
        preset.created_at_ms = now_ms;
        preset.updated_at_ms = now_ms;
        preset.id = new_preset_id(now_ms);
        if preset.subtitles.external_path.is_some() {
            return Err(
                "预设不应包含外部字幕文件路径（该路径与机器相关），请移除后重试".to_string(),
            );
        }
        preset.name = self.unique_import_name(preset.name.trim());
        validate_preset(&preset)?;
        self.write(&preset)?;
        Ok(preset)
    }

    /// 导出预设为可直接分享 / 再次导入的 JSON 文本。
    ///
    /// 内置预设导出为普通用户预设（`builtin: false`），路径字段同样被剥离。
    pub fn export_text(&self, id: &str) -> Result<String, String> {
        let mut preset = self.find(id).ok_or_else(|| format!("预设不存在：{id}"))?;
        preset.builtin = false;
        preset.subtitles.external_path = None;
        let mut text =
            serde_json::to_string_pretty(&preset).map_err(|e| format!("序列化预设失败：{e}"))?;
        text.push('\n');
        Ok(text)
    }

    // ---------- 内部 ----------

    fn preset_path(&self, id: &str) -> Result<PathBuf, String> {
        if !is_safe_preset_id(id) {
            return Err(format!("预设 ID 无效：{id}"));
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    fn user_preset(&self, id: &str) -> Result<Preset, String> {
        if id.starts_with(BUILTIN_PRESET_PREFIX) {
            return Err("内置预设不可修改或删除".to_string());
        }
        self.load_user(id)?
            .ok_or_else(|| format!("预设不存在：{id}"))
    }

    fn load_user(&self, id: &str) -> Result<Option<Preset>, String> {
        let path = self.preset_path(id)?;
        if !path.is_file() {
            return Ok(None);
        }
        load_preset_file(&path).map(Some)
    }

    fn load_user_presets(&self, errors: &mut Vec<String>) -> Vec<Preset> {
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
            Err(e) => {
                errors.push(format!("读取预设目录失败（{}）：{e}", self.dir.display()));
                return Vec::new();
            }
        };
        let mut presets = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let is_json = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("json"));
            if !is_json {
                continue;
            }
            match load_preset_file(&path) {
                Ok(preset) => presets.push(preset),
                Err(e) => errors.push(format!("{}：{e}", path.display())),
            }
        }
        presets
    }

    fn ensure_name_available(&self, name: &str, exclude_id: Option<&str>) -> Result<(), String> {
        let existing = self.list();
        let taken = existing.presets.iter().any(|preset| {
            Some(preset.id.as_str()) != exclude_id && preset.name.eq_ignore_ascii_case(name)
        });
        if taken {
            return Err(format!("已存在同名预设：{name}"));
        }
        Ok(())
    }

    fn unique_import_name(&self, base: &str) -> String {
        let existing = self.list();
        let taken = |candidate: &str| {
            existing
                .presets
                .iter()
                .any(|preset| preset.name.eq_ignore_ascii_case(candidate))
        };
        if !taken(base) {
            return base.to_string();
        }
        for index in 1..100 {
            let suffix = if index == 1 {
                "（导入）".to_string()
            } else {
                format!("（导入 {index}）")
            };
            let candidate = with_suffix_within_limit(base, &suffix);
            if !taken(&candidate) {
                return candidate;
            }
        }
        with_suffix_within_limit(base, "（导入）")
    }

    /// 原子写入：先写临时文件，再重命名覆盖（Windows 上同样是替换语义）。
    fn write(&self, preset: &Preset) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|e| format!("创建预设目录失败（{}）：{e}", self.dir.display()))?;
        let path = self.preset_path(&preset.id)?;
        let temp = self.dir.join(format!("{}.json.tmp", preset.id));
        let mut text =
            serde_json::to_string_pretty(preset).map_err(|e| format!("序列化预设失败：{e}"))?;
        text.push('\n');
        std::fs::write(&temp, text)
            .map_err(|e| format!("写入预设文件失败（{}）：{e}", temp.display()))?;
        std::fs::rename(&temp, &path).map_err(|e| {
            let _ = std::fs::remove_file(&temp);
            format!("保存预设文件失败（{}）：{e}", path.display())
        })
    }
}

/// 读取并校验单个预设文件；用户文件中的内置标记一律忽略。
fn load_preset_file(path: &Path) -> Result<Preset, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("读取预设文件失败（{}）：{e}", path.display()))?;
    let mut preset: Preset = serde_json::from_str(&text)
        .map_err(|e| format!("预设 JSON 解析失败（{}）：{e}", path.display()))?;
    preset.builtin = false;
    validate_preset(&preset).map_err(|e| format!("{}：{e}", path.display()))?;
    Ok(preset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::QualityMode;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-tmp")
            .join(format!("aozora-preset-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn store(tag: &str) -> (PresetStore, PathBuf) {
        let dir = temp_dir(tag);
        (PresetStore::new(dir.clone()), dir)
    }

    fn input(name: &str) -> PresetInput {
        PresetInput {
            name: name.to_string(),
            description: None,
            video: VideoConfig {
                encoder_id: "libx265".to_string(),
                quality_mode: QualityMode::Crf,
                quality_value: 16,
                preset: "slow".to_string(),
                pixel_format: Some("yuv420p10le".to_string()),
            },
            audio: AudioConfig::default(),
            subtitles: SubtitleConfig::default(),
            output: PresetOutput {
                container: crate::encode::Container::Mkv,
                overwrite: true,
            },
        }
    }

    #[test]
    fn builtin_presets_are_valid_and_marked_builtin() {
        let presets = builtin_presets();
        assert_eq!(presets.len(), 3);
        for preset in &presets {
            assert!(preset.builtin);
            assert!(preset.id.starts_with(BUILTIN_PRESET_PREFIX));
            validate_preset(preset).unwrap_or_else(|e| panic!("内置预设无效：{e}"));
        }
        assert!(presets.iter().any(|p| p.video.encoder_id == "libx264"));
        assert!(presets.iter().any(|p| p.video.encoder_id == "libx265"));
        assert!(presets.iter().any(|p| p.video.encoder_id == "hevc_nvenc"));
    }

    #[test]
    fn json_roundtrip_keeps_schema_and_has_no_machine_paths() {
        let (store, _dir) = store("roundtrip");
        let preset = store.create(input("我的预设"), 1_700_000_000_000).unwrap();
        let json = serde_json::to_string(&preset).unwrap();
        assert!(json.contains("\"schemaVersion\":1"), "json: {json}");
        assert!(json.contains("\"externalPath\":null"), "json: {json}");
        assert!(
            !json.contains("path\":"),
            "预设不应包含任何路径字段：{json}"
        );
        let restored: Preset = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, preset);
    }

    #[test]
    fn create_save_list_and_reload_from_disk() {
        let (store, dir) = store("persist");
        let created = store
            .create(input("持久化预设"), 1_700_000_000_000)
            .unwrap();
        assert!(dir.join(format!("{}.json", created.id)).is_file());

        // 模拟应用重启：用同一个目录新建存储实例
        let reopened = PresetStore::new(dir.clone());
        let list = reopened.list();
        assert_eq!(list.errors, Vec::<String>::new());
        assert_eq!(list.presets.len(), builtin_presets().len() + 1);
        assert_eq!(reopened.find(&created.id).unwrap(), created);
    }

    #[test]
    fn create_rejects_duplicate_name_and_empty_name() {
        let (store, _dir) = store("names");
        store.create(input("重复名称"), 1).unwrap();
        let err = store.create(input("重复名称"), 2).unwrap_err();
        assert!(err.contains("已存在同名预设"), "error: {err}");
        // 内置预设名称同样被占用
        let err = store
            .create(input("H.264 兼容（CRF 18 / slow）"), 3)
            .unwrap_err();
        assert!(err.contains("已存在同名预设"), "error: {err}");

        let err = store.create(input("   "), 4).unwrap_err();
        assert!(err.contains("名称"), "error: {err}");
    }

    #[test]
    fn create_rejects_invalid_video_config() {
        let (store, _dir) = store("invalid");
        let mut bad = input("错误参数");
        bad.video.encoder_id = "libvpx-vp8".to_string();
        let err = store.create(bad, 1).unwrap_err();
        assert!(err.contains("未知编码器"), "error: {err}");

        let mut bad = input("超范围");
        bad.video.quality_value = 99;
        let err = store.create(bad, 2).unwrap_err();
        assert!(err.contains("超出"), "error: {err}");
    }

    #[test]
    fn create_strips_machine_paths() {
        let (store, _dir) = store("strip");
        let mut draft = input("烧录预设");
        draft.subtitles.mode = crate::encode::SubtitleMode::Burn;
        draft.subtitles.external_path = Some(PathBuf::from(r"C:\影片\字幕.srt"));
        let preset = store.create(draft, 1).unwrap();
        assert_eq!(preset.subtitles.mode, crate::encode::SubtitleMode::Burn);
        assert_eq!(preset.subtitles.external_path, None);
        // 该预设可以直接应用到编码配置（烧录文件在压制页选择）
        let config = preset.to_encode_config();
        assert!(config.output.path.as_os_str().is_empty());
    }

    #[test]
    fn container_survives_preset_round_trip_and_defaults_to_mkv() {
        let (store, _dir) = store("container");
        // 显式选择 WebM
        let mut draft = input("WebM 预设");
        draft.output.container = crate::encode::Container::Webm;
        let preset = store.create(draft, 1).unwrap();
        assert_eq!(preset.output.container, crate::encode::Container::Webm);
        assert_eq!(
            preset.to_encode_config().output.container,
            crate::encode::Container::Webm
        );

        // 未指定容器（旧预设文件）时默认 MKV
        let legacy = input("旧预设");
        assert_eq!(legacy.output.container, crate::encode::Container::Mkv);
        let preset = store.create(legacy, 2).unwrap();
        let json = serde_json::to_string(&preset).unwrap();
        assert!(json.contains("\"container\":\"mkv\""), "json: {json}");
    }

    #[test]
    fn preset_json_without_container_field_is_accepted() {
        let (store, _dir) = store("legacy-container");
        let text = r#"{
            "schemaVersion":1,"id":"p-legacy","name":"容器前旧预设",
            "video":{"encoderId":"libx264","qualityMode":"crf","qualityValue":18,"preset":"slow","pixelFormat":null},
            "audio":{"mode":"copy"},
            "output":{"overwrite":false}}"#;
        let preset = store.import_text(text, 1).unwrap();
        assert_eq!(preset.output.container, crate::encode::Container::Mkv);
    }

    #[test]
    fn rename_and_delete_follow_builtin_rules() {
        let (store, dir) = store("rename");
        let created = store.create(input("旧名称"), 100).unwrap();
        let renamed = store.rename(&created.id, "新名称", 200).unwrap();
        assert_eq!(renamed.name, "新名称");
        assert_eq!(renamed.updated_at_ms, 200);
        assert_eq!(renamed.created_at_ms, 100);

        let err = store.rename(&created.id, "  ", 300).unwrap_err();
        assert!(err.contains("名称"), "error: {err}");

        let builtin_id = builtin_presets()[0].id.clone();
        let err = store.rename(&builtin_id, "改内置", 400).unwrap_err();
        assert!(err.contains("内置预设"), "error: {err}");
        let err = store.delete(&builtin_id).unwrap_err();
        assert!(err.contains("内置预设"), "error: {err}");

        store.delete(&created.id).unwrap();
        assert!(store.find(&created.id).is_none());
        assert!(!dir.join(format!("{}.json", created.id)).exists());
        let err = store.delete(&created.id).unwrap_err();
        assert!(err.contains("不存在"), "error: {err}");
    }

    #[test]
    fn delete_rejects_unsafe_ids() {
        let (store, _dir) = store("unsafe");
        let err = store.delete("../../settings").unwrap_err();
        assert!(err.contains("ID 无效"), "error: {err}");
        let err = store.delete("a/b").unwrap_err();
        assert!(err.contains("ID 无效"), "error: {err}");
    }

    #[test]
    fn import_assigns_new_id_and_unique_name() {
        let (store, _dir) = store("import");
        let source = store.create(input("导入源"), 1).unwrap();
        let text = store.export_text(&source.id).unwrap();

        let imported = store.import_text(&text, 500).unwrap();
        assert_ne!(imported.id, source.id);
        assert_eq!(imported.name, "导入源（导入）");
        assert!(!imported.builtin);
        assert_eq!(imported.created_at_ms, 500);
        assert_eq!(imported.video, source.video);
        assert!(imported.output.overwrite);

        let list = store.list();
        assert_eq!(list.errors, Vec::<String>::new());
        assert_eq!(list.presets.len(), builtin_presets().len() + 2);
    }

    #[test]
    fn import_rejects_unknown_fields_and_missing_required_fields() {
        let (store, _dir) = store("import-invalid");
        let err = store
            .import_text(r#"{"schemaVersion":1,"id":"x","name":"y","video":{},"audio":{"mode":"copy"},"extra":1}"#, 1)
            .unwrap_err();
        assert!(err.contains("解析失败"), "error: {err}");

        // 输出路径字段在预设格式中不存在，必须被拒绝而不是静默丢弃
        let err = store
            .import_text(
                r#"{"schemaVersion":1,"id":"x","name":"y",
                    "video":{"encoderId":"libx264","qualityMode":"crf","qualityValue":18,"preset":"slow","pixelFormat":null},
                    "audio":{"mode":"copy"},
                    "output":{"overwrite":false,"path":"D:/out.mkv"}}"#,
                1,
            )
            .unwrap_err();
        assert!(err.contains("解析失败"), "error: {err}");

        // 嵌套结构中的未知字段同样必须被拒绝（例如字段名拼写错误）
        let err = store
            .import_text(
                r#"{"schemaVersion":1,"id":"x","name":"y",
                    "video":{"encoderId":"libx264","qualityMode":"crf","qualityValue":18,"preset":"slow","pixelFormat":null,"crfTune":"film"},
                    "audio":{"mode":"copy"},
                    "output":{"overwrite":false}}"#,
                2,
            )
            .unwrap_err();
        assert!(err.contains("解析失败"), "error: {err}");

        let err = store
            .import_text(
                r#"{"schemaVersion":1,"id":"x","name":"y",
                    "video":{"encoderId":"libx264","qualityMode":"crf","qualityValue":18,"preset":"slow","pixelFormat":null},
                    "audio":{"mode":"copy","bitrate":"320k"},
                    "subtitles":{"mode":"drop","externalPath":null,"font":"Arial"},
                    "output":{"overwrite":false}}"#,
                3,
            )
            .unwrap_err();
        assert!(err.contains("解析失败"), "error: {err}");
    }

    #[test]
    fn import_truncates_name_before_adding_suffix() {
        let (store, _dir) = store("import-long-name");
        let long_name = "名".repeat(MAX_PRESET_NAME_CHARS);
        let mut draft = input(&long_name);
        draft.name = long_name.clone();
        store.create(draft, 1).unwrap();

        let mut source = store.create(input("导入源"), 2).unwrap();
        source.name = long_name;
        let text = serde_json::to_string(&source).unwrap();
        // 加"（导入）"后缀后仍须在名称长度上限内
        let imported = store.import_text(&text, 3).unwrap();
        assert!(
            imported.name.ends_with("（导入）"),
            "name: {}",
            imported.name
        );
        assert!(
            imported.name.chars().count() <= MAX_PRESET_NAME_CHARS,
            "name length: {}",
            imported.name.chars().count()
        );
    }

    #[test]
    fn import_rejects_unsupported_schema_version_and_external_subtitle_path() {
        let (store, _dir) = store("import-version");
        let mut preset = store.create(input("版本"), 1).unwrap();
        preset.schema_version = 99;
        let text = serde_json::to_string(&preset).unwrap();
        let err = store.import_text(&text, 2).unwrap_err();
        assert!(err.contains("版本不受支持"), "error: {err}");

        let mut preset = store.create(input("字幕"), 3).unwrap();
        preset.subtitles.external_path = Some(PathBuf::from("sub.srt"));
        let text = serde_json::to_string(&preset).unwrap();
        let err = store.import_text(&text, 4).unwrap_err();
        assert!(err.contains("外部字幕文件路径"), "error: {err}");
    }

    #[test]
    fn export_of_builtin_is_importable() {
        let (store, _dir) = store("export-builtin");
        let builtin = builtin_presets()[1].clone();
        let text = store.export_text(&builtin.id).unwrap();
        assert!(text.contains("\"builtin\": false"), "json: {text}");
        assert!(text.ends_with('\n'));
        let imported = store.import_text(&text, 10).unwrap();
        assert_eq!(imported.video, builtin.video);
        assert!(!imported.builtin);
    }

    #[test]
    fn corrupt_file_is_reported_without_breaking_other_presets() {
        let (store, dir) = store("corrupt");
        let good = store.create(input("正常预设"), 1).unwrap();
        std::fs::write(dir.join("broken.json"), "{ this is not json").unwrap();

        let list = store.list();
        assert_eq!(list.errors.len(), 1, "errors: {:?}", list.errors);
        assert!(list.errors[0].contains("broken.json"));
        assert!(list.presets.iter().any(|p| p.id == good.id));
    }

    #[test]
    fn user_file_cannot_shadow_builtin_flag() {
        let (store, dir) = store("flag");
        let created = store.create(input("标记"), 1).unwrap();
        let path = dir.join(format!("{}.json", created.id));
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(
            &path,
            text.replace("\"builtin\": false", "\"builtin\": true"),
        )
        .unwrap();
        let loaded = store.find(&created.id).unwrap();
        assert!(!loaded.builtin, "用户文件不应被当作内置预设");
    }

    #[test]
    fn ids_are_unique_and_filename_safe() {
        let first = new_preset_id(1);
        let second = new_preset_id(1);
        assert_ne!(first, second);
        assert!(is_safe_preset_id(&first));
        assert!(!is_safe_preset_id(".."));
        assert!(!is_safe_preset_id("a b"));
        assert!(!is_safe_preset_id(""));
    }
}
