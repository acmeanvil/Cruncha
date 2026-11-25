+++
cruncha-template = 1
id = "mechanical.gdtPosition.spherical"
version = "1.0.0"
title = "True position of a spherical feature from measured coordinates (ASME Y14.5.1)"
summary = "The position deviation of a measured centre point from its true position in a spherical zone (S⌀), 2√(dx² + dy² + dz²), against the positional tolerance allowed at the produced size, with the bonus at MMC or LMC (and a MAX) or none at RFS."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "true position", "position", "spherical zone", "sphere", "ball", "position deviation", "coordinate measurement", "CMM", "bonus tolerance", "ASME Y14.5", "ASME Y14.5.1"]
from = "lacewing:mechanical.gdtPosition"
license = "GPL-3.0-or-later"
inputs = ["X", "Y", "Z", "x", "y", "z", "hole", "LL", "UL", "t", "mc", "t_MAX", "AS"]
outputs = ["dx", "dy", "dz", "r", "d_p", "MMC", "LMC", "b", "t_a", "m"]

[[references]]
work = "ASME Y14.5.1-2019 Mathematical Definition of Dimensioning and Tolerancing Principles"
edition = "2019"
locator = "7.2.2(b), (c) (position deviation of a centre point, conformance); Table 7-3"

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "10.4.6 (spherical features); 5.9.4 and 5.9.6 (bonus)"

[[references]]
work = "ISO 1101:2012 Geometrical tolerancing"
edition = "2012"
locator = "18.12.1 (position of a point: spherical zone)"

[[tests]]
name = "Centre 1, 2, 2 off true position: S⌀6"
source = "Lacewing Lab tolerance_calcs_test: a spherical zone takes dz too, 2√(1 + 4 + 4) = 6"
inputs = { X = "0 mm", Y = "0 mm", Z = "0 mm", x = "1 mm", y = "2 mm", z = "2 mm", hole = "0", LL = "24 mm", UL = "25 mm", t = "6 mm", mc = "3", t_MAX = "0 mm", AS = "24.5 mm" }
expect = { r = "3 mm", d_p = "6 mm", t_a = "6 mm", m = "0 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: S⌀20.0-20.2 ball at MMC, S⌀0.2, produced at 20.05"
source = "By hand: MMC 20.2 (external), bonus 0.15, allowed 0.35; d = (0.06, −0.08, 0.12), r = √0.0244 = 0.156205, S⌀0.312410"
inputs = { X = "50 mm", Y = "20 mm", Z = "10 mm", x = "50.06 mm", y = "19.92 mm", z = "10.12 mm", hole = "0", LL = "20.0 mm", UL = "20.2 mm", t = "0.2 mm", mc = "1", t_MAX = "0 mm", AS = "20.05 mm" }
expect = { MMC = "20.2 mm", b = "0.15 mm", t_a = "0.35 mm", r = "0.1562050 mm", d_p = "0.3124100 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same ball produced at MMC: out of position"
source = "By hand: at 20.2 no bonus, S⌀0.31241 against S⌀0.2"
inputs = { X = "50 mm", Y = "20 mm", Z = "10 mm", x = "50.06 mm", y = "19.92 mm", z = "10.12 mm", hole = "0", LL = "20.0 mm", UL = "20.2 mm", t = "0.2 mm", mc = "1", t_MAX = "0 mm", AS = "20.2 mm" }
expect = { t_a = "0.2 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## True position of a spherical feature

Whether a measured spherical feature (a ball, a spherical seat) is in position: S⌀ before the tolerance makes the zone a sphere about the true position, which must hold the feature's centre point (ASME Y14.5-2018 10.4.6; ISO 1101 18.12.1). All three coordinates count. The position deviation is the diameter of the smallest sphere about true position that holds the measured centre (Y14.5.1-2019 7.2.2(c)).

### True and measured position

```calc {description="Basic (true) position, x"}
X = 50 mm_
```

```calc {description="Basic (true) position, y"}
Y = 20 mm_
```

```calc {description="Basic (true) position, z"}
Z = 10 mm_
```

```calc {description="Measured centre, x"}
x = 50.06 mm_
```

```calc {description="Measured centre, y"}
y = 19.92 mm_
```

```calc {description="Measured centre, z"}
z = 10.12 mm_
```

### Feature and tolerance

```calc {description="1 for an internal feature (spherical seat), 0 for an external one (ball)"}
hole = 0_
```

```calc {description="Lower limit of size"}
LL = 20.0 mm_
```

```calc {description="Upper limit of size"}
UL = 20.2 mm_
```

```calc {description="Positional tolerance in the frame (S⌀, zone diameter)"}
t = 0.2 mm_
```

```calc {description="Material condition: 1 MMC, 2 LMC, 3 RFS"}
mc = 1_
```

```calc {description="MAX after the tolerance, the cap on it (0 for none)"}
t_MAX = 0 mm_
```

```calc {description="Actual size of the produced feature"}
AS = 20.05 mm_
```

### Position deviation

```calc
> Departures in x, y and z
dx = x - X = ?? mm
dy = y - Y = ?? mm
dz = z - Z = ?? mm
> Distance from true position
r = sqrt(dx^2 + dy^2 + dz^2) = ?? mm
> Position deviation, the diameter of the smallest sphere holding the centre
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

```calc
@test d_p ≤ t_a
```

```calc
@test LL ≤ AS ≤ UL
```
