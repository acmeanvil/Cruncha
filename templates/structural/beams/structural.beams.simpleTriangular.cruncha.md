+++
cruncha-template = 1
id = "structural.beams.simpleTriangular"
version = "1.0.0"
title = "Simply supported beam, triangular load: reactions, moment, slopes and deflection"
summary = "Reactions, the largest moment and where it occurs, the end slopes and the largest deflection of a simply supported beam under a load that rises linearly from zero at one support to w at the other (Roark Table 8.1, case 2e with a uniformly increasing load)."
category = ["Structural engineering", "Beams"]
tags = ["beam", "simply supported", "triangular load", "linearly varying load", "hydrostatic", "deflection", "moment", "Roark"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "E", "I"]
outputs = ["R_A", "R_B", "M_max", "x_M", "θ_A", "θ_B", "x_m", "y_max"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2e (distributed load, both ends simply supported; uniformly increasing load, a = 0, w_a = 0), p. 193"

[[tests]]
name = "Roark 8.1 case 2e, increasing load: 0 to 10 kN/m over 6 m, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 2e, uniformly increasing): RA = wl/6 = 10 kN, RB = wl/3 = 20 kN, Mmax ≈ 0.0641wl² (exact wl²/(9√3) = 23.094 kN·m), ymax: Roark prints 0.00653wl⁴/EI at 0.5195l; the exact 0.0065222 at 0.5193l gives 4.2264 mm. Checked against a stiffness solution; the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { w = "10 kN/m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "10 kN", R_B = "20 kN", M_max = "23.094011 kN·m", x_M = "3.4641016 m", "θ_A" = "0.0021", "θ_B" = "0.0024", x_m = "3.1159777 m", y_max = "4.2263754 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "0 to 1.2 kip/ft over 24 ft, I = 510 in⁴"
source = "By hand: RA = 1.2 × 24/6 = 4.8 kips, RB = 9.6 kips, Mmax = 1.2 × 576/(9√3) = 44.341 kip·ft at 24/√3 = 13.856 ft; ymax = 0.0065222 × 100 lbf/in × 288⁴/(29e6 × 510) = 0.30339 in"
inputs = { w = "1.2 kip/ft", L = "24 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "4.8 kips", R_B = "9.6 kips", M_max = "44.340501 kip·ft", x_M = "13.856406 ft", y_max = "0.30338551 in" }
tolerance = "1e-7"
checks = "pass"
+++

## Simply supported beam, triangular load

A prismatic beam on a pin and a roller, carrying a load that rises linearly from zero at the left support A to w at the right support B: a hydrostatic load on a wall panel, a soil or bin pressure, or a load tapering along a span (Roark Table 8.1, case 2e with a uniformly increasing load). Linear elastic, small deflections, shear deformation neglected. Deflections and slopes are magnitudes, deflection downward. The total load is wL/2.

### Beam and load

```calc {description="Load intensity at B (zero at A)"}
w = 10 kN/m_
```

```calc {description="Span between supports"}
L = 6 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Second moment of area about the bending axis"}
I = 1e8 mm⁴_
```

### Reactions and moment

```calc
> Reaction at A (the light end)
R_A = w L / 6 = ?? kN
> Reaction at B (the heavy end), the largest shear
R_B = w L / 3 = ?? kN
> Position of the largest moment, from A (zero shear)
x_M = L / sqrt(3) = ?? m
> Largest moment
M_max = w L^2 / (9 sqrt(3)) = ?? kN·m
```

### Slopes and deflection

The deflected shape is y = w x (7L⁴ − 10L²x² + 3x⁴)/(360EIL) from A; it peaks where x² = L²(1 − √(8/15)), at about 0.5193L.

```calc
> Slope at A
θ_A = 7 w L^3 / (360 E I) = ??
> Slope at B
θ_B = 8 w L^3 / (360 E I) = ??
> Position of the largest deflection, from A
x_m = L sqrt(1 - sqrt(8 / 15)) = ?? m
> Largest deflection
y_max = w x_m (7 L^4 - 10 L^2 x_m^2 + 3 x_m^4) / (360 E I L) = ?? mm
```
