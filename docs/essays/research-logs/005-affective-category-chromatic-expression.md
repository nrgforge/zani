# Research Log: Affective Category Chromatic Expression

## Question 1: What is the OKLCH chroma distribution of all 40 palette backgrounds and accents, grouped by Affective Category? Are Vivid vs Muted backgrounds actually distinguishable, or do they overlap?

**Method:** Spike — measured OKLCH lightness, chroma, and hue for all 40 palettes' backgrounds and accents. Computed category means, min/max ranges, and cross-category comparisons. Code at `scratch/spike-chroma-audit/`.

**Findings:**

### Background chroma ranking (by category mean)

| Rank | Category | Bg Chroma Mean | Expected Rank |
|------|----------|---------------|---------------|
| 1 | Light Warm | 0.0417 | Should be below Vivid |
| 2 | Dark Warm | 0.0281 | Should be below Vivid |
| 3 | Light Vivid | 0.0212 | Should be #1 among Light |
| 4 | Dark Vivid | 0.0191 | Should be #1 among Dark |
| 5 | Dark Cool | 0.0170 | — |
| 6 | Light Cool | 0.0143 | — |
| 7 | Light Muted | 0.0106 | Correct: lowest among Light |
| 8 | Dark Muted | 0.0072 | Correct: lowest overall |

**Warm backgrounds are more saturated than Vivid backgrounds.** This contradicts the taxonomy: Vivid is supposed to be the high-chroma pole of the character axis, but Warm outranks it in background saturation.

### Vivid vs Muted overlap

**Light categories overlap.** The lowest Light Vivid background (Farewell, 0.0095) is lower than the highest Light Muted background (Goatsbeard, 0.0147). Three of five Light Vivid backgrounds (Paintbrush 0.020, Farewell 0.010, Camas 0.010) are in "near-neutral" territory (< 0.020).

**Dark categories barely separate.** Lowest Dark Vivid bg (0.0126) is just above highest Dark Muted bg (0.0109). Dark palettes have an inherent constraint: dark colors have lower chroma ceiling in sRGB.

### Where the distinction actually lives

**Accents, not backgrounds.** The accent chroma separation is strong:
- Dark: Vivid accents 0.1574 mean vs Muted 0.0425 (3.7x ratio)
- Light: Vivid accents 0.1307 mean vs Muted 0.0524 (2.5x ratio)

The vivid/muted distinction is carried entirely by accent colors. Since the background is ~90% of screen area and accents are punctuation, the dominant visual experience of "Vivid" and "Muted" categories is nearly identical.

### Near-neutral backgrounds (chroma < 0.020)

