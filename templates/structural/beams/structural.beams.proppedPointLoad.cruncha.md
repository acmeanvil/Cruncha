+++
cruncha-template = 1
id = "structural.beams.proppedPointLoad"
version = "1.0.0"
title = "Propped cantilever, point load: reactions, moments, slope and deflection"
summary = "Reactions, the fixed-end moment, the moment under the load, the slope at the pinned end and the deflections of a beam pinned at one end and fixed at the other, carrying one concentrated load (Roark Table 8.1, case 1c)."
category = ["Structural engineering", "Beams"]
tags = ["beam", "propped cantilever", "fixed-pinned", "point load", "concentrated load", "statically indeterminate", "deflection", "Roark"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["P", "a", "L", "E", "I"]
outputs = ["R_A", "R_B", "M_B", "M_P", "θ_A", "y_P", "y_max", "x_m"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 1c (concentrated intermediate load, left end simply supported, right end fixed), p. 190"

[[tests]]
name = "Roark 8.1 case 1c: 10 kN at 2 m on a 6 m span, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 1c): RA = W(l − a)²(2l + a)/(2l³) = 5.1852 kN, MB = −Wa(l² − a²)/(2l²) = −8.8889 kN·m, θA = Wa(l − a)²/(4EIl) = 0.00066667; deflections by hand from the same case and checked against a stiffness solution (1200 elements). The formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { P = "10 kN", a = "2 m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "5.1851852 kN", R_B = "4.8148148 kN", M_B = "-8.8888889 kN·m", M_P = "10.37037 kN·m", "θ_A" = "0.00066666667", y_P = "0.98765432 mm", y_max = "1.0098619 mm", x_m = "2.3076923 m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "The load nearer the fixed end: the other branch of the largest deflection"
source = "By hand, a = 4 m > 0.414 l: RA = 10 × 2² × 16/(2 × 216) = 1.4815 kN, MB = −10 × 4 × 20/72 = −11.111 kN·m; ymax = Wab²√(a/(2l + a))/(6EI) = 0.66667 mm at x = l√(a/(2l + a)) = 3 m; checked against a stiffness solution"
inputs = { P = "10 kN", a = "4 m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "1.4814815 kN", R_B = "8.5185185 kN", M_B = "-11.111111 kN·m", M_P = "5.9259259 kN·m", "θ_A" = "0.00033333333", y_P = "0.54320988 mm", y_max = "0.66666667 mm", x_m = "3 m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "12 kips at 8 ft on a 24 ft propped span, I = 510 in⁴"
source = "By hand: b = 16 ft, RA = 12 × 16² × 56/(2 × 24³) = 6.2222 kips, MB = −12 × 8 × (576 − 64)/(2 × 576) = −42.667 kip·ft, M under the load = 6.2222 × 8 = 49.778 kip·ft"
inputs = { P = "12 kips", a = "8 ft", L = "24 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "6.2222222 kips", R_B = "5.7777778 kips", M_B = "-42.666667 kip·ft", M_P = "49.777778 kip·ft", y_max = "0.18122945 in" }
tolerance = "1e-7"
checks = "pass"
+++

## Propped cantilever, point load

A prismatic beam on a pin or roller at its left end A and built in at its right end B, carrying one concentrated load P at a distance a from A (Roark Table 8.1, case 1c). Linear elastic, small deflections, shear deformation neglected. The load is positive downward; deflections and the slope are magnitudes, deflection downward. Moments are positive sagging: the fixed-end moment is negative (hogging).

### Beam and load

```calc {description="Concentrated load, downward"}
P = 10 kN_
```

```calc {description="Distance of the load from the pinned end A"}
a = 2 m_
```

```calc {description="Span, pinned end to fixed end"}
L = 6 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Second moment of area about the bending axis"}
I = 1e8 mm⁴_
```

### Reactions and moments

```calc {description="Distance of the load from the fixed end B"}
b = L - a = ?? m
```

```calc
> Reaction at the pinned end
R_A = P b^2 (2 L + a) / (2 L^3) = ?? kN
> Reaction at the fixed end
R_B = P - R_A = ?? kN
> Moment at the fixed end (hogging)
M_B = -P a (L^2 - a^2) / (2 L^2) = ?? kN·m
> Moment under the load (sagging)
M_P = R_A a = ?? kN·m
```

The larger of |M_B| and M_P governs the moment: the fixed end when the load is near mid-span or beyond.

### Slope and deflection

The largest deflection is at x_m from A: x_m = L(L² + a²)/(3L² − a²) when the load is within (√2 − 1)L ≈ 0.414L of A (the deflection peaks beyond the load), otherwise x_m = L√(a/(2L + a)) (it peaks before the load).

```calc
> Slope at the pinned end
θ_A = P a b^2 / (4 E I L) = ??
> Deflection under the load
y_P = P a^2 b^3 (3 L + a) / (12 E I L^3) = ?? mm
```

```calc
> Position of the largest deflection, from A
x_m = { L (L^2 + a^2) / (3 L^2 - a^2) if a < (sqrt(2) - 1) L; L sqrt(a / (2 L + a)) otherwise } = ?? m
> Largest deflection
y_max = { P a (L^2 - a^2)^3 / (3 E I (3 L^2 - a^2)^2) if a < (sqrt(2) - 1) L; P a b^2 sqrt(a / (2 L + a)) / (6 E I) otherwise } = ?? mm
```

### Checks

```calc
@test 0 'm' < a < L
```
