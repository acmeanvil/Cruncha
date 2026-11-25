+++
cruncha-template = 1
id = "mechanical.gdtBonus.mmc"
version = "1.0.0"
title = "Bonus tolerance and virtual condition at MMC (ASME Y14.5)"
summary = "A hole, slot, shaft or tab with a geometric tolerance at maximum material condition: its MMC and LMC sizes, the bonus a produced size earns and the tolerance it is then allowed (capped by a MAX if the frame has one), the virtual and resultant conditions and the inner and outer boundaries."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "bonus tolerance", "MMC", "maximum material condition", "virtual condition", "resultant condition", "inner boundary", "outer boundary", "feature of size", "position", "functional gauge", "ASME Y14.5"]
from = "lacewing:mechanical.gdtBonus"
license = "GPL-3.0-or-later"
inputs = ["hole", "LL", "UL", "t", "t_MAX", "AS"]
outputs = ["MMC", "LMC", "b", "t_a", "VC", "t_LMC", "RC", "IB", "OB"]

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "3.2, 3.5, 3.40, 3.41, 3.54, 3.70 (definitions); 5.9.4, 5.9.4.2 (MMC, bonus, MAX); Figs. 5-14 and 5-17"

[[references]]
work = "ASME Y14.5.1-2019 Mathematical Definition of Dimensioning and Tolerancing Principles"
edition = "2019"
locator = "Table 7-3 (positional zone at MMC, axis method)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 20-6, Fig. 20-33"

