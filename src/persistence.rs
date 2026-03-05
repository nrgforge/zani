use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use crate::buffer::Buffer;
use crate::draft_name;

/// File identity and autosave state.
pub struct Persistence {
    pub file_path: Option<PathBuf>,
    pub is_scratch: bool,
    pub save_error: Option<String>,
    pub load_error: Option<String>,
    pub last_save: Option<Instant>,
    pub autosave_interval: Duration,
    last_known_mtime: Option<SystemTime>,
}

impl Persistence {
    pub fn new() -> Self {
        Self {
            file_path: None,
            is_scratch: false,
            save_error: None,
            load_error: None,
            last_save: None,
            autosave_interval: Duration::from_secs(3),
            last_known_mtime: None,
        }
    }

    /// Check if autosave should trigger (dirty + enough time elapsed).
    /// Returns false when load_error is set to prevent overwriting files
    /// that failed to load correctly.
    pub fn should_autosave(&self, dirty: bool) -> bool {
        if !dirty || self.load_error.is_some() {
            return false;
        }
        match self.last_save {
            Some(last) => last.elapsed() >= self.autosave_interval,
            None => true,
        }
    }

    /// Perform autosave if a file path is set. Returns true if saved.
    pub fn autosave(&mut self, buffer: &Buffer, dirty: &mut bool) -> bool {
        if !*dirty || self.file_path.is_none() || self.load_error.is_some() {
            return false;
        }

        if let Some(path) = &self.file_path {
            let content = buffer.to_string();
            match std::fs::write(path, &content) {
                Ok(()) => {
                    *dirty = false;
                    self.last_save = Some(Instant::now());
                    self.save_error = None;
                    self.record_mtime();
                    return true;
                }
                Err(e) => {
                    self.save_error = Some(e.to_string());
                }
            }
        }
        false
    }

    /// Stat the file on disk and return its modified time.
    pub fn current_mtime(&self) -> Option<SystemTime> {
        self.file_path
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.modified().ok())
    }

    /// Snapshot the file's current mtime as our baseline.
    pub fn record_mtime(&mut self) {
        self.last_known_mtime = self.current_mtime();
    }

    /// True when the file on disk has a different mtime than our baseline.
    /// Returns false if there is no file path or no baseline recorded yet.
    pub fn mtime_changed(&self) -> bool {
        let Some(baseline) = self.last_known_mtime else {
            return false;
        };
        match self.current_mtime() {
            Some(current) => current != baseline,
            None => false,
        }
    }

    /// Set up with an existing file.
    pub fn with_file(&mut self, path: PathBuf, buffer: &mut Buffer, content: &str) {
        *buffer = Buffer::from_text(content);
        self.file_path = Some(path);
        self.record_mtime();
    }

    /// Set up as a scratch buffer with a generated name.
    pub fn with_scratch_name(&mut self) {
        self.file_path = Some(PathBuf::from(draft_name::generate()));
        self.is_scratch = true;
    }
}

impl Default for Persistence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_dirty_returns_false() {
        let p = Persistence::new();
        assert!(!p.should_autosave(false));
    }

    #[test]
    fn no_previous_save_returns_true() {
        let p = Persistence::new();
        assert!(p.should_autosave(true));
    }

    #[test]
    fn recent_save_returns_false() {
        let mut p = Persistence::new();
        p.last_save = Some(Instant::now());
        assert!(!p.should_autosave(true));
    }

    #[test]
    fn elapsed_past_interval_returns_true() {
        let mut p = Persistence::new();
        p.last_save = Some(Instant::now() - Duration::from_secs(5));
        assert!(p.should_autosave(true));
    }

    #[test]
    fn load_error_suppresses_autosave() {
        let mut p = Persistence::new();
        p.load_error = Some("corrupt file".into());
        assert!(!p.should_autosave(true));
    }

    #[test]
    fn autosave_refuses_when_load_error_set() {
        let dir = std::env::temp_dir().join("zani_test_load_error");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.md");
        std::fs::write(&path, "original").unwrap();

        let mut p = Persistence::new();
        p.file_path = Some(path.clone());
        p.load_error = Some("corrupt".into());

        let buffer = Buffer::from_text("overwritten");
        let mut dirty = true;
        assert!(!p.autosave(&buffer, &mut dirty));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mtime_tracks_after_save() {
        let dir = std::env::temp_dir().join("zani_test_mtime");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("mtime_test.md");
        std::fs::write(&path, "initial").unwrap();

        let mut p = Persistence::new();
        let mut buffer = Buffer::from_text("initial");
        p.with_file(path.clone(), &mut buffer, "initial");

        // After with_file, mtime is recorded — no change detected
        assert!(!p.mtime_changed());

        // Autosave re-records mtime
        let mut dirty = true;
        let buffer = Buffer::from_text("updated");
        p.autosave(&buffer, &mut dirty);
        assert!(!p.mtime_changed());

        // External write changes mtime
        std::thread::sleep(std::time::Duration::from_millis(50));
        std::fs::write(&path, "external").unwrap();
        assert!(p.mtime_changed());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mtime_no_path_returns_false() {
        let p = Persistence::new();
        assert!(!p.mtime_changed());
    }
}
