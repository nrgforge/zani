# ADR-018: Chroma Targets for Affective Categories

**Status:** Proposed

## Context

Essay 005 established through empirical measurement that the eight Affective Categories are taxonomically correct but chromatically unexpressed in palette backgrounds. An OKLCH chroma audit of all 40 palettes revealed:

- Light Warm backgrounds (mean ΔE2000 12.73 from neutral) register as more vivid than Light Vivid (mean ΔE2000 7.14) — the hierarchy is inverted.
- Three of five Light Vivid backgrounds fall in the "noticeably tinted" perceptual band (ΔE2000 2.5–5.0), the same band as Light Muted palettes.
- The vivid/muted distinction is carried entirely by accent colors, which occupy ~5–10% of the visual field. Per Invariant 17, the background dominates the writer's immersive experience (~90% of screen area).
- Light Muted and Dark Muted backgrounds are correctly restrained — the bottom of the hierarchy works.

Invariant 18 establishes that each Affective Category must have a Chroma Target specifying the ΔE2000 perceptual band its backgrounds must achieve. This ADR defines the specific ranges.

Perceptual thresholds from color science (CIEDE2000) and a readability review (Essay 005, Q3) established:
- Extended reading on moderately tinted backgrounds (ΔE2000 5–15, OKLCH chroma 0.02–0.06) is within the comfort zone and may be preferred over pure white.
- Light background chroma should stay below ~0.09 OKLCH (ΔE2000 ~20) to avoid interfering with content perception.
- Dark backgrounds face sRGB gamut constraints — dark saturated colors have a lower chroma ceiling, making accent chroma a legitimate compensatory channel.

## Decision

### Chroma Target Ranges

Each Affective Category's palette backgrounds must achieve a ΔE2000 (from nearest-lightness neutral gray) within these ranges:

**Light categories:**

| Category | ΔE2000 Range | OKLCH Chroma Range | Perceptual keyword |
|----------|-------------|-------------------|--------------------|
| Light Vivid | 10–20 | 0.04–0.09 | "colored paper" |
| Light Warm | 5–10 | 0.02–0.04 | "warm tint" |
| Light Cool | 5–10 | 0.02–0.04 | "cool tint" |
| Light Muted | 2–5 | 0.007–0.015 | "barely there" |

Light Warm and Light Cool share the same ΔE2000 band. These categories are distinguished by hue (warm vs. cool undertone), not by chroma intensity. The Chroma Target constrains how much color the background carries; the 15° OKLCH hue diversity rule and species Signature Colors provide the distinguishing dimension.

The boundary at ΔE2000 10 aligns with the perceptual science threshold: above 10 is "vivid — strongly colored, unmistakable"; below 10 is "clearly colored" or less. This creates a natural perceptual boundary between Vivid and Warm/Cool categories.

**Dark categories (ΔE2000 ranges only — raw chroma varies with lightness due to sRGB gamut):**

| Category | ΔE2000 Range | Perceptual keyword |
|----------|-------------|--------------------|
| Dark Vivid | 6–10 | "clearly tinted" |
| Dark Warm | 5–9 | "warm undertone" |
| Dark Cool | 4–8 | "cool undertone" |
| Dark Muted | 1–4 | "near-neutral" |

Dark Vivid and Dark Warm ranges overlap (6–9). This is an intentional consequence of the sRGB gamut constraint: dark saturated colors have a lower chroma ceiling, limiting background-only separation. In the dark tier, Vivid distinguishes itself from Warm primarily through accent chroma (the compensatory strategy), not background chroma alone. The ranges ensure both sit above Cool and Muted, while the accent requirement (Enforcement Rule 4) provides the additional separation.

Dark Warm and Dark Cool also overlap (5–8), distinguished by hue rather than chroma intensity — the same principle as Light Warm vs. Light Cool.

### Enforcement Rules

1. **Individual palette conformance.** Every palette's background ΔE2000 from nearest-lightness neutral must fall within its category's target range.
2. **No downward band crossing.** No individual palette in a higher-chroma category may have a background ΔE2000 below the *floor* of the next-lower category. Concretely: no Light Vivid palette below ΔE2000 10; no Light Warm or Light Cool palette below ΔE2000 5; no Dark Vivid palette below ΔE2000 6. Upward excursions (a Warm palette at ΔE2000 10) are compliant — a palette that runs hotter than its category's minimum is less confusing than one that runs cooler.
3. **Category mean hierarchy.** Within each brightness tier, category mean ΔE2000 values must follow: Vivid > Warm ≥ Cool > Muted. In the dark tier, the Vivid > Warm ordering may be narrow due to sRGB constraints; the accent compensation rule provides the perceptual separation that background chroma alone cannot.
4. **Dark Vivid accent compensation.** Dark Vivid palettes must maintain accent chroma (mean across accent_heading, accent_emphasis, accent_link, accent_code) substantially above Dark Muted levels. This is a design judgment, not a perceptually derived threshold: the current Dark Vivid accent mean is 0.1574 vs. Dark Muted's 0.0425 (3.7x ratio). Accent chroma compensates for the sRGB ceiling on dark background chroma.

