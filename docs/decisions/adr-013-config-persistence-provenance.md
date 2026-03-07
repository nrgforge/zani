# ADR-013: Config Persistence Provenance

**Status:** Proposed

## Context

ADR-011 established Config Resolution — the walk-up lookup from Local Config (`.zani.toml`) to global config (`~/.config/zani/config.toml`) to built-in default. The resolution side works: opening a file in a project with a `.zani.toml` correctly loads the local settings.

The persistence side does not. `save_config()` fires eagerly on every settings change and always writes to the global config file regardless of where the config was resolved from. This creates two problems:

1. **Source mismatch:** A writer who opens a project with a Local Config, changes the palette, and closes the session has written the change globally — the local config still overrides on next open, so the change appears lost.

2. **Scope pollution:** A writer who adjusts settings intending to save them as project-specific has already modified the global config before they can act on "Save to project." The global config should not be affected by changes that the writer intends to be project-local.

Additionally, there is no way to create a Local Config from within Zani. The only path to a `.zani.toml` is manual file creation or the `Config::bind_to_project` API, which is not wired to the Settings Layer.

ADR-011 described a "Bind to project" action in the Palette Browser. This ADR supersedes that approach in favor of a dedicated Config row in the Settings Layer, which is more general (covers all settings, not just palette) and separates config scope management from palette selection.

## Decision

**Persistence timing depends on config scope.** Settings changes behave differently depending on whether the session is bound to a project:

- **Local source (config_source is Local):** Changes persist immediately to the `.zani.toml` on every settings change. The writer has already committed to project scope, and other files opened in the same project directory should see the updated settings immediately.

- **Global/Default source (config_source is Global or Default):** Changes are held in memory during the session. They persist to the global config on quit. This gives the writer time to experiment with settings and optionally choose "Save to project" before the global config is touched.

This means "Save to project" has a clear behavioral shift: once you bind to project, subsequent changes become immediately persistent (to `.zani.toml`). Before that, they are exploratory and uncommitted.

**Write-back respects provenance.** On quit, the config writes to the source it was loaded from:

- **Local:** write to the `.zani.toml` that was found during Config Resolution.
- **Global/Default:** write to `~/.config/zani/config.toml` (creating it if needed).

To support write-back, Config Resolution must also return the path of the found `.zani.toml`, not just the `ConfigSource` enum.

**A Config row in the Settings Layer** shows the current config scope and offers scope management:

- When config_source is Global or Default: the row reads `Config: global [enter]`. Pressing Enter creates a `.zani.toml` containing all current settings, switches config_source to Local, and from that point forward changes persist immediately to the `.zani.toml`.
- When config_source is Local: the row reads `Config: project`. No action — the writer is already saving locally.
- When no file is open (scratch buffer): the row reads `Config: global` with no `[enter]` affordance — there is no project directory to bind to.

**"Save to project" writes the full current config.** When the writer creates a new Local Config via the Config row, all current settings (palette, focus mode, column width, editing mode, scroll mode) are written to the `.zani.toml`. This makes the project config fully self-contained — no surprises from global changes leaking through.

**Project directory for new bindings** is the parent directory of the currently opened file. If a `.zani.toml` already exists in an ancestor directory (found by the walk-up search), "Save to project" writes to that existing location rather than creating a new file in the file's parent.

**Rejected alternatives:**

- **Eager save to global on every change (current behavior):** Pollutes the global config with changes the writer may intend to be project-specific. The writer has no chance to choose scope before the save happens.
- **Deferred persistence for both local and global:** Loses the benefit of immediate project-level sharing — a second file opened in the same directory would not see the updated settings until quit.
- **Bind action in the Palette Browser:** Couples config scope management to palette selection. A writer who wants to bind focus mode or column width to a project would have no path. The Config row is more general.
- **Prompt on every save ("global or project?"):** Interrupts the writer on every settings change.
- **Separate "local" and "global" save buttons per setting:** Overengineers the UI. Config scope is a property of the session, not individual fields.

## Consequences

**Positive:**
- Global config is protected from exploratory changes — the writer can experiment during a session without commitment.
- Project config updates are immediately visible to other files in the same project.
- "Save to project" captures exactly the settings the writer has dialed in, without polluting the global config.

**Negative:**
- Config Resolution must track the found `.zani.toml` path, not just the source type. Slightly more state in App.
- "Save to project" writes all fields, which means a project config will not inherit future global config changes for fields the writer hasn't explicitly set. This is a feature (predictability), but may surprise writers who expect inheritance.
- If the writer quits unexpectedly (crash, kill) while in global scope, in-session settings changes are lost. This is acceptable — settings are low-stakes compared to document content (which is covered by Autosave).

**Neutral:**
- The "Bind to project" scenario from ADR-011's Palette Browser section is superseded. The domain model's Bind action remains — its mechanism changes from browser to Config row.
- The existing `Config::bind_to_project` function can be generalized to `Config::save_local` taking the full config.
- Autosave behavior is unchanged — it writes the document content, not config.