[[tests]]
name = "Y14.5-2018 Fig. 5-14: Ø30.1-30.5 hole, Ø0.1 at MMC, produced at 30.3"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Fig. 5-14: MMC 30.1, LMC 30.5, VC 30.0, RC 31.0; at 30.3 the tolerance is 0.3"
inputs = { hole = "1", LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", t_MAX = "0 mm", AS = "30.3 mm" }
expect = { MMC = "30.1 mm", LMC = "30.5 mm", b = "0.2 mm", t_a = "0.3 mm", VC = "30.0 mm", t_LMC = "0.5 mm", RC = "31.0 mm", IB = "30.0 mm", OB = "31.0 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Y14.5-2018 Fig. 5-17: Ø29.5-29.9 pin, Ø0.1 at MMC, produced at LMC"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Fig. 5-17: MMC 29.9, VC 30.0, RC 29.0; 0.5 allowed at 29.5"
inputs = { hole = "0", LL = "29.5 mm", UL = "29.9 mm", t = "0.1 mm", t_MAX = "0 mm", AS = "29.5 mm" }
expect = { MMC = "29.9 mm", LMC = "29.5 mm", b = "0.4 mm", t_a = "0.5 mm", VC = "30.0 mm", RC = "29.0 mm", IB = "29.0 mm", OB = "30.0 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Fig. 5-14's hole with a MAX of 0.3, produced at LMC"
source = "Lacewing Lab tolerance_calcs_test after ASME Y14.5-2018 5.9.4.2: the MAX caps the tolerance at 0.3, so at 30.5 it is 0.3 and RC = 30.5 + 0.3 = 30.8"
inputs = { hole = "1", LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", t_MAX = "0.3 mm", AS = "30.5 mm" }
expect = { b = "0.4 mm", t_a = "0.3 mm", VC = "30.0 mm", RC = "30.8 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Shigley Fig. 20-33: boss Ø50.0-50.3, Ø0.2 at MMC, produced at 50.2"
source = "Lacewing Lab tolerance_calcs_test, after Shigley 10th Fig. 20-33: bonus 0.1, tolerance 0.3"
inputs = { hole = "0", LL = "50.0 mm", UL = "50.3 mm", t = "0.2 mm", t_MAX = "0 mm", AS = "50.2 mm" }
expect = { b = "0.1 mm", t_a = "0.3 mm", VC = "50.5 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Shigley Fig. 20-33: hole Ø30.0-30.3, Ø0.2 at MMC, produced at 30.1"
source = "Lacewing Lab tolerance_calcs_test, after Shigley 10th Fig. 20-33: tolerance 0.3 at 30.1 (0.5 at 30.3)"
inputs = { hole = "1", LL = "30.0 mm", UL = "30.3 mm", t = "0.2 mm", t_MAX = "0 mm", AS = "30.1 mm" }
expect = { t_a = "0.3 mm", t_LMC = "0.5 mm", VC = "29.8 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Fig. 5-14's hole produced at 30.6: outside its limits of size"
source = "ASME Y14.5-2018 Fig. 5-14: the size limits are 30.1 to 30.5, so 30.6 is rejected whatever its position"
inputs = { hole = "1", LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", t_MAX = "0 mm", AS = "30.6 mm" }
expect = { b = "0.5 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Bonus tolerance and virtual condition at MMC

A feature of size (a hole or slot, a shaft or tab) carries a geometric tolerance that applies at maximum material condition: Ⓜ after the tolerance in the feature control frame, typically a position, orientation or axis straightness tolerance (ASME Y14.5-2018 5.9.4). The stated tolerance holds when the feature is at MMC; as the produced size departs from MMC towards LMC, the tolerance grows by the same amount, the bonus. A MAX after the tolerance caps it.

The bonus is taken by the axis method (5.9.4.2; Y14.5.1-2019 Table 7-3): the zone grows by the departure of the actual size from MMC. The actual size is the unrelated actual mating envelope, the largest pin that enters the hole or the smallest ring over the shaft, not a two-point measurement. The surface method, the feature not violating its virtual condition, takes precedence; the two agree for a feature of perfect form.

### Feature

```calc {description="1 for an internal feature (hole, slot), 0 for an external one (shaft, tab)"}
hole = 1_
```

```calc {description="Lower limit of size"}
LL = 30.1 mm_
```

```calc {description="Upper limit of size"}
UL = 30.5 mm_
```

```calc {description="Geometric tolerance in the frame, at MMC"}
t = 0.1 mm_
```

```calc {description="MAX after the tolerance, the cap on it (0 for none)"}
t_MAX = 0 mm_
```

```calc {description="Actual size: the unrelated actual mating envelope"}
AS = 30.3 mm_
```

### Material conditions

MMC is the size with the most material, the smallest hole or the largest shaft; LMC the size with the least (3.41, 3.40).

```calc
> Maximum material condition size
MMC = { LL if hole == 1; UL otherwise } = ?? mm
> Least material condition size
LMC = { UL if hole == 1; LL otherwise } = ?? mm
```

### Tolerance at the actual size

```calc
> Bonus, the actual size's departure from MMC
b = abs(AS - MMC) = ?? mm
> Tolerance allowed at the actual size, no more than MAX
t_a = { t + b if t_MAX == 0 'mm'; min(t + b, t_MAX) otherwise } = ?? mm
```

### Boundaries

The virtual condition is the constant boundary made by MMC and the tolerance at MMC (3.70): the size of a functional gauge pin or ring, and the boundary a mating part must clear. The resultant condition is the worst-case boundary on the other side (3.54), at LMC with the whole tolerance allowed there. The inner and outer boundaries are the smaller and larger of the two (3.2, 3.5).

```calc
> Virtual condition
VC = { MMC - t if hole == 1; MMC + t otherwise } = ?? mm
> Tolerance allowed at LMC
t_LMC = { t + abs(LMC - MMC) if t_MAX == 0 'mm'; min(t + abs(LMC - MMC), t_MAX) otherwise } = ?? mm
> Resultant condition
RC = { LMC + t_LMC if hole == 1; LMC - t_LMC otherwise } = ?? mm
> Inner boundary
IB = min(VC, RC) = ?? mm
> Outer boundary
OB = max(VC, RC) = ?? mm
```

### Check

The bonus counts only for a feature within its limits of size. Whether the feature's position or orientation is within t_a is a measurement: see the true position template.

```calc
@test LL ≤ AS ≤ UL
```
