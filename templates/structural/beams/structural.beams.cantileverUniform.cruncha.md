+++
cruncha-template = 1
id = "structural.beams.cantileverUniform"
version = "1.0.0"
title = "Cantilever, uniform load: reaction, fixed-end moment, slope and deflection"
summary = "Reaction and moment at the fixed end, and the slope and deflection at the free end, of a cantilever under a uniform load over its whole length (Roark Table 8.1, case 2a)."
category = ["Structural engineering", "Beams"]
tags = ["beam", "cantilever", "uniform load", "UDL", "deflection", "moment", "Roark"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "E", "I"]
outputs = ["R_A", "M_A", "θ_tip", "y_tip"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2a (distributed load, left end free, right end fixed; uniform over the whole length), p. 191"

[[tests]]
name = "Roark 8.1 case 2a: 10 kN/m on a 6 m cantilever, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 2a): M = −wl²/2 = −180 kN·m, y = wl⁴/(8EI) = 81 mm; θ = wl³/(6EI) = 0.018 by hand. Checked against a stiffness solution; the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { w = "10 kN/m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "60 kN", M_A = "-180 kN·m", "θ_tip" = "0.018", y_tip = "81 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "500 lbf/ft on a 12 ft cantilever, I = 510 in⁴"
source = "By hand: R = 6 kips, M = −0.5 × 12²/2 = −36 kip·ft; y = (500/12) × 144⁴/(8 × 29e6 × 510) = 0.15142 in; θ = (500/12) × 144³/(6 × 29e6 × 510) = 0.0014020"
inputs = { w = "500 lbf/ft", L = "12 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "6 kips", M_A = "-36 kip·ft", "θ_tip" = "0.0014020284", y_tip = "0.15141907 in" }
tolerance = "1e-7"
checks = "pass"
+++

## Cantilever, uniform load

A prismatic beam built in at its left end A and free at the other, carrying a uniform load w over its whole length (Roark Table 8.1, case 2a). Linear elastic, small deflections, shear deformation neglected. Load the beam's self-weight into w if it matters. Deflection and slope are magnitudes, deflection downward; the moment at the fixed end is negative (hogging).

### Beam and load

```calc {description="Uniform load, including self-weight"}
w = 10 kN/m_
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
> Reaction at the fixed end, and the largest shear
R_A = w L = ?? kN
> Moment at the fixed end (hogging)
M_A = -w L^2 / 2 = ?? kN·m
> Slope at the free end
θ_tip = w L^3 / (6 E I) = ??
> Deflection at the free end
y_tip = w L^4 / (8 E I) = ?? mm
```
