+++
cruncha-template = 1
id = "structural.beams.twoSpanUniform"
version = "1.0.0"
title = "Two equal continuous spans, uniform load: reactions, moments and deflection"
summary = "Reactions, the moment over the middle support, the largest span moment and the largest deflection of a beam continuous over two equal spans under a uniform load on both."
category = ["Structural engineering", "Beams"]
tags = ["beam", "continuous beam", "two-span", "uniform load", "UDL", "statically indeterminate", "deflection", "Roark"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "E", "I"]
outputs = ["R_A", "R_B", "M_B", "M_pos", "x_m", "y_max"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2c (by symmetry each span of a two-span beam is a propped cantilever fixed at the middle support), p. 192"

[[tests]]
name = "Two 3 m spans under 10 kN/m: 3wL/8, 10wL/8, 3wL/8"
source = "Lacewing Lab beam_test (continuous, two equal spans): end reactions 3wL/8 = 11.25 kN, middle 10wL/8 = 37.5 kN, moment over the middle support −wL²/8 = −11.25 kN·m; span moment 9wL²/128 and deflection by hand from Roark case 2c; checked against a stiffness solution"
inputs = { w = "10 kN/m", L = "3 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "11.25 kN", R_B = "37.5 kN", M_B = "-11.25 kN·m", M_pos = "6.328125 kN·m", x_m = "1.2646055 m", y_max = "0.21935293 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Two 20 ft spans under 1.2 kip/ft, I = 510 in⁴"
source = "By hand: RA = 3 × 1.2 × 20/8 = 9 kips, RB = 10 × 1.2 × 20/8 = 30 kips, MB = −1.2 × 400/8 = −60 kip·ft, M+ = 9 × 1.2 × 400/128 = 33.75 kip·ft; ymax = 0.0054161 × 100 lbf/in × 240⁴/(29e6 × 510) = 0.12150 in"
inputs = { w = "1.2 kip/ft", L = "20 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "9 kips", R_B = "30 kips", M_B = "-60 kip·ft", M_pos = "33.75 kip·ft", y_max = "0.1214969 in" }
tolerance = "1e-7"
checks = "pass"
+++

## Two equal continuous spans, uniform load

A prismatic beam continuous over three supports, A, B and C, with two equal spans L, carrying a uniform load w over both spans. Linear elastic, small deflections, shear deformation neglected, supports that don't settle. By symmetry the beam has no slope over the middle support, so each span behaves as a propped cantilever fixed at B (Roark Table 8.1, case 2c). Moments are positive sagging; the deflection is a magnitude, downward.

Loading only one span (pattern loading for a live load) gives a larger span moment and a smaller support moment than this; this template is for the full load on both spans.

### Beam and load

```calc {description="Uniform load on both spans, including self-weight"}
w = 10 kN/m_
```

```calc {description="Each span, centre to centre of supports"}
L = 3 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Second moment of area about the bending axis"}
I = 1e8 mm⁴_
```

### Reactions and moments

```calc
> Reaction at each end support, A and C
R_A = 3 w L / 8 = ?? kN
> Reaction at the middle support B
R_B = 10 w L / 8 = ?? kN
> Moment over the middle support (hogging), the largest moment
M_B = -w L^2 / 8 = ?? kN·m
> Largest sagging moment, at 3L/8 from each end
M_pos = 9 w L^2 / 128 = ?? kN·m
```

The largest shear is 5wL/8, on either side of B.

### Deflection

```calc
> Position of the largest deflection, from an end support
x_m = L (1 + sqrt(33)) / 16 = ?? m
> Largest deflection, in each span
y_max = w x_m (L^3 - 3 L x_m^2 + 2 x_m^3) / (48 E I) = ?? mm
```
