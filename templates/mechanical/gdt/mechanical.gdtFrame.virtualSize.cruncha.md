+++
cruncha-template = 1
id = "mechanical.gdtFrame.virtualSize"
version = "1.0.0"
title = "Maximum and least material virtual sizes (ISO 2692)"
summary = "A toleranced feature of size (a cylinder or two parallel opposite planes) with Ⓜ or Ⓛ in its tolerance indicator: its maximum and least material sizes, the maximum and least material virtual sizes, and the geometrical deviation allowed at its actual size by the requirement chosen, with the size checked within its limits."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "GPS", "ISO 2692", "MMVS", "LMVS", "maximum material requirement", "least material requirement", "MMR", "LMR", "virtual size", "virtual condition", "feature control frame", "tolerance indicator", "ASME Y14.5"]
from = "lacewing:mechanical.gdtFrame"
license = "GPL-3.0-or-later"
inputs = ["hole", "LL", "UL", "δ", "req", "AS"]
outputs = ["MMS", "LMS", "MMVS", "LMVS", "δ_M", "δ_L", "δ_a"]

[[references]]
work = "ISO 2692:2021 Geometrical product specifications (GPS) - Geometrical tolerancing - Maximum material requirement (MMR), least material requirement (LMR) and reciprocity requirement (RPR)"
edition = "2021"
locator = "3.4 to 3.16 (sizes and virtual sizes); 4.1.2 Formulas (1) to (4); rules A to D and H to K; Annex A Figs. A.1, A.2, A.5, A.12, A.13"

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "3.70 (virtual condition); 5.9; Fig. 5-14"

