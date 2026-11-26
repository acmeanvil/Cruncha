+++
cruncha-template = 1
id = "structural.beams.proppedUniform"
version = "1.0.0"
title = "Propped cantilever, uniform load: reactions, moments, slope and deflection"
summary = "Reactions, the fixed-end moment, the largest sagging moment, the slope at the pinned end and the largest deflection of a beam pinned at one end and fixed at the other, under a uniform load over its whole span (Roark Table 8.1, case 2c)."
category = ["Structural engineering", "Beams"]
tags = ["beam", "propped cantilever", "fixed-pinned", "uniform load", "UDL", "statically indeterminate", "deflection", "Roark"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "E", "I"]
outputs = ["R_A", "R_B", "M_B", "M_pos", "θ_A", "x_m", "y_max"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2c (distributed load, left end simply supported, right end fixed; uniform over the whole span), p. 192"

[[tests]]
name = "Roark 8.1 case 2c: 10 kN/m on a 6 m span, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test and frame2d_test (Roark 7th Table 8.1 case 2c): RA = 3wl/8 = 22.5 kN, MB = −wl²/8 = −45 kN·m, θA = wl³/(48EI) = 0.00225, M+ = 9wl²/128 = 25.3125 kN·m, ymax: Roark prints 0.0054wl⁴/EI at 0.4215l; the exact 0.0054161 gives 3.5096 mm. Checked against a stiffness solution; the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { w = "10 kN/m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "22.5 kN", R_B = "37.5 kN", M_B = "-45 kN·m", M_pos = "25.3125 kN·m", "θ_A" = "0.00225", x_m = "2.529211 m", y_max = "3.5096468 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "1.2 kip/ft on a 24 ft propped span, I = 510 in⁴"
source = "By hand: RA = 3 × 1.2 × 24/8 = 10.8 kips, RB = 18 kips, MB = −1.2 × 24²/8 = −86.4 kip·ft, M+ = 9 × 1.2 × 576/128 = 48.6 kip·ft; ymax = 0.0054161 × 100 lbf/in × 288⁴/(29e6 × 510) = 0.25194 in"
inputs = { w = "1.2 kip/ft", L = "24 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "10.8 kips", R_B = "18 kips", M_B = "-86.4 kip·ft", M_pos = "48.6 kip·ft", y_max = "0.25193597 in" }
tolerance = "1e-7"
checks = "pass"
+++

## Propped cantilever, uniform load

A prismatic beam on a pin or roller at its left end A and built in at its right end B, carrying a uniform load w over its whole span (Roark Table 8.1, case 2c). Linear elastic, small deflections, shear deformation neglected. Load the beam's self-weight into w if it matters. Deflections and the slope are magnitudes, deflection downward; moments are positive sagging.

### Beam and load

```calc {description="Uniform load, including self-weight"}
w = 10 kN/m_
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

```calc
> Reaction at the pinned end
R_A = 3 w L / 8 = ?? kN
> Reaction at the fixed end, and the largest shear
R_B = 5 w L / 8 = ?? kN
> Moment at the fixed end (hogging), the largest moment
M_B = -w L^2 / 8 = ?? kN·m
> Largest sagging moment, at 3L/8 from A
M_pos = 9 w L^2 / 128 = ?? kN·m
```

### Slope and deflection

The deflected shape is y = w x (L³ − 3L x² + 2x³)/(48EI) from A; it peaks at x = L(1 + √33)/16 ≈ 0.4215L, where y ≈ wL⁴/(185EI).

```calc
> Slope at the pinned end
θ_A = w L^3 / (48 E I) = ??
> Position of the largest deflection, from A
x_m = L (1 + sqrt(33)) / 16 = ?? m
> Largest deflection
y_max = w x_m (L^3 - 3 L x_m^2 + 2 x_m^3) / (48 E I) = ?? mm
```
