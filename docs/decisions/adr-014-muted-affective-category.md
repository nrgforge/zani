# ADR-014: Muted as Fourth Character Value

**Status:** Proposed

## Context

ADR-009 established the Affective Category taxonomy with two axes: brightness (Dark, Light) and character (Warm, Cool, Vivid). This produced six categories. Essay 003 identified a gap in this taxonomy through convergent evidence from three independent factor-analytic programs:

- Ou et al. (2004): Activity (chroma) axis — "Active" vs. "Passive"
- Kobayashi (1981): "Clear" vs. "Grayish" dimension
- Valdez & Mehrabian (1994): Arousal driven by saturation — high vs. low

The current "Vivid" captures the high pole of the chroma/saturation axis. The low pole — muted, desaturated, grayish — is absent. This is not the absence of vividness but a distinct affective register: contemplative, subdued, reflective (Jonauskaite & Mohr 2025 systematic review, 132 studies, 42,266 participants). Suk & Irtel (2010) found that emotional response varies more strongly with tone (chroma level) than with hue, making this the single most well-supported missing dimension.

The El-Nasr emotional affordances framework (Essay 002) is not in conflict — it established that color primes affect through a subconscious pathway with saturation as the primary arousal lever. Muted is the calm end of that same lever.

## Decision

Add "Muted" as a fourth character value on the Affective Category taxonomy, producing eight categories:

| | Warm | Cool | Vivid | Muted |
|---|---|---|---|---|
| **Dark** | Intimate, sheltering | Intellectual, deep | Electric, energetic | Contemplative, subdued |
| **Light** | Gentle, morning | Crisp, alpine | Bright, solar | Airy, soft |

The `AffectiveCategory` enum gains `DarkMuted` and `LightMuted` variants. Display order places Muted after Vivid: DarkWarm, DarkCool, DarkVivid, DarkMuted, LightWarm, LightCool, LightVivid, LightMuted.

**Rejected alternatives:**

- **Keep three character values, add Muted palettes under Cool:** Muted is perceptually distinct from Cool. Cool palettes have chromatic character (blue-green undertones); Muted palettes are desaturated across all hue families. Conflating them misrepresents the factor-analytic evidence.
- **Split Muted into Muted-Warm and Muted-Cool:** Over-categorization. Yan et al. (2015) found that excessive categorization becomes a source of overload. Eight categories is within the researched range; ten would push against it. Muted palettes can have warm or cool undertones within the single Muted category.
- **Add a third axis (saturation) instead of a fourth character value:** A three-axis taxonomy (brightness × temperature × saturation) produces 8 categories — the same count — but requires users to navigate three independent dimensions. The flat character axis (Warm, Cool, Vivid, Muted) is simpler to browse.

## Consequences

**Positive:**
- Completes the chroma/saturation axis that every major color-emotion factor model identifies.
- Supports core writing use cases (journaling, reflective prose, quiet literary fiction) that a high-arousal-only taxonomy underserves.
- The Palette Browser gains two categories, expanding from 6 to 8 rows — still within the research-supported range for categorized browsing (Sharma 2023, Yan et al. 2015).

**Negative:**
- Code change required: `AffectiveCategory` enum, `all()`, `label()`, `is_dark()`, and all match arms throughout the codebase.
- Ten new Muted palettes need to be designed (5 Dark-Muted + 5 Light-Muted) to fill the new categories.

**Neutral:**
- ADR-009 is amended: its character axis expands from (Warm, Cool, Vivid) to (Warm, Cool, Vivid, Muted). ADR-009's core decision (organize by Affective Category with Perceptual Sort Order) is unchanged.
