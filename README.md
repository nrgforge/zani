# Zani

A terminal writing app. Beautiful, distraction-free writing with focus modes, curated palettes, and vim keybindings.

## Why

There’s no terminal-based writing app with a thoughtful visual design. Other tools have their own file format. Or are baked into heavier dev environments. The best GUI apps are beautiful but closed — no composability, no scripting, no terminal. So here’s a tool built for creative writing that anyone in the world can use freely from their terminal.


## Design

Every design decision traces to research on flow states, cognitive load, typography, and color science.

- **Zero chrome by default.** Text, cursor, empty space. Settings appear only when summoned (Ctrl+P).
- **Centered ~60-character column.** Research consensus for optimal line length. Configurable.
- **Curated color palettes.** No pure black or white (causes halation). WCAG AA contrast minimum. Warm, muted defaults.
- **Focus modes.** Sentence and paragraph — surrounding text fades via per-character color interpolation, keeping the writer in generative mode.
- **Scroll modes.** Edge scrolling or typewriter (cursor stays vertically centered). Independent of focus modes.
- **Markdown as native format.** Syntax is always visible and editable, never hidden. Bold renders bold, headings get color, syntax characters are dimmed.
- **Two editing modes.** Vim (modal, default) or Standard (modeless, CUA-style). Switch in Settings Layer.
- **Vim keybindings.** Normal/Insert mode, hjkl movement, word motions (w/b/e), line operations (0/$), multi-key sequences (gg, dd, o/O). The cursor shape changes with the mode.
- **Smart typography.** Straight quotes become curly quotes, `--` becomes em dashes, `...` becomes ellipses — as you type.

## Install

```
cargo install --path .
```

Or build from source:

```
cargo build --release
```

## Usage

```
zani                     # New scratch document
zani draft.md            # Open existing file
zani --window draft.md   # Dedicated Writing Window with custom font
```

Zani runs inline by default — it works inside tmux, over SSH, in any terminal. The `--window` flag spawns a dedicated terminal window with writing-optimized typography (font, line height, colors). Supports Ghostty, Kitty, WezTerm, Alacritty, and iTerm2.

### Keybindings

| Key | Mode | Action |
|-----|------|--------|
| `i` | Normal | Enter Insert mode |
| `a` | Normal | Append after cursor |
| `A` | Normal | Append at end of line |
| `Esc` | Insert | Return to Normal mode |
| `h` `j` `k` `l` | Normal | Move left/down/up/right |
| `w` `b` `e` | Normal | Word forward / backward / end |
| `0` `$` | Normal | Line start / end |
| `G` | Normal | Go to last line |
| `gg` | Normal | Go to first line |
| `x` | Normal | Delete character |
| `dd` | Normal | Delete line |
| `o` `O` | Normal | Open line below / above |
| `Ctrl+F` | Any | Find |
| `Ctrl+P` | Any | Toggle Settings Layer |
| `Ctrl+S` | Any | Save |
| `Ctrl+Z` | Any | Undo |
| `Ctrl+Y` | Any | Redo |
| `Ctrl+C` | Any | Copy selection |
| `Ctrl+X` | Any | Cut selection |
| `Ctrl+V` | Any | Paste |
| `Ctrl+A` | Any | Select all |
| `Ctrl+Q` | Any | Quit |

### Settings Layer

Press `Ctrl+P` to summon the Settings Layer. Navigate with `j`/`k`, apply with `Enter`. Palette colors preview live as you browse. Column width adjusts with `h`/`l`. File rename on `Enter`.

Settings persist to `~/.config/zani/config.toml`.

### Focus Modes

Switch via Settings Layer (`Ctrl+P`): Off, Sentence, Paragraph.

- **Sentence** — the current sentence stays bright, everything else fades.
- **Paragraph** — the current paragraph stays bright, surrounding paragraphs dim gradually.

### Scroll Modes

Switch via Settings Layer (`Ctrl+P`): Edge, Typewriter.

- **Edge** — the viewport scrolls when the cursor nears the top or bottom edge.
- **Typewriter** — the cursor stays vertically centered, text scrolls around it.

### Color Profiles

Zani detects your terminal's color capability automatically:

- **TrueColor** (24-bit) — full RGB palettes and smooth dimming gradients.
- **256-color** — palettes mapped to the nearest 256-color indices.
- **Basic** (16 ANSI) — focus dimming uses the terminal's dim attribute.

### Palettes

Three built-in palettes, each respecting the no-pure-black/white constraint:

- **Ember** — warm dark. The default.
- **Inkwell** — cool dark.
- **Parchment** — warm light.

## Architecture

Built in Rust for predictable latency — zero GC means no pauses during sustained typing.

| Crate | Role |
|-------|------|
| [ratatui](https://github.com/ratatui/ratatui) | App shell, layout, immediate-mode rendering |
| [crossterm](https://github.com/crossterm-rs/crossterm) | Cross-platform terminal backend |
| [ropey](https://github.com/cessen/ropey) | Rope data structure for the text buffer |

The **Writing Surface** is a custom widget that renders directly to ratatui's cell buffer, bypassing the Paragraph widget. This gives full control over soft-wrapping, scroll positioning, and per-character styling — composing markdown formatting with focus dimming in a single render pass.

The **animation subsystem** drives all visual transitions — palette crossfades, overlay fade-ins, and per-line focus dimming. All dimming uses opacity-based color interpolation (foreground blended toward background), not distance-based falloff. Two layers: `AnimationManager` for global discrete transitions, `DimLayer`/`AnimatedValue` for per-line chase-to-target dimming. Zero steady-state allocations after first frame.

```
Ropey buffer → Writing Surface → ratatui cell buffer → Crossterm → terminal
```

## Development

```
cargo test       # 380 unit + 3 integration + 1 alloc bench
cargo clippy     # Lint
cargo run        # Run in development
```

## License

MIT
