+++
cruncha-template = 1
id = "mechanical.gdtBonus.lmc"
version = "1.0.0"
title = "Bonus tolerance and virtual condition at LMC (ASME Y14.5)"
summary = "A hole, slot, shaft or tab with a geometric tolerance at least material condition, as used to keep a minimum wall: its MMC and LMC sizes, the bonus a produced size earns and the tolerance it is then allowed (capped by a MAX if the frame has one), the virtual and resultant conditions and the inner and outer boundaries."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "bonus tolerance", "LMC", "least material condition", "virtual condition", "resultant condition", "inner boundary", "outer boundary", "minimum wall", "feature of size", "ASME Y14.5"]
from = "lacewing:mechanical.gdtBonus"
license = "GPL-3.0-or-later"
inputs = ["hole", "LL", "UL", "t", "t_MAX", "AS"]
outputs = ["MMC", "LMC", "b", "t_a", "VC", "t_MMC", "RC", "IB", "OB"]

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "3.2, 3.5, 3.40, 3.41, 3.54, 3.70 (definitions); 5.9.6, 5.9.6.2 (LMC, bonus, MAX); Figs. 5-15 and 5-18"

[[references]]
work = "ASME Y14.5.1-2019 Mathematical Definition of Dimensioning and Tolerancing Principles"
edition = "2019"
locator = "Table 7-3 (positional zone at LMC, axis method)"

[[tests]]
name = "Y14.5-2018 Fig. 5-15: Ø30.1-30.5 hole, Ø0.1 at LMC, produced at 30.3"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Fig. 5-15: VC 30.6, RC 29.6; 0.3 allowed at 30.3, 0.5 at MMC"
inputs = { hole = "1", LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", t_MAX = "0 mm", AS = "30.3 mm" }
expect = { MMC = "30.1 mm", LMC = "30.5 mm", b = "0.2 mm", t_a = "0.3 mm", VC = "30.6 mm", t_MMC = "0.5 mm", RC = "29.6 mm", IB = "29.6 mm", OB = "30.6 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Y14.5-2018 Fig. 5-18: Ø29.5-29.9 pin, Ø0.1 at LMC, produced at MMC"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Fig. 5-18: VC 29.4, RC 30.4; its table runs from 0.5 at MMC to 0.1 at LMC"
inputs = { hole = "0", LL = "29.5 mm", UL = "29.9 mm", t = "0.1 mm", t_MAX = "0 mm", AS = "29.9 mm" }
expect = { MMC = "29.9 mm", LMC = "29.5 mm", b = "0.4 mm", t_a = "0.5 mm", VC = "29.4 mm", RC = "30.4 mm", IB = "29.4 mm", OB = "30.4 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Fig. 5-18's pin at 29.7, the middle of its table"
source = "ASME Y14.5-2018 Fig. 5-18 table: 0.3 allowed at 29.7"
inputs = { hole = "0", LL = "29.5 mm", UL = "29.9 mm", t = "0.1 mm", t_MAX = "0 mm", AS = "29.7 mm" }
expect = { b = "0.2 mm", t_a = "0.3 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: Fig. 5-15's hole with a MAX of 0.25"
source = "By hand: at 30.1 the bonus 0.4 would give 0.5, capped at 0.25; RC = MMC − 0.25 = 29.85"
inputs = { hole = "1", LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", t_MAX = "0.25 mm", AS = "30.1 mm" }
expect = { t_a = "0.25 mm", t_MMC = "0.25 mm", RC = "29.85 mm", VC = "30.6 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Fig. 5-18's pin produced at 29.4: undersize"
source = "ASME Y14.5-2018 Fig. 5-18: the size limits are 29.5 to 29.9"
inputs = { hole = "0", LL = "29.5 mm", UL = "29.9 mm", t = "0.1 mm", t_MAX = "0 mm", AS = "29.4 mm" }
expect = { b = "0.1 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Bonus tolerance and virtual condition at LMC

A feature of size carries a geometric tolerance at least material condition: Ⓛ after the tolerance in the feature control frame (ASME Y14.5-2018 5.9.6). LMC is used to protect a minimum wall or edge distance. The stated tolerance holds when the feature is at LMC; as the produced size moves towards MMC, the tolerance grows by the same amount, the bonus. A MAX after the tolerance caps it.

The bonus is taken by the axis method (5.9.6.2; Y14.5.1-2019 Table 7-3). The actual size here is the unrelated actual minimum material envelope: the smallest cylinder that contains the hole's material side, or the largest inside the shaft.

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

```calc {description="Geometric tolerance in the frame, at LMC"}
t = 0.1 mm_
```

```calc {description="MAX after the tolerance, the cap on it (0 for none)"}
t_MAX = 0 mm_
```

```calc {description="Actual size: the unrelated actual minimum material envelope"}
AS = 30.3 mm_
```

### Material conditions

```calc
> Maximum material condition size
MMC = { LL if hole == 1; UL otherwise } = ?? mm
> Least material condition size
LMC = { UL if hole == 1; LL otherwise } = ?? mm
```

### Tolerance at the actual size

```calc
> Bonus, the actual size's departure from LMC
b = abs(LMC - AS) = ?? mm
> Tolerance allowed at the actual size, no more than MAX
t_a = { t + b if t_MAX == 0 'mm'; min(t + b, t_MAX) otherwise } = ?? mm
```

### Boundaries

The virtual condition is the constant boundary of LMC and the tolerance at LMC (3.70), which stays inside the material. The resultant condition is the worst-case boundary on the other side (3.54), at MMC with the whole tolerance allowed there.

```calc
> Virtual condition
VC = { LMC + t if hole == 1; LMC - t otherwise } = ?? mm
> Tolerance allowed at MMC
t_MMC = { t + abs(MMC - LMC) if t_MAX == 0 'mm'; min(t + abs(MMC - LMC), t_MAX) otherwise } = ?? mm
> Resultant condition
RC = { MMC - t_MMC if hole == 1; MMC + t_MMC otherwise } = ?? mm
> Inner boundary
IB = min(VC, RC) = ?? mm
> Outer boundary
OB = max(VC, RC) = ?? mm
```

### Check

The bonus counts only for a feature within its limits of size.

```calc
@test LL ≤ AS ≤ UL
```
