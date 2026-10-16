# ADR 0013: Preference-based import unit defaults

Date: 2026-10-16

Status: Accepted (owner request)

The owner requested automatic recognition of common unit spellings and US-centric
defaults, with metric equivalents when measurement mode is metric. This intentionally
replaces the previous rule that every unqualified imported cup required resolution.

All import formats share a Rust whole-token unit matcher. Singulars, plurals,
abbreviations and trailing abbreviation periods resolve to existing unit IDs.
Unqualified cups, tablespoons and teaspoons use US variants in mixed/US mode and
metric variants in metric mode. Fluid ounces, pints, quarts and gallons default
to US because the registry has no metric variants of those named units.

Explicit source variants and units remain authoritative: metric grams do not
become ounces, and pounds do not become grams by changing the unit ID. This
change resolves names and regional ambiguity; it does not convert numeric values.
Source amounts and raw wording are preserved, and all choices remain editable
before saving. Per the owner?s follow-up, unresolved or missing units default
to Piece in every mode. The original unknown unit, package size and ingredient
description remain in the raw text and review note. Missing/to-taste quantities
remain unknown; no count is invented. This is an editable import default, not
a conversion rule between pieces, mass or volume.

The persisted measurement mode is read by import preview. A visible warning
explains the applied default. The former cup review preference control is removed;
its stored value is retained for backup compatibility but does not override mode.
The general measurement conversion API still rejects ambiguous unit aliases.
No migrations, browser capture code, capabilities or external access changes are
needed.

The regression fixture records ingredient measurement facts from the owner's
[example recipe](https://www.spendwithpennies.com/philly-cheesesteak-casserole/),
inspected on 2026-10-16. It omits publisher instructions and uses a synthetic test
instruction. Tests feed this deterministic website-capture-shaped fixture through
production import preview in all three modes; live WebView extraction is not
claimed by these tests.
