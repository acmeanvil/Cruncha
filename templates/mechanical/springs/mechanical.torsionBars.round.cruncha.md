+++
cruncha-template = 1
id = "mechanical.torsionBars.round"
version = "1.0.0"
title = "Torsion bar spring, round bar or tube"
summary = "A round torsion bar or tube twisted by a torque: polar second moment, torsional rate, twist, surface shear stress, factor of safety and twist at yield, energy stored, and with a lever arm the force, travel and rate at the arm's end and along an inclined load."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "torsion bar", "torsion", "round bar", "tube", "shaft", "rate", "twist", "shear stress", "lever arm", "Shigley"]
from = "lacewing:mechanical.torsionBars"
license = "GPL-3.0-or-later"
inputs = ["d", "d_i", "l", "G", "T", "S_sy", "R", "φ"]
outputs = ["J", "k", "θ", "τ_max", "n", "θ_max", "U", "F", "s", "k_R", "k_φ"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-12: Eq. 3-35 (angle of twist), 3-37 (shear stress), 3-38 (J of a round bar), 3-39 (J of a tube)"

[[tests]]
name = "Lacewing: 20 mm bar, 1 m long, 100 N·m"
source = "Lacewing Lab coil_springs_test (torsion bar): τmax = 16T/(πd³) = 16 × 100 / (π 0.02³) = 63.662 MPa"
inputs = { d = "20 mm", d_i = "0 mm", l = "1 m", G = "79 GPa", T = "100 N·m" }
expect = { "τ_max" = "63.66198 MPa", J = "15707.96 mm^4" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Lacewing: 30 mm bar wound 0.1 rad, 300 mm arm at 0.5 rad from square, Ssy 900 MPa"
source = "Lacewing Lab coil_springs_test (torsion bar lever): J = π 0.03⁴/32 = 7.952156e-8 m⁴, k = GJ/l = 6282.204 N·m/rad, so 0.1 rad takes T = 628.2204 N·m; F = T/R = 2094.068 N, k/R² = 69 802.26 N/m, k/(R cos φ)² = 90 634.50 N/m, θ at yield = 0.1 × Ssy/τ"
inputs = { d = "30 mm", d_i = "0 mm", l = "1 m", G = "79 GPa", T = "628.22036 N·m", S_sy = "900 MPa", R = "300 mm", "φ" = "0.5 rad" }
expect = { k = "6282.2036 N·m/rad", "θ" = "0.1 rad", F = "2094.0679 N", s = "30 mm", k_R = "69802.26 N/m", "k_φ" = "90634.50 N/m", "τ_max" = "118.5 MPa", "θ_max" = "0.75949 rad" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Lacewing: a 30/20 mm tube is the solid bar less its bore"
source = "Lacewing Lab coil_springs_test (torsion bar tube): k_tube/k_solid = 1 − (2/3)⁴; J = π(0.03⁴ − 0.02⁴)/32 = 6.381360e-8 m⁴, k = 79e9 J = 5041.274 N·m/rad"
inputs = { d = "30 mm", d_i = "20 mm", l = "1 m", G = "79 GPa", T = "100 N·m" }
expect = { J = "63813.60 mm^4", k = "5041.2744 N·m/rad", "τ_max" = "23.5059 MPa" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Hand check: 1 in bar, 30 in long, 11.5 Mpsi, 5000 lbf·in over a 10 in arm, Ssy 80 ksi"
source = "By hand: J = π/32 = 0.0981748 in⁴, k = 11.5e6 × 0.0981748 / 30 = 37 633.7 lbf·in/rad, θ = 5000/37 633.7 = 0.1328597 rad = 7.6123°, τ = 16 × 5000/π = 25 464.8 psi, n = 3.14159, U = 332.149 lbf·in, F = 500 lbf, s = 1.328597 in, k/R² = 376.337 lbf/in"
inputs = { d = "1 in", d_i = "0 in", l = "30 in", G = "11.5 Mpsi", T = "5000 lbf·in", S_sy = "80 ksi", R = "10 in", "φ" = "0 °" }
expect = { J = "0.09817477 in^4", k = "37633.66 lbf·in/rad", "θ" = "7.61230 °", "τ_max" = "25.46479 ksi", n = "3.141593", U = "332.1494 lbf·in", F = "500 lbf", s = "1.328597 in", k_R = "376.3366 lbf/in", "k_φ" = "376.3366 lbf/in" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Hand check overloaded: the 1 in bar at 18 000 lbf·in"
source = "By hand: τ = 16 × 18 000/π = 91.67 ksi, over Ssy = 80 ksi, n = 0.873"
inputs = { d = "1 in", d_i = "0 in", l = "30 in", G = "11.5 Mpsi", T = "18000 lbf·in", S_sy = "80 ksi", R = "10 in", "φ" = "0 °" }
expect = { n = "0.8726646" }
tolerance = "1e-6"
checks = "fail"
+++

## Torsion bar spring, round bar or tube

A straight bar of solid round section or a round tube, fixed at one end and twisted at the other by a torque T, often through a lever arm. The active length l is the part between the end fittings that twists; the ends are made larger so they don't yield first. Twist and stress follow elementary torsion (Shigley Sec. 3-12): the section stays plane and the material elastic. For square, rectangular or laminated bars, use the rectangular torsion bar template.

### Bar

```calc {description="Outside diameter of the bar"}
d = 30 mm_
```

```calc {description="Bore of a tube (0 for a solid bar)"}
d_i = 0 mm_
```

```calc {description="Active length between the end fittings"}
l = 1 m_
```

Spring steel's shear modulus is about 79 GPa (11.5 Mpsi).

```calc {description="Shear modulus"}
G = 79 GPa_
```

```calc
> Polar second moment of area, Eqs. 3-38 and 3-39
J = π (d^4 - d_i^4) / 32 = ?? mm^4
> Torsional rate, torque per radian, from Eq. 3-35
k = G J / l = ?? N·m/rad
```

### Torque

```calc {description="Torque on the bar"}
T = 500 N·m_
```

```calc {description="Torsional yield strength of the bar"}
S_sy = 900 MPa_
```

Torsion bars are often preset, twisted past yield in the working direction, which leaves residual stresses that allow a higher working stress in that direction only. S_sy here is the plain material's.

```calc
> Twist, Eq. 3-35
θ = T / k = ?? °
> Shear stress at the surface, Eq. 3-37
τ_max = T (d / 2) / J = ?? MPa
> Factor of safety against torsional yield
n = S_sy / τ_max = ??
> Twist at which the surface reaches S_sy
θ_max = n θ = ?? °
> Energy stored
U = T θ / 2 = ?? J
```

### Lever arm

A lever of length R turns the bar's rate into a linear rate k/R² at the arm's end, for a load square to the arm. With the load inclined at φ from square, the rate along the load is about k/(R cos φ)² for small motions about that position; geometric stiffening is left out.

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
@test 0 'mm' ≤ d_i < d
```

```calc
@test n ≥ 1
```
