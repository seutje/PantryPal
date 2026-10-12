# ADR 0009: Practical shopping quantities and alternative measures

Date: 2026-10-12. Status: Accepted. Refines ADR 0006's display-only formatting.

The owner requested upward rounding, reasonable units and alternatives after
seeing an exact conversion displayed as `59 1176473/8000000 mL` for olive oil.
Keep exact rationals for generation, scaling, grouping, stored overrides and
source provenance. Apply the new policy only when Rust builds shopping views
and clipboard text; no schema migration or regeneration is required.

- Whole mL, g, mg and count amounts round upward to the next whole unit.
  Counts are not converted into mass or volume.
- At one litre/kilogram or more, metric mL/g displays use L/kg. Other chosen
  units remain selected. Litres use decimals; kilograms retain simple exact
  fractions when preferred, otherwise decimals.
- Preserve simple exact fractions with denominators 1, 2, 3, 4, 5, 6, 8, 12 or
  16 when fraction display is enabled. More complex fractions in nonmetric
  measures round upward to eighths. Decimal displays round upward with at most
  two decimal places, respecting a lower configured precision. This shopping
  readability policy deliberately caps precision even if other measurement
  displays request more decimal places.
- Rounding uses checked rational ceiling, never floating point or rounding to
  nearest. Zero remains zero; unsupported precision/overflow produces an error.
- Volume alternatives include L/mL, explicitly named US/metric/Imperial cups
  and US/Imperial fluid ounces. Mass alternatives use g/kg/oz/lb. Only registered
  same-dimension universal factors are used: never infer density or package size.
  Avoid redundant large mL/g alternatives when the primary display uses L/kg.
- Each alternative is converted from the exact effective requirement (including
  an override), then independently rounded upward. These are alternative ways
  to measure the requirement, not amounts to add together or exact equalities
  between the rounded displays. UI and copied text label this rounding.
- Unknown/to-taste amounts retain their wording and have no fabricated numeric
  alternatives. Original values remain available in source details and editors.

For the reported quantity the primary display is `60 mL`; alternatives include
`0.06 L`, `¼ US cup` and `2 US fl oz`. Tests cover this regression, regional
variants, upward-rounding invariants across preferences/units, zero/tiny/large
amounts, metric scale boundaries, incompatible dimensions, retained exact totals,
manual overrides, refresh/restart and the same backend formatting in copied text.
