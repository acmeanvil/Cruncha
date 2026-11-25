+++
cruncha-template = 1
id = "mechanical.flatSprings.table19"
version = "1.0.0"
title = "Flat spring: supported, cantilever, triangular or tapered (Machinery's Table 19)"
summary = "A flat spring of rectangular strip, simply supported with the load at the centre, or a cantilever of uniform width, of triangular plan or tapering to a quarter of its root width: rate, deflection, bending stress, factor of safety on yield and energy stored, by the beam formulas of Machinery's Handbook Table 19."
category = ["Mechanical engineering", "Springs"]
tags = ["flat spring", "leaf spring", "strip", "cantilever", "simply supported", "triangular", "tapered", "beam of uniform strength", "spring rate", "bending stress", "Machinery's Handbook", "Shigley"]
from = "lacewing:mechanical.flatSprings"
license = "GPL-3.0-or-later"
inputs = ["form", "L", "b", "t", "E", "P", "S_y"]
outputs = ["c_f", "c_s", "k", "f", "S_b", "n", "U"]

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "Table 19, flat springs (p. 327): deflection and stress formulas for the four plan forms"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 10-14, Eq. 10-64 (triangular cantilever plate)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 12.10, Fig. 12.22, Eqs. 12.13 to 12.15 (triangular plate); Sample Problem 12.4"

[[tests]]
name = "Uniform cantilever: PL³/(3EI)"
source = "Lacewing Lab coil_springs_test: L = 100 mm, b = 20 mm, t = 1 mm, E = 200 GPa, P = 10 N; f = 10 × 0.1³ / (3 × 200e9 × 0.02 × 1e-9/12) = 10 mm; by hand Sb = 6 × 10 × 100 / (20 × 1) = 300 MPa, U = 10 × 0.01/2 = 0.05 J"
inputs = { form = "2", L = "100 mm", b = "20 mm", t = "1 mm", E = "200 GPa", P = "10 N", S_y = "1000 MPa" }
expect = { k = "1 N/mm", f = "10 mm", S_b = "300 MPa", n = "3.333333", U = "0.05 J" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Triangular cantilever: Shigley Eq. 10-64"
source = "Lacewing Lab coil_springs_test: the same strip as a triangle, k = b t³ E / (6 L³) = 0.02 × 1e-9 × 200e9 / (6 × 1e-3) = 0.666667 N/mm (Shigley Eq. 10-64); by hand f = 15 mm, Sb = 300 MPa at every section"
inputs = { form = "3", L = "100 mm", b = "20 mm", t = "1 mm", E = "200 GPa", P = "10 N", S_y = "1000 MPa" }
expect = { k = "0.6666667 N/mm", f = "15 mm", S_b = "300 MPa", U = "0.075 J" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Simply supported, load at the centre"
source = "By hand, Table 19's simply supported case: k = 4 E b t³ / L³ = 4 × 200e9 × 0.02 × 1e-9 / 1e-3 = 16 N/mm, f = 0.625 mm, Sb = 3PL/(2bt²) = 75 MPa"
inputs = { form = "1", L = "100 mm", b = "20 mm", t = "1 mm", E = "200 GPa", P = "10 N", S_y = "1000 MPa" }
expect = { k = "16 N/mm", f = "0.625 mm", S_b = "75 MPa", n = "13.33333" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Cantilever tapering to b/4"
source = "By hand, Table 19's coefficient 5.22: k = 200e9 × 0.02 × 1e-9 / (5.22 × 1e-3) = 0.766284 N/mm, f = 13.05 mm, Sb = 300 MPa"
inputs = { form = "4", L = "100 mm", b = "20 mm", t = "1 mm", E = "200 GPa", P = "10 N", S_y = "1000 MPa" }
expect = { k = "0.7662835 N/mm", f = "13.05 mm", S_b = "300 MPa", U = "0.06525 J" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Juvinall SP 12.4: half of the five-leaf spring as one triangular plate"
source = "Juvinall & Marshek 5th, Sample Problem 12.4: the semi-elliptic spring of 1364 mm span, 416 mm total leaf width and 7 mm leaves has k = 30 N/mm, each half a triangular cantilever of 682 mm carrying half the load, so one half is 15 N/mm (b t³ E/(6L³) = 14.994 N/mm)"
inputs = { form = "3", L = "682 mm", b = "416 mm", t = "7 mm", E = "200 GPa", P = "1 N", S_y = "1430 MPa" }
expect = { k = "15 N/mm" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Overloaded cantilever: 40 N"
source = "By hand: four times the first case, Sb = 1200 MPa over Sy = 1000 MPa, n = 0.833"
inputs = { form = "2", L = "100 mm", b = "20 mm", t = "1 mm", E = "200 GPa", P = "40 N", S_y = "1000 MPa" }
expect = { S_b = "1200 MPa", n = "0.8333333" }
tolerance = "1e-6"
checks = "fail"
+++

## Flat spring (Machinery's Table 19)

A flat spring of rectangular strip, treated as a beam in bending: good while the deflection is small against the length and the strip stays elastic. The tapered cantilevers are beams of uniform strength, with the same bending stress at every section. For multi-leaf springs use the leaf spring template; for a strip wound in a spiral, the spiral spring template.

### Form

1. Simply supported, the load at the centre of the span.
2. Cantilever of uniform width b, the load at the free end.
3. Cantilever of triangular plan, width b at the clamp tapering to zero at the load (Shigley Eq. 10-64 gives the same rate).
4. Cantilever whose width tapers from b at the clamp to b/4 at the load.

Juvinall & Marshek model a multi-leaf spring as the triangular plate of form 3. Their Sample Problem 12.5 splits a trapezoidal plate into a triangle and a rectangle acting in parallel; that split is an approximation, and Machinery's coefficient for form 4 is a third value again (beam theory gives a coefficient of about 5.13 against Machinery's 5.22).

```calc {description="Form: 1 simply supported, 2 uniform cantilever, 3 triangular, 4 tapering to b/4"}
form = 2_
```

```calc
@test form == round(form) and 1 ≤ form ≤ 4
```

### Spring

L is the span between the supports, or the cantilever's length from the clamp to the load; b is the width (at the clamp for the tapered forms). Spring steel strip has E of about 200 GPa (29 Mpsi).

```calc {description="Span (simply supported) or length from the clamp to the load (cantilever)"}
L = 100 mm_
```

```calc {description="Strip width (at the clamp for the tapered forms)"}
b = 20 mm_
```

```calc {description="Strip thickness"}
t = 1 mm_
```

```calc {description="Young's modulus of the strip"}
E = 200 GPa_
```

```calc {description="Yield strength of the strip"}
S_y = 1000 MPa_
```

### Load

```calc {description="Load at the centre (simply supported) or the free end (cantilever)"}
P = 10 N_
```

### Rate, deflection and stress

Table 19 gives the deflection as c_f P L³ / (E b t³) and the stress as c_s P L / (b t²) for each form.

```calc
> Deflection coefficient, Table 19
c_f = { 0.25 if form == 1; 4 if form == 2; 6 if form == 3; 5.22 if form == 4 } = ??
> Stress coefficient, Table 19
c_s = { 1.5 if form == 1; 6 otherwise } = ??
> Rate
k = E b t^3 / (c_f L^3) = ?? N/mm
> Deflection under the load
f = P / k = ?? mm
> Bending stress: at the centre or the clamp, and all along the tapered forms
S_b = c_s P L / (b t^2) = ?? MPa
> Factor of safety on yield
n = S_y / S_b = ??
> Energy stored
U = P f / 2 = ?? J
```

### Check

```calc
@test S_b ≤ S_y
```
