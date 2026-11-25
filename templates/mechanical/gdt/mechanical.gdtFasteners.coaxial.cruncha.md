+++
cruncha-template = 1
id = "mechanical.gdtFasteners.coaxial"
version = "1.0.0"
title = "Coaxial features: positional tolerance of two mating bores and diameters (ASME Y14.5 App. B)"
summary = "A part with two coaxial bores mating with a stepped shaft, one feature the datum for the other: the total positional tolerance available for both parts, T₁ + T₂ = (H₁ + H₂) − (F₁ + F₂), the second part's share left by the first's, and the chosen split checked against the total."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "coaxial features", "coaxiality", "stepped shaft", "bore", "positional tolerance", "MMC", "datum", "assembly", "Appendix B", "ASME Y14.5", "ISO 2692"]
from = "lacewing:mechanical.gdtFasteners"
license = "GPL-3.0-or-later"
inputs = ["H_1", "H_2", "F_1", "F_2", "T_1", "T_2"]
outputs = ["T_tot", "T_(2,av)", "T_ISO"]

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "Nonmandatory Appendix B, B-6 (coaxial features), Fig. B-3"

[[references]]
work = "ISO 2692:2021 Geometrical product specifications (GPS) - Geometrical tolerancing - Maximum material requirement (MMR), least material requirement (LMR) and reciprocity requirement (RPR)"
edition = "2021"
locator = "4.1.2 Formulas (1) and (2); 4.2.2 rule F (a datum feature at Ⓜ: its MMVS is its MMS)"

[[tests]]
name = "Y14.5-2018 Fig. B-3: bores Ø20 and Ø10 on shaft diameters Ø19.89 and Ø9.92, 0.12 to the datum part"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Appendix B-6 and Fig. B-3: (20 + 10) − (19.89 + 9.92) = 0.19, split 0.12 and 0.07"
inputs = { H_1 = "20 mm", H_2 = "10 mm", F_1 = "19.89 mm", F_2 = "9.92 mm", T_1 = "0.12 mm", T_2 = "0.07 mm" }
expect = { T_tot = "0.19 mm", "T_(2,av)" = "0.07 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Fig. B-3's parts, split in half; ISO's MMVC pairs leave 0.08"
source = "Lacewing Lab tolerance_calcs_test and gdt_iso_test: 0.095 each; with the datum at Ⓜ ISO 2692 rule F gives T₁ + T₂ = H₂ − F₂ = 0.08"
inputs = { H_1 = "20 mm", H_2 = "10 mm", F_1 = "19.89 mm", F_2 = "9.92 mm", T_1 = "0.095 mm", T_2 = "0.095 mm" }
expect = { T_tot = "0.19 mm", "T_(2,av)" = "0.095 mm", T_ISO = "0.08 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Fig. B-3's parts asked for 0.12 and 0.10: more than the 0.19 available"
source = "By hand from B-6: 0.22 over 0.19"
inputs = { H_1 = "20 mm", H_2 = "10 mm", F_1 = "19.89 mm", F_2 = "9.92 mm", T_1 = "0.12 mm", T_2 = "0.10 mm" }
expect = { T_tot = "0.19 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Coaxial features

A part with two coaxial bores mates with a part with two coaxial diameters (a stepped shaft): the first pair is the datum feature for the second on each part (ASME Y14.5-2018 Nonmandatory Appendix B, B-6). At MMC the tolerance available to both parts together is the sum of the clearances, T₁ + T₂ = (H₁ + H₂) − (F₁ + F₂), divided between them as the designer chooses. Appendix B notes the formula holds for simple coaxial parts of similar length.

ISO 2692 has no fastener formulas. With the datum at Ⓜ, rule F takes the datum feature's MMVS as its MMS, so each part is a pair of coaxial virtual conditions: they fit when H₁ ≥ F₁ and H₂ − T₁ ≥ F₂ + T₂, so T₁ + T₂ = H₂ − F₂. B-6 also spends the datum clearance H₁ − F₁, because ASME keeps the datum sizes within their limits. T_ISO is that ISO figure, derived from the virtual sizes rather than given by the standard.

### Inputs

```calc {description="First (datum) bore at MMC, its smallest diameter"}
H_1 = 20 mm_
```

```calc {description="Second bore at MMC"}
H_2 = 10 mm_
```

```calc {description="First (datum) shaft diameter at MMC, its largest"}
F_1 = 19.89 mm_
```

```calc {description="Second shaft diameter at MMC"}
F_2 = 9.92 mm_
```

```calc {description="Positional tolerance at MMC of the part with the bores"}
T_1 = 0.12 mm_
```

```calc {description="Positional tolerance at MMC of the stepped shaft"}
T_2 = 0.07 mm_
```

### Tolerance available

```calc
> Total positional tolerance for both parts, B-6
T_tot = (H_1 + H_2) - (F_1 + F_2) = ?? mm
> The shaft's share left by the bores' T₁
T_(2,av) = T_tot - T_1 = ?? mm
> The total by ISO 2692's virtual conditions, datum at Ⓜ (derived)
T_ISO = H_2 - F_2 = ?? mm
```

### Check

```calc
@test T_1 + T_2 ≤ T_tot
```
