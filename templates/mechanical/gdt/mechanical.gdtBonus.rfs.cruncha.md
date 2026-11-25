+++
cruncha-template = 1
id = "mechanical.gdtBonus.rfs"
version = "1.0.0"
title = "Inner and outer boundaries, regardless of feature size (ASME Y14.5)"
summary = "A hole, slot, shaft or tab with a geometric tolerance regardless of feature size (no modifier): the tolerance holds at every size, with no bonus and no virtual condition, and the inner and outer boundaries are the smallest size less the tolerance and the largest size plus it."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "RFS", "regardless of feature size", "inner boundary", "outer boundary", "feature of size", "position", "ASME Y14.5"]
from = "lacewing:mechanical.gdtBonus"
license = "GPL-3.0-or-later"
inputs = ["LL", "UL", "t", "AS"]
outputs = ["IB", "OB", "t_a"]

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "3.2, 3.5 (inner and outer boundary); 5.9.3 (RFS, Rule #2); Figs. 5-16 and 5-19"

[[tests]]
name = "Y14.5-2018 Fig. 5-16: Ø30.1-30.5 hole, Ø0.1 RFS"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Fig. 5-16: inner boundary 30.0, outer 30.6, 0.1 at every size"
inputs = { LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", AS = "30.4 mm" }
expect = { IB = "30.0 mm", OB = "30.6 mm", t_a = "0.1 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Y14.5-2018 Fig. 5-19: Ø29.5-29.9 pin, Ø0.1 RFS"
source = "Lacewing Lab tolerance_calcs_test, after ASME Y14.5-2018 Fig. 5-19: inner boundary 29.4, outer 30.0"
inputs = { LL = "29.5 mm", UL = "29.9 mm", t = "0.1 mm", AS = "29.7 mm" }
expect = { IB = "29.4 mm", OB = "30.0 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Fig. 5-16's hole produced at 30.0: undersize"
source = "ASME Y14.5-2018 Fig. 5-16: the size limits are 30.1 to 30.5"
inputs = { LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", AS = "30.0 mm" }
expect = { IB = "30.0 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Inner and outer boundaries, regardless of feature size

Without Ⓜ or Ⓛ after it, a geometric tolerance on a feature of size applies regardless of feature size (ASME Y14.5-2018 5.9.3, Rule #2): the same tolerance at every produced size, with no bonus. There is then no virtual or resultant condition, only the inner and outer boundaries the feature's surface can reach: the smallest size less the tolerance and the largest size plus it (3.2, 3.5; Figs. 5-16 and 5-19). They are the same for a hole or a shaft.

### Feature

```calc {description="Lower limit of size"}
LL = 30.1 mm_
```

```calc {description="Upper limit of size"}
UL = 30.5 mm_
```

```calc {description="Geometric tolerance in the frame, no modifier"}
t = 0.1 mm_
```

```calc {description="Actual size of the produced feature"}
AS = 30.4 mm_
```

### Boundaries

```calc
> Tolerance allowed at any size, no bonus
t_a = t = ?? mm
> Inner boundary
IB = LL - t = ?? mm
> Outer boundary
OB = UL + t = ?? mm
```

### Check

```calc
@test LL ≤ AS ≤ UL
```
