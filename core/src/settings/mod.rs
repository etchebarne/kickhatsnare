use std::{collections::HashMap, path::Path};

use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::{
    CoreError,
    storage::{Database, database_error},
};

pub const AUDIO_BUFFER_SIZE_ID: &str = "audio.bufferSize";
pub const DEFAULT_AUDIO_BUFFER_SIZE: u32 = 512;

const BUFFER_SIZE_OPTIONS: &[u32] = &[128, 256, 512, 1_024, 2_048, 4_096];
const CATEGORIES: &[CategoryDefinition] = &[CategoryDefinition {
    id: "audio",
    label: "Audio",
    description: "Audio output and playback performance.",
}];
const SETTINGS: &[SettingDefinition] = &[SettingDefinition {
    id: AUDIO_BUFFER_SIZE_ID,
    category_id: "audio",
    label: "Buffer size",
    description: "Lower values reduce latency but may cause playback interruptions. Changes apply the next time playback starts.",
    default_value: DEFAULT_AUDIO_BUFFER_SIZE,
    unit: "samples",
    options: BUFFER_SIZE_OPTIONS,
}];

#[derive(Debug)]
pub struct Settings {
    database: Database,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSnapshot {
    pub categories: Vec<SettingCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingCategory {
    pub id: String,
    pub label: String,
    pub description: String,
    pub settings: Vec<Setting>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setting {
    IntegerSelect {
        id: String,
        label: String,
        description: String,
        value: u32,
        default_value: u32,
        unit: String,
        options: Vec<IntegerSettingOption>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerSettingOption {
    pub value: u32,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum SettingValue {
    Integer(u32),
}

struct CategoryDefinition {
    id: &'static str,
    label: &'static str,
    description: &'static str,
}

struct SettingDefinition {
    id: &'static str,
    category_id: &'static str,
    label: &'static str,
    description: &'static str,
    default_value: u32,
    unit: &'static str,
    options: &'static [u32],
}

impl Settings {
    /// Opens the settings registry and its persisted overrides.
    ///
    /// # Errors
    ///
    /// Returns an error if the application database cannot be initialized.
    pub fn open(data_directory: impl AsRef<Path>) -> Result<Self, CoreError> {
        Ok(Self {
            database: Database::open(data_directory)?,
        })
    }

    pub(crate) fn in_memory() -> Result<Self, CoreError> {
        Ok(Self {
            database: Database::in_memory()?,
        })
    }

    /// Returns the ordered backend registry with each setting's effective value.
    ///
    /// # Errors
    ///
    /// Returns an error if persisted values cannot be read or decoded.
    pub fn snapshot(&self) -> Result<SettingsSnapshot, CoreError> {
        let values = self.persisted_values()?;
        let categories = CATEGORIES
            .iter()
            .map(|category| {
                let settings = SETTINGS
                    .iter()
                    .filter(|setting| setting.category_id == category.id)
                    .map(|setting| setting.snapshot(values.get(setting.id).copied()))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(SettingCategory {
                    id: category.id.to_owned(),
                    label: category.label.to_owned(),
                    description: category.description.to_owned(),
                    settings,
                })
            })
            .collect::<Result<Vec<_>, CoreError>>()?;
        Ok(SettingsSnapshot { categories })
    }

    /// Validates and persists a setting override.
    ///
    /// # Errors
    ///
    /// Returns an error if the setting is unknown, the value is invalid, or it cannot be stored.
    pub fn set(&mut self, id: &str, value: SettingValue) -> Result<SettingsSnapshot, CoreError> {
        let definition = SETTINGS
            .iter()
            .find(|setting| setting.id == id)
            .ok_or_else(|| CoreError::new(format!("unknown setting: {id}")))?;
        definition.validate(value)?;
        let value_json = serde_json::to_string(&value)
            .map_err(|error| CoreError::new(format!("failed to encode setting {id}: {error}")))?;
        self.database
            .connection()
            .execute(
                "INSERT INTO settings (id, value_json) VALUES (?1, ?2) \
                 ON CONFLICT(id) DO UPDATE SET value_json = excluded.value_json",
                params![id, value_json],
            )
            .map_err(database_error("persist setting"))?;
        self.snapshot()
    }

    /// Returns the effective audio buffer size.
    ///
    /// # Errors
    ///
    /// Returns an error if the persisted value cannot be read or decoded.
    pub fn audio_buffer_size(&self) -> Result<u32, CoreError> {
        self.value(AUDIO_BUFFER_SIZE_ID).map(|value| match value {
            SettingValue::Integer(value) => value,
        })
    }

    fn value(&self, id: &str) -> Result<SettingValue, CoreError> {
        let definition = SETTINGS
            .iter()
            .find(|setting| setting.id == id)
            .ok_or_else(|| CoreError::new(format!("unknown setting: {id}")))?;
        let value_json = self
            .database
            .connection()
            .query_row(
                "SELECT value_json FROM settings WHERE id = ?1",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(database_error("read setting"))?;
        let value = value_json.map_or(
            Ok(SettingValue::Integer(definition.default_value)),
            |value| decode_value(id, &value),
        )?;
        definition.validate(value)?;
        Ok(value)
    }

    fn persisted_values(&self) -> Result<HashMap<String, SettingValue>, CoreError> {
        let mut statement = self
            .database
            .connection()
            .prepare("SELECT id, value_json FROM settings")
            .map_err(database_error("prepare settings query"))?;
        let records = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(database_error("query settings"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database_error("read setting record"))?;
        records
            .into_iter()
            .map(|(id, value)| decode_value(&id, &value).map(|value| (id, value)))
            .collect()
    }
}

impl SettingDefinition {
    fn snapshot(&self, value: Option<SettingValue>) -> Result<Setting, CoreError> {
        let value = value.unwrap_or(SettingValue::Integer(self.default_value));
        self.validate(value)?;
        let SettingValue::Integer(value) = value;
        Ok(Setting::IntegerSelect {
            id: self.id.to_owned(),
            label: self.label.to_owned(),
            description: self.description.to_owned(),
            value,
            default_value: self.default_value,
            unit: self.unit.to_owned(),
            options: self
                .options
                .iter()
                .map(|value| IntegerSettingOption {
                    value: *value,
                    label: value.to_string(),
                })
                .collect(),
        })
    }

    fn validate(&self, value: SettingValue) -> Result<(), CoreError> {
        let SettingValue::Integer(value) = value;
        if !self.options.contains(&value) {
            return Err(CoreError::new(format!(
                "invalid value for setting {}: {value}",
                self.id
            )));
        }
        Ok(())
    }
}

fn decode_value(id: &str, value: &str) -> Result<SettingValue, CoreError> {
    serde_json::from_str(value)
        .map_err(|error| CoreError::new(format!("failed to decode setting {id}: {error}")))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{AUDIO_BUFFER_SIZE_ID, DEFAULT_AUDIO_BUFFER_SIZE, Setting, SettingValue, Settings};

    #[test]
    fn registry_returns_the_default_buffer_size() {
        let settings = Settings::in_memory().expect("settings should initialize");
        let snapshot = settings.snapshot().expect("settings should load");

        assert_eq!(snapshot.categories.len(), 1);
        assert_eq!(snapshot.categories[0].id, "audio");
        assert!(matches!(
            snapshot.categories[0].settings[0],
            Setting::IntegerSelect {
                value: DEFAULT_AUDIO_BUFFER_SIZE,
                default_value: DEFAULT_AUDIO_BUFFER_SIZE,
                ..
            }
        ));
    }

    #[test]
    fn buffer_size_override_persists() {
        let data = tempdir().expect("data directory should be created");
        let mut settings = Settings::open(data.path()).expect("settings should open");
        settings
            .set(AUDIO_BUFFER_SIZE_ID, SettingValue::Integer(1_024))
            .expect("buffer size should update");
        drop(settings);

        let settings = Settings::open(data.path()).expect("settings should reopen");
        assert_eq!(settings.audio_buffer_size().unwrap(), 1_024);
    }

    #[test]
    fn invalid_values_do_not_replace_the_current_value() {
        let mut settings = Settings::in_memory().expect("settings should initialize");

        assert!(
            settings
                .set(AUDIO_BUFFER_SIZE_ID, SettingValue::Integer(42))
                .is_err()
        );
        assert_eq!(
            settings.audio_buffer_size().unwrap(),
            DEFAULT_AUDIO_BUFFER_SIZE
        );
    }
}
