# Affective Category Chromatic Expression
*2026-03-07*

## Abstract

This essay investigates whether Zani's eight Affective Categories — Dark/Light crossed with Warm/Cool/Vivid/Muted — are chromatically distinguishable in the current palette collection. Through an empirical spike measuring OKLCH chroma and CIEDE2000 perceptual differences across all 40 palettes, a perceptual threshold study mapping chroma to human-visible color difference, and a literature review on reading comfort with saturated backgrounds, the investigation found that the categories are taxonomically correct but chromatically unexpressed. Light Warm backgrounds register as more vivid than Light Vivid; three Light Vivid backgrounds fall in the same perceptual band as Light Muted; and the vivid/muted distinction lives entirely in accent colors that occupy a fraction of screen area. The essay proposes evidence-based chroma targets per category that would make the affective taxonomy perceptually real.

## The Problem: Categories Without Chromatic Identity

Zani's Affective Categories exist to organize 40 palettes into mood-based groups: a writer browsing the Palette Browser should perceive that Dark Warm palettes *feel* warmer than Dark Cool, that Light Vivid palettes *feel* more colorful than Light Muted. The two axes — brightness (Dark/Light) and character (Warm/Cool/Vivid/Muted) — imply four distinct chromatic profiles within each brightness tier.

The question is whether the current palette backgrounds actually produce these distinct profiles. Domain model Invariant 17 establishes that "the background dominates the visual field (~90% of screen area) and determines the writer's immersive experience." If background chroma doesn't differ between Vivid and Muted, the writer's immersive experience doesn't differ either — regardless of how different the accent colors are.

## Empirical Baseline: What the Numbers Say

An audit of all 40 palette backgrounds in OKLCH color space produced the following category-mean chroma ranking:

| Rank | Category | Bg Chroma Mean | Expected Position |
|------|----------|---------------|-------------------|
| 1 | Light Warm | 0.0417 | Should be below Vivid |
| 2 | Dark Warm | 0.0281 | Should be below Vivid |
| 3 | Light Vivid | 0.0212 | Should be #1 among Light |
| 4 | Dark Vivid | 0.0191 | Should be #1 among Dark |
| 5 | Dark Cool | 0.0170 | — |
| 6 | Light Cool | 0.0143 | — |
| 7 | Light Muted | 0.0106 | Correct: lowest among Light |
| 8 | Dark Muted | 0.0072 | Correct: lowest overall |

The ranking is inverted at the top: Warm backgrounds are more saturated than Vivid backgrounds. This contradicts the taxonomy's intent. Vivid is the high-chroma pole of the character axis, yet it ranks third and fourth.

### Overlap Between Categories

Light Vivid and Light Muted overlap. The lowest Light Vivid background (Farewell, chroma 0.0095) falls below the highest Light Muted background (Goatsbeard, 0.0147). Three of five Light Vivid backgrounds sit in "near-neutral" territory (chroma < 0.020), indistinguishable from Muted palettes by background alone.

Dark categories barely separate: lowest Dark Vivid (0.0126) is just above highest Dark Muted (0.0109). Dark palettes face a real physical constraint — dark colors have a lower chroma ceiling within the sRGB gamut — but this does not explain why Dark Warm outranks Dark Vivid.

### Where the Distinction Actually Lives

Accent chroma separation is strong:
- Dark Vivid accents average 0.1574 vs Dark Muted 0.0425 (3.7x ratio)
- Light Vivid accents average 0.1307 vs Light Muted 0.0524 (2.5x ratio)

The vivid/muted distinction is carried entirely by accent colors. But accents are typographic punctuation — headings, emphasis, links, code spans — occupying perhaps 5–10% of the visual field. The dominant experience, the one the writer sits inside for hours, is the background. By background chroma, "Vivid" and "Muted" are nearly the same room.

## Perceptual Thresholds: When Tinted Becomes Colored

Raw OKLCH chroma numbers are not directly interpretable by humans. To translate chroma into perceptual experience, a second spike computed CIEDE2000 color differences (ΔE2000) between each palette's background and a neutral gray at the same lightness. The ΔE2000 metric is the standard measure for perceptual color difference, with well-established thresholds from color science:

| ΔE2000 | Perception |
|--------|-----------|
| < 1.0 | Imperceptible |
| 1.0–2.5 | Barely tinted — detectable but marginal |
| 2.5–5.0 | Noticeably tinted — clearly different from neutral |
| 5.0–10.0 | Clearly colored — unambiguously tinted |
| > 10.0 | Vivid — strongly colored, unmistakable |

Applying these thresholds to category means:

| Category | ΔE Mean | Perceptual Band | Diagnosis |
|----------|---------|----------------|-----------|
| Light Warm | 12.73 | Vivid | Too vivid for "Warm" |
| Dark Warm | 9.88 | Clearly colored | Appropriate |
| Light Vivid | 7.14 | Clearly colored | Should be "Vivid" |
| Dark Vivid | 6.92 | Clearly colored | sRGB constraint applies |
| Dark Cool | 6.45 | Clearly colored | Appropriate |
| Light Cool | 5.42 | Clearly colored | Appropriate |
| Light Muted | 4.30 | Noticeably tinted | Appropriate |
| Dark Muted | 2.87 | Barely tinted | Appropriate |

Light Warm registers as more vivid than Light Vivid. More concerning, individual Light Vivid palettes cross into Muted territory: Farewell (ΔE 4.38) and Camas (ΔE 3.47) are "noticeably tinted" — the same band as Light Muted's mean.

The bottom of the ranking is correctly ordered: Muted categories sit in the appropriate perceptual bands, and the Muted-to-Cool-to-Warm gradient makes sense. The problem is at the top — Vivid doesn't separate from its neighbors upward.

