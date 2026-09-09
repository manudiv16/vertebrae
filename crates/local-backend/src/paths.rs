//! Default database path resolution.
//!
//! Uses `vertebrae_installer::paths::data_dir()` for the per-OS root,
//! falling back to `VTB_DB_PATH` env override.

use std::path::PathBuf;

use vertebrae_installer::paths::data_dir;

use crate::error::{LocalError, LocalResult};

/// Default database filename.
const DB_FILENAME: &str = "vertebrae.db";

/// Resolve the default database path.
///
/// Resolution order:
/// 1. `VTB_DB_PATH` environment variable (if set and non-empty)
/// 2. `data_dir()/vertebrae.db` (macOS: `~/Library/Application Support/Vertebrae/vertebrae.db`,
///    Linux: `~/.local/share/vertebrae/vertebrae.db`)
pub fn default_db_path() -> LocalResult<PathBuf> {
    if let Ok(path) = std::env::var("VTB_DB_PATH") {
        if !path.is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    let dir = data_dir().map_err(|e| {
        LocalError::Database(format!("cannot resolve data directory: {}", e))
    })?;
    Ok(dir.join(DB_FILENAME))
}
