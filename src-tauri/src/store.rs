use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::Model;

pub const SCHEMA_VERSION: u32 = 4;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    schema_version: u32,
    model: Model,
}

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn new(dir: &Path) -> Self {
        Self {
            path: dir.join("state.json"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read the model from disk. Any problem — missing file, corrupt JSON, a schema
    /// version we do not understand — falls back to a freshly seeded model rather than
    /// failing to launch. An unrecognised version is preserved as `state.json.bak` first
    /// so a downgrade never silently destroys the user's data.
    pub fn load(&self) -> Model {
        let Ok(raw) = fs::read_to_string(&self.path) else {
            return fresh();
        };

        let version = serde_json::from_str::<serde_json::Value>(&raw)
            .ok()
            .and_then(|v| v.get("schemaVersion").and_then(|v| v.as_u64()));

        match version {
            Some(v) if v == SCHEMA_VERSION as u64 => serde_json::from_str::<Envelope>(&raw)
                .map(|e| e.model)
                .unwrap_or_else(|_| fresh()),
            Some(_) => {
                let _ = fs::write(self.path.with_extension("json.bak"), &raw);
                fresh()
            }
            None => fresh(),
        }
    }

    /// Write the model atomically: a sibling temp file, then a rename, so a crash
    /// mid-write can never leave a half-written state.json.
    pub fn save(&self, model: &Model) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let envelope = Envelope {
            schema_version: SCHEMA_VERSION,
            model: model.clone(),
        };
        let json = serde_json::to_string_pretty(&envelope)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

/// A first launch: no tasks (the sidebar has an empty state for that), the
/// four built-in reminders. Demo rows were seeded here once; they read as the
/// user's own data and, before the list was editable, could not be removed.
fn fresh() -> Model {
    let mut model = Model::default();
    model.seed_reminders();
    model
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("momo-store-test-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn load_seeds_a_fresh_model_when_no_file_exists() {
        let store = Store::new(&temp_dir("fresh"));
        let model = store.load();
        assert!(model.tasks.is_empty());
        assert_eq!(model.timer.active_task, None);
        assert_eq!(model.reminders.len(), 4);
        assert_eq!(model.settings.focus_secs, 1500);
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = temp_dir("roundtrip");
        let store = Store::new(&dir);
        let mut model = store.load();
        model.timer.remaining_secs = 42;
        model.add_task("新任务".into(), 2);
        store.save(&model).expect("save");

        let back = Store::new(&dir).load();
        assert_eq!(back.timer.remaining_secs, 42);
        assert_eq!(back.tasks.len(), 1);
        assert_eq!(back.tasks[0].name, "新任务");
    }

    #[test]
    fn mini_mode_and_its_placement_survive_a_round_trip() {
        let dir = temp_dir("mini");
        let store = Store::new(&dir);
        let mut model = store.load();
        model.mini_enabled = true;
        model.mini_placement = Some(crate::core::desk::Placement { x: 1180.0, y: 0.0 });
        store.save(&model).expect("save");

        let back = Store::new(&dir).load();
        assert!(back.mini_enabled);
        assert_eq!(
            back.mini_placement,
            Some(crate::core::desk::Placement { x: 1180.0, y: 0.0 })
        );
    }

    #[test]
    fn a_save_written_before_mini_mode_existed_still_loads() {
        let dir = temp_dir("mini-missing");
        let store = Store::new(&dir);
        let mut model = store.load();
        model.add_task("留下来的".into(), 1);
        store.save(&model).expect("save");

        // Strip the two keys the way a state.json from the previous build looks.
        let raw = fs::read_to_string(dir.join("state.json")).expect("read");
        let mut value: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        let obj = value["model"].as_object_mut().expect("model object");
        obj.remove("miniEnabled");
        obj.remove("miniPlacement");
        fs::write(dir.join("state.json"), value.to_string()).expect("write");

        let back = Store::new(&dir).load();
        assert!(!back.mini_enabled);
        assert_eq!(back.mini_placement, None);
        assert_eq!(back.tasks.len(), 1); // and it is the real model, not a fresh fallback
    }

    #[test]
    fn save_writes_the_schema_version() {
        let dir = temp_dir("version");
        let store = Store::new(&dir);
        store.save(&store.load()).expect("save");
        let raw = fs::read_to_string(dir.join("state.json")).expect("read");
        let value: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(value["schemaVersion"], SCHEMA_VERSION);
    }

    #[test]
    fn an_unknown_schema_version_is_backed_up_not_overwritten() {
        let dir = temp_dir("migrate");
        fs::write(
            dir.join("state.json"),
            r#"{"schemaVersion":999,"model":{"whatever":true}}"#,
        )
        .expect("write");

        let model = Store::new(&dir).load();
        assert!(model.tasks.is_empty()); // fell back to a fresh model
        assert_eq!(model.reminders.len(), 4);
        assert!(dir.join("state.json.bak").exists());
        let backup = fs::read_to_string(dir.join("state.json.bak")).expect("read backup");
        assert!(backup.contains("999"));
    }

    #[test]
    fn a_corrupt_file_falls_back_without_panicking() {
        let dir = temp_dir("corrupt");
        fs::write(dir.join("state.json"), "{ not json at all").expect("write");
        let model = Store::new(&dir).load();
        assert!(model.tasks.is_empty());
        assert_eq!(model.reminders.len(), 4);
    }

    #[test]
    fn save_leaves_no_temp_file_behind() {
        let dir = temp_dir("atomic");
        let store = Store::new(&dir);
        store.save(&store.load()).expect("save");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "left behind {leftovers:?}");
    }
}