## Reading Comfort: How Much Saturation Is Too Much?

Before proposing higher background chroma for Vivid palettes, the investigation examined whether saturated backgrounds harm reading comfort.

A 2025 Frontiers in Psychology study found that a light green background (approximately OKLCH chroma 0.03) significantly reduced eye fatigue and negative emotion compared to white. The study's authors explicitly noted their stimulus was "low-saturation" and cautioned that findings from "high-saturation colors" might not generalize. This supports moderate background tinting — enough to be perceived as colored, not enough to create chromatic fatigue.

Multiple sources cite light yellow as preferred for extended reading, consistent with centuries of colored stationery and the "sepia mode" convention in e-readers. The comfort zone for extended prose reading extends well into the "clearly colored" range (ΔE 5–15, chroma 0.02–0.06 for light backgrounds).

The practical ceiling sits around chroma 0.09 (ΔE ~20 from neutral). Above this, backgrounds enter "colored paper" territory that could interfere with content perception for some users. The range 0.03–0.07 (ΔE 8–17) represents "noticeably and pleasantly colored" — the sweet spot for a vivid writing surface.

Dark palettes naturally stay within comfortable ranges due to the sRGB gamut constraint on dark saturated colors. No readability concern was found for any dark category at the proposed targets.

## Proposed Chroma Targets

Combining the empirical audit, perceptual thresholds, and readability research, the following targets would make each Affective Category chromatically distinct:

### Light Categories

| Category | Target ΔE Range | OKLCH Chroma Range | Keyword | Current ΔE Mean |
|----------|----------------|-------------------|---------|--------------------|
| Light Vivid | 10–20 | 0.04–0.09 | "colored paper" | 7.14 (under) |
| Light Warm | 5–12 | 0.02–0.05 | "warm tint" | 12.73 (over) |
| Light Cool | 5–12 | 0.02–0.05 | "cool tint" | 5.42 (on target) |
| Light Muted | 2–5 | 0.007–0.015 | "barely there" | 4.30 (on target) |

Light Vivid needs to move substantially upward. Light Warm needs to move downward — its current mean overshoots into Vivid territory. Light Cool and Light Muted are already well-placed.

### Dark Categories

Dark backgrounds face a physical constraint: the sRGB gamut limits how much chroma dark colors can carry. Targets are expressed as ΔE ranges rather than raw chroma, since the chroma needed to reach a given ΔE varies with lightness.

| Category | Target ΔE Range | Keyword | Current ΔE Mean |
|----------|----------------|---------|--------------------|
| Dark Vivid | 6–10 | "clearly tinted" + vivid accents | 6.92 (on target) |
| Dark Warm | 6–10 | "warm undertone" | 9.88 (high end) |
| Dark Cool | 4–8 | "cool undertone" | 6.45 (on target) |
| Dark Muted | 1–4 | "near-neutral" | 2.87 (on target) |

Dark categories are mostly well-placed. Dark Vivid compensates for the gamut constraint through accent chroma — this is appropriate and should be preserved. Dark Warm sits at the high end of its range but is not problematic.

### Separation Guarantees

The targets create non-overlapping or minimally overlapping bands:
- **Light Vivid floor (ΔE 10) > Light Warm ceiling (ΔE 12)**: Slight overlap, but Vivid's mean should clearly exceed Warm's mean
- **Light Warm floor (ΔE 5) > Light Muted ceiling (ΔE 5)**: Clean separation
- **Dark Vivid floor (ΔE 6) > Dark Muted ceiling (ΔE 4)**: Clean separation with 2-point gap

No individual palette in a higher-chroma category should fall below the mean of the next-lower category. This is the critical constraint the current collection violates.

## The Dark Vivid Compensation Strategy

Dark Vivid backgrounds cannot reach the chroma levels available to Light Vivid — this is a physics limitation, not a design failure. The strategy for Dark Vivid is two-pronged:

1. **Maximize background chroma within sRGB limits** (target ΔE 6–10)
2. **Lean harder on accent chroma** to carry the "vivid" character

This is already happening: Dark Vivid accent mean (0.1574) is the highest of any category. The current approach is correct; it just needs to be recognized as an intentional compensatory strategy rather than an incidental pattern.

## Invariant Alignment

The proposed targets directly serve existing invariants:

- **Invariant 17** (background dominance): By making backgrounds chromatically distinct across categories, the dominant visual experience — not just the accent punctuation — expresses the Affective Category.
- **Invariant 16** (traceable provenance): Some species may need category reassignment if their Signature Colors cannot produce backgrounds in the target chroma range. This is consistent with the invariant's directive that "when the essence cannot be authentically represented in a given Affective Category, the species belongs in a different category."
- **Invariant 3** (WCAG AA): Higher background chroma makes contrast maintenance harder. Each retuned palette must be verified against the 4.5:1 minimum.

No invariant needs to change. The proposed targets are an operational specification of what the existing invariants already require.

## What Changes

The remediation affects different categories differently:

**Needs significant work:**
- Light Vivid (5 palettes): All backgrounds need higher chroma. Three palettes (Farewell, Camas, Paintbrush) need substantial increases.
- Light Warm (5 palettes): Some backgrounds may need chroma reduction to avoid overshooting into Vivid territory.

**Needs minor adjustment:**
- Dark Warm (5 palettes): Minor, if any. Currently at the high end of its target range.

**Already well-placed:**
- Light Cool, Light Muted, Dark Cool, Dark Muted, Dark Vivid: Category means are within or near their target ranges. Individual outliers may need adjustment.

Each retuned palette must preserve its species' Signature Color associations, maintain WCAG AA contrast ratios, and keep the 15° OKLCH hue diversity within its category. The chroma targets add a new constraint; they do not replace existing ones.
