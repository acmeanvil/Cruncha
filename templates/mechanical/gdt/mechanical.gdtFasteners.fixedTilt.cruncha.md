+++
cruncha-template = 1
id = "mechanical.gdtFasteners.fixedTilt"
version = "1.0.0"
title = "Fixed fastener without a projected tolerance zone (ASME Y14.5 App. B)"
summary = "Screws in tapped holes or studs whose tapped holes have no projected tolerance zone, so the fastener can tilt within the tapped hole's zone over the part's thickness: the clearance hole needed, H = F + T₁ + T₂(1 + 2P/D), the tolerances it leaves, and the smaller hole a projected zone would allow."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "fixed fastener", "tapped hole", "stud", "cap screw", "clearance hole", "tilt", "thread engagement", "projected tolerance zone", "positional tolerance", "MMC", "Appendix B", "ASME Y14.5"]
from = "lacewing:mechanical.gdtFasteners"
license = "GPL-3.0-or-later"
inputs = ["F", "H", "T_1", "T_2", "P", "D"]
outputs = ["k", "H_req", "H_proj", "T_(2,av)", "T_(1,av)", "T_eq"]

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "Nonmandatory Appendix B, B-1, B-5 (fixed fastener without a projected tolerance zone), Fig. B-2; 10.4.1 (projected tolerance zones)"

[[tests]]
name = "Y14.5-2018 B-5: Ø6 screws, T₁ 0.2, T₂ 0.4, a 12 thick plate on 8 of thread"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Appendix B-5: H = 6 + 0.2 + 0.4(1 + 2·12/8) = 7.8"
inputs = { F = "6 mm", H = "7.8 mm", T_1 = "0.2 mm", T_2 = "0.4 mm", P = "12 mm", D = "8 mm" }
expect = { k = "4", H_req = "7.8 mm", H_proj = "6.6 mm", "T_(2,av)" = "0.4 mm", "T_(1,av)" = "0.2 mm", T_eq = "0.36 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "B-5's parts in a Ø7.5 hole: too small without a projected zone"
source = "By hand from B-5: 7.8 is needed; a projected zone (B-4) would need only 6.6"
inputs = { F = "6 mm", H = "7.5 mm", T_1 = "0.2 mm", T_2 = "0.4 mm", P = "12 mm", D = "8 mm" }
expect = { H_req = "7.8 mm", H_proj = "6.6 mm", "T_(2,av)" = "0.325 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Fixed fastener without a projected tolerance zone

A fastener fixed in a tapped or tight-fitting hole whose positional tolerance has no projected zone: the hole's axis may lean anywhere within its zone over the depth of engagement, and the fastener carries that lean up through the part with the clearance hole (ASME Y14.5-2018 Nonmandatory Appendix B, B-5). Over the projection P the lean grows by 2P/D times the tapped hole's tolerance, so the clearance hole must be enlarged: H = F + T₁ + T₂(1 + 2P/D). A projected tolerance zone on the tapped hole (10.4.1) removes the term; H_proj shows the hole it would allow.

With equal tolerances T₁ = T₂ = T in both parts, the formula is solved as H = F + T(2 + 2P/D). ISO 2692 has no counterpart of the tilt term; under ISO, specify Ⓟ on the tapped hole (ISO 1101 Clause 13) and use the projected-zone arithmetic.

### Inputs

```calc {description="Fastener at MMC, its largest diameter"}
F = 6 mm_
```

```calc {description="Clearance hole at MMC, its smallest diameter"}
H = 7.8 mm_
```

```calc {description="Positional tolerance of the clearance holes at MMC"}
T_1 = 0.2 mm_
```

```calc {description="Positional tolerance of the tapped or tight-fitting holes at MMC, no projected zone"}
T_2 = 0.4 mm_
```

```calc {description="Maximum projection: the thickness of the part with the clearance hole, or the stud's height"}
P = 12 mm_
```

```calc {description="Minimum depth of thread or engagement of the fixed fastener"}
D = 8 mm_
```

### Hole and tolerances

```calc
> What the tapped hole's tolerance costs at the clearance hole, B-5
k = 1 + 2 P / D = ??
> Clearance hole needed, B-5
H_req = F + T_1 + T_2 k = ?? mm
> Clearance hole a projected zone on the tapped hole would allow, B-4
H_proj = F + T_1 + T_2 = ?? mm
> Tapped hole tolerance left by the clearance hole's T₁
T_(2,av) = (H - F - T_1) / k = ?? mm
> Clearance hole tolerance left by the tapped hole's T₂
T_(1,av) = H - F - T_2 k = ?? mm
> Equal tolerance in both parts, B-5 with T₁ = T₂
T_eq = (H - F) / (1 + k) = ?? mm
```

### Check

```calc
@test H_req ≤ H
```
