+++
cruncha-template = 1
id = "structural.beams.cantileverPointLoad"
version = "1.0.0"
title = "Cantilever, point load: reaction, fixed-end moment, slope and deflection"
summary = "Reaction and moment at the fixed end, and the slope and deflection at the load and at the free end, of a cantilever carrying one concentrated load anywhere along it (Roark Table 8.1, case 1a)."
category = ["Structural engineering", "Beams"]
tags = ["beam", "cantilever", "point load", "concentrated load", "deflection", "moment", "Roark"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["P", "a", "L", "E", "I"]
outputs = ["R_A", "M_A", "θ_tip", "y_P", "y_tip"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 1a (concentrated intermediate load, left end free, right end fixed), p. 189"

[[tests]]
name = "Roark 8.1 case 1a: 10 kN 2 m from the free end of a 6 m cantilever, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 1a, a measured from the free end; here a = 4 m from the fixed end): MB = −W(l − aR) = −40 kN·m, y free end = W/(6EI)·(2l³ − 3l²aR + aR³) = 18.667 mm, slope W(l − aR)²/(2EI) = 0.004. Checked against a stiffness solution; the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { P = "10 kN", a = "4 m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "10 kN", M_A = "-40 kN·m", "θ_tip" = "0.004", y_P = "10.666667 mm", y_tip = "18.666667 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "2 kips at the tip of a 10 ft cantilever, I = 510 in⁴"
source = "By hand: M = −2 × 10 = −20 kip·ft; y = PL³/(3EI) = 2000 × 120³/(3 × 29e6 × 510) = 0.077890 in; θ = PL²/(2EI) = 0.00097363"
inputs = { P = "2 kips", a = "10 ft", L = "10 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "2 kips", M_A = "-20 kip·ft", "θ_tip" = "0.00097363083", y_P = "0.077890467 in", y_tip = "0.077890467 in" }
tolerance = "1e-7"
checks = "pass"
+++

## Cantilever, point load

A prismatic beam built in at its left end A and free at the other, carrying one concentrated load P at a distance a from the fixed end (Roark Table 8.1, case 1a; Roark draws the fixed end on the right and measures a from the free end, so his a is L − a here). Linear elastic, small deflections, shear deformation neglected. The load is positive downward; deflections and slopes are magnitudes, deflection downward. The moment is negative (hogging: tension on top).

Beyond the load the beam carries no moment, so it stays straight: the tip deflects by the deflection at the load plus the slope there times the remaining length.

### Beam and load

```calc {description="Concentrated load, downward"}
P = 10 kN_
```

```calc {description="Distance of the load from the fixed end A"}
a = 4 m_
```

```calc {description="Length of the cantilever"}
L = 6 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Second moment of area about the bending axis"}
I = 1e8 mm⁴_
```

### Results

```calc
> Reaction at the fixed end
R_A = P = ?? kN
> Moment at the fixed end (hogging)
M_A = -P a = ?? kN·m
> Slope at the load and beyond it, to the free end
θ_tip = P a^2 / (2 E I) = ??
> Deflection under the load
y_P = P a^3 / (3 E I) = ?? mm
> Deflection at the free end
y_tip = P a^2 (3 L - a) / (6 E I) = ?? mm
```

### Checks

```calc
@test 0 'm' < a ≤ L
```
