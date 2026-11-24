+++
cruncha-template = 1
id = "structural.steel.flexureF2"
version = "1.0.0"
title = "W-shape flexure with lateral-torsional buckling, AISC 360 F2 (LRFD)"
summary = "Design flexural strength of a compact doubly symmetric I-shape about its major axis: yielding, or lateral-torsional buckling by unbraced length, checked against the factored moment."
category = ["Structural engineering", "Steel members"]
tags = ["AISC 360", "F2", "flexure", "beam", "lateral-torsional buckling", "LTB", "Lp", "Lr", "Cb", "W-shape", "LRFD"]
from = "lacewing:structural.aisc360"
license = "GPL-3.0-or-later"
inputs = ["F_y", "E", "Z_x", "S_x", "r_y", "r_ts", "J", "h_o", "λ_f", "λ_w", "L_b", "C_b", "M_u"]
outputs = ["M_p", "L_p", "L_r", "M_n", "ϕM_n"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. F2 (Eqs. F2-1 to F2-6), F1 (ϕb = 0.90), Table B4.1b cases 10 and 15"

[[references]]
work = "AISC Steel Construction Manual"
edition = "16th"
locator = "Table 1-1 (W-shape properties); Table 3-2 (ϕbMpx, Lp, Lr, Fy = 50 ksi)"

[[tests]]
name = "W14×159, fully braced"
source = "Lacewing Lab aisc360_test, after AISC Manual Table 3-2: ϕbMpx 1080 kip·ft, Lp 14.1 ft, Lr 66.7 ft"
inputs = { F_y = "50 ksi", E = "29000 ksi", Z_x = "287 in^3", S_x = "254 in^3", r_y = "4.00 in", r_ts = "4.51 in", J = "19.7 in^4", h_o = "13.8 in", "λ_f" = "6.54", "λ_w" = "15.3", L_b = "0 ft", C_b = "1", M_u = "900 kip·ft" }
expect = { "ϕM_n" = "1080 kip·ft", L_p = "14.1 ft", L_r = "66.7 ft" }
tolerance = "0.6%"
checks = "pass"

[[tests]]
name = "W33×130, fully braced"
source = "Lacewing Lab aisc360_test, after AISC Manual Table 3-2: ϕbMpx 1750 kip·ft, Lp 8.44 ft, Lr 24.2 ft"
inputs = { F_y = "50 ksi", E = "29000 ksi", Z_x = "467 in^3", S_x = "406 in^3", r_y = "2.39 in", r_ts = "2.94 in", J = "7.37 in^4", h_o = "32.2 in", "λ_f" = "6.73", "λ_w" = "51.7", L_b = "0 ft", C_b = "1", M_u = "1500 kip·ft" }
expect = { "ϕM_n" = "1750 kip·ft", L_p = "8.44 ft", L_r = "24.2 ft" }
tolerance = "0.6%"
checks = "pass"

[[tests]]
name = "W18×50 braced at Lr: Mn = 0.7 Fy Sx"
source = "Lacewing Lab aisc360_test: F2-2 and F2-4 meet at Lr; Lr = 16.9 ft from AISC Manual Table 3-2; 0.7 × 50 × 88.9 = 3112 kip·in"
inputs = { F_y = "50 ksi", E = "29000 ksi", Z_x = "101 in^3", S_x = "88.9 in^3", r_y = "1.65 in", r_ts = "1.98 in", J = "1.24 in^4", h_o = "17.4 in", "λ_f" = "6.57", "λ_w" = "45.2", L_b = "16.95 ft", C_b = "1", M_u = "200 kip·ft" }
expect = { L_r = "16.9 ft", M_n = "259.3 kip·ft" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "W18×50 over 25 ft: elastic LTB governs, and 250 kip·ft is too much"
source = "By hand, Eq. F2-4: Lb/rts = 300/1.98 = 151.5; Fcr = π² × 29000/151.5² × √(1 + 0.078 × 8.016e-4 × 151.5²) = 12.47 × 1.561 = 19.46 ksi; Mn = 19.46 × 88.9 = 1730 kip·in = 144.1 kip·ft"
inputs = { F_y = "50 ksi", E = "29000 ksi", Z_x = "101 in^3", S_x = "88.9 in^3", r_y = "1.65 in", r_ts = "1.98 in", J = "1.24 in^4", h_o = "17.4 in", "λ_f" = "6.57", "λ_w" = "45.2", L_b = "25 ft", C_b = "1", M_u = "250 kip·ft" }
expect = { M_n = "144.1 kip·ft" }
tolerance = "0.2%"
checks = "fail"
+++

## W-shape flexure, AISC 360 F2 (LRFD)

Design flexural strength about the major axis of a doubly symmetric I-shape with **compact** web and flanges (AISC 360 F2), bent in its plane with the compression flange braced against lateral movement and twist at intervals L_b. The limit states are yielding (F2.1) and lateral-torsional buckling (F2.2).

Most rolled W-shapes in Fy = 50 ksi steel are compact; the two checks below say when one isn't, and then F3 applies instead. Shear, web local yielding at loads and deflection are separate checks.

### Material and section

Section properties are the AISC Manual's Table 1-1 values for the shape, or the I-section template's for a built-up one.

```calc {description="Specified minimum yield stress"}
F_y = 50 ksi_
```

```calc {description="Modulus of elasticity of steel"}
E = 29000 ksi_
```

```calc {description="Plastic section modulus about x"}
Z_x = 287 in³_
```

```calc {description="Elastic section modulus about x"}
S_x = 254 in³_
```

```calc {description="Radius of gyration about y"}
r_y = 4.00 in_
```

```calc {description="Effective radius of gyration for LTB, Table 1-1"}
r_ts = 4.51 in_
```

```calc {description="Torsional constant"}
J = 19.7 in⁴_
```

```calc {description="Distance between flange centroids"}
h_o = 13.8 in_
```

```calc {description="Flange slenderness b_f / 2t_f"}
λ_f = 6.54_
```

```calc {description="Web slenderness h / t_w"}
λ_w = 15.3_
```

### Compactness (Table B4.1b)

```calc
@test λ_f ≤ 0.38 sqrt(E / F_y)
```

```calc
@test λ_w ≤ 3.76 sqrt(E / F_y)
```

### Bracing and load

```calc {description="Unbraced length of the compression flange"}
L_b = 0 ft_
```

```calc {description="Lateral-torsional buckling modification factor (1.0 is conservative)"}
C_b = 1_
```

```calc {description="Factored moment"}
M_u = 900 kip·ft_
```

### Strength

```calc {description="Plastic moment, Eq. F2-1"}
M_p = F_y Z_x = ?? kip·ft
```

```calc
> Limiting unbraced length for yielding, Eq. F2-5
L_p = 1.76 r_y sqrt(E / F_y) = ?? ft
> J c / (S_x h_o), with c = 1 for a doubly symmetric I
j_c = J / (S_x h_o) = ?? 
> Limiting unbraced length for inelastic LTB, Eq. F2-6
L_r = 1.95 r_ts E / (0.7 F_y) sqrt(j_c + sqrt(j_c^2 + 6.76 (0.7 F_y / E)^2)) = ?? ft
```

Below L_p the beam yields; between L_p and L_r the strength falls linearly to 0.7 F_y S_x (F2-2); beyond L_r it buckles elastically at F_cr (F2-3, F2-4). C_b raises the buckling strength, never above M_p.

```calc
> Critical stress for elastic LTB, Eq. F2-4 (beyond L_r only)
F_cr = { C_b π^2 E / (L_b / r_ts)^2 sqrt(1 + 0.078 j_c (L_b / r_ts)^2) if L_b > L_r; 0 'ksi' otherwise } = ?? ksi
> Nominal flexural strength, F2.1 and F2.2
M_n = { M_p if L_b ≤ L_p; min(C_b (M_p - (M_p - 0.7 F_y S_x) (L_b - L_p) / (L_r - L_p)), M_p) if L_b ≤ L_r; min(F_cr S_x, M_p) otherwise } = ?? kip·ft
```

```calc {description="Resistance factor for flexure (F1)"}
ϕ_b = 0.90
```

```calc {description="Design flexural strength"}
ϕM_n = ϕ_b M_n = ?? kip·ft
```

### Check

```calc
@test M_u ≤ ϕM_n
```
