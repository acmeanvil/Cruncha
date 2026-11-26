+++
cruncha-template = 1
id = "structural.bolts.shearTension"
version = "1.1.0"
title = "Bolt shear and tension, AISC 360 (LRFD or ASD)"
summary = "Available shear and tension strength of one high-strength bolt, and tension reduced by shear (combined), by LRFD or ASD, checked against the required loads on it."
category = ["Structural engineering", "Connections"]
tags = ["bolt", "AISC 360", "J3", "shear", "tension", "combined", "LRFD", "ASD", "A325", "A490", "F3125", "Table J3.2"]
from = "lacewing:structural.bolts"
license = "GPL-3.0-or-later"
inputs = ["LRFD", "d_b", "n_s", "F_nv", "F_nt", "V_u", "T_u"]
outputs = ["A_b", "R_(a,nv)", "R_(a,nt)", "f_rv", "F_nt′", "R_(a,ntc)"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J3.7 (Eq. J3-1, ϕ 0.75, Ω 2.00), J3.8 (Eqs. J3-2, J3-3a, J3-3b), Table J3.2 (pp. 16.1-137 to 16.1-142)"

[[references]]
work = "AISC Steel Construction Manual"
edition = "15th"
locator = "Table 7-1, p. 7-22, and Table 7-2, p. 7-23 (available shear and tensile strength of bolts)"

[[tests]]
name = "3/4 in. Group 120, threads included (N), LRFD"
source = "AISC 360-22 Eq. J3-1 and Table J3.2 (verified: Group 120 F_nt 90 ksi, F_nv 54 ksi N): 0.75 × 54 × 0.4418 = 17.89 kips, 0.75 × 90 × 0.4418 = 29.82 kips; AISC Manual 15th Table 7-1 (p. 7-22) and Table 7-2 (p. 7-23), 3/4 in. Group A (verified): ϕr_n 17.9 kips single shear N, 29.8 kips tension; also Lacewing Lab engine test"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "5 kips", T_u = "10 kips" }
expect = { "R_(a,nv)" = "17.9 kips", "R_(a,nt)" = "29.8 kips" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "3/4 in. Group 120-N, ASD"
source = "AISC Manual 15th Tables 7-1 and 7-2, 3/4 in. Group A (verified): r_n/Ω 11.9 kips single shear N, 19.9 kips tension; by hand, Eq. J3-1 with Ω = 2.00: 54 × 0.44179 / 2 = 11.93 kips, 90 × 0.44179 / 2 = 19.88 kips"
inputs = { LRFD = "0", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "3 kips", T_u = "6 kips" }
expect = { "R_(a,nv)" = "11.928 kips", "R_(a,nt)" = "19.880 kips" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "3/4 in. Group 120, threads excluded (X), LRFD"
source = "Table J3.2 (verified: F_nv 68 ksi X): 0.75 × 68 × 0.4418 = 22.5 kips; AISC Manual 15th Table 7-1 (verified): 3/4 in. Group A, X, single shear ϕr_n 22.5 kips (r_n/Ω 15.0)"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "68 ksi", F_nt = "90 ksi", V_u = "5 kips", T_u = "10 kips" }
expect = { "R_(a,nv)" = "22.5 kips" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Combined tension and shear at f_rv = 20 ksi, LRFD"
source = "AISC 360-22 Eq. J3-3a (verified): F′nt = 1.3 × 90 − 90 / (0.75 × 54) × 20 = 72.56 ksi; also Lacewing Lab engine test"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "8.83573 kips", T_u = "10 kips" }
expect = { f_rv = "20 ksi", "F_nt′" = "72.5556 ksi" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Combined tension and shear at f_rv = 20 ksi, ASD"
source = "AISC 360-22 Eq. J3-3b (verified): F′nt = 1.3 × 90 − 2.00 × 90 / 54 × 20 = 50.33 ksi; R_(a,ntc) = 50.333 × 0.44179 / 2.00 = 11.12 kips"
inputs = { LRFD = "0", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "8.83573 kips", T_u = "10 kips" }
expect = { f_rv = "20 ksi", "F_nt′" = "50.3333 ksi", "R_(a,ntc)" = "11.118 kips" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Combined, light shear: F′nt is capped at Fnt"
source = "Eq. J3-3a (verified): at f_rv = 1 ksi it exceeds F_nt, so F′nt = F_nt = 90 ksi; also Lacewing Lab engine test"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "0.441786 kips", T_u = "10 kips" }
expect = { "F_nt′" = "90 ksi" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Overloaded in tension: the check fails (LRFD)"
source = "As the LRFD combined case, with 30 kips of tension against 24.04 kips available"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", F_nt = "90 ksi", V_u = "8.83573 kips", T_u = "30 kips" }
expect = { "R_(a,ntc)" = "24.04 kips" }
tolerance = "0.05%"
checks = "fail"
+++

## Bolt shear and tension (AISC 360)

Available strength of **one** bolt in a bearing-type connection, for the shear and tension it carries, by LRFD (ϕ R_n against factored loads) or ASD (R_n / Ω against service loads). The bolt group's force distribution, bearing and tearout at the holes, and slip resistance are separate checks.

### Method

```calc {description="Design method: 1 LRFD, 0 ASD"}
LRFD = 1_
```

### Bolt and loads

The nominal stresses come from AISC 360 Table J3.2, by bolt group and by whether the threads are in the shear plane (N) or excluded from it (X). For end-loaded patterns longer than 38 in., the table's note reduces F_nv to 83.3 %.

```calc {description="Nominal bolt diameter"}
d_b = 0.75 in_
```

```calc {description="Shear planes through the bolt (1 single shear, 2 double shear)"}
n_s = 1_
```

```calc {description="Nominal shear stress, AISC 360 Table J3.2"}
F_nv = 54 ksi_
```

```calc {description="Nominal tensile stress, AISC 360 Table J3.2"}
F_nt = 90 ksi_
```

```calc {description="Required shear on the bolt (factored for LRFD, service for ASD)"}
V_u = 5 kips_
```

```calc {description="Required tension on the bolt (factored for LRFD, service for ASD)"}
T_u = 10 kips_
```

### Strength

The nominal area is the unthreaded body; Table J3.2's stresses already allow for the threads.

```calc {description="Nominal unthreaded body area"}
A_b = π d_b^2 / 4 = ?? in²
```

```calc {description="Resistance factor for bolts in shear and tension (J3.7, J3.8)"}
ϕ = 0.75
```

```calc {description="Safety factor for bolts in shear and tension (J3.7, J3.8)"}
Ω = 2.00
```

```calc
> Nominal shear strength, Eq. J3-1
R_nv = F_nv A_b n_s = ?? kips
> Available shear strength
R_(a,nv) = { ϕ R_nv if LRFD == 1; R_nv / Ω otherwise } = ?? kips
```

```calc
> Nominal tensile strength, Eq. J3-1
R_nt = F_nt A_b = ?? kips
> Available tensile strength
R_(a,nt) = { ϕ R_nt if LRFD == 1; R_nt / Ω otherwise } = ?? kips
```

### Combined tension and shear

Shear reduces the tension a bolt can take (J3.8). The available tensile stress falls linearly with the required shear stress, and never exceeds F_nt.

```calc
> Required shear stress
f_rv = V_u / (A_b n_s) = ?? ksi
> Nominal tensile stress reduced for shear, Eq. J3-3a (LRFD) or J3-3b (ASD)
F_nt′ = { min(1.3 F_nt - F_nt / (ϕ F_nv) f_rv, F_nt) if LRFD == 1; min(1.3 F_nt - Ω F_nt / F_nv f_rv, F_nt) otherwise } = ?? ksi
> Available tensile strength with shear, Eq. J3-2
R_(a,ntc) = { ϕ F_nt′ A_b if LRFD == 1; F_nt′ A_b / Ω otherwise } = ?? kips
```

### Checks

```calc
@test V_u ≤ R_(a,nv)
```

```calc
@test T_u ≤ R_(a,ntc)
```