24 of 40 palettes have near-neutral backgrounds. This includes:
- 3 of 5 Light Vivid (Paintbrush, Farewell, Camas)
- 3 of 5 Dark Vivid (Fly Agaric, Lobaria, Jack-o'-Lantern)
- All 5 Dark Muted
- 4 of 5 Light Cool

### Accent chroma anomalies

Three Muted palettes have accent chromas that exceed some Vivid palettes:
- Map Lichen (Dark Muted): accent mean 0.074
- Sword Fern (Light Muted): accent mean 0.076
- Fringecup (Light Muted): accent mean 0.080

**Implications:**

The affective categories are structurally correct in the taxonomy but not chromatically expressed in the palette data. The "character" axis (Warm/Cool/Vivid/Muted) should produce four distinct chromatic profiles, but currently:
1. Background chroma doesn't differentiate Vivid from Muted (overlap exists)
2. Warm backgrounds are more saturated than Vivid backgrounds (ranking inverted)
3. The accent separation is real but insufficient — accents are too small a fraction of screen area to carry the entire affective distinction
4. Dark palettes have a legitimate constraint (dark + high chroma is limited in sRGB) but Light Vivid has no such excuse

This confirms the user's observation: "these should feel affective, across the board according to their affect." The data shows they don't.

## Question 2: At what OKLCH chroma does a background cross from "barely tinted" to "clearly colored"? What are the perceptual thresholds?

**Method:** Spike + web search. Computed CIEDE2000 color differences between neutral gray and tinted versions at increasing OKLCH chroma levels. Applied established ΔE2000 perceptual thresholds from color science literature.

**Findings:**

### ΔE2000 perceptual thresholds (from literature)

| ΔE2000 | Perception |
|--------|-----------|
| < 1.0 | Imperceptible — no human observer can detect difference |
| 1.0–2.5 | Barely tinted — detectable but marginal |
| 2.5–5.0 | Noticeably tinted — clearly different from neutral |
| 5.0–10.0 | Clearly colored — unambiguously tinted |
| > 10.0 | Vivid — strongly colored, unmistakable |

### OKLCH chroma to ΔE2000 mapping (light backgrounds, ~L=0.93)

| OKLCH Chroma | ΔE2000 | Perception |
|-------------|--------|-----------|
| 0.005 | ~1.6 | Barely tinted |
| 0.011 | ~3.7 | Noticeably tinted |
| 0.019 | ~6.2 | Clearly colored |
| 0.029 | ~8.8 | Clearly colored |
| 0.038 | ~11.0 | Vivid |
| 0.049 | ~13.2 | Vivid |
| 0.072 | ~17.5 | Vivid |
| 0.089 | ~20.0 | Vivid |

### Current category ΔE2000 means (from nearest-lightness neutral)

| Category | ΔE Mean | ΔE Min | ΔE Max | Diagnosis |
|----------|---------|--------|--------|-----------|
| Light Warm | 12.73 | 8.64 | 15.76 | Vivid! (should be moderate) |
| Dark Warm | 9.88 | 7.35 | 13.08 | Clearly colored (appropriate) |
| Light Vivid | 7.14 | 3.47 | 10.29 | Mixed (Farewell 4.4, Camas 3.5 are weak) |
| Dark Vivid | 6.92 | 4.08 | 9.08 | Clearly colored (sRGB constraint) |
| Dark Cool | 6.45 | 4.73 | 8.64 | Clearly colored |
| Light Cool | 5.42 | 2.77 | 7.78 | Mixed (Pearly Everlasting 2.8 is weak) |
| Light Muted | 4.30 | 3.01 | 6.10 | Noticeably tinted (appropriate) |
| Dark Muted | 2.87 | 1.37 | 4.27 | Barely tinted to noticeably (appropriate) |

### Critical findings

**Light Warm is more vivid than Light Vivid.** Light Warm mean ΔE=12.73 vs Light Vivid mean ΔE=7.14. This is backwards.

**Three Light Vivid palettes fail the "vivid" threshold.** Farewell (ΔE=4.38) and Camas (ΔE=3.47) register as merely "noticeably tinted" — the same perceptual band as Light Muted palettes. Oregon Sunshine (ΔE=9.92) is "clearly colored" but not "vivid."

**Dark Vivid is inherently constrained.** Dark saturated colors hit sRGB gamut limits. Dark Vivid backgrounds average ΔE=6.92 — "clearly colored" but not dramatically different from Dark Cool (6.45). This is a physics limitation, not a design failure. Dark Vivid must rely more on accent chroma.

**Muted backgrounds are correctly restrained.** Dark Muted (2.87) and Light Muted (4.30) sit in the right perceptual bands.

**Implications:**

Proposed chroma targets by category (light backgrounds):

| Category | Target ΔE Range | OKLCH Chroma Range | Keyword |
|----------|----------------|-------------------|---------|
| Light Vivid | 10–20 | 0.04–0.09 | "colored paper" |
| Light Warm | 5–12 | 0.02–0.05 | "warm tint" |
| Light Cool | 5–12 | 0.02–0.05 | "cool tint" |
| Light Muted | 2–5 | 0.007–0.015 | "barely there" |

For dark backgrounds, the sRGB constraint means targeting ΔE rather than raw chroma:

| Category | Target ΔE Range | Keyword |
|----------|----------------|---------|
| Dark Vivid | 6–10 | "clearly tinted" + vivid accents |
| Dark Warm | 6–10 | "warm undertone" |
| Dark Cool | 4–8 | "cool undertone" |
| Dark Muted | 1–4 | "near-neutral" |

## Question 3: Does background color saturation affect reading comfort for extended use?

**Method:** Web search — surveyed recent research on colored backgrounds, reading performance, eye fatigue, and visual comfort.

**Findings:**

### Light green backgrounds reduce eye fatigue

A 2025 Frontiers in Psychology study (Frontiers, 2025) found that a light green background (RGB 207, 232, 204 — OKLCH chroma ~0.03) significantly increased pupil diameter (indicating lower fatigue) and reduced negative emotion compared to white. Participants reported white backgrounds caused glare and eye fatigue. However, actual reading speed/accuracy differences were not statistically significant.

### Low saturation is key

The study authors explicitly note their light green is "low-saturation" and that findings from "high-saturation colors (e.g., red, blue, yellow, green)" may not generalize. This supports moderate background tinting — enough to be perceived as colored, not enough to create chromatic fatigue.

### Dark mode and visual comfort

Research (PMC, 2024) found dark mode provides better visual comfort and reduces eye fatigue in low-light conditions, while light mode was better for overall readability. This validates offering both dark and light palettes but doesn't constrain saturation within either.

### Irlen syndrome / colored overlays

Proponents of colored overlays (Irlen) claim reading through correctly selected tints improves reading speed and comprehension. While controversial, the principle of individually optimal background tinting is consistent with offering varied palette tints.

### Yellow backgrounds are preferred for extended reading

Multiple sources cite light yellow as a popular choice for reducing eye strain in extended reading, with lighter shades preferred over bright yellow. Classic "sepia" modes in e-readers use warm yellows for this reason.

**Implications:**

Background tinting at "noticeably colored" levels (ΔE 5–15, OKLCH chroma 0.02–0.06 for light backgrounds) is within the comfort zone for extended reading. The research suggests low-to-moderate saturation tints are actually preferred over pure white for long sessions. Vivid backgrounds (ΔE > 10) push into "colored paper" territory — still comfortable based on centuries of colored stationery use, but representing a distinct aesthetic choice. High-saturation backgrounds (chroma > 0.10) would be unusual for extended prose writing and should be avoided.

The sRGB gamut ceiling for dark saturated colors (noted in Q2) means dark palettes naturally stay within comfortable ranges. Light palettes have more room to push saturation, and the research suggests moderate tinting is actively preferred over neutrality.

**Key constraint:** Keep light background chroma below ~0.09 OKLCH (ΔE ~20 from neutral). Above this, the background becomes "colored paper" that could interfere with content perception for some users. The range 0.03–0.07 (ΔE 8–17) represents "noticeably and pleasantly colored" — the sweet spot for a vivid writing surface.
