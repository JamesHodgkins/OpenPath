//! Native project container (§3.3) — Phase 0 spike (P0-08).
//!
//! A `.rproj` file is a magic header + BLAKE3-checksummed CBOR payload. This spike
//! implements `encode`/`decode`, atomic save with a `.bak`, and a migration dispatch with
//! one demonstrated fixture. Zstd payload compression is flagged (container `flags` bit 0)
//! but not yet applied.
//!
//! Container layout (little-endian):
//! ```text
//! magic "RPROJ\0" (6) · version u16 · flags u16 · schema_version u32 ·
//! created i64 · modified i64 · app_version_len u16 · app_version bytes ·
//! payload_len u64 · checksum [u8;32] (BLAKE3 of payload) · payload (CBOR)
//! ```

#![forbid(unsafe_code)]

use std::io::{Cursor, Read, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use domain::Project;

const MAGIC: &[u8; 6] = b"RPROJ\0";
const CONTAINER_VERSION: u16 = 1;
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CBOR error: {0}")]
    Cbor(String),
    #[error("not a native project file (bad magic)")]
    BadMagic,
    #[error("unsupported container version {0}")]
    UnsupportedVersion(u16),
    #[error("unsupported schema version {0}")]
    UnsupportedSchema(u32),
    #[error("payload checksum mismatch (file corrupt)")]
    BadChecksum,
}

/// Serialize `project` into a checksummed native container.
pub fn encode(project: &Project) -> Result<Vec<u8>, StoreError> {
    let mut payload = Vec::new();
    ciborium::into_writer(project, &mut payload).map_err(|e| StoreError::Cbor(e.to_string()))?;
    let checksum = blake3::hash(&payload);
    let now = now_unix();

    let mut out = Vec::new();
    out.write_all(MAGIC)?;
    out.write_u16::<LittleEndian>(CONTAINER_VERSION)?;
    out.write_u16::<LittleEndian>(0)?; // flags: bit 0 = zstd (unused)
    out.write_u32::<LittleEndian>(SCHEMA_VERSION)?;
    out.write_i64::<LittleEndian>(now)?; // created
    out.write_i64::<LittleEndian>(now)?; // modified
    let app = env!("CARGO_PKG_VERSION").as_bytes();
    out.write_u16::<LittleEndian>(app.len() as u16)?;
    out.write_all(app)?;
    out.write_u64::<LittleEndian>(payload.len() as u64)?;
    out.write_all(checksum.as_bytes())?;
    out.write_all(&payload)?;
    Ok(out)
}

/// Deserialize a native container into a `Project`, migrating the payload if needed.
pub fn decode(bytes: &[u8]) -> Result<Project, StoreError> {
    let (schema, payload) = parse_container(bytes)?;
    let payload = if schema == SCHEMA_VERSION {
        payload
    } else {
        migrate(schema, SCHEMA_VERSION, &payload)?
    };
    ciborium::from_reader(payload.as_slice()).map_err(|e| StoreError::Cbor(e.to_string()))
}

/// Verify the header and checksum, returning the schema version and verified payload.
fn parse_container(bytes: &[u8]) -> Result<(u32, Vec<u8>), StoreError> {
    let mut r = Cursor::new(bytes);
    let mut magic = [0u8; 6];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(StoreError::BadMagic);
    }
    let version = r.read_u16::<LittleEndian>()?;
    if version != CONTAINER_VERSION {
        return Err(StoreError::UnsupportedVersion(version));
    }
    let _flags = r.read_u16::<LittleEndian>()?;
    let schema = r.read_u32::<LittleEndian>()?;
    let _created = r.read_i64::<LittleEndian>()?;
    let _modified = r.read_i64::<LittleEndian>()?;
    let app_len = r.read_u16::<LittleEndian>()?;
    let mut app = vec![0u8; app_len as usize];
    r.read_exact(&mut app)?;
    let payload_len = r.read_u64::<LittleEndian>()?;
    let mut checksum = [0u8; 32];
    r.read_exact(&mut checksum)?;
    let mut payload = vec![0u8; payload_len as usize];
    r.read_exact(&mut payload)?;
    if blake3::hash(&payload).as_bytes() != &checksum {
        return Err(StoreError::BadChecksum);
    }
    Ok((schema, payload))
}

