# ADR-017: Flora Reference as Botanical Source of Truth

**Status:** Proposed

## Context

Essay 004's validation revealed a systematic problem: Provenance Descriptions had drifted toward rationalizing palette colors rather than documenting species colors. Balsamroot's provenance says "warm amber-gold" when field guides say "bright yellow." Witch's Hair says "olive-black" when *A. sarmentosa* is pale yellow-green. The descriptions were written after the palette colors were chosen, creating a circular reference where the evidence for the naming register's color congruence was derived from the palette itself.

Invariant 16 (strengthened after Essay 004) now requires Signature Colors to be "verified against external botanical sources." The domain model defines Flora Reference as a "structured reference document documenting each species' Signature Colors with source citations, Essence Mode, geographic range, and provenance notes." But no such document exists. The botanical knowledge accumulated during Essay 004's audit (40 species researched against Oregon Flora Project, USDA PLANTS, USFS Silvics/FEIS, Burke Herbarium, McCune & Geiser, regional field guides) lives only in the research log and essay — artifacts that are hard to consult during routine palette work.

The user identified this gap directly: "keeping our sort of corpus handy enables that reflection."

## Decision

Create `docs/flora-reference.md` as the durable botanical corpus for all species in the Palette collection. The Flora Reference:

1. **Documents each species independently of its palette.** Signature Colors, Essence Mode, geographic range, and source citations are recorded based on what the species actually looks like — not what the palette renders.

2. **Is the source of truth for Provenance Descriptions.** When writing or updating a Provenance Description in code, the Flora Reference is consulted first. Provenance text must be consistent with the Flora Reference entry.

3. **Records the compositional rationale.** For each species, the entry documents which Signature Color occupies which palette slot and why (the Species Essence and Essence Mode). This makes the design reasoning traceable rather than implicit.

4. **Is consulted before species decisions.** When considering new species, replacements, or category moves, the Flora Reference provides the baseline for comparison.

The reference is structured per-species with these fields:

- **Species name** (common and scientific)
- **Signature Colors** with source citations
- **Essence Mode** (Feature, Throughline, or Place) with rationale
- **Geographic range** within Cascadia
- **Palette assignment** (which Affective Category, which slots use which colors)
- **Validation score** (A/B/C/D from most recent audit)
- **Notes** (seasonal variation, provenance pitfalls, related species)

The Flora Reference is a documentation artifact, not a code artifact. It lives alongside the domain model and essays, not in `src/`.

**Rejected alternatives:**

- **Embed botanical data in code comments.** Provenance Descriptions in `collection.rs` already serve this role partially, but they are constrained to one line and are coupled to the palette definition. The Flora Reference provides the full context that a one-line description cannot.
- **Rely on the research log and essay.** These are chronological artifacts documenting a research process. They are not structured for lookup. Finding "what are Licorice Fern's real colors?" requires reading through a narrative rather than consulting a reference entry.
- **Create a structured data file (TOML/JSON).** The Flora Reference is primarily for human consultation during design work, not machine parsing. Markdown is the right format for a document that mixes prose rationale with structured data.

## Consequences

**Positive:**
- Breaks the circular reference: botanical knowledge exists independently of palette code, so Provenance Descriptions can be validated against an external source.
- Accumulated knowledge from Essay 004's 40-species audit is preserved in a consultable format rather than buried in chronological research artifacts.
- Future palette work (species replacements, provenance corrections, new designs) has a canonical reference to consult.
- The Flora Reference enables the periodic validation cadence described in domain model Open Question 12.

**Negative:**
- Maintenance burden: when species are added or replaced, both the Flora Reference and the palette code must be updated.
- Initial creation requires structuring the Essay 004 audit findings into per-species entries — approximately 40 entries.

**Neutral:**
- The Flora Reference does not change any code. It is purely a documentation artifact that informs code decisions.
- The relationship between Flora Reference and Provenance Description is advisory, not enforced by tooling. Consistency depends on the design process consulting the reference.
