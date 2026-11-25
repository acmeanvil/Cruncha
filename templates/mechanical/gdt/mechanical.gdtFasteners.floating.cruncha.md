+++
cruncha-template = 1
id = "mechanical.gdtFasteners.floating"
version = "1.0.0"
title = "Floating fastener: clearance holes and positional tolerance (ASME Y14.5 App. B)"
summary = "Bolts and nuts through clearance holes in every part: the positional tolerance at MMC each part's holes may take, T = H − F, and the hole needed for a chosen tolerance, H = F + T, with the chosen tolerance checked so the parts assemble at the worst case."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "floating fastener", "clearance hole", "bolt pattern", "positional tolerance", "MMC", "assembly", "Appendix B", "ASME Y14.5"]
from = "lacewing:mechanical.gdtFasteners"
license = "GPL-3.0-or-later"
inputs = ["F", "H", "T"]
outputs = ["T_av", "H_req"]

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "Nonmandatory Appendix B, B-1 to B-3 (floating fastener formula), Fig. B-1; 10.3.3.2 and Fig. 10-11"

[[tests]]
name = "Y14.5-2018 B-3: Ø6 fasteners in Ø6.44 holes"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Appendix B-3: T = 6.44 − 6 = 0.44; H = 6 + 0.44 = 6.44"
inputs = { F = "6 mm", H = "6.44 mm", T = "0.44 mm" }
expect = { T_av = "0.44 mm", H_req = "6.44 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Y14.5-2018 Fig. 10-11: Ø14 fasteners in Ø14.25 holes"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 10.3.3.2 and Fig. 10-11: Ø0.25 at MMC"
inputs = { F = "14 mm", H = "14.25 mm", T = "0.25 mm" }
expect = { T_av = "0.25 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "B-3's parts with Ø0.5 asked of the holes: too much"
source = "By hand: H = 6 + 0.5 = 6.5 needed, over the 6.44 hole"
inputs = { F = "6 mm", H = "6.44 mm", T = "0.5 mm" }
expect = { H_req = "6.5 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Floating fastener

Two or more parts held by bolts and nuts (or other fasteners) that pass through clearance holes in every part: each fastener floats in its holes (ASME Y14.5-2018 Nonmandatory Appendix B, B-3). With the holes positioned at MMC, the parts assemble at the worst case, every feature at MMC and at the extreme of its positional tolerance, when H = F + T. The formula gives a no-interference, no-clearance fit at that worst case. It applies to each part on its own, so parts with different holes are each worked alone.

The formulas assume the parts' interfacing datum features or other means keep them aligned (B-1). They leave no further clearance for assembly.

### Inputs

```calc {description="Fastener at MMC, its largest diameter"}
F = 6 mm_
```

```calc {description="Clearance hole at MMC, its smallest diameter"}
H = 6.44 mm_
```

```calc {description="Positional tolerance of the holes at MMC (zone diameter)"}
T = 0.44 mm_
```

### Tolerance and hole

```calc
> Positional tolerance the holes may take, B-3
T_av = H - F = ?? mm
> Clearance hole needed for the chosen tolerance, B-3
H_req = F + T = ?? mm
```

### Check

```calc
@test T ≤ T_av
```
