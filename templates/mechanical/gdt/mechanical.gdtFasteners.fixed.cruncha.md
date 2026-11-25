+++
cruncha-template = 1
id = "mechanical.gdtFasteners.fixed"
version = "1.0.0"
title = "Fixed fastener with a projected tolerance zone (ASME Y14.5 App. B)"
summary = "Screws in tapped holes or studs, with a projected tolerance zone on the tapped or tight-fitting holes: the equal positional tolerance each part may take, T = (H − F)/2, the tapped hole's share left by a chosen clearance hole tolerance, and the clearance hole needed, H = F + T₁ + T₂, checked against the hole given."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "fixed fastener", "tapped hole", "stud", "cap screw", "clearance hole", "projected tolerance zone", "positional tolerance", "MMC", "assembly", "Appendix B", "ASME Y14.5"]
from = "lacewing:mechanical.gdtFasteners"
license = "GPL-3.0-or-later"
inputs = ["F", "H", "T_1", "T_2"]
outputs = ["T_eq", "T_(2,av)", "H_req"]

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "Nonmandatory Appendix B, B-1, B-2, B-4 (fixed fastener formula, equal and unequal tolerances), Fig. B-2; 10.4.1 to 10.4.1.3 (projected tolerance zones)"

[[references]]
work = "ISO 2692:2021 Geometrical product specifications (GPS) - Geometrical tolerancing - Maximum material requirement (MMR), least material requirement (LMR) and reciprocity requirement (RPR)"
edition = "2021"
locator = "4.1.2, Formulas (1) and (2); Fig. A.10"

[[tests]]
name = "Y14.5-2018 B-4: Ø6 screws, Ø6.44 clearance holes, equal tolerances"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Appendix B-4: T = (6.44 − 6)/2 = 0.22, half the floating fastener tolerance"
inputs = { F = "6 mm", H = "6.44 mm", T_1 = "0.22 mm", T_2 = "0.22 mm" }
expect = { T_eq = "0.22 mm", "T_(2,av)" = "0.22 mm", H_req = "6.44 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Y14.5-2018 B-4, the total split: 0.18 to the clearance holes leaves 0.26 to the tapped holes"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Appendix B-4: 2T = T₁ + T₂ = 0.44"
inputs = { F = "6 mm", H = "6.44 mm", T_1 = "0.18 mm", T_2 = "0.26 mm" }
expect = { "T_(2,av)" = "0.26 mm", H_req = "6.44 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "ISO 2692 Fig. A.10: pins Ø11.4 0/−0.5 in holes Ø12 +0.5/0, both Ø0.3 Ⓜ"
source = "Lacewing Lab gdt_iso_test, after ISO 2692:2021 Fig. A.10: both virtual conditions are 11.7 (pin 11.4 + 0.3, hole 12 − 0.3), so H = F + T₁ + T₂ = 12 exactly"
inputs = { F = "11.4 mm", H = "12 mm", T_1 = "0.3 mm", T_2 = "0.3 mm" }
expect = { H_req = "12 mm", T_eq = "0.3 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "B-4's screws with 0.2 and 0.4: the 6.44 hole is too small"
source = "Lacewing Lab gdt_iso_test after ASME Y14.5-2018 B-4: H = 6 + 0.2 + 0.4 = 6.6 is needed"
inputs = { F = "6 mm", H = "6.44 mm", T_1 = "0.2 mm", T_2 = "0.4 mm" }
expect = { H_req = "6.6 mm", "T_(2,av)" = "0.24 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Fixed fastener with a projected tolerance zone

One part has clearance holes; the other has tapped or tight-fitting holes that hold the fastener fixed: cap screws, or studs and dowels pressed in (ASME Y14.5-2018 Nonmandatory Appendix B, B-4). The parts assemble at the worst case, every feature at MMC and at the extreme of its positional tolerance, when H = F + T₁ + T₂. With equal tolerances in both parts, T = (H − F)/2: half what a floating fastener gets for the same H and F.

The formula assumes a projected tolerance zone (Ⓟ) on the tapped or tight-fitting holes, of a height at least the maximum thickness of the part with the clearance holes or the stud's installed height (10.4.1). Without one, the fastener's tilt within the tapped hole's zone costs more: use the fixed-fastener-without-projected-zone template. ISO 2692 reaches the same arithmetic from the virtual sizes (the clearance hole's MMVS, H − T₁, at least the fastener's, F + T₂), with Ⓟ from ISO 1101 Clause 13.

### Inputs

```calc {description="Fastener at MMC, its largest diameter"}
F = 6 mm_
```

```calc {description="Clearance hole at MMC, its smallest diameter"}
H = 6.44 mm_
```

```calc {description="Positional tolerance of the clearance holes at MMC"}
T_1 = 0.22 mm_
```

```calc {description="Positional tolerance of the tapped or tight-fitting holes at MMC, projected zone"}
T_2 = 0.22 mm_
```

### Tolerances and hole

```calc
> Equal positional tolerance each part may take, B-4
T_eq = (H - F) / 2 = ?? mm
> Tapped hole tolerance left by the clearance hole's T₁, B-4
T_(2,av) = H - F - T_1 = ?? mm
> Clearance hole needed, B-4
H_req = F + T_1 + T_2 = ?? mm
```

### Check

```calc
@test H_req ≤ H
```
