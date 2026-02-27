use serde::{Deserialize, Serialize};

/// Top-level editing paradigm: Vim (modal) or Standard (modeless, CUA-style).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditingMode {
    #[default]
    Vim,
    Standard,
}
