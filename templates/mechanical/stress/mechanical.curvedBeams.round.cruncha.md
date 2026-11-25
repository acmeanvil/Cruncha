+++
cruncha-template = 1
id = "mechanical.curvedBeams.round"
version = "1.0.0"
title = "Curved beam in bending: round section"
summary = "Neutral-axis radius, eccentricity and inner and outer fibre stresses of a curved beam of solid round section under a moment and an axial force at the centroid (hooks, rings, chain links, bent rods), with the curvature factors Ki, Ko against straight-beam theory."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["curved beam", "hook", "ring", "chain link", "eye bolt", "neutral axis", "eccentricity", "bending", "round", "circular", "curvature factor"]
from = "lacewing:mechanical.curvedBeams"
license = "GPL-3.0-or-later"
inputs = ["d", "r_i", "M", "F"]
outputs = ["A", "r_c", "r_n", "e", "c_i", "c_o", "σ_i", "σ_o", "K_i", "K_o", "e_approx"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-18, Eqs. 3-63 to 3-66, Table 3-4 (circle); Example 3-16"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 4.6, Eqs. 4.9 to 4.11, Fig. 4.11"

[[tests]]
name = "Shigley Ex. 3-16: circle of radius 1 in at rc = 3 in"
source = "Shigley 10th ed., Example 3-16: rn = 2.914 21 in from Table 3-4, e = 0.085 79 in; Eq. 3-66 approximates e ≈ 0.083 33 in (Lacewing Lab curved_beams_test)."
inputs = { d = "2 in", r_i = "2 in" }
expect = { r_n = "2.91421 in", e = "0.08579 in", e_approx = "0.08333 in" }
tolerance = "0.01%"

[[tests]]
name = "The same ring under 1000 lbf·in"
source = "By hand: A = π in², ci = 0.914 214 in, co = 1.085 786 in; σi = 1000 × 0.914 214/(π × 0.085 786 × 2) = 1696.09 psi, σo = −1000 × 1.085 786/(π × 0.085 786 × 4) = −1007.20 psi; Mc/I = 1000 × 1/(π/4) = 1273.24 psi, Ki = 1.332 11, Ko = 0.791 05."
inputs = { d = "2 in", r_i = "2 in", M = "1000 lbf·in", F = "0 lbf" }
expect = { "σ_i" = "1696.09 psi", "σ_o" = "-1007.20 psi", K_i = "1.332107", K_o = "0.791053" }
tolerance = "1e-5"
+++

## Curved beam in bending: round section

A beam curved in its plane of bending (a hook, a ring, a chain link, an eye bolt) has a hyperbolic stress distribution: the neutral axis moves toward the centre of curvature by e, and the inner fibre carries more stress than straight-beam theory gives.

The section is a solid circle of diameter d, with its inner fibre at radius r_i from the centre of curvature. The section carries a bending moment M, positive when it straightens the beam (tension at the inner fibre), and an axial force F at the centroid, tension positive. A load F whose line lies ℓ from the centroidal axis toward the centre of curvature gives M = F·ℓ and the axial force F.

The neutral-axis radius is Shigley's closed form for the circle (Table 3-4). Assumptions: plane sections stay plane, elastic material, and radial stresses neglected. For gentle curves (r_c/c over about 10) the straight-beam formula is within about 10 %.

### Section and loads

```calc {description="Diameter of the section"}
d = 2 in_
```

```calc {description="Radius of the inner fibre, from the centre of curvature"}
r_i = 2 in_
```

```calc {description="Bending moment, positive straightening the beam"}
M = 1000 lbf·in_
```

```calc {description="Axial force at the centroid, tension positive"}
F = 0 lbf_
```

### Section properties

```calc
> Radius of the section
R = d / 2 = ?? in
> Radius of the outer fibre
r_o = r_i + d = ?? in
> Area
A = π R^2 = ?? in²
> Radius of the centroidal axis
r_c = r_i + R = ?? in
> Radius of the neutral axis, Table 3-4
r_n = R^2 / (2 (r_c - sqrt(r_c^2 - R^2))) = ?? in
> Distance from the centroidal to the neutral axis
e = r_c - r_n = ?? in
> Neutral axis to inner fibre
c_i = r_n - r_i = ?? in
> Neutral axis to outer fibre
c_o = r_o - r_n = ?? in
```

### Stresses

```calc
> Inner fibre stress, Eq. 3-65 with F/A
σ_i = F / A + M c_i / (A e r_i) = ?? psi
> Outer fibre stress
σ_o = F / A - M c_o / (A e r_o) = ?? psi
```

### Against straight-beam theory

```calc
> Second moment of area about the centroidal axis
I = π R^4 / 4 = ?? in⁴
> Shigley's approximation e ≈ I/(r_c A), Eq. 3-66
e_approx = I / (r_c A) = ?? in
> Straight-beam bending stress M c/I at the inner fibre
σ_st = M R / I = ?? psi
> Curvature factor, inner fibre, Eq. 4.11
K_i = (c_i / (A e r_i)) / (R / I) = ??
> Curvature factor, outer fibre
K_o = (c_o / (A e r_o)) / (R / I) = ??
```
