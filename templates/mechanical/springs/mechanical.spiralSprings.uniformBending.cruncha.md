+++
cruncha-template = 1
id = "mechanical.spiralSprings.uniformBending"
version = "1.0.0"
title = "Flat spiral spring, coils free (strip in uniform bending)"
summary = "A flat strip wound in a spiral (a clock, power or hair spring) with its coils clear of one another: rate, wind-up angle and turns under a moment, bending stress, factor of safety on yield and energy stored, treating the strip as a beam in pure bending."
category = ["Mechanical engineering", "Springs"]
tags = ["spiral spring", "clock spring", "power spring", "hair spring", "flat strip", "wind-up", "torsional rate", "bending stress", "energy stored"]
from = "lacewing:mechanical.spiralSprings"
license = "GPL-3.0-or-later"
inputs = ["b", "t", "L", "E", "M", "S_y"]
outputs = ["I", "k", "θ", "N_θ", "σ", "n", "U"]

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "p. 327 (flat spiral springs; tightly wound and barrel springs to be designed with the springmaker)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 3-10 (bending stress Mc/I) and 4-3 (beam curvature 1/ρ = M/EI), applied to a strip of length L in uniform bending"

[[tests]]
name = "10 × 0.5 mm strip, 1 m active, 0.05 N·m"
source = "Lacewing Lab coil_springs_test (spiral spring σ = 6M/(bt²) = 120 MPa), the rest by hand: I = 0.01 × 0.5e-3³/12 = 1.041667e-13 m⁴, k = 200e9 × 1.041667e-13 / 1 = 0.0208333 N·m/rad, θ = 0.05/0.0208333 = 2.4 rad (0.38197 turns), U = 0.05 × 2.4 / 2 = 0.06 J"
inputs = { b = "10 mm", t = "0.5 mm", L = "1 m", E = "200 GPa", M = "0.05 N·m", S_y = "1000 MPa" }
expect = { I = "1.0416667e-13 m^4", k = "0.02083333 N·m/rad", "θ" = "2.4 rad", "N_θ" = "0.3819719", "σ" = "120 MPa", n = "8.333333", U = "0.06 J" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same strip wound to 0.5 N·m: yields"
source = "By hand: ten times the moment, σ = 1200 MPa over Sy = 1000 MPa, n = 0.833, θ = 24 rad (3.82 turns)"
inputs = { b = "10 mm", t = "0.5 mm", L = "1 m", E = "200 GPa", M = "0.5 N·m", S_y = "1000 MPa" }
expect = { "θ" = "24 rad", "N_θ" = "3.819719", "σ" = "1200 MPa", n = "0.8333333" }
tolerance = "1e-6"
checks = "fail"
+++

## Flat spiral spring

A strip wound in a spiral between an arbor and an outer anchor, its coils clear of one another. A moment M on the arbor bends every section of the strip by the same moment, so the strip is a beam in pure bending: its curvature changes by M/(EI) everywhere, the arbor turns through θ = M L/(E I), and the bending stress 6M/(b t²) is the same all along the strip. Coils that rub on one another, and springs wound tight in a barrel (clock mainsprings), depart from this; Machinery's Handbook advises taking such designs from the springmaker.

### Strip

b is the strip's width (along the arbor), t its thickness, and L the length of strip that bends, between the arbor and the outer anchor. Spring steel strip has E of about 200 GPa (29 Mpsi).

```calc {description="Strip width, along the arbor's axis"}
b = 10 mm_
```

```calc {description="Strip thickness"}
t = 0.5 mm_
```

```calc {description="Active length of strip between the arbor and the outer anchor"}
L = 1 m_
```

```calc {description="Young's modulus of the strip"}
E = 200 GPa_
```

```calc {description="Yield strength of the strip"}
S_y = 1000 MPa_
```

### Moment

```calc {description="Winding moment on the arbor"}
M = 0.05 N·m_
```

### Rate, wind-up and stress

```calc
> Second moment of area of the strip
I = b t^3 / 12 = ?? mm^4
> Rate, moment per radian
k = E I / L = ?? N·m/rad
> Wind-up angle, in radians
θ = M / k = ?? rad
> Wind-up in turns
N_θ = θ / (2 π) = ??
> Bending stress, the same all along the strip
σ = 6 M / (b t^2) = ?? MPa
> Factor of safety on yield
n = S_y / σ = ??
> Energy stored
U = M θ / 2 = ?? J
```

### Check

```calc
@test σ ≤ S_y
```