[[tests]]
name = "ISO 2692 Fig. A.1 a): pins Ø10 0/−0.2, position Ø0.3 Ⓜ"
source = "Lacewing Lab iso_material_requirements_test, after ISO 2692:2021 Fig. A.1 a): MMS 10, LMS 9.8, MMVS 10.3; at a local size of 9.9 the deviation allowed is 10.3 − 9.9 = 0.4 (derived)"
inputs = { hole = "0", LL = "9.8 mm", UL = "10 mm", "δ" = "0.3 mm", req = "1", AS = "9.9 mm" }
expect = { MMS = "10 mm", LMS = "9.8 mm", MMVS = "10.3 mm", "δ_a" = "0.4 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "ISO 2692 Fig. A.2 a): straightness Ø0.1 Ⓜ on a Ø35 0/−0.1 shaft"
source = "Lacewing Lab iso_material_requirements_test, after ISO 2692:2021 Fig. A.2 a): MMVS 35.1"
inputs = { hole = "0", LL = "34.9 mm", UL = "35 mm", "δ" = "0.1 mm", req = "1", AS = "35 mm" }
expect = { MMVS = "35.1 mm", "δ_a" = "0.1 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "ISO 2692 Fig. A.5 a): coaxiality Ø0.1 Ⓛ on a Ø70 0/−0.1 shaft"
source = "Lacewing Lab iso_material_requirements_test, after ISO 2692:2021 Fig. A.5 a): LMVS 69.8"
inputs = { hole = "0", LL = "69.9 mm", UL = "70 mm", "δ" = "0.1 mm", req = "2", AS = "70 mm" }
expect = { LMS = "69.9 mm", LMVS = "69.8 mm", "δ_a" = "0.2 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "ISO 2692 Fig. A.5 b): coaxiality Ø0.1 Ⓛ on a Ø35 +0.1/0 hole"
source = "Lacewing Lab iso_material_requirements_test, after ISO 2692:2021 Fig. A.5 b): LMVS 35.2"
inputs = { hole = "1", LL = "35 mm", UL = "35.1 mm", "δ" = "0.1 mm", req = "2", AS = "35.1 mm" }
expect = { LMS = "35.1 mm", LMVS = "35.2 mm", "δ_a" = "0.1 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "ISO 2692 Fig. A.12: hole Ø10 ± 0.1, position Ø0.1 Ⓜ"
source = "Lacewing Lab iso_material_requirements_test, after ISO 2692:2021 Fig. A.12: MMVS 9.8"
inputs = { hole = "1", LL = "9.9 mm", UL = "10.1 mm", "δ" = "0.1 mm", req = "1", AS = "10 mm" }
expect = { MMS = "9.9 mm", MMVS = "9.8 mm", "δ_a" = "0.2 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "ISO 2692 Fig. A.13: 4× holes Ø8 +0.1/0, position Ø0.5 Ⓜ"
source = "Lacewing Lab iso_material_requirements_test, after ISO 2692:2021 Fig. A.13: MMVS 7.5"
inputs = { hole = "1", LL = "8 mm", UL = "8.1 mm", "δ" = "0.5 mm", req = "1", AS = "8.05 mm" }
expect = { MMVS = "7.5 mm", LMVS = "8.6 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Y14.5-2018 Fig. 5-14's hole read by ISO: MMVS is ASME's virtual condition"
source = "Lacewing Lab iso_material_requirements_test: Ø30.1-30.5, Ø0.1 Ⓜ gives MMVS 30.0, and 0.3 is allowed at 30.3, as ASME's bonus gives"
inputs = { hole = "1", LL = "30.1 mm", UL = "30.5 mm", "δ" = "0.1 mm", req = "1", AS = "30.3 mm" }
expect = { MMVS = "30 mm", "δ_M" = "0.3 mm", "δ_a" = "0.3 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Fig. A.1 a)'s pin at a local size of 10.05: over its MMS"
source = "ISO 2692:2021 rule A: the local sizes keep between MMS and LMS (without Ⓡ)"
inputs = { hole = "0", LL = "9.8 mm", UL = "10 mm", "δ" = "0.3 mm", req = "1", AS = "10.05 mm" }
expect = { MMVS = "10.3 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Maximum and least material virtual sizes

A geometrical tolerance with Ⓜ (the maximum material requirement) or Ⓛ (the least material requirement) in its tolerance indicator ties the feature's size and its geometrical deviation together through a virtual condition: a perfect form of the virtual size that the feature must not violate (ISO 2692:2021 4.1, rules A to D for Ⓜ and H to K for Ⓛ). Ⓜ or Ⓛ applies only to a derived feature of a cylinder or of two parallel opposite planes. The local sizes still keep within MMS and LMS (rules A and H) unless Ⓡ, the reciprocity requirement, is added. Without either modifier the tolerance holds at every size (the independency principle of ISO 8015).

For a toleranced feature, MMVS is the same number as ASME Y14.5's virtual condition at MMC, and LMVS its virtual condition at LMC. A datum feature with Ⓜ after its letter differs: by rule F its MMVS is its MMS unless rule G brings its own tolerance in; this template is for the toleranced feature. Whether the frame itself is legal, and what it means, is the feature control frame builder's work in Lacewing Lab and not a calculation.

### Feature

```calc {description="1 for an internal feature (hole, slot), 0 for an external one (shaft, tab)"}
hole = 0_
```

```calc {description="Lower limit of size"}
LL = 9.8 mm_
```

```calc {description="Upper limit of size"}
UL = 10 mm_
```

```calc {description="Geometrical tolerance in the indicator"}
δ = 0.3 mm_
```

```calc {description="Requirement after the tolerance: 1 for Ⓜ, 2 for Ⓛ"}
req = 1_
```

```calc {description="Actual local size of the produced feature"}
AS = 9.9 mm_
```

### Sizes

The maximum material size is the upper limit of an external feature and the lower limit of an internal one; the least material size the other (ISO 2692 3.4 to 3.7).

```calc
> Maximum material size
MMS = { LL if hole == 1; UL otherwise } = ?? mm
> Least material size
LMS = { UL if hole == 1; LL otherwise } = ?? mm
> Maximum material virtual size, Formulas (1) and (2)
MMVS = { MMS - δ if hole == 1; MMS + δ otherwise } = ?? mm
> Least material virtual size, Formulas (3) and (4)
LMVS = { LMS + δ if hole == 1; LMS - δ otherwise } = ?? mm
```

### Deviation allowed at the actual size

What a feature of perfect form at the actual size may deviate and still keep within the virtual condition: its distance from MMVS with Ⓜ, from LMVS with Ⓛ. ISO 2692 states the boundary, not this number; it follows from rules C and J.

```calc
> Allowed with Ⓜ
δ_M = abs(MMVS - AS) = ?? mm
> Allowed with Ⓛ
δ_L = abs(LMVS - AS) = ?? mm
> Allowed by the requirement in the indicator
δ_a = { δ_M if req == 1; δ_L if req == 2 } = ?? mm
```

### Check

```calc
@test LL ≤ AS ≤ UL
```
