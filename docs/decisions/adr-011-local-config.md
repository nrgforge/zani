# ADR-011: Local Config for Per-Project Settings

**Status:** Proposed

## Context

The mood-instrument framing (Essay 002) implies that Palette should follow creative context. A writer associating a neon palette with a cyberpunk project and a warm muted palette with journal entries should not switch manually each time. The domain model defines Local Config (`.zani.toml`) and Config Resolution (local → global → default).

Currently, configuration is global only — `~/.config/zani/config.toml`. There is no mechanism for per-project settings.

## Decision

Add local config file support via `.zani.toml`. Config Resolution walks up from the opened file's parent directory, checking each ancestor for `.zani.toml`, until one is found or the filesystem root is reached. Then falls back to global config (`~/.config/zani/config.toml`), then built-in defaults.

The same `Config` fields are supported in `.zani.toml`. A local config need only specify fields it wants to override — unspecified fields inherit from global config, then defaults. Initially the primary use case is `palette`, but all fields (focus_mode, column_width, editing_mode, scroll_mode) are available for override.

When the resolved Palette differs from the currently active one (e.g., opening a file in a project with a different Local Config), the existing crossfade animation handles the transition.

> **Superseded:** The Bind-to-project action was originally located in the Palette Browser. ADR-013 moves it to a dedicated Config row in the Settings Layer, which is more general (covers all settings, not just palette).

The Palette Browser (ADR-010) ~~offers two apply actions: apply globally (write to global config) or Bind to project (write `.zani.toml` in the nearest project directory or current file's directory)~~ selects palettes; where changes persist is governed by the config scope (ADR-013).

**Rejected alternatives:**

- **Per-document metadata (frontmatter):** Overengineering for v1. Per-folder covers the "project = mood" use case. Noted as Open Question 8 in the domain model.
- **Environment variables:** Not discoverable, do not travel with the project directory.
- **Separate project config format:** Unnecessary. Reusing the `Config` struct format keeps one schema.

## Consequences

**Positive:**
- Zero-friction palette switching per project. Opening a file immediately resolves to the project's palette.
- Familiar pattern for terminal-native users (`.editorconfig`, `.prettierrc`).
- The Settings Layer can indicate provenance: "Neon Noir (project)" vs "Ember (global)".
- Generalizes beyond palette — any per-project writing preference (column width, focus mode) works the same way.

**Negative:**
- Config Resolution adds a filesystem walk at file open — a handful of `stat()` calls per ancestor directory. Negligible for performance but must handle errors gracefully (permission denied, symlink loops).
- Partial overrides require merging: local config fields override global, unspecified fields fall through. The merge logic must be explicit.

**Neutral:**
- Local Config is optional. Zani works identically without any `.zani.toml` files.
- The walk-up search terminates at filesystem root, not at any git or project boundary. This is simpler and matches `.editorconfig` behavior.
