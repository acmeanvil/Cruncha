+++
cruncha-template = 1
id = "mechanical.impactLoading.torsion"
version = "1.0.0"
title = "Torsional impact: a spinning disc stopped by its shaft"
summary = "Juvinall's energy method for a solid disc spinning on a round shaft that is stopped suddenly at its far end: the kinetic energy ½Iω², the shaft's torsional rate, the equivalent torque, the shear stress 2√(UG/V), the twist, and the stress against the shear yield strength."
category = ["Mechanical engineering", "Mechanics"]
tags = ["impact", "torsional impact", "torsion", "shaft", "flywheel", "grinding wheel", "jam", "sudden stop", "shear stress", "twist", "energy method", "Juvinall"]
from = "lacewing:mechanical.impactLoading"
license = "GPL-3.0-or-later"
inputs = ["r", "t", "ρ", "n", "d", "L_s", "G", "S_sy"]
outputs = ["I", "U", "K", "T_e", "τ", "θ", "n_s"]

[[references]]
work = "Juvinall, R.C., Marshek, K.M., Fundamentals of Machine Component Design"
edition = "5th"
locator = "Ch. 7, Sec. 7.3: Eqs. 7.3bt, 7.4bt and 7.6, Eq. i (torsional rate); Sample Problem 7.4 and its comment on a 2024-T4 shaft"

[[tests]]
name = "Juvinall Sample Problem 7.4: grinding wheel jammed at 2400 rpm on a 20 mm steel shaft"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.4: wheel r 60 mm, t 20 mm, 2000 kg/m³; shaft 20 mm × 250 mm, G 79 GPa; U = 25.72 J, τ = 322 MPa, θ = 0.10 rad (0.1018 before the book rounds). S_sy = 400 MPa is chosen for the check, not from the book"
inputs = { r = "60 mm", t = "20 mm", "ρ" = "2000 kg/m^3", n = "2400 rpm", d = "20 mm", L_s = "250 mm", G = "79 GPa", S_sy = "400 MPa" }
expect = { U = "25.72 J", "τ" = "321.7 MPa", "θ" = "0.1018" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 7.4, comment: a 2024-T4 aluminum shaft"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.4 (comment 3): with G = 27 GPa the stress is 188 MPa and the twist 0.174 rad. By hand: K = π 0.02⁴ × 27e9/(32 × 0.25) = 1 696.4600 N·m, T_e = √(2 × 25.717846 × 1696.46) = 295.39566 N·m"
inputs = { r = "60 mm", t = "20 mm", "ρ" = "2000 kg/m^3", n = "2400 rpm", d = "20 mm", L_s = "250 mm", G = "27 GPa", S_sy = "400 MPa" }
expect = { "τ" = "188.05472 MPa", "θ" = "0.17412474", K = "1696.4600 N·m", T_e = "295.39566 N·m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Sample Problem 7.4 against a lower shear yield strength"
source = "By hand: the steel shaft's 321.7 MPa exceeds an S_sy of 300 MPa (chosen for the check): n = 0.93"
inputs = { r = "60 mm", t = "20 mm", "ρ" = "2000 kg/m^3", n = "2400 rpm", d = "20 mm", L_s = "250 mm", G = "79 GPa", S_sy = "300 MPa" }
expect = { n_s = "0.93262" }
tolerance = "1e-4"
checks = "fail"
+++

## Torsional impact

A solid disc spinning on a round shaft is stopped suddenly at the shaft's far end, as when a grinding wheel's spindle jams (Juvinall Sec. 7.3, Sample Problem 7.4). The disc's kinetic energy is taken up as elastic torsional energy in the shaft between the disc and the inertia that stops. The shear stress 2√(UG/V) (Eq. 7.6) depends on the shaft's volume, not its diameter alone: a longer shaft of the same diameter takes the same energy at a lower stress, and so does a shaft of lower modulus.

The shaft is taken as massless and elastic, the disc as rigid, and damping is neglected (Juvinall p. 290): the stresses are underestimated. No stress concentration or bending is included. The shear yield strength of a ductile steel is about 0.58 S_y.

### Disc

```calc {description="Radius of the disc"}
r = 60 mm_
```

```calc {description="Thickness of the disc"}
t = 20 mm_
```

```calc {description="Density of the disc (abrasive wheels about 2000 kg/m³)"}
ρ = 2000 kg/m³_
```

```calc {description="Speed at which it is stopped"}
n = 2400 rpm_
```

### Shaft

```calc {description="Diameter of the solid round shaft"}
d = 20 mm_
```

```calc {description="Length of shaft between the disc and the inertia that stops"}
L_s = 250 mm_
```

```calc {description="Shear modulus of the shaft"}
G = 79 GPa_
```

```calc {description="Shear yield strength of the shaft (about 0.58 S_y)"}
S_sy = 400 MPa_
```

### Impact

```calc
> Polar moment of inertia of the solid disc, ½mr²
I = π r^4 t ρ / 2 = ?? kg·m²
> Kinetic energy of the disc
U = I n^2 / 2 = ?? J
> Torsional rate of the shaft, Eq. i
K = π d^4 G / (32 L_s) = ?? N·m
> Equivalent static torque, Eq. 7.4bt
T_e = sqrt(2 U K) = ?? N·m
> Volume of the shaft
V = π d^2 L_s / 4 = ?? mm³
> Shear stress, Eq. 7.6
τ = 2 sqrt(U G / V) = ?? MPa
> Twist of the shaft (radians), Eq. 7.3bt
θ = sqrt(2 U / K) = ??
> Safety factor against shear yield
n_s = S_sy / τ = ??
```

```calc
@test τ ≤ S_sy
```
