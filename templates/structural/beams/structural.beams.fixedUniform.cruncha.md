+++
cruncha-template = 1
id = "structural.beams.fixedUniform"
version = "1.0.0"
title = "Fixed-ended beam, uniform load: reactions, end and mid-span moments, deflection"
summary = "Reactions, the fixed-end moments, the mid-span moment and the mid-span deflection of a beam built in at both ends under a uniform load over its whole span (Roark Table 8.1, case 2d)."
category = ["Structural engineering", "Beams"]
tags = ["beam", "fixed ends", "fixed-fixed", "built-in", "uniform load", "UDL", "fixed-end moment", "statically indeterminate", "Roark"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "E", "I"]
outputs = ["R", "M_A", "M_mid", "y_max"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2d (distributed load, both ends fixed; uniform over the whole span), p. 193"

[[tests]]
name = "Roark 8.1 case 2d: 10 kN/m on a 6 m span, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 2d): M ends −wl²/12 = −30 kN·m, mid-span wl²/24 = 15 kN·m, ymax = wl⁴/(384EI) = 1.6875 mm. Checked against a stiffness solution; the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { w = "10 kN/m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R = "30 kN", M_A = "-30 kN·m", M_mid = "15 kN·m", y_max = "1.6875 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "1.2 kip/ft on a 24 ft fixed-ended span, I = 510 in⁴"
source = "By hand: R = 14.4 kips, M ends = −1.2 × 576/12 = −57.6 kip·ft, mid-span 28.8 kip·ft, y = 100 lbf/in × 288⁴/(384 × 29e6 × 510) = 0.12114 in"
inputs = { w = "1.2 kip/ft", L = "24 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R = "14.4 kips", M_A = "-57.6 kip·ft", M_mid = "28.8 kip·ft", y_max = "0.12113525 in" }
tolerance = "1e-7"
checks = "pass"
+++

## Fixed-ended beam, uniform load

A prismatic beam built in at both ends, carrying a uniform load w over its whole span (Roark Table 8.1, case 2d). Linear elastic, small deflections, shear deformation neglected; the ends neither rotate nor move. Load the beam's self-weight into w if it matters. Moments are positive sagging; the deflection is a magnitude, downward. The end moment is the fixed-end moment a frame analysis starts from. The moment is zero at 0.2113L from each end.

### Beam and load

```calc {description="Uniform load, including self-weight"}
w = 10 kN/m_
```

```calc {description="Span between the fixed ends"}
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
> Each reaction, and the largest shear
R = w L / 2 = ?? kN
> Moment at each end (hogging), the largest moment
M_A = -w L^2 / 12 = ?? kN·m
> Moment at mid-span (sagging)
M_mid = w L^2 / 24 = ?? kN·m
> Deflection at mid-span, the largest
y_max = w L^4 / (384 E I) = ?? mm
```
