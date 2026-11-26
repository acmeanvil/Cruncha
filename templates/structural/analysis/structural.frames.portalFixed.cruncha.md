+++
cruncha-template = 1
id = "structural.frames.portalFixed"
version = "1.0.0"
title = "Portal frame, fixed bases: reactions, base and corner moments, drift"
summary = "A single-bay rigid portal frame on fixed bases under a uniform load on the beam and a lateral load at beam level: base reactions and moments, the moments at the knees and mid-span, and the sway, in closed form."
category = ["Structural engineering", "Analysis"]
tags = ["frame", "portal frame", "rigid frame", "moment frame", "fixed base", "sway", "drift", "slope-deflection", "stiffness method"]
from = "lacewing:structural.frames"
license = "GPL-3.0-or-later"
inputs = ["L", "h", "I_b", "I_c", "E", "w", "P"]
outputs = ["k", "H_g", "H_A", "H_D", "V_A", "V_D", "M_A", "M_B", "M_C", "M_D", "M_mid", "Δ"]

[[references]]
work = "McGuire, Gallagher and Ziemian, Matrix Structural Analysis"
edition = "2nd (2000)"
locator = "Ch. 4 (the plane-frame member stiffness, used to check these closed forms)"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2d (fixed-end moment wL²/12 of the beam, the limit k → 0)"

