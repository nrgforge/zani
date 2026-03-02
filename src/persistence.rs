use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::buffer::Buffer;
use crate::draft_name;

/// File identity and autosave state.
pub struct Persistence {
    pub file_path: Option<PathBuf>,
    pub is_scratch: bool,
    pub save_error: Option<String>,
    pub last_save: Option<Instant>,
    pub autosave_interval: Duration,
}

impl Persistence {
    pub fn new() -> Self {
        Self {
            file_path: None,
            is_scratch: false,
            save_error: None,
            last_save: None,
            autosave_interval: Duration::from_secs(3),
        }
    }

    /// Check if autosave should trigger (dirty + enough time elapsed).
    pub fn should_autosave(&self, dirty: bool) -> bool {
        if !dirty {
            return false;
        }
        match self.last_save {
            Some(last) => last.elapsed() >= self.autosave_interval,
            None => true,
        }
    }

    /// Perform autosave if a file path is set. Returns true if saved.
    pub fn autosave(&mut self, buffer: &Buffer, dirty: &mut bool) -> bool {
        if !*dirty || self.file_path.is_none() {
            return false;
        }

        if let Some(path) = &self.file_path {
            let content = buffer.to_string();
            match std::fs::write(path, &content) {
                Ok(()) => {
                    *dirty = false;
                    self.last_save = Some(Instant::now());
                    self.save_error = None;
                    return true;
                }
                Err(e) => {
                    self.save_error = Some(e.to_string());
                }
            }
        }
        false
    }

    /// Set up with an existing file.
    pub fn with_file(&mut self, path: PathBuf, buffer: &mut Buffer, content: &str) {
        *buffer = Buffer::from_text(content);
        self.file_path = Some(path);
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
