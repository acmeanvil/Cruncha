+++
cruncha-template = 1
id = "mechanical.gdtPosition.cylindrical"
version = "1.0.0"
title = "True position of a hole or pin from measured coordinates (ASME Y14.5.1)"
summary = "The position deviation of a measured axis from its true position, 2√(dx² + dy²), against the cylindrical positional tolerance it is allowed at its produced size, with the bonus at MMC or LMC (and a MAX) or none at RFS; the margin left and the size checked within its limits."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "true position", "position", "positional tolerance", "position deviation", "coordinate measurement", "CMM", "inspection", "bonus tolerance", "MMC", "LMC", "RFS", "ASME Y14.5", "ASME Y14.5.1"]
from = "lacewing:mechanical.gdtPosition"
license = "GPL-3.0-or-later"
inputs = ["X", "Y", "x", "y", "hole", "LL", "UL", "t", "mc", "t_MAX", "AS"]
outputs = ["dx", "dy", "r", "d_p", "MMC", "LMC", "b", "t_a", "m"]

[[references]]
work = "ASME Y14.5.1-2019 Mathematical Definition of Dimensioning and Tolerancing Principles"
edition = "2019"
locator = "7.2.2(b), (c) (position deviation, conformance); Table 7-3 (zone size with bonus); Fig. 7-5"

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "5.9.4 and 5.9.6 (bonus at MMC and LMC), 10.3.3 (positional tolerance at MMC); Fig. 5-14"

[[references]]
work = "ISO 1101:2012 Geometrical tolerancing"
edition = "2012"
locator = "18.12.2 (position of a line: cylindrical zone about the theoretically exact position)"

