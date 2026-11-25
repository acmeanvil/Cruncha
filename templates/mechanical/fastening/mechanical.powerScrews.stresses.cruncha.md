+++
cruncha-template = 1
id = "mechanical.powerScrews.stresses"
version = "1.0.0"
title = "Power screw stresses: body and thread root"
summary = "Torsional and axial stresses in a power screw's body at the root diameter, bearing and bending stresses on the first engaged thread, their von Mises combinations at the thread root and in the body, the average thread bearing stress over the nut, and the factor of safety on yield checked against a design factor."
category = ["Mechanical engineering", "Fastening"]
tags = ["power screw", "lead screw", "jack", "Acme", "square thread", "thread bending", "bearing stress", "von Mises", "root diameter", "factor of safety"]
from = "lacewing:mechanical.powerScrews"
license = "GPL-3.0-or-later"
inputs = ["d", "p", "F", "T_raise", "x_1", "t_n", "S_y", "n_d"]
outputs = ["d_m", "d_r", "τ", "σ_a", "σ_B", "σ_b", "σ′", "σ_body′", "σ_Bav", "n"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-2, Fig. 8-3, Eqs. 8-7, 8-8, 8-10 and 8-11 (body and thread stresses, the first thread's share of the load); Eq. 5-14; Example 8-1"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 10.4: 10.4.1 to 10.4.3 (torsion, compression, distortion energy), Eq. 10.10 (average thread bearing stress)"

[[tests]]
name = "Shigley Ex. 8-1: 32 mm square thread, 4 mm pitch, 6.4 kN, 26.18 N·m"
source = "Lacewing Lab threads_test, after Shigley 10th Example 8-1: body τ = 6.07 MPa, σ = −10.39 MPa; first thread σB = −12.9 MPa, σb = 41.5 MPa; von Mises at the root σ′ = 48.7 MPa"
inputs = { d = "32 mm", p = "4 mm", F = "6.4 kN", T_raise = "26.18 N·m", x_1 = "0.38", t_n = "16 mm", S_y = "370 MPa", n_d = "2" }
expect = { "τ" = "6.07 MPa", "σ_a" = "-10.39 MPa", "σ_B" = "-12.9 MPa", "σ_b" = "41.5 MPa", "σ′" = "48.7 MPa" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 8-1's screw with a 16 mm nut, worked Juvinall's way too"
source = "Lacewing Lab threads_test and by hand: dr = 28 mm; τ = 16·26.176955/(π·0.028³) = 6.073166 MPa; σ = −4·6400/(π·0.028²) = −10.393792 MPa; σ′ body = √(σ² + 3τ²) = 14.787865 MPa; σB av = 4·6400·0.004/(π(0.032² − 0.028²)·0.016) = 8.488264 MPa; n = 370/48.678407 = 7.600906"
inputs = { d = "32 mm", p = "4 mm", F = "6.4 kN", T_raise = "26.176954745 N·m", x_1 = "0.38", t_n = "16 mm", S_y = "370 MPa", n_d = "2" }
expect = { d_m = "30 mm", d_r = "28 mm", "τ" = "6.07316581 MPa", "σ_a" = "-10.3937922 MPa", "σ_B" = "-12.9021607 MPa", "σ_b" = "41.4712309 MPa", "σ′" = "48.6784068 MPa", "σ_body′" = "14.7878648 MPa", "σ_Bav" = "8.48826363 MPa", n = "7.60090611" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same screw in a weak material: not enough margin"
source = "By hand: σ′ = 48.68 MPa against Sy = 90 MPa gives n = 1.85, under the design factor of 2"
inputs = { d = "32 mm", p = "4 mm", F = "6.4 kN", T_raise = "26.18 N·m", x_1 = "0.38", t_n = "16 mm", S_y = "90 MPa", n_d = "2" }
expect = { n = "1.849" }
tolerance = "0.1%"
checks = "fail"
+++

## Power screw stresses: body and thread root

Static stresses in a square or Acme power screw lifting an axial load: torsion and compression in the body at the root diameter, and bearing and bending on the first engaged thread, which experiments show carrying about 38 % of the load (Shigley Sec. 8-2). The thread depth is p/2, so the mean diameter is d − p/2 and the root diameter d − p. The torque is the whole raising torque, collar included, as both Shigley and Juvinall take it: bind it to the torque template's T_raise.

The nut's own strength, buckling of a long screw (use a column template on the root diameter) and fatigue are separate checks.

### Screw and load

```calc {description="Major (outside) diameter of the screw"}
d = 32 mm_
```

```calc {description="Pitch, the axial distance between adjacent threads"}
p = 4 mm_
```

```calc {description="Axial load raised (compressive in the screw)"}
F = 6.4 kN_
```

```calc {description="Torque to raise the load, collar included"}
T_raise = 26.18 N·m_
```

```calc {description="Share of the load carried by the first engaged thread (0.38, Shigley Sec. 8-2)"}
x_1 = 0.38_
```

```calc {description="Engaged length of the nut"}
t_n = 16 mm_
```

```calc {description="Yield strength of the screw"}
S_y = 370 MPa_
```

```calc {description="Design factor the factor of safety must reach"}
n_d = 2_
```

### Diameters

```calc
> Mean thread diameter, Fig. 8-3
d_m = d - p / 2 = ?? mm
> Root (minor) diameter
d_r = d - p = ?? mm
```

### Body

```calc
> Torsional shear at the root diameter, Eq. 8-7
τ = 16 T_raise / (π d_r^3) = ?? MPa
> Axial stress, Eq. 8-8: compressive when the screw pushes the load
σ_a = -4 F / (π d_r^2) = ?? MPa
> Distortion-energy stress of the body, Juvinall Sec. 10.4.3
σ_body′ = sqrt(σ_a^2 + 3 τ^2) = ?? MPa
```

### First engaged thread

```calc
> Bearing stress on the first thread, Eq. 8-10
σ_B = -2 x_1 F / (π d_m p) = ?? MPa
> Bending stress at the root of the first thread, Eq. 8-11
σ_b = 6 x_1 F / (π d_r p) = ?? MPa
```

At the root of the first thread, the thread's bending stress, the body's axial stress and its torsional shear act together. Shigley Example 8-1 combines them by Eq. 5-14 with σx = σb, σy = 0, σz = σa and τyz = τ.

```calc {description="Von Mises stress at the root of the first thread"}
σ′ = sqrt((σ_b^2 + σ_a^2 + (σ_a - σ_b)^2) / 2 + 3 τ^2) = ?? MPa
```

### Thread bearing over the nut

The average over the t/p threads in contact, with the nut's minor diameter taken as the screw's root diameter (Juvinall Eq. 10.10). The first thread carries more than its share, so compare it with a low allowable bearing pressure for the screw and nut materials.

```calc {description="Average thread bearing stress over the nut"}
σ_Bav = 4 F p / (π (d^2 - d_r^2) t_n) = ?? MPa
```

### Check

```calc {description="Factor of safety on yield at the worst of the thread root and the body"}
n = S_y / max(σ′, σ_body′) = ??
```

```calc
@test n ≥ n_d
```