### Measurement Method

ΔE2000 is computed between the palette background (converted to CIELAB via sRGB → XYZ → CIELAB) and the neutral gray at the same CIELAB lightness (L* matched, a*=0, b*=0). This is the standard CIEDE2000 formula (ISO/CIE 11664-6:2014).

OKLCH chroma is used as a secondary reference for light backgrounds where the chroma-to-ΔE relationship is approximately linear. For dark categories, OKLCH chroma ranges are not provided because the mapping is non-linear and lightness-dependent.

### Retuning Constraints

When retuning a palette's background to meet its Chroma Target:

- The retuned background must preserve the species' Signature Color association (Invariant 16). A Bracken palette must still feel like bracken.
- All foreground/accent vs. background pairs must maintain WCAG AA 4.5:1 contrast ratio (Invariant 3). Higher background chroma may require foreground adjustment.
- Sibling palettes must maintain 15° OKLCH hue diversity within their category.
- Accent colors may need adjustment to maintain visual coherence with the new background.

### Light Vivid Feasibility Note

Before retuning, verify that each Light Vivid species' Signature Colors can authentically produce a background in the ΔE2000 10–20 range. Current Light Vivid palettes (Farewell ΔE2000 4.38, Camas ΔE2000 3.47) need background chroma roughly tripled. These palettes' current compositions connect to foliage, not flowers — recomposition toward higher-chroma Signature Colors changes the Species Essence and must remain authentic per Invariant 16. Species whose essence requires a low-chroma background may need category reassignment or species replacement.

**Rejected alternatives:**

- **Target only category means, not individual palettes:** Allows individual outliers to cross category boundaries (the current problem). Rejected because the writer encounters individual palettes, not category means.
- **Use OKLCH chroma ranges for all categories:** OKLCH chroma maps to different ΔE2000 values at different lightness levels. A single chroma threshold means different perceptual experiences for dark vs. light palettes. ΔE2000 is perceptually uniform across lightness.
- **Tighter ranges with no overlap between Warm/Cool and Vivid:** Would over-constrain palette design, leaving insufficient room for species-authentic color expression. The ΔE2000 10 boundary already aligns with the perceptual "vivid" threshold from color science.
- **No enforcement on individual palettes — advisory targets only:** The current state is the result of advisory intentions without enforcement. Advisory targets drift; enforced targets hold.
- **Specific numeric threshold for Dark Vivid accent compensation (e.g., > 0.12):** No perceptual threshold study exists for accent chroma analogous to the ΔE2000 background thresholds. A specific number would be a magic number masquerading as evidence. The principle (substantially above Dark Muted levels) is stated; the specific minimum should be established through implementation experience.

## Consequences

**Positive:**
- The character axis of the Affective Category taxonomy becomes perceptually real — a writer browsing Vivid palettes will see and feel distinctly more color than when browsing Muted.
- Background chroma carries the affective distinction, aligning with Invariant 17 (background dominance).
- ΔE2000 is a standard, reproducible measurement — palette conformance is testable, not subjective.
- The targets are evidence-based (Essay 005, Q1–Q3), grounded in color science thresholds and readability research.

**Negative:**
- Light Vivid palettes (5) need significant background chroma increases. Some may require category reassignment or species replacement if their Signature Colors cannot authentically produce backgrounds in the 0.04–0.09 chroma range.
- Light Warm palettes need background chroma reduction (current mean ΔE2000 12.73, target ceiling 10), which changes their current character.
- Each retuned palette requires re-verification of Invariant 3 (WCAG AA contrast), hue diversity, and species Signature Color fidelity — this is labor-intensive.

**Neutral:**
- ADR-009 is further amended: its Affective Category organization now includes chromatic targets per category. ADR-009's core decision (organize by category with Perceptual Sort Order) is unchanged.
- Dark categories are mostly already compliant. The effort concentrates on Light Vivid and Light Warm.
- The Chroma Target concept and Invariant 18 were established in the domain model before this ADR. This ADR operationalizes them with specific numbers.
