+++
cruncha-template = 1
id = "mechanical.curvedBeams.trapezoid"
version = "1.0.0"
title = "Curved beam in bending: trapezoidal section"
summary = "Neutral-axis radius, eccentricity and inner and outer fibre stresses of a curved beam of trapezoidal section (the usual crane-hook shape, wide at the inner fibre) under a moment and an axial force at the centroid, with the curvature factors Ki, Ko against straight-beam theory."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["curved beam", "hook", "crane hook", "frame", "neutral axis", "eccentricity", "bending", "trapezoid", "curvature factor"]
from = "lacewing:mechanical.curvedBeams"
license = "GPL-3.0-or-later"
inputs = ["h", "b_i", "b_o", "r_i", "M", "F"]
outputs = ["A", "r_c", "r_n", "e", "c_i", "c_o", "σ_i", "σ_o", "K_i", "K_o", "e_approx"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-18, Eqs. 3-63 to 3-66, Table 3-4 (trapezoid); Example 3-15"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 4.6, Eqs. 4.9 to 4.11, Fig. 4.11"

[[tests]]
name = "Equal widths reduce to Shigley Ex. 3-15's rectangular hook"
source = "Shigley 10th ed., Example 3-15 (0.75 × 4 in at ri = 2 in, M = 20 000 lbf·in, F = 5000 lbf): rn = 3.641 in, e = 0.359 in, σi = 16.9 kpsi, σo = −5.63 kpsi."
inputs = { h = "4 in", b_i = "0.75 in", b_o = "0.75 in", r_i = "2 in", M = "20000 lbf·in", F = "5000 lbf" }
expect = { r_n = "3.641 in", e = "0.359 in", "σ_i" = "16.9 ksi", "σ_o" = "-5.63 ksi" }
tolerance = "0.1%"

[[tests]]
name = "Trapezoid 30 to 10 mm wide, 40 deep, at ri = 30 mm, 500 N·m and 2 kN"
source = "By hand: A = 800 mm², rc = 30 + 40 × 50/120 = 46.6667 mm; rn = 800/(10 − 30 + ((30 × 70 − 10 × 30)/40) ln(70/30)) = 44.1296 mm (a numerical integration of dA/r agrees); e = 2.537 02 mm; σi = 2000/800 + 500 000 × 14.129 64/(800 × 2.537 02 × 30) = 118.529 MPa, σo = 2.5 − 500 000 × 25.870 36/(800 × 2.537 02 × 70) = −88.546 MPa; I = 40³(30² + 4 × 30 × 10 + 10²)/(36 × 40) = 97 777.8 mm⁴, Ki = 1.361 40, Ko = 0.763 05."
inputs = { h = "40 mm", b_i = "30 mm", b_o = "10 mm", r_i = "30 mm", M = "500 N·m", F = "2 kN" }
expect = { A = "800 mm^2", r_c = "46.66667 mm", r_n = "44.129644 mm", e = "2.537023 mm", "σ_i" = "118.5287 MPa", "σ_o" = "-88.5458 MPa", K_i = "1.361404", K_o = "0.763051" }
tolerance = "1e-5"
+++

## Curved beam in bending: trapezoidal section

A beam curved in its plane of bending has a hyperbolic stress distribution: the neutral axis moves toward the centre of curvature by e, and the inner fibre carries more stress than straight-beam theory gives. A trapezoid wide at the inner fibre puts material where the stress is highest, which is why crane hooks are shaped so.

The section is a trapezoid h deep (radially), b_i wide at the inner fibre and b_o wide at the outer, symmetric about the plane of bending, with its inner fibre at radius r_i from the centre of curvature. The section carries a bending moment M, positive when it straightens the beam (tension at the inner fibre), and an axial force F at the centroid, tension positive. A load F whose line lies ℓ from the centroidal axis toward the centre of curvature gives M = F·ℓ and the axial force F; for a hook loaded through the centre of curvature, ℓ = r_c.

The neutral-axis radius is Shigley's closed form for the trapezoid (Table 3-4). Assumptions: plane sections stay plane, elastic material, and radial stresses neglected. For gentle curves (r_c/c over about 10) the straight-beam formula is within about 10 %.

### Section and loads

```calc {description="Depth, radial"}
h = 40 mm_
```

```calc {description="Width at the inner fibre"}
b_i = 30 mm_
```

```calc {description="Width at the outer fibre"}
b_o = 10 mm_
```

```calc {description="Radius of the inner fibre, from the centre of curvature"}
r_i = 30 mm_
```

```calc {description="Bending moment, positive straightening the beam"}
M = 500 N·m_
```

```calc {description="Axial force at the centroid, tension positive"}
F = 2 kN_
```

### Section properties

```calc
> Radius of the outer fibre
r_o = r_i + h = ?? mm
> Area
A = (b_i + b_o) h / 2 = ?? mm²
> Radius of the centroidal axis
r_c = r_i + h (b_i + 2 b_o) / (3 (b_i + b_o)) = ?? mm
> Radius of the neutral axis, Table 3-4
r_n = A / (b_o - b_i + ((b_i r_o - b_o r_i) / h) ln(r_o / r_i)) = ?? mm
> Distance from the centroidal to the neutral axis
e = r_c - r_n = ?? mm
> Neutral axis to inner fibre
c_i = r_n - r_i = ?? mm
> Neutral axis to outer fibre
c_o = r_o - r_n = ?? mm
```

### Stresses

```calc
> Inner fibre stress, Eq. 3-65 with F/A
σ_i = F / A + M c_i / (A e r_i) = ?? MPa
> Outer fibre stress
σ_o = F / A - M c_o / (A e r_o) = ?? MPa
```

### Against straight-beam theory

```calc
> Second moment of area about the centroidal axis
I = h^3 (b_i^2 + 4 b_i b_o + b_o^2) / (36 (b_i + b_o)) = ?? mm⁴
> Shigley's approximation e ≈ I/(r_c A), Eq. 3-66
e_approx = I / (r_c A) = ?? mm
> Straight-beam bending stress M c/I at the inner fibre
σ_st = M (r_c - r_i) / I = ?? MPa
> Curvature factor, inner fibre, Eq. 4.11
K_i = (c_i / (A e r_i)) / ((r_c - r_i) / I) = ??
> Curvature factor, outer fibre
K_o = (c_o / (A e r_o)) / ((r_o - r_c) / I) = ??
```
