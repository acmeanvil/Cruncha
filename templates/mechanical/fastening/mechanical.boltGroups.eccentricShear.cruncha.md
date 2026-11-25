+++
cruncha-template = 1
id = "mechanical.boltGroups.eccentricShear"
version = "1.0.0"
title = "Bolt group under eccentric shear (Shigley)"
summary = "The load on each fastener of a pattern under an in-plane force acting off the centroid and a torque: the pattern's centroid, direct shear shared by area, moment shear in proportion to the distance from the centroid, their vector sum on every fastener, and the largest shear and bearing stresses checked against allowables."
category = ["Mechanical engineering", "Fastening"]
tags = ["bolt group", "bolt pattern", "eccentric load", "shear", "torsion", "moment shear", "primary shear", "secondary shear", "centroid", "bearing", "rivet group", "fitted bolt", "Shigley"]
from = "lacewing:mechanical.boltGroups"
license = "GPL-3.0-or-later"
inputs = ["x_b", "y_b", "d_b", "F_x", "F_y", "a_x", "a_y", "M_z", "t_min", "τ_all", "σ_(br,all)"]
outputs = ["A_i", "x_c", "y_c", "T_c", "J_A", "V_d", "V_m", "V_b", "V_max", "τ_max", "σ_br"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-12: Eq. 8-55 (bearing), Eq. 8-56 (centroid), Eq. 8-57 (moment shear in proportion to the radius); Example 8-7"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 10.10, Fig. 10.33, Sample Problem 10.5"

[[tests]]
name = "Shigley Ex. 8-7: four M16 bolts at ±75, ±60 mm, 16 kN acting 425 mm off the centroid, 15 mm bar"
source = "Lacewing Lab bolted_joint_test, after Shigley 10th Example 8-7: FA = FB = 21.0 kN, FC = FD = 14.8 kN, moment shear 17.7 kN, τ = 104 MPa, bearing on the 15 mm bar 87.4 MPa"
inputs = { x_b = "[-75; 75; -75; 75] mm", y_b = "[60; 60; -60; -60] mm", d_b = "16 mm", F_x = "0 kN", F_y = "-16 kN", a_x = "425 mm", a_y = "0 mm", M_z = "0 N·m", t_min = "15 mm", "τ_all" = "150 MPa", "σ_(br,all)" = "200 MPa" }
expect = { x_c = "0 mm", T_c = "-6.8 kN·m", V_max = "21.0 kN", "τ_max" = "104 MPa", "σ_br" = "87.4 MPa" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 10.5: three fasteners, 144 kN acting 150 mm off the centroid"
source = "Lacewing Lab riveted_joints_test, after Juvinall & Marshek 5th Sample Problem 10.5 (Fig. 10.33): fasteners at (±150, 100) and (0, −200) mm, 144 kN down at x = −150 mm; 37.1 kN of moment shear on the top pair, 81.5 kN on the most loaded"
inputs = { x_b = "[-150; 150; 0] mm", y_b = "[100; 100; -200] mm", d_b = "18 mm", F_x = "0 kN", F_y = "-144 kN", a_x = "-150 mm", a_y = "0 mm", M_z = "0 N·m", t_min = "20 mm", "τ_all" = "400 MPa", "σ_(br,all)" = "650 MPa" }
expect = { y_c = "0 mm", T_c = "21.6 kN·m", V_max = "81.5 kN" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 8-7 unrounded"
source = "By hand: J = 4 × π(16)²/4 × (75² + 60²) = 7.419185e6 mm⁴, T = 0.425 × (−16 000) = −6800 N·m; moment shear 6800 × 96.047 × 201.062/7.419185e6 = 17 699.69 N; top bolts: |(−11 056.9, 13 820.9 − 4000)| = 20 972.56 N; τ = 20 972.56/201.062 = 104.3090 MPa, bearing 20 972.56/(15 × 16) = 87.3857 MPa"
inputs = { x_b = "[-75; 75; -75; 75] mm", y_b = "[60; 60; -60; -60] mm", d_b = "16 mm", F_x = "0 kN", F_y = "-16 kN", a_x = "425 mm", a_y = "0 mm", M_z = "0 N·m", t_min = "15 mm", "τ_all" = "150 MPa", "σ_(br,all)" = "200 MPa" }
expect = { J_A = "7.419185e6 mm^4", V_max = "20972.56 N", "τ_max" = "104.3090 MPa", "σ_br" = "87.3857 MPa" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Mixed sizes: the middle of three bolts, twice the area, takes half of a centred load"
source = "Lacewing Lab bolted_joint_test (mixed sizes): areas 1, 2, 1 cm² in a row at 100 mm pitch, 10 kN through the centroid; by hand the middle bolt takes 10 × 2/4 = 5 kN"
inputs = { x_b = "[-100; 0; 100] mm", y_b = "[0; 0; 0] mm", d_b = "[11.28379; 15.95769; 11.28379] mm", F_x = "0 kN", F_y = "-10 kN", a_x = "0 mm", a_y = "0 mm", M_z = "0 N·m", t_min = "10 mm", "τ_all" = "150 MPa", "σ_(br,all)" = "200 MPa" }
expect = { x_c = "0 mm", V_max = "5 kN", "τ_max" = "25 MPa" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Ex. 8-7 against an allowable shear stress of 90 MPa: the top bolts fail"
source = "Shigley 10th Example 8-7: τ = 104 MPa on the most loaded bolt, over 90 MPa"
inputs = { x_b = "[-75; 75; -75; 75] mm", y_b = "[60; 60; -60; -60] mm", d_b = "16 mm", F_x = "0 kN", F_y = "-16 kN", a_x = "425 mm", a_y = "0 mm", M_z = "0 N·m", t_min = "15 mm", "τ_all" = "90 MPa", "σ_(br,all)" = "200 MPa" }
expect = { "τ_max" = "104.3090 MPa" }
tolerance = "1e-6"
checks = "fail"
+++

## Bolt group under eccentric shear

The share of an in-plane load that each fastener of a pattern carries (Shigley Sec. 8-12). The connected parts are taken as rigid and the fasteners as elastic, so the load splits into a direct (primary) shear shared in proportion to each fastener's area, and a moment (secondary) shear from the torque about the pattern's centroid, in proportion to each fastener's area and distance from the centroid and perpendicular to its radius. The two are added as vectors.

This assumes the fasteners bear: rivets, fitted or shoulder bolts. Bolts in clearance holes carry the shear by friction until the joint slips (see the NASA-STD-5020B margins template for slip). Tension and bending out of the plane are the bolt group in tension template. Bending of the fasteners and of the connected bar are separate checks.

### Pattern

Enter the fastener positions as column vectors, in any axes in the plane of the joint; the centroid is found from them. The diameter is the one that carries shear (the minor diameter where threads are in the shear plane): one value for equal fasteners, or a column with one per fastener.

```calc {description="x coordinate of each fastener"}
x_b = [-75; 75; -75; 75] mm_
```

```calc {description="y coordinate of each fastener"}
y_b = [60; 60; -60; -60] mm_
```

```calc {description="Shear diameter (one value, or one per fastener)"}
d_b = 16 mm_
```

### Loads

```calc {description="In-plane force on the pattern, x component"}
F_x = 0 kN_
```

```calc {description="In-plane force on the pattern, y component"}
F_y = -16 kN_
```

```calc {description="x coordinate of the force's line of action"}
a_x = 425 mm_
```

```calc {description="y coordinate of the force's line of action"}
a_y = 0 mm_
```

```calc {description="Applied in-plane torque, anticlockwise positive"}
M_z = 0 N·m_
```

### Member and allowables

```calc {description="Thinnest member the fasteners bear on"}
t_min = 15 mm_
```

```calc {description="Allowable shear stress of the fasteners"}
τ_all = 150 MPa_
```

```calc {description="Allowable bearing stress (the lesser of fastener and member)"}
σ_(br,all) = 200 MPa_
```

### Centroid and torque

```calc
> Number of fasteners
n_b = length(x_b) = ??
> Shear area of each fastener
A_i = π d_b^2 / 4 ∘ ones(n_b, 1) = ?? mm²
> Centroid of the pattern, Eq. 8-56
x_c = sum(A_i ∘ x_b) / sum(A_i) = ?? mm
y_c = sum(A_i ∘ y_b) / sum(A_i) = ?? mm
> Each fastener's offset from the centroid
Δx = x_b - x_c = ?? mm
Δy = y_b - y_c = ?? mm
> Torque about the centroid: the applied torque plus the moment of the force
T_c = M_z + (a_x - x_c) F_y - (a_y - y_c) F_x = ?? kN·m
> Polar moment of the fastener areas about the centroid
J_A = sum(A_i ∘ (Δx^2 + Δy^2)) = ?? mm^4
```

### Load on each fastener

```calc
> Direct shear, shared by area: x and y components
V_dx = F_x A_i / sum(A_i) = ?? kN
V_dy = F_y A_i / sum(A_i) = ?? kN
> Moment shear, perpendicular to the radius, Eq. 8-57: x and y components
V_mx = -T_c Δy ∘ A_i / J_A = ?? kN
V_my = T_c Δx ∘ A_i / J_A = ?? kN
> Magnitude of the direct shear on each fastener
V_d = sqrt(V_dx^2 + V_dy^2) = ?? kN
> Magnitude of the moment shear on each fastener
V_m = sqrt(V_mx^2 + V_my^2) = ?? kN
> Resultant shear on each fastener
V_b = sqrt((V_dx + V_mx)^2 + (V_dy + V_my)^2) = ?? kN
```

```calc {description="Largest shear on one fastener"}
V_max = max(V_b) = ?? kN
```

```calc
> Largest shear stress on a fastener body
τ_max = max(V_b / A_i) = ?? MPa
> Largest bearing stress on the thinnest member, Eq. 8-55
σ_br = max(V_b / (t_min d_b ∘ ones(n_b, 1))) = ?? MPa
```

### Checks

```calc
@test τ_max ≤ τ_all
```

```calc
@test σ_br ≤ σ_(br,all)
```
