+++
cruncha-template = 1
id = "structural.bolts.shearTension"
version = "1.0.0"
title = "Bolt shear and tension, AISC 360 (LRFD)"
summary = "Design shear and tension strength of one high-strength bolt, and tension reduced by shear (combined), checked against the factored loads on it."
category = ["Structural engineering", "Connections"]
tags = ["bolt", "AISC 360", "J3", "shear", "tension", "combined", "LRFD", "A325", "A490", "F3125"]
from = "lacewing:structural.bolts"
license = "GPL-3.0-or-later"
inputs = ["d_b", "n_s", "F_nv", "F_nt", "V_u", "T_u"]
outputs = ["A_b", "ϕR_nv", "ϕR_nt", "f_rv", "F_nt′", "ϕR_ntc"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J3.7 (Eq. J3-1), J3.8 (Eqs. J3-3a, J3-3b), Table J3.2"

[[references]]
work = "AISC Steel Construction Manual"
edition = "16th"
locator = "Tables 7-1 and 7-2 (available bolt strengths)"

[[tests]]
name = "3/4 in. Group 120, threads included (N)"
source = "Lacewing Lab engine test, after AISC Manual 16th Tables 7-1 and 7-2: single shear 17.9 kips, tension 29.8 kips"
inputs = { d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "5 kips", T_u = "10 kips" }
expect = { "ϕR_nv" = "17.9 kips", "ϕR_nt" = "29.8 kips" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "3/4 in. Group 120, threads excluded (X)"
source = "Lacewing Lab engine test, after AISC Manual 16th Table 7-1: single shear 22.5 kips"
inputs = { d_b = "0.75 in", n_s = "1", F_nv = "68 ksi", F_nt = "90 ksi", V_u = "5 kips", T_u = "10 kips" }
expect = { "ϕR_nv" = "22.5 kips" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Combined tension and shear at f_rv = 20 ksi"
source = "Lacewing Lab engine test, AISC 360-22 Eq. J3-3a: F′nt = 1.3·90 − 90/(0.75·54)·20 = 72.56 ksi"
inputs = { d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "8.83573 kips", T_u = "10 kips" }
expect = { f_rv = "20 ksi", "F_nt′" = "72.5556 ksi" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Combined, light shear: F′nt is capped at Fnt"
source = "Lacewing Lab engine test: at f_rv = 1 ksi, Eq. J3-3a exceeds Fnt, so F′nt = Fnt = 90 ksi"
inputs = { d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "0.441786 kips", T_u = "10 kips" }
expect = { "F_nt′" = "90 ksi" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Overloaded in tension: the check fails"
source = "As the combined case, with 30 kips of tension against 24.0 kips available"
inputs = { d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "8.83573 kips", T_u = "30 kips" }
expect = { "ϕR_ntc" = "24.04 kips" }
tolerance = "0.05%"
checks = "fail"
+++

## Bolt shear and tension (AISC 360, LRFD)

Design strength of **one** bolt in a bearing-type connection, for the factored shear and tension it carries. The bolt group's force distribution, bearing and tearout at the holes, and slip resistance are separate checks.

### Bolt and loads

Nominal stresses come from AISC 360 Table J3.2, by bolt group and whether the threads are in the shear plane (N: included, X: excluded):

- **A307:** F_nt 45 ksi; F_nv 27 ksi.
- **Group 120** (A325, F3125 Grade A325): F_nt 90 ksi; F_nv 54 ksi (N) or 68 ksi (X).
- **Group 150** (A490, F3125 Grade A490): F_nt 113 ksi; F_nv 68 ksi (N) or 84 ksi (X).

```calc {description="Nominal bolt diameter"}
d_b = 0.75 in_
```

```calc {description="Shear planes through the bolt (1 single shear, 2 double shear)"}
n_s = 1_
```

```calc {description="Nominal shear stress, Table J3.2"}
F_nv = 54 ksi_
```

```calc {description="Nominal tensile stress, Table J3.2"}
F_nt = 90 ksi_
```

```calc {description="Factored shear on the bolt"}
V_u = 5 kips_
```

```calc {description="Factored tension on the bolt"}
T_u = 10 kips_
```

### Strength

The nominal area is the unthreaded body; Table J3.2's stresses already allow for the threads.

```calc {description="Nominal unthreaded body area"}
A_b = π d_b^2 / 4 = ?? in²
```

```calc {description="Resistance factor for bolts in shear and tension (J3.7)"}
ϕ = 0.75
```

```calc {description="Design shear strength, Eq. J3-1"}
ϕR_nv = ϕ F_nv A_b n_s = ?? kips
```

```calc {description="Design tensile strength, Eq. J3-1"}
ϕR_nt = ϕ F_nt A_b = ?? kips
```

### Combined tension and shear

Shear reduces the tension a bolt can take (J3.8). The available tensile stress falls linearly with the required shear stress, and never exceeds F_nt.

```calc
> Required shear stress
f_rv = V_u / (A_b n_s) = ?? ksi
> Nominal tensile stress reduced for shear, Eq. J3-3a
F_nt′ = min(1.3 F_nt - F_nt / (ϕ F_nv) f_rv, F_nt) = ?? ksi
> Design tensile strength with shear, Eq. J3-2
ϕR_ntc = ϕ F_nt′ A_b = ?? kips
```

### Checks

```calc
@test V_u ≤ ϕR_nv
```

```calc
@test T_u ≤ ϕR_ntc
```
