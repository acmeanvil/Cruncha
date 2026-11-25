+++
cruncha-template = 1
id = "mechanical.curvedBeams.rectangle"
version = "1.0.0"
title = "Curved beam in bending: rectangular section"
summary = "Neutral-axis radius, eccentricity and inner and outer fibre stresses of a curved beam of rectangular section under a moment and an axial force at the centroid (crane hooks, clamp and press frames), with the curvature factors Ki, Ko against straight-beam theory."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["curved beam", "hook", "crane hook", "C-clamp", "frame", "neutral axis", "eccentricity", "bending", "rectangle", "curvature factor"]
from = "lacewing:mechanical.curvedBeams"
license = "GPL-3.0-or-later"
inputs = ["b", "h", "r_i", "M", "F"]
outputs = ["A", "r_c", "r_n", "e", "c_i", "c_o", "σ_i", "σ_o", "K_i", "K_o", "e_approx"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-18, Eqs. 3-63 to 3-66, Table 3-4 (rectangle); Example 3-15"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 4.6, Eqs. 4.9 to 4.11, Fig. 4.11; Sample Problem 4.1"

[[tests]]
name = "Shigley Ex. 3-15: crane hook 0.75 × 4 in at ri = 2 in, 5000 lbf through the centre of curvature"
source = "Shigley 10th ed., Example 3-15: rn = 3.641 in, e = 0.359 in, σi = 16.9 kpsi, σo = −5.63 kpsi (M = F rc = 20 000 lbf·in, F = 5000 lbf at the centroid)."
inputs = { b = "0.75 in", h = "4 in", r_i = "2 in", M = "20000 lbf·in", F = "5000 lbf" }
expect = { r_n = "3.641 in", e = "0.359 in", "σ_i" = "16.9 ksi", "σ_o" = "-5.63 ksi" }
tolerance = "0.1%"

[[tests]]
name = "Juvinall SP 4.1: ri = h, pure bending"
source = "Juvinall 5th ed., Sample Problem 4.1: e = 0.089761h, σi = 9.141M/(bh²), σo = −4.380M/(bh²); with h = 50 mm, b = 20 mm, M = 1 kN·m: e = 4.4880 mm, σi = 182.82 MPa, σo = −87.60 MPa, and Ki = 9.141/6 = 1.5235, Ko = 4.380/6 = 0.7300 (Fig. 4.11 at rc/c = 2; Lacewing Lab curved_beams_test)."
inputs = { b = "20 mm", h = "50 mm", r_i = "25 mm", M = "1 kN·m", F = "0 N" }
expect = { e = "4.48805 mm", "σ_i" = "182.82 MPa", "σ_o" = "-87.60 MPa", K_i = "1.5235", K_o = "0.7300" }
tolerance = "0.05%"
+++

## Curved beam in bending: rectangular section

A beam curved in its plane of bending (a crane hook, a C-clamp or press frame, a chain link) has a hyperbolic stress distribution: the neutral axis moves toward the centre of curvature by e, and the inner fibre carries more stress than straight-beam theory gives.

The section is a rectangle b wide (across the plane of bending) and h deep (radially), with its inner fibre at radius r_i from the centre of curvature. The section carries a bending moment M, positive when it straightens the beam (tension at the inner fibre), and an axial force F at the centroid, tension positive. A load F whose line lies ℓ from the centroidal axis toward the centre of curvature gives M = F·ℓ and the axial force F; for a hook loaded through the centre of curvature, ℓ = r_c.

Assumptions: plane sections stay plane, elastic material, and radial stresses neglected (small in compact sections). For gentle curves (r_c/c over about 10) the straight-beam formula is within about 10 %.

### Section and loads

```calc {description="Width, across the plane of bending"}
b = 0.75 in_
```

```calc {description="Depth, radial"}
h = 4 in_
```

```calc {description="Radius of the inner fibre, from the centre of curvature"}
r_i = 2 in_
```

```calc {description="Bending moment, positive straightening the beam"}
M = 20000 lbf·in_
```

```calc {description="Axial force at the centroid, tension positive"}
F = 5000 lbf_
```

### Section properties

```calc
> Radius of the outer fibre
r_o = r_i + h = ?? in
> Area
A = b h = ?? in²
> Radius of the centroidal axis
r_c = r_i + h / 2 = ?? in
> Radius of the neutral axis, Table 3-4
r_n = h / ln(r_o / r_i) = ?? in
> Distance from the centroidal to the neutral axis, Eq. 3-66
e = r_c - r_n = ?? in
> Neutral axis to inner fibre
c_i = r_n - r_i = ?? in
> Neutral axis to outer fibre
c_o = r_o - r_n = ?? in
```

### Stresses

```calc
> Inner fibre stress, Eq. 3-65 with F/A
σ_i = F / A + M c_i / (A e r_i) = ?? ksi
> Outer fibre stress
σ_o = F / A - M c_o / (A e r_o) = ?? ksi
```

### Against straight-beam theory

```calc
> Second moment of area about the centroidal axis
I = b h^3 / 12 = ?? in⁴
> Shigley's approximation e ≈ I/(r_c A), Eq. 3-66
e_approx = I / (r_c A) = ?? in
> Straight-beam bending stress M c/I at the inner fibre
σ_st = M (r_c - r_i) / I = ?? ksi
> Curvature factor, inner fibre, Eq. 4.11
K_i = (c_i / (A e r_i)) / ((r_c - r_i) / I) = ??
> Curvature factor, outer fibre
K_o = (c_o / (A e r_o)) / ((r_o - r_c) / I) = ??
```
