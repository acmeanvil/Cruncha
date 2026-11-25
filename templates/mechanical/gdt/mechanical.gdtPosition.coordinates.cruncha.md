+++
cruncha-template = 1
id = "mechanical.gdtPosition.coordinates"
version = "1.0.0"
title = "Coordinate (±) tolerances and a positional zone"
summary = "A ± coordinate tolerance square compared with a cylindrical positional zone: the zone through the square's corners, the largest round zone inside it, the equal ± that fits inside a zone of given diameter and the ± in y that goes with a chosen ± in x, with the square checked to lie inside the zone."
category = ["Mechanical engineering", "GD&T"]
tags = ["GD&T", "position", "positional tolerance", "coordinate tolerance", "plus minus", "square zone", "cylindrical zone", "conversion", "ASME Y14.5"]
from = "lacewing:mechanical.gdtPosition"
license = "GPL-3.0-or-later"
inputs = ["t_x", "t_y", "T"]
outputs = ["T_c", "T_i", "t_eq", "t_(y,T)"]

[[references]]
work = "ASME Y14.5.1-2019 Mathematical Definition of Dimensioning and Tolerancing Principles"
edition = "2019"
locator = "7.2.2(c) (position deviation as the diameter of the smallest zone about true position)"

[[references]]
work = "ASME Y14.5-2018 Dimensioning and Tolerancing"
edition = "2018"
locator = "10.3 (positional tolerancing)"

[[tests]]
name = "±0.1 square: corners on Ø0.2√2; equal ± inside Ø0.2√2 is 0.1"
source = "Lacewing Lab tolerance_calcs_test: circumscribed 2√(0.1² + 0.1²) = 0.282843; equal ± for T = 0.2√2 is T/(2√2) = 0.1"
inputs = { t_x = "0.1 mm", t_y = "0.1 mm", T = "0.2828427125 mm" }
expect = { T_c = "0.2828427125 mm", T_i = "0.2 mm", t_eq = "0.1 mm", "t_(y,T)" = "0.1 mm" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "±0.03 in x inside Ø0.1: ±0.04 in y"
source = "Lacewing Lab tolerance_calcs_test: √(0.1²/4 − 0.03²) = 0.04; the 0.03 × 0.05 square's inner zone is 2 × 0.03"
inputs = { t_x = "0.03 mm", t_y = "0.04 mm", T = "0.1 mm" }
expect = { "t_(y,T)" = "0.04 mm", T_c = "0.1 mm", T_i = "0.06 mm", t_eq = "0.03535534 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "±0.1 × ±0.05 square: inner zone Ø0.1"
source = "Lacewing Lab tolerance_calcs_test: 2 min(0.1, 0.05) = 0.1"
inputs = { t_x = "0.1 mm", t_y = "0.05 mm", T = "0.3 mm" }
expect = { T_i = "0.1 mm", T_c = "0.2236068 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "±0.1 square against Ø0.25: its corners are outside the zone"
source = "By hand: 2√0.02 = 0.2828 over 0.25"
inputs = { t_x = "0.1 mm", t_y = "0.1 mm", T = "0.25 mm" }
expect = { T_c = "0.2828427 mm" }
tolerance = "1e-6"
checks = "fail"
+++

## Coordinate tolerances and a positional zone

A hole located by ± dimensions in x and y may lie anywhere in a square (or rectangle) about its nominal position; a positional tolerance gives it a round zone instead. Neither ASME Y14.5-2018 nor Y14.5.1-2019 states a conversion between them. The relations here are the geometry of the square and the circle: a centre at a corner of the ± square is as far off as it can be, and its position deviation (Y14.5.1 7.2.2(c)) is the diameter of the circle through the corners. A round zone that holds the whole square is π/2, about 57 %, larger in area than the square of equal ± inside it, which is why replacing ± location with position usually gains tolerance.

### Inputs

```calc {description="± coordinate tolerance in x"}
t_x = 0.1 mm_
```

```calc {description="± coordinate tolerance in y"}
t_y = 0.1 mm_
```

```calc {description="Positional tolerance zone diameter to compare with"}
T = 0.3 mm_
```

### Square and zone

```calc
> Zone through the corners of the ± square, the worst position the square allows
T_c = 2 hypot(t_x, t_y) = ?? mm
> Largest round zone inside the square
T_i = 2 min(t_x, t_y) = ?? mm
> Equal ± whose square fits inside the zone T
t_eq = T / (2 sqrt(2)) = ?? mm
> ± in y that, with ± t_x, puts the square's corners on the zone T
t_(y,T) = sqrt(T^2 / 4 - t_x^2) = ?? mm
```

### Check

The ± square lies inside the positional zone, so a part that meets the coordinate tolerances also meets the position tolerance.

```calc
@test T_c ≤ T
```
