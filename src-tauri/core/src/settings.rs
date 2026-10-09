use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

/// 应用设置（持久化到系统应用配置目录下的 `settings.json`，见 PLAN §8）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub schema_version: u32,
    /// 用户手动指定的 FFmpeg / FFprobe 所在文件夹（仅 Windows 使用）。
    pub custom_tool_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            custom_tool_dir: None,
        }
    }
}

/// 校验用户指定的工具目录：必须存在且为文件夹。
pub fn validate_tool_dir(dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("路径不存在或不是文件夹：{}", dir.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn defaults_use_current_schema_version() {
        let settings = Settings::default();
        assert_eq!(settings.schema_version, SETTINGS_SCHEMA_VERSION);
        assert_eq!(settings.custom_tool_dir, None);
    }

    #[test]
    fn roundtrip_via_json_keeps_camel_case_fields() {
        let settings = Settings {
            schema_version: SETTINGS_SCHEMA_VERSION,
            custom_tool_dir: Some(PathBuf::from(r"C:\tools\ffmpeg\bin")),
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("schemaVersion"));
        assert!(json.contains("customToolDir"));
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, settings);
    }

    #[test]
    fn validate_tool_dir_accepts_directory_and_rejects_missing() {
        // 放在 crate 目录内的 test-tmp（位于 workspace 内），避免依赖受限的系统 %TEMP%。
        let temp = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-tmp")
            .join(format!("aozora-settings-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        assert!(validate_tool_dir(&temp).is_ok());
        let missing = temp.join("missing");
        assert!(validate_tool_dir(&missing).is_err());
        fs::remove_dir_all(&temp).ok();
    }
}
