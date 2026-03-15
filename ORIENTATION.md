# Zani — Project Orientation

*Tier 1 entry point. Readable in under 5 minutes.*

---

## What this system is

Zani is a zen writing application for the terminal. It does one thing: give a writer a beautiful, distraction-free environment to write prose in. The tool disappears by design — no chrome, no menus, no notifications — leaving text, cursor, and empty space. Palettes are not themes; they are calibrated mood instruments, 40 of them named after Cascadia bioregion flora and grounded in color science research. The philosophy is that environment shapes creative state, and that every detail — color, focus dimming, typewriter scroll, sub-millisecond keystroke response — either serves the writer's generative momentum or it shouldn't be there.

---

## Who it serves

**The Writer** — a prose writer who is terminal-comfortable, values local-first tools and plain markdown, and wants an environment that actively creates a feeling conducive to writing.
Reading path: [docs/product-discovery.md](docs/product-discovery.md) → [docs/scenarios.md](docs/scenarios.md)

**The Builder** — the developer-writer who maintains Zani. Every design decision is grounded in research or direct writing experience; nothing is "vibes-based."
Reading path: [docs/essays/001-zen-terminal-writing.md](docs/essays/001-zen-terminal-writing.md) → [docs/domain-model.md](docs/domain-model.md) → [docs/system-design.md](docs/system-design.md)

**The Broader Ecosystem** — Zani is one tool in a suite sharing the philosophy "tools that help you be more generative rather than generate for you." Plexus handles knowledge graphs; llm-orc handles model orchestration; Zani handles the writing itself.

---

## Key constraints

1. **The tool disappears.** Default visual state is text, cursor, and empty space. No chrome is visible unless explicitly summoned. (Invariant 1)

2. **Sub-millisecond keystroke latency.** Every keystroke must produce a visible result within the app layer's control in under 1ms. Rust, zero-GC, and immediate-mode rendering are architectural consequences of this requirement, not preferences. (Invariant 6, ADR-001, ADR-002)

3. **WCAG AA contrast floor, no pure black or white.** All foreground/background pairs maintain at least 4.5:1 contrast. The palette system is aesthetically free within this floor. (Invariant 3, ADR-006)

4. **Markdown is the native format.** Documents are plain markdown files on disk. No proprietary format, no database. What git sees is what the writer wrote. (Invariant 7)

5. **Palette names have traceable botanical provenance.** Every palette is named after a real Cascadia species whose documented colors are moderately congruent with the palette composition. Names are not decorative; they are a constitutive part of the instrument. (Invariant 16, ADR-015, ADR-017)

---

## How artifacts fit together

### Tier 1 — Orientation (this document)
Entry point. Answers: what is this, who is it for, what must always be true?

### Tier 2 — Research and Design
- **[docs/essays/](docs/essays/)** — 6 research essays grounding the methodology (affective essentialism, color science, palette design, species validation)
- **[docs/product-discovery.md](docs/product-discovery.md)** — stakeholders, jobs-to-be-done, mental models, value tensions, open questions
- **[docs/domain-model.md](docs/domain-model.md)** — 18 invariants that must always hold; the authoritative list of open questions
- **[docs/system-design.md](docs/system-design.md)** — architectural drivers, component map, rendering pipeline

### Tier 3 — Decisions and References
- **[docs/decisions/](docs/decisions/)** — 18 ADRs recording what was decided and why (supersedes system-design when they conflict)
- **[docs/references/field-guide.md](docs/references/field-guide.md)** — domain-to-code mapping for all modules
- **[docs/essays/reflections/](docs/essays/reflections/)** and **[docs/essays/research-logs/](docs/essays/research-logs/)** — epistemic trail: what was learned, what changed, what remains speculative

---

## Current state

**Complete:**
- 6 research essays establishing the methodological foundation
- 18 ADRs covering all major architectural decisions (language, rendering, focus system, palette architecture, naming register, color validation)
- Domain model with 18 invariants, fully settled
- System design document (v1.0, current)
- Substantial implementation: 448 tests passing, palette browser, focus dimming, typewriter mode, markdown styling, color degradation, local config, accent rendering

**Settled decisions:**
- Rust + ratatui + crossterm + ropey as the stack (ADR-001)
- Immediate-mode rendering with a custom WritingSurface (ADR-002)
- Composable dimming with Scroll/Focus orthogonality (ADR-008)
- 40 palettes in 8 affective categories, Cascadia flora naming register (ADR-009, ADR-015)
- Species color validation methodology and Flora Reference corpus (ADR-016, ADR-017)
- Chroma targets as measurable category invariants (ADR-018)

**Open questions (selected):**
- Save error visibility without violating Invariant 1 (domain-model OQ 1)
- Focus dimming logic scattered across four modules — consolidation candidate (domain-model OQ 2)
- Practical effect size of color-affect priming for writing output (domain-model OQ 7)
- Cold-start discoverability: how does a new writer learn what's available? (domain-model OQ 16)
- Distribution and onboarding: no install mechanism or packaging exists yet (domain-model OQ 19)
- Provenance bias in existing descriptions — systematic audit needed (domain-model OQ 11)
