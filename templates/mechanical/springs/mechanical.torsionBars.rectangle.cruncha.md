+++
cruncha-template = 1
id = "mechanical.torsionBars.rectangle"
version = "1.0.0"
title = "Torsion bar spring, square, rectangular or laminated"
summary = "A torsion bar of square or rectangular section, or a pack of rectangular leaves each twisting about its own axis: torsion constant and peak shear stress by Roark's closed forms, with Shigley's approximate stress beside them, the rate, twist, factor of safety, energy stored, and with a lever arm the force, travel and rate at the arm's end."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "torsion bar", "torsion", "rectangular bar", "square bar", "laminated", "leaf pack", "torsion constant", "rate", "twist", "shear stress", "lever arm", "Roark", "Shigley"]
from = "lacewing:mechanical.torsionBars"
license = "GPL-3.0-or-later"
inputs = ["b", "c", "n_L", "l", "G", "T", "S_sy", "R", "φ"]
outputs = ["K", "k", "θ", "τ_max", "τ_S", "n", "θ_max", "U", "F", "s", "k_R", "k_φ"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 10.1, cases 3 (square) and 4 (rectangle): K and τmax"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-12: Eq. 3-40 (approximate stress in a rectangular bar), the α, β table after Eq. 3-41; Example 3-8"

[[tests]]
name = "Shigley Ex. 3-8: arm of 1.25 × 0.25 in under 450 lbf·in"
source = "Lacewing Lab coil_springs_test, after Shigley 10th Example 3-8: τ = T/(bc²)(3 + 1.8c/b) = 19.4 kpsi by Eq. 3-40; Roark's closed form gives 19.78 kpsi, near the table's α at b/c = 5 (Shigley's approximate form runs about 2 % low there)"
inputs = { b = "1.25 in", c = "0.25 in", n_L = "1", l = "4 in", G = "11.5 Mpsi", T = "450 lbf·in" }
expect = { "τ_S" = "19.4 kpsi", "τ_max" = "19.78 kpsi" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shigley's table at b/c = 1: a 20 mm square bar"
source = "Lacewing Lab coil_springs_test, after the table after Shigley 10th Eq. 3-41 (from Timoshenko): β = 0.141 and α = 0.208 at b/c = 1, so K = 0.141 × 0.02⁴ = 22 560 mm⁴ and τ = 100 N·m / (0.208 × 0.02³) = 60.10 MPa"
inputs = { b = "20 mm", c = "20 mm", n_L = "1", l = "1 m", G = "79 GPa", T = "100 N·m" }
expect = { K = "22560 mm^4", "τ_max" = "60.10 MPa" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shigley's table at b/c = 4: a 40 × 10 mm bar"
source = "Lacewing Lab coil_springs_test, after the table after Shigley 10th Eq. 3-41 (from Timoshenko): β = 0.281 and α = 0.282 at b/c = 4, so K = 0.281 × 40 × 10³ = 11 240 mm⁴ and τ = 1 N·m / (0.282 × 0.04 × 0.01²) = 0.8865 MPa"
inputs = { b = "40 mm", c = "10 mm", n_L = "1", l = "1 m", G = "79 GPa", T = "1 N·m" }
expect = { K = "11240 mm^4", "τ_max" = "0.8865 MPa" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: a pack of two 50 × 5 mm leaves, 1 m, 20 N·m, Ssy 600 MPa"
source = "By hand, Roark Table 10.1 case 4 with a = 25, b = 2.5 mm (b/a = 0.1): K = 25 × 2.5³ (16/3 − 0.336(1 − 0.1⁴/12)) = 1952.0844 mm⁴ per leaf; k = 2 × 79e3 × 1952.0844e-12 / 1 = 308.42934 N·m/rad; τ = 3 × 20/(8 × 2 × 25 × 2.5²) mm⁻³ × (1 + 0.06095 + 0.008865 − 0.0018023 + 0.000091) = 25.634489 MPa; θ = 20/308.42934 = 0.064845 rad"
inputs = { b = "50 mm", c = "5 mm", n_L = "2", l = "1 m", G = "79 GPa", T = "20 N·m", S_sy = "600 MPa", R = "200 mm", "φ" = "0 °" }
expect = { K = "1952.0844 mm^4", k = "308.42934 N·m/rad", "τ_max" = "25.634489 MPa", "θ" = "0.06484467 rad", F = "100 N", k_R = "7710.734 N/m" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The Ex. 3-8 arm at 2000 lbf·in: over a torsional yield of 60 ksi"
source = "By hand: Roark's τ = 19.78 × 2000/450 = 87.9 kpsi, over Ssy = 60 ksi"
inputs = { b = "1.25 in", c = "0.25 in", n_L = "1", l = "4 in", G = "11.5 Mpsi", T = "2000 lbf·in", S_sy = "60 ksi" }
expect = { "τ_max" = "87.89 kpsi" }
tolerance = "0.1%"
checks = "fail"
+++

## Torsion bar spring, square, rectangular or laminated

A straight bar of b × c rectangular section (b the longer side; b = c for a square), fixed at one end and twisted at the other by a torque T, often through a lever arm. The section warps as it twists, so neither the stiffness nor the stress follows the round-bar formulas. Roark's Table 10.1 (cases 3 and 4) gives closed forms on the half-sides, accurate over every side ratio; the peak shear stress is at the middle of the long sides. Shigley's Eq. 3-40 is the simpler approximate stress, shown beside it.

A laminated pack is n_L such leaves side by side, free to slide on each other, each twisting about its own axis: the rate is n_L times a leaf's and each leaf carries T/n_L. A pack clamped so that it twists about its own centre is stiffer, because its outer leaves also stretch; that case is not covered. For round bars and tubes, use the round torsion bar template.

### Bar

```calc {description="Longer side of the section (of each leaf, for a pack)"}
b = 50 mm_
```

```calc {description="Shorter side of the section"}
c = 10 mm_
```

```calc {description="Number of leaves in a laminated pack (1 for a single bar)"}
n_L = 1_
```

```calc {description="Active length between the end fittings"}
l = 1 m_
```

```calc {description="Shear modulus"}
G = 79 GPa_
```

```calc
> Half-sides, Roark's a ≥ b
a_h = b / 2 = ?? mm
b_h = c / 2 = ?? mm
> Side ratio
r = b_h / a_h = ??
> Torsion constant of one bar or leaf, Roark Table 10.1 case 4
K = a_h b_h^3 (16 / 3 - 3.36 r (1 - r^4 / 12)) = ?? mm^4
> Torsional rate, torque per radian
k = n_L G K / l = ?? N·m/rad
```

### Torque

```calc {description="Torque on the bar or pack"}
T = 200 N·m_
```

```calc {description="Torsional yield strength of the bar"}
S_sy = 600 MPa_
```

```calc
> Twist
θ = T / k = ?? °
> Largest shear stress, middle of the long side, Roark Table 10.1 case 4
τ_max = 3 T / (8 n_L a_h b_h^2) (1 + 0.6095 r + 0.8865 r^2 - 1.8023 r^3 + 0.9100 r^4) = ?? MPa
> Shigley's approximate stress, Eq. 3-40
τ_S = T / (n_L b c^2) (3 + 1.8 c / b) = ?? MPa
> Factor of safety against torsional yield
n = S_sy / τ_max = ??
> Twist at which the bar reaches S_sy
θ_max = n θ = ?? °
> Energy stored
U = T θ / 2 = ?? J
```

### Lever arm

A lever of length R turns the bar's rate into k/R² at the arm's end, for a load square to the arm; with the load at φ from square, about k/(R cos φ)² along it for small motions.

```calc {description="Lever arm, from the bar's axis to the load"}
R = 300 mm_
```

```calc {description="Angle of the load from square to the arm"}
φ = 0°_
```

```calc
> Force at the arm's end, square to the arm
F = T / R = ?? N
> Arc travelled by the arm's end
s = R θ = ?? mm
> Rate at the arm's end
k_R = k / R^2 = ?? N/mm
> Rate along the inclined load
k_φ = k / (R cos(φ))^2 = ?? N/mm
```

### Checks

```calc
@test b ≥ c
```

```calc
@test n ≥ 1
```
