+++
cruncha-template = 1
id = "structural.ltb.rectBarF11"
version = "1.0.0"
title = "Rectangular bar in major-axis bending, AISC 360 F11 (LRFD and ASD)"
summary = "Design flexural strength of a solid rectangular steel bar bent about its major axis: yielding, and lateral-torsional buckling by the bar's Lb d/t², checked against the required moment by LRFD and by ASD."
category = ["Structural engineering", "Members"]
tags = ["AISC 360", "F11", "rectangular bar", "flat bar", "plate", "flexure", "lateral-torsional buckling", "LTB", "Cb", "LRFD", "ASD"]
from = "lacewing:structural.lateralBuckling"
license = "GPL-3.0-or-later"
inputs = ["F_y", "E", "d", "t", "L_b", "C_b", "M_u", "M_a"]
outputs = ["S_x", "Z_x", "M_p", "ψ", "M_n", "ϕM_n", "M_(n,ASD)"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. F11 (Eq. F11-1 yielding, FyZ ≤ 1.5 Fy Sx for rectangular bars; F11.2 lateral-torsional buckling, Eqs. F11-3 to F11-5); Sec. F1 (ϕb 0.90, Ωb 1.67)"

[[tests]]
name = "6 × 3/4 in. A36 bar, Lb 12 in.: inelastic LTB"
source = "By hand: Sx = 0.75 × 6²/6 = 4.5 in³, Z = 6.75 in³, Mp = min(36 × 6.75, 1.5 × 36 × 4.5) = 243 kip-in; Lb d/t² = 12 × 6/0.5625 = 128, between 0.08E/Fy = 64.44 and 1.9E/Fy = 1530.6; Mn = (1.52 − 0.274 × 128 × 36/29 000) × 162 = 239.1869 kip-in; ϕMn 215.268, Mn/Ω 143.226 kip-in. Equations F11-1 and F11-3 checked against AISC 360-22"
inputs = { F_y = "36 ksi", E = "29000 ksi", d = "6 in", t = "0.75 in", L_b = "12 in", C_b = "1", M_u = "200 kip·in", M_a = "140 kip·in" }
expect = { M_p = "243 kip·in", "ψ" = "128", M_n = "239.1869 kip·in", "ϕM_n" = "215.2682 kip·in", "M_(n,ASD)" = "143.2257 kip·in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same bar braced at 5 in.: yielding"
source = "By hand: Lb d/t² = 53.3 ≤ 64.44, Mn = Mp = 243 kip-in (Lacewing Lab lateral_torsional_buckling_test: Z/S = 1.5 exactly for a rectangle, so FyZ = 1.5 Fy Sx)"
inputs = { F_y = "36 ksi", E = "29000 ksi", d = "6 in", t = "0.75 in", L_b = "5 in", C_b = "1", M_u = "200 kip·in", M_a = "140 kip·in" }
expect = { M_n = "243 kip·in" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "The same bar over 15 ft: elastic LTB, too weak; Cb 1.3 raises it"
source = "By hand: Lb d/t² = 180 × 6/0.5625 = 1920 > 1530.6; Fcr = 1.9 × 29 000 × Cb/1920; Cb 1: 28.698 ksi, Mn = 129.1406 kip-in, ϕMn 116.23 < 200"
inputs = { F_y = "36 ksi", E = "29000 ksi", d = "6 in", t = "0.75 in", L_b = "180 in", C_b = "1", M_u = "200 kip·in", M_a = "140 kip·in" }
expect = { M_n = "129.140625 kip·in" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "Over 15 ft with Cb 1.3"
source = "By hand: Fcr = 1.9 × 29 000 × 1.3/1920 = 37.307 ksi, Mn = 167.8828 kip-in ≤ Mp, ϕMn 151.09 ≥ 150"
inputs = { F_y = "36 ksi", E = "29000 ksi", d = "6 in", t = "0.75 in", L_b = "180 in", C_b = "1.3", M_u = "150 kip·in", M_a = "100 kip·in" }
expect = { M_n = "167.8828125 kip·in" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Lb 12 in. with 145 kip-in by ASD: the ASD check fails"
source = "Mn/Ω = 239.1869/1.67 = 143.226 kip-in < 145 kip-in; 200 kip-in by LRFD is below ϕMn 215.27 kip-in"
inputs = { F_y = "36 ksi", E = "29000 ksi", d = "6 in", t = "0.75 in", L_b = "12 in", C_b = "1", M_u = "200 kip·in", M_a = "145 kip·in" }
expect = { "M_(n,ASD)" = "143.2257 kip·in" }
tolerance = "1e-6"
checks = "fail"
+++

## Rectangular bar in major-axis bending (AISC 360 F11)

A solid rectangular bar (a flat bar or plate on edge) of depth d and thickness t, bent about its major axis, with the compression edge braced against lateral movement and twist at intervals L_b. Its strength depends on ψ = L_b d/t²:

- up to 0.08 E/F_y, the bar yields: M_n = M_p = F_y Z ≤ 1.5 F_y S_x (Eq. F11-1);
- up to 1.9 E/F_y, inelastic lateral-torsional buckling (Eq. F11-3);
- beyond that, elastic buckling at F_cr = 1.9 E C_b/ψ (Eqs. F11-4, F11-5).

Both the LRFD required moment M_u (ϕb = 0.90) and the ASD required moment M_a (Ωb = 1.67) are checked; use the one you design by.

Bending about the minor axis, and round bars, only yield (Eq. F11-1 and F11-2's 1.6 F_y S_x for rounds) and are not covered here. C_b is from Eq. F1-1 (see the C_b template).

### Bar

```calc {description="Specified minimum yield stress"}
F_y = 36 ksi_
```

```calc {description="Modulus of elasticity of steel"}
E = 29000 ksi_
```

```calc {description="Bar depth, in the plane of bending"}
d = 6 in_
```

```calc {description="Bar thickness"}
t = 0.75 in_
```

### Bracing and load

```calc {description="Unbraced length of the compression edge"}
L_b = 12 in_
```

```calc {description="Lateral-torsional buckling modification factor (1.0 is conservative)"}
C_b = 1_
```

```calc {description="Required moment, LRFD (factored)"}
M_u = 200 kip·in_
```

```calc {description="Required moment, ASD (service)"}
M_a = 140 kip·in_
```

### Strength

```calc
> Elastic and plastic section moduli
S_x = t d^2 / 6 = ?? in³
Z_x = t d^2 / 4 = ?? in³
> Yield and plastic moments, Eq. F11-1
M_y = F_y S_x = ?? kip·in
M_p = min(F_y Z_x, 1.5 F_y S_x) = ?? kip·in
> Bar slenderness for lateral-torsional buckling
ψ = L_b d / t^2 = ??
```

```calc {description="Nominal flexural strength, Eqs. F11-1, F11-3 to F11-5"}
M_n = { M_p if ψ ≤ 0.08 E / F_y; min(C_b (1.52 - 0.274 ψ F_y / E) M_y, M_p) if ψ ≤ 1.9 E / F_y; min(1.9 E C_b / ψ S_x, M_p) otherwise } = ?? kip·in
```

```calc {description="Design flexural strength (LRFD, ϕb = 0.90)"}
ϕM_n = 0.90 M_n = ?? kip·in
```

```calc {description="Allowable flexural strength (ASD, Ωb = 1.67)"}
M_(n,ASD) = M_n / 1.67 = ?? kip·in
```

### Check

```calc
@test M_u ≤ ϕM_n
```

```calc
@test M_a ≤ M_(n,ASD)
```