[[tests]]
name = "Y14.5-2018 Fig. 5-14's hole produced at 30.3, measured 0.084 and 0.112 off"
source = "Lacewing Lab tolerance_calcs_test: r = √(0.084² + 0.112²) = 0.14, Ø0.28 against the 0.3 allowed at 30.3 (Ø30.1-30.5, Ø0.1 at MMC), margin 0.02; basic coordinates 40, 25 chosen for the example"
inputs = { X = "40 mm", Y = "25 mm", x = "40.084 mm", y = "25.112 mm", hole = "1", LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", mc = "1", t_MAX = "0 mm", AS = "30.3 mm" }
expect = { dx = "0.084 mm", dy = "0.112 mm", r = "0.14 mm", d_p = "0.28 mm", b = "0.2 mm", t_a = "0.3 mm", m = "0.02 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "The same hole produced at MMC, 30.1: no bonus, out of position"
source = "Lacewing Lab tolerance_calcs_test: Ø0.28 off true position fails a Ø30.1 hole at MMC, allowed Ø0.1"
inputs = { X = "40 mm", Y = "25 mm", x = "40.084 mm", y = "25.112 mm", hole = "1", LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", mc = "1", t_MAX = "0 mm", AS = "30.1 mm" }
expect = { d_p = "0.28 mm", t_a = "0.1 mm", m = "-0.18 mm" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "The same hole produced at 30.6: in position, but outside its size limits"
source = "Lacewing Lab tolerance_calcs_test: rejected for its size"
inputs = { X = "40 mm", Y = "25 mm", x = "40.084 mm", y = "25.112 mm", hole = "1", LL = "30.1 mm", UL = "30.5 mm", t = "0.1 mm", mc = "1", t_MAX = "0 mm", AS = "30.6 mm" }
expect = { d_p = "0.28 mm" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "3-4-5 departure, Ø0.1 RFS"
source = "Lacewing Lab tolerance_calcs_test: dx = 0.03, dy = 0.04 is Ø0.1 exactly, which a Ø0.1 RFS zone just holds"
inputs = { X = "0 mm", Y = "0 mm", x = "0.03 mm", y = "-0.04 mm", hole = "0", LL = "9.9 mm", UL = "10 mm", t = "0.1 mm", mc = "3", t_MAX = "0 mm", AS = "9.95 mm" }
expect = { r = "0.05 mm", d_p = "0.1 mm", b = "0 mm", t_a = "0.1 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: pin Ø9.9-10.0 at LMC, Ø0.05, produced at 9.98"
source = "By hand: LMC 9.9, bonus 9.98 − 9.9 = 0.08, allowed 0.13; dx = 0.05, dy = 0.03, Ø = 2√0.0034 = 0.116619"
inputs = { X = "12 mm", Y = "8 mm", x = "12.05 mm", y = "8.03 mm", hole = "0", LL = "9.9 mm", UL = "10 mm", t = "0.05 mm", mc = "2", t_MAX = "0 mm", AS = "9.98 mm" }
expect = { MMC = "10 mm", LMC = "9.9 mm", b = "0.08 mm", t_a = "0.13 mm", d_p = "0.1166190 mm" }
tolerance = "1e-6"
checks = "pass"
+++

## True position from measured coordinates

Whether a measured hole or pin is in position: the departure of its axis from true position, expressed as the diameter of the smallest cylindrical zone about true position that holds it, against the positional tolerance it is allowed at its produced size (ASME Y14.5.1-2019 7.2.2). The zone is cylindrical (⌀ in the frame), so only the two coordinates across the axis count. For a spherical zone (S⌀) use the spherical template.

The axis is that of the feature's actual mating envelope. Y14.5.1 requires every point of it to lie in the zone, so measure the axis at both ends of the feature (or wherever the inspection plan says) and check each point. ISO 1101 18.12.2 defines the same zone about the theoretically exact position; with Ⓜ or Ⓛ the ISO allowance at a size is the distance of the size from the virtual size, the same number.

### True and measured position

Both pairs of coordinates are in the same datum reference frame: the basic ones from the drawing, the measured ones from the inspection report.

```calc {description="Basic (true) position, x"}
X = 40 mm_
```

```calc {description="Basic (true) position, y"}
Y = 25 mm_
```

```calc {description="Measured centre, x"}
x = 40.084 mm_
```

```calc {description="Measured centre, y"}
y = 25.112 mm_
```

### Feature and tolerance

The material condition is the modifier after the tolerance in the frame: 1 for Ⓜ (MMC), 2 for Ⓛ (LMC), 3 for none (RFS). The actual size is the unrelated actual mating envelope at MMC, or the unrelated actual minimum material envelope at LMC.

```calc {description="1 for an internal feature (hole, slot), 0 for an external one (shaft, pin)"}
hole = 1_
```

```calc {description="Lower limit of size"}
LL = 30.1 mm_
```

```calc {description="Upper limit of size"}
UL = 30.5 mm_
```

```calc {description="Positional tolerance in the frame (zone diameter)"}
t = 0.1 mm_
```

```calc {description="Material condition: 1 MMC, 2 LMC, 3 RFS"}
mc = 1_
```

```calc {description="MAX after the tolerance, the cap on it (0 for none)"}
t_MAX = 0 mm_
```

```calc {description="Actual size of the produced feature"}
AS = 30.3 mm_
```

### Position deviation

```calc
> Departure in x
dx = x - X = ?? mm
> Departure in y
dy = y - Y = ?? mm
> Radial distance from true position
r = hypot(dx, dy) = ?? mm
> Position deviation, the diameter of the smallest zone holding the centre, Y14.5.1 7.2.2(c)
d_p = 2 r = ?? mm
```

### Tolerance allowed at the actual size

```calc
> Maximum material condition size
MMC = { LL if hole == 1; UL otherwise } = ?? mm
> Least material condition size
LMC = { UL if hole == 1; LL otherwise } = ?? mm
> Bonus: departure from MMC at MMC, from LMC at LMC, none at RFS
b = { abs(AS - MMC) if mc == 1; abs(LMC - AS) if mc == 2; 0 'mm' if mc == 3 } = ?? mm
> Positional tolerance allowed, no more than MAX
t_a = { t + b if t_MAX == 0 'mm'; min(t + b, t_MAX) otherwise } = ?? mm
> What is left of the zone
m = t_a - d_p = ?? mm
```

### Checks

The feature conforms when its centre lies in the zone allowed at its size and the size is within its limits.

```calc
@test d_p ≤ t_a
```

```calc
@test LL ≤ AS ≤ UL
```
