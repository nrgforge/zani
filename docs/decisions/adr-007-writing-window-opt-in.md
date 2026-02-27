# ADR-007: Writing Window Is Opt-In

**Status:** Accepted

## Context

Invariant 9 originally stated: "The Writing Window is the default. When launched without `--inline`, Zani spawns a dedicated terminal window with writing-optimized settings."

During implementation (wiring the event loop), this proved problematic:
- Writers using tmux — a common workflow — got an unwanted window spawn instead of running inline in their current pane.
- The default spawn behavior surprised users who expected `zani draft.md` to edit in-place, like `vim` or `nano`.
- On macOS, spawning a Ghostty window launches a separate application instance, which feels heavy for the default action.

The Writing Window remains valuable for writers who want a dedicated, font-optimized writing environment. But it should be a deliberate choice, not a surprise.

## Decision

Inline Mode is the default. The Writing Window is opt-in via `--window`:

- `zani draft.md` — runs inline in the current terminal
- `zani --window draft.md` — spawns a dedicated Writing Window with writing-optimized font settings

The `--inline` flag is retained for explicit inline mode (e.g., inside a Writing Window to prevent re-spawn), but it is no longer needed to suppress window spawning.

**Invariant 9 is amended** to: "The Writing Window is opt-in. When launched with `--window`, Zani spawns a dedicated terminal window with writing-optimized settings. Without `--window`, Zani runs inline in the current terminal."

## Supersedes

This supersedes ADR-003's assertion that the Writing Window is the default launch mode. ADR-003's terminal detection and spawn mechanics remain valid — only the trigger changes from default to `--window`.

## Consequences

**Positive:**
- Predictable behavior: `zani file.md` works like every other terminal editor.
- Tmux, screen, and SSH workflows work without flags.
- Writers who want the font-optimized window explicitly request it.

**Negative:**
- Writers who always want the Writing Window must remember `--window` (mitigated by shell alias).
- The zero-friction "just type `zani`" experience now gives inline mode, not the optimized window.

**Neutral:**
- `ZANI_WINDOW=1` still prevents re-spawn inside an existing Writing Window.
- Terminal detection logic is unchanged — it's just gated behind `--window`.
