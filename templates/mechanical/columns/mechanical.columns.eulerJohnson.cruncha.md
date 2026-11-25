+++
cruncha-template = 1
id = "mechanical.columns.eulerJohnson"
version = "1.0.1"
title = "Column buckling: Euler or Johnson, by slenderness"
summary = "Critical load of a centrally loaded column: Euler's formula for long columns, the Johnson parabola for intermediate ones, chosen by comparing the slenderness ratio with the tangent point."
category = ["Mechanical engineering", "Columns"]
tags = ["column", "buckling", "Euler", "Johnson", "slenderness", "critical load", "strut"]
from = "lacewing:structural.columnBuckling"
license = "GPL-3.0-or-later"
inputs = ["E", "S_y", "l_e", "A", "I", "P", "n_d"]
outputs = ["k", "λ", "λ_1", "P_cr"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 4-12 to 4-14 (Eqs. 4-43, 4-44, 4-48); Examples 4-17 and 4-19"

[[tests]]
name = "Shigley Ex. 4-17: a 1.5 m pinned round steel column, d 37.48 mm"
source = "Lacewing Lab columns_test: d = 37.48 mm gives Pcr = 88 kN by Euler, (l/k)₁ = 90.4 at Sy 500 MPa, E 207 GPa"
inputs = { E = "207 GPa", S_y = "500 MPa", l_e = "1.5 m", A = "1103.288 mm^2", I = "96865.30 mm^4", P = "40 kN", n_d = "2" }
expect = { "λ_1" = "90.4", P_cr = "88 kN" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 4-19: a 5/8 × 0.76 in link, 15 in long, is a Johnson column"
source = "Lacewing Lab columns_test, after Shigley Table 4-3: (l/k)₁ = 88.9 at Sy 75 kpsi, E 30 Mpsi; l/k = 83; Pcr ≈ 20 000 lbf"
inputs = { E = "30000000 psi", S_y = "75000 psi", l_e = "15 in", A = "0.475 in^2", I = "0.0154622 in^4", P = "8000 lbf", n_d = "2.5" }
expect = { "λ" = "83.14", "λ_1" = "88.9", P_cr = "20000 lbf" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "The same link at 12 000 lbf: not enough margin"
source = "Pcr ≈ 20 000 lbf over a design factor of 2.5 allows 8 000 lbf"
inputs = { E = "30000000 psi", S_y = "75000 psi", l_e = "15 in", A = "0.475 in^2", I = "0.0154622 in^4", P = "12000 lbf", n_d = "2.5" }
expect = { P_cr = "20031.7 lbf" }
tolerance = "0.01%"
checks = "fail"
+++

## Column buckling: Euler or Johnson

Critical load of a straight, centrally loaded column with uniform section. Long columns buckle elastically (Euler); intermediate ones fail at a lower stress, which the Johnson parabola gives. The two meet tangentially at the slenderness ratio (l/k)₁, where the critical stress is half the yield strength.

The effective length l_e allows for the end conditions: l for both ends pinned, 0.5 l fixed–fixed, 2 l for a flagpole (Shigley recommends more conservative values for real ends). Eccentric loads need the secant formula instead.

### Material and column

```calc {description="Modulus of elasticity"}
E = 207 GPa_
```

```calc {description="Yield strength"}
S_y = 500 MPa_
```

```calc {description="Effective length for the end conditions"}
l_e = 1.5 m_
```

```calc {description="Cross-sectional area"}
A = 1103.288 mm²_
```

```calc {description="Least second moment of area"}
I = 96865.30 mm⁴_
```

```calc {description="Axial load"}
P = 40 kN_
```

```calc {description="Design factor on the critical load"}
n_d = 2_
```

### Slenderness

```calc
> Least radius of gyration
k = sqrt(I / A) = ?? mm
> Slenderness ratio
λ = l_e / k = ??
> Slenderness ratio at the tangent point of Euler and Johnson, Eq. 4-48
λ_1 = sqrt(2 π^2 E / S_y) = ??
```

### Critical load

```calc {description="Critical load: Euler (Eq. 4-43) above λ₁, Johnson (Eq. 4-44) below"}
P_cr = { π^2 E A / λ^2 if λ ≥ λ_1; A (S_y - (S_y λ / (2 π))^2 / E) otherwise } = ?? kN
```

### Check

```calc
@test P ≤ P_cr / n_d
```