/// Write `project` atomically: write `.tmp`, fsync, rotate the previous file to `.bak`,
/// then rename into place (§3.3).
pub fn save_atomic(path: &Path, project: &Project) -> Result<(), StoreError> {
    let bytes = encode(project)?;
    let tmp = append_suffix(path, ".tmp");
    let mut f = std::fs::File::create(&tmp)?;
    f.write_all(&bytes)?;
    f.sync_all()?;
    drop(f);
    if path.exists() {
        let bak = append_suffix(path, ".bak");
        let _ = std::fs::remove_file(&bak);
        let _ = std::fs::rename(path, &bak);
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Read and decode a native project file.
pub fn load(path: &Path) -> Result<Project, StoreError> {
    decode(&std::fs::read(path)?)
}

/// Migrate a payload between schema versions. The dispatch table grows one entry per
/// historical version pair; unknown versions fail loudly, never panic (§3.3).
pub fn migrate(from: u32, to: u32, payload: &[u8]) -> Result<Vec<u8>, StoreError> {
    if from == to {
        return Ok(payload.to_vec());
    }
    match (from, to) {
        (0, 1) => migrate_0_to_1(payload),
        _ => Err(StoreError::UnsupportedSchema(from)),
    }
}

/// Demo migration (0 → 1): the field `Name` was renamed to `name`. A contrived fixture
/// proving the CBOR → generic value → transform → CBOR mechanism.
fn migrate_0_to_1(payload: &[u8]) -> Result<Vec<u8>, StoreError> {
    let mut value: serde_json::Value =
        ciborium::from_reader(payload).map_err(|e| StoreError::Cbor(e.to_string()))?;
    if let Some(obj) = value.as_object_mut() {
        if let Some(name) = obj.remove("Name") {
            obj.insert("name".into(), name);
        }
    }
    let mut out = Vec::new();
    ciborium::into_writer(&value, &mut out).map_err(|e| StoreError::Cbor(e.to_string()))?;
    Ok(out)
}

fn append_suffix(path: &Path, suffix: &str) -> std::path::PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(suffix);
    std::path::PathBuf::from(s)
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::{
        Calendar, ConstraintType, Dependency, Duration, Lag, RelationType, Task, TaskUid,
    };

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, mi, 0)
            .unwrap()
    }

    fn sample_project() -> Project {
        let t = Task {
            uid: Some(TaskUid(1)),
            id: Some(1),
            name: Some("Task A".into()),
            duration: Some(Duration::from_minutes(480)),
            start: Some(dt(2026, 11, 9, 8, 0)),
            finish: Some(dt(2026, 11, 9, 17, 0)),
            constraint_type: Some(ConstraintType::AsSoonAsPossible),
            ..Default::default()
        };
        Project {
            name: Some("Spike".into()),
            start_date: Some(dt(2026, 11, 9, 8, 0)),
            finish_date: Some(dt(2026, 11, 9, 17, 0)),
            minutes_per_day: Some(480),
            tasks: vec![t],
            calendars: vec![Calendar::standard("Standard")],
            dependencies: vec![Dependency {
                predecessor: TaskUid(1),
                successor: TaskUid(2),
                relation_type: RelationType::FinishToStart,
                lag: Lag::from_minutes(0),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn roundtrip() {
        let p = sample_project();
        let bytes = encode(&p).unwrap();
        let q = decode(&bytes).unwrap();
        assert_eq!(p, q);
    }

    #[test]
    fn bad_magic_is_rejected() {
        assert!(matches!(decode(b"not a project"), Err(StoreError::BadMagic)));
    }

    #[test]
    fn corrupt_payload_is_detected() {
        let mut bytes = encode(&sample_project()).unwrap();
        // Flip a byte near the end (inside the payload).
        let n = bytes.len();
        bytes[n - 1] ^= 0xFF;
        assert!(matches!(decode(&bytes), Err(StoreError::BadChecksum)));
    }

    #[test]
    fn atomic_save_then_load() {
        let dir = std::env::temp_dir();
        let path = dir.join("openpath_spike_roundtrip.rproj");
        let p = sample_project();
        save_atomic(&path, &p).unwrap();
        let q = load(&path).unwrap();
        assert_eq!(p, q);
        // Clean up the temp files.
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(append_suffix(&path, ".tmp"));
        let _ = std::fs::remove_file(append_suffix(&path, ".bak"));
    }

    #[test]
    fn migration_demo_renames_field() {
        // Build a synthetic v0 payload: a CBOR object with the old field name.
        let v0 = serde_json::json!({ "Name": "old-name" });
        let mut payload = Vec::new();
        ciborium::into_writer(&v0, &mut payload).unwrap();

        let migrated = migrate(0, 1, &payload).unwrap();
        let value: serde_json::Value = ciborium::from_reader(migrated.as_slice()).unwrap();
        assert_eq!(value["name"], "old-name");
        assert!(value.get("Name").is_none());
    }

    #[test]
    fn unknown_schema_is_rejected_not_panicked() {
        let mut bytes = encode(&sample_project()).unwrap();
        // Overwrite the schema_version (offset 10) with an unknown value.
        bytes[10] = 0xFF;
        bytes[11] = 0xFF;
        bytes[12] = 0xFF;
        bytes[13] = 0xFF;
        // Re-checksum is irrelevant: the schema is checked before the payload is parsed,
        // but after checksum. The checksum still matches (we only changed the header), so
        // we expect the unsupported-schema error.
        assert!(matches!(decode(&bytes), Err(StoreError::UnsupportedSchema(_))));
    }
}
