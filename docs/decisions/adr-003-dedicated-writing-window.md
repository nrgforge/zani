# ADR-003: Dedicated Writing Window for Typography Control

**Status:** Proposed

## Context

Terminal applications cannot change the font family at runtime. No modern terminal supports changing font via escape codes for the current window. The research identified three approaches to the font friction problem: the writer must reconfigure their terminal before using Zani, Zani modifies the current terminal (not possible), or Zani spawns a new window.

Invariant 9 states: "The Writing Window is the default." The research found that most major modern terminals support CLI config overrides for font, size, line height, and colors. iTerm2 uses a profile-based approach instead of inline CLI flags.

Three options were evaluated:
1. **Dedicated Writing Window** — spawn a new terminal window with writing-optimized settings.
2. **Modify current terminal in-place** — Kitty can change font size but not family at runtime. No terminal supports font family changes. Dead end.
3. **Graphics protocol rendering** — rasterize text as images via Kitty/Sixel protocol. Breaks text selection, copy/paste, accessibility. Impractical.

## Decision

When launched without `--inline`, Zani detects the terminal emulator and spawns a dedicated Writing Window with writing-optimized settings (font, font size, line height, colors). Zani then runs inside that window. When the writer quits, the window closes. The development terminal is unchanged.

Detection uses environment variables (`$TERM_PROGRAM`, `$KITTY_PID`, `$WEZTERM_EXECUTABLE`, etc.). A `ZANI_WINDOW=1` environment variable prevents re-spawning when already in a Writing Window.

Inline Mode (`--inline` flag) skips window creation for SSH, tmux, or pre-configured terminals.

Terminal-specific launch commands:
- Ghostty: `ghostty --font-family="..." --font-size=16 -e zani --inline`
- Kitty: `kitty -o font_family="..." -o font_size=16 zani --inline`
- WezTerm: `wezterm --config-file=... start -- zani --inline`
- Alacritty: `alacritty -o font.normal.family="..." -e zani --inline`
- iTerm2: profile-based — Zani creates/uses a "Zani" profile with writing-optimized settings, launched via `open -a iTerm --args -p "Zani"` or the iTerm2 Python API

## Consequences

**Positive:**
- Zero friction: `zani draft.md` opens a beautiful writing environment with the right font, line height, and colors.
- Clean separation between writing window and development terminal.
- The act of launching creates a distinct space for writing — aligns with the zen philosophy.
- Works across all modern terminals that support CLI config overrides.

**Negative:**
- Requires maintaining terminal-specific launch configurations.
- Terminal detection via environment variables is heuristic, not authoritative. Edge cases (tmux, nested terminals, screen) may misidentify the outer terminal. A config override or `--terminal=<name>` flag may be needed.
- Unknown terminals fall back to Inline Mode (no font control), per Invariant 11 (graceful degradation).
- Users on terminals without CLI config override support get Inline Mode only.

**Neutral:**
- Zani ships with a config file (`~/.config/zani/config.toml`) specifying font preferences and terminal-specific overrides.
- First-run experience could offer to install iA Writer Duo (free, MIT-licensed) if not present.