[[tests]]
name = "8 m × 4 m frame, k = 1, 10 kN/m and 20 kN"
source = "By hand: k = 1; gravity Hg = 640/(4 × 4 × 3) = 13.333 kN, bases 640/36 = 17.778 kN·m, knees −640/18 = −35.556 kN·m; lateral bases 80 × 4/(2 × 7) = 22.857 kN·m, knees 3 × 80/14 = 17.143 kN·m, V = 3 × 80/(8 × 7) = 4.2857 kN; Δ = 20 000 × 64 × 5/(12 × 200e9 × 1e-4 × 7) = 3.8095 mm. A plane-frame stiffness solution (axially rigid members) gives the same to 1e-6"
inputs = { L = "8 m", h = "4 m", I_b = "2e8 mm^4", I_c = "1e8 mm^4", E = "200 GPa", w = "10 kN/m", P = "20 kN" }
expect = { k = "1", H_g = "13.333333 kN", H_A = "3.3333333 kN", H_D = "-23.333333 kN", V_A = "35.714286 kN", V_D = "44.285714 kN", M_A = "-5.0793651 kN·m", M_B = "-18.412698 kN·m", M_C = "-52.698413 kN·m", M_D = "40.634921 kN·m", M_mid = "44.444444 kN·m", "Δ" = "3.8095238 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Lacewing Lab's opening frame: W16X40 beam on W10X49 columns, 15 kN/m and 20 kN"
source = "Lacewing Lab frame page preset (fixed bases, 8 m × 4 m, W10X49 columns Ix 272 in⁴, W16X40 beam Ix 518 in⁴, AISC Shapes Database v16.0), here without self-weight and with axially rigid members: k = 518 × 4/(272 × 8) = 0.95221; checked against a plane-frame stiffness solution (to 1e-6). Lacewing itself counts axial shortening and self-weight, so its page differs slightly"
inputs = { L = "8 m", h = "4 m", I_b = "518 in^4", I_c = "272 in^4", E = "200 GPa", w = "15 kN/m", P = "20 kN" }
expect = { k = "0.95220588", H_A = "10.323786 kN", H_D = "-30.323786 kN", V_A = "55.744797 kN", V_D = "64.255203 kN", M_A = "4.1191916 kN·m", M_B = "-37.175952 kN·m", M_C = "-71.217573 kN·m", M_D = "50.077571 kN·m", M_mid = "65.803238 kN·m", "Δ" = "3.4079797 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "A very stiff beam: the columns bend in double curvature, Ph/4 top and bottom"
source = "By hand, k → ∞ (beam I 10⁶ times): each column is fixed-fixed with sway, M = (P/2)(h/2) = 20 kN·m at base and knee, V = Ph/(2L) = 5 kN, Δ = (P/2)h³/(12EIc) = 2.6667 mm"
inputs = { L = "8 m", h = "4 m", I_b = "2e14 mm^4", I_c = "1e8 mm^4", E = "200 GPa", w = "0 kN/m", P = "20 kN" }
expect = { M_A = "-20 kN·m", M_B = "20 kN·m", M_D = "20 kN·m", V_D = "5 kN", "Δ" = "2.6666667 mm" }
tolerance = "1e-5"
checks = "pass"
+++

## Portal frame, fixed bases

A single-bay, single-storey rigid frame: two columns of height h built in at their bases A and D, joined rigidly at the knees B and C by a beam of span L. The beam carries a uniform load w over its whole span, and a lateral load P acts at beam level, from A's side towards D. The full frame calculator (any nodes, members, hinges and loads, by the stiffness method) is a solver this template doesn't hold; this is the closed form for the commonest case.

Assumptions: prismatic members, both columns alike, bases that don't rotate; linear elastic, small displacements, no P-Δ effect; axial and shear deformation neglected. The gravity case is symmetric and doesn't sway; the lateral case is antisymmetric. Their results are added.

Signs: reactions positive to the right (H) and upward (V); moments positive when they put the inside face of the frame in tension. Under gravity the knees hog (negative) and the bases are positive.

### Frame and loads

```calc {description="Span of the beam, column centre lines"}
L = 8 m_
```

```calc {description="Height of the columns, base to beam centre line"}
h = 4 m_
```

```calc {description="Second moment of area of the beam"}
I_b = 2e8 mm⁴_
```

```calc {description="Second moment of area of each column"}
I_c = 1e8 mm⁴_
```

```calc {description="Modulus of elasticity (for the drift only)"}
E = 200 GPa_
```

```calc {description="Uniform load on the beam, downward"}
w = 10 kN/m_
```

```calc {description="Lateral load at beam level, towards D"}
P = 20 kN_
```

### Relative stiffness

```calc {description="Stiffness of the beam relative to the columns, (I_b/L)/(I_c/h)"}
k = I_b h / (I_c L) = ??
```

### Gravity load on the beam

```calc
> Horizontal thrust at each base, inwards
H_g = w L^2 / (4 h (k + 2)) = ?? kN
> Moment at each base from gravity
M_(A,g) = w L^2 / (12 (k + 2)) = ?? kN·m
> Moment at each knee from gravity (hogging)
M_(B,g) = -w L^2 / (6 (k + 2)) = ?? kN·m
> Moment at mid-span of the beam (sagging)
M_mid = w L^2 / 8 + M_(B,g) = ?? kN·m
```

### Lateral load

Each column takes P/2 of shear. A stiff beam (large k) makes the columns bend in double curvature with Ph/4 at each end; a flexible beam (k → 0) leaves them as cantilevers with Ph/2 at the base.

```calc
> Moment at each base from the lateral load
M_(A,lat) = P h (3 k + 1) / (2 (6 k + 1)) = ?? kN·m
> Moment at each knee from the lateral load
M_(B,lat) = 3 k P h / (2 (6 k + 1)) = ?? kN·m
> Vertical reaction from overturning, down at A and up at D
V_lat = 3 P h k / (L (6 k + 1)) = ?? kN
> Sway of the beam (drift)
Δ = P h^3 (3 k + 2) / (12 E I_c (6 k + 1)) = ?? mm
```

### Combined

```calc
> Horizontal reaction at A (positive to the right)
H_A = H_g - P / 2 = ?? kN
> Horizontal reaction at D (positive to the right)
H_D = -H_g - P / 2 = ?? kN
> Vertical reaction at A
V_A = w L / 2 - V_lat = ?? kN
> Vertical reaction at D
V_D = w L / 2 + V_lat = ?? kN
> Moment at base A, windward (positive: tension inside)
M_A = M_(A,g) - M_(A,lat) = ?? kN·m
> Moment at knee B, windward (positive: tension inside)
M_B = M_(B,g) + M_(B,lat) = ?? kN·m
> Moment at knee C, leeward (positive: tension inside)
M_C = M_(B,g) - M_(B,lat) = ?? kN·m
> Moment at base D, leeward (positive: tension inside)
M_D = M_(A,g) + M_(A,lat) = ?? kN·m
```

The lateral load adds nothing at mid-span, but it moves the beam's largest sagging moment towards the windward knee; with a large P, check the beam's moment diagram between the knees. A negative V_A means the base must hold the column down.

### Checks

```calc
@test h > 0 'm' and L > 0 'm'
```
