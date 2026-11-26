+++
cruncha-template = 1
id = "structural.aluminum.flexureIBeam"
version = "1.0.0"
title = "Aluminum I-beam in bending and shear, ADM 2015 F.2, F.3.1, F.4 and G.2 (LRFD and ASD)"
summary = "Major-axis flexural strength of an unwelded aluminum I-beam (artificially aged temper): yielding, rupture, weighted-average local buckling of the flanges and web, and lateral-torsional buckling; and the shear strength of its web; checked against the required moment and shear by LRFD and by ASD."
category = ["Structural engineering", "Members"]
tags = ["Aluminum Design Manual", "ADM 2015", "aluminum", "aluminium", "flexure", "beam", "I-beam", "lateral-torsional buckling", "LTB", "local buckling", "web shear", "rye", "Cb", "6061-T6", "LRFD", "ASD"]
from = "lacewing:structural.aluminumMember"
license = "GPL-3.0-or-later"
inputs = ["F_ty", "F_tu", "k_t", "E", "Z_x", "S_x", "I_x", "d", "t_f", "t_w", "λ_f", "λ_w", "k_1", "k_2", "k_1b", "k_2b", "r_ye", "L_b", "C_b", "M_u", "M_a", "V_u", "V_a"]
outputs = ["M_np", "M_(n,rup)", "M_(n,lb)", "λ_b", "M_(n,mb)", "M_n", "ϕM_n", "M_(n,ASD)", "F_s", "V_n", "ϕV_n", "V_(n,ASD)"]

[[references]]
work = "Aluminum Design Manual 2015, Part I Specification for Aluminum Structures (Aluminum Association)"
edition = "2015"
locator = "Table A.3.1; Tables B.4.2 and B.4.3; Secs. B.5.4.1, B.5.5.1; F.1 (ϕb 0.90, Ωb 1.65; rupture 0.75, 1.95); F.2 (Eq. F.2-1); F.3.1 (Eq. F.3-1); F.4 (Eqs. F.4-1 to F.4-4; F.4.2.1); G.2 (Eqs. G.1-1, G.2-1 to G.2-3)"

[[references]]
work = "Aluminum Design Manual 2015, Part VII"
edition = "2015"
locator = "Example 15 (I 5 × 3.70 beam), pp. VII-28 to VII-29"

[[tests]]
name = "ADM Part VII Example 15: 6061-T6 I 5 × 3.70, Lb 4 ft"
source = "Aluminum Design Manual 2015 Part VII Example 15, pp. VII-28 to VII-29 (checked against the PDF): Mnp 215.3 in-k, rupture 119.8 in-k allowable (with Z 6.15), local buckling 133.3 in-k allowable, LTB 153.0 in-k (with Cc rounded to 66), Mn/Ω 92.8 in-k, web shear Fs/Ω 12.7 ksi, 12.1 k allowable. Section Zx 6.14, Sx 5.58 in³, Ix 13.9 in⁴, d 5, tf 0.32, tw 0.19 in., flange b/t 5.2, web 22.9, rye = 1.2 ry = 1.0236 in. (F.4.2.1)"
inputs = { F_ty = "35 ksi", F_tu = "38 ksi", k_t = "1", E = "10100 ksi", Z_x = "6.14 in^3", S_x = "5.58 in^3", I_x = "13.9 in^4", d = "5 in", t_f = "0.32 in", t_w = "0.19 in", "λ_f" = "5.2", "λ_w" = "22.9", k_1 = "0.35", k_2 = "2.27", k_1b = "0.50", k_2b = "2.04", r_ye = "1.0236 in", L_b = "48 in", C_b = "1", M_u = "100 kip·in", M_a = "90 kip·in", V_u = "10 kips", V_a = "12 kips" }
expect = { M_np = "215 kip·in", "M_(n,mb)" = "153 kip·in", "M_(n,ASD)" = "92.8 kip·in", "V_(n,ASD)" = "12.1 kips" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "Example 15, exactly"
source = "By hand (script): Mnp = min(6.14 × 35, 1.5 × 5.58 × 35) = 214.9 kip-in; flanges 5.2 < λ1 6.659, Fc 35 ksi; web 22.9 below B.5.5.1's λ1, Fb = 1.5 Fcy = 52.5 ksi; h = 4.36 in., Iw = 1.31229 in⁴, Mnlb = 35 × 12.5877/2.34 + 52.5 × 1.31229/2.18 = 219.881 kip-in; λ = 48/1.0236 = 46.893 < Cc 65.673, Mnmb = 214.9 (1 − 46.893/65.673) + π² × 10 100 × 46.893 × 5.58/65.673³ = 153.5405 kip-in; Mn/Ω = 93.0549 kip-in; rupture Z Ftu = 233.32 kip-in, over Ω 1.95 = 119.651 kip-in; shear: web 22.9 < λ1 35.29, Fs = 0.6 Fty = 21 ksi, Vn = 21 × 5 × 0.19 = 19.95 kips, Vn/Ω 12.0909 kips"
inputs = { F_ty = "35 ksi", F_tu = "38 ksi", k_t = "1", E = "10100 ksi", Z_x = "6.14 in^3", S_x = "5.58 in^3", I_x = "13.9 in^4", d = "5 in", t_f = "0.32 in", t_w = "0.19 in", "λ_f" = "5.2", "λ_w" = "22.9", k_1 = "0.35", k_2 = "2.27", k_1b = "0.50", k_2b = "2.04", r_ye = "1.0236 in", L_b = "48 in", C_b = "1", M_u = "100 kip·in", M_a = "90 kip·in", V_u = "10 kips", V_a = "12 kips" }
expect = { M_np = "214.9 kip·in", "M_(n,lb)" = "219.8811 kip·in", "M_(n,mb)" = "153.5405 kip·in", "M_(n,ASD)" = "93.05485 kip·in", "M_(n,rup)" = "233.32 kip·in", F_s = "21 ksi", V_n = "19.95 kips", "V_(n,ASD)" = "12.09091 kips" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "The same beam unbraced over 12 ft: elastic LTB, too weak"
source = "By hand: λ = 144/1.0236 = 140.68 > Cc, Mnmb = π² × 10 100 × 5.58/140.68² = 28.1055 kip-in; ϕMn = 25.29 < 100"
inputs = { F_ty = "35 ksi", F_tu = "38 ksi", k_t = "1", E = "10100 ksi", Z_x = "6.14 in^3", S_x = "5.58 in^3", I_x = "13.9 in^4", d = "5 in", t_f = "0.32 in", t_w = "0.19 in", "λ_f" = "5.2", "λ_w" = "22.9", k_1 = "0.35", k_2 = "2.27", k_1b = "0.50", k_2b = "2.04", r_ye = "1.0236 in", L_b = "144 in", C_b = "1", M_u = "100 kip·in", M_a = "20 kip·in", V_u = "10 kips", V_a = "12 kips" }
expect = { "M_(n,mb)" = "28.1055 kip·in" }
tolerance = "0.02%"
checks = "fail"

[[tests]]
name = "Example 15 with 95 kip-in by ASD: the ASD check fails"
source = "Mn/Ω = 153.5405/1.65 = 93.055 kip-in < 95 kip-in; 100 kip-in by LRFD is below ϕMn 138.19 kip-in"
inputs = { F_ty = "35 ksi", F_tu = "38 ksi", k_t = "1", E = "10100 ksi", Z_x = "6.14 in^3", S_x = "5.58 in^3", I_x = "13.9 in^4", d = "5 in", t_f = "0.32 in", t_w = "0.19 in", "λ_f" = "5.2", "λ_w" = "22.9", k_1 = "0.35", k_2 = "2.27", k_1b = "0.50", k_2b = "2.04", r_ye = "1.0236 in", L_b = "48 in", C_b = "1", M_u = "100 kip·in", M_a = "95 kip·in", V_u = "10 kips", V_a = "12 kips" }
expect = { "M_(n,ASD)" = "93.05485 kip·in" }
tolerance = "1e-5"
checks = "fail"
+++

## Aluminum I-beam in bending and shear (ADM 2015)

The major-axis flexural strength and web shear strength of an unwelded, doubly symmetric aluminum I-beam of an artificially aged temper (T5 to T9) in a building-type structure. Flexure takes the least of:

- **yielding** (F.2): the lesser of Z F_cy and 1.5 S F_ty; and **rupture** (Eq. F.2-1), Z F_tu/k_t, with its own resistance factor;
- **local buckling** (F.3.1): the flanges at their uniform-compression strength (B.5.4.1) and the web at its flexural strength (B.5.5.1, a symmetric web), each over its share of the second moment of area;
- **lateral-torsional buckling** (F.4): slenderness L_b/(r_ye √C_b) against the member buckling constants.

r_ye is the effective radius of gyration for lateral-torsional buckling: Eq. F.4-4 in general, or one of the simpler values F.4.2.1 allows for doubly symmetric I-shapes loaded through the shear centre (Example 15 uses 1.2 r_y). Web shear (G.2) uses the web's clear slenderness against the shear buckling constants, and shear rupture is not checked here.

F_cy is F_ty and F_sy = 0.6 F_ty for these tempers (Table A.3.1); k_t is 1.0 except for a few alloys (1.25 for 6005-T5). E is 10 100 ksi. The buckling constants are Table B.4.2's for tempers T5 to T9.

Both LRFD (M_u, V_u; ϕ 0.90, rupture 0.75) and ASD (M_a, V_a; Ω 1.65, rupture 1.95, building-type structures) are checked; use the one you design by.

Not covered: welded members, bridge-type safety factors, channels and other shapes, minor-axis bending, O, H and T1 to T4 tempers, and combined forces (H.1).

### Material

```calc {description="Tensile yield strength, Table A.3.3"}
F_ty = 35 ksi_
```

```calc {description="Tensile ultimate strength, Table A.3.3"}
F_tu = 38 ksi_
```

```calc {description="Tension coefficient k_t, Table A.3.3"}
k_t = 1_
```

```calc {description="Modulus of elasticity"}
E = 10100 ksi_
```

```calc {description="Unit stress of the buckling formulas"}
κ = 1 ksi
```

```calc
> Compressive and shear yield strengths, Table A.3.1 (T5 to T9)
F_cy = F_ty = ?? ksi
F_sy = 0.6 F_ty = ?? ksi
> Member buckling constants, Table B.4.2
B_c = F_cy (1 + (F_cy / (2250 κ))^(1/2)) = ?? ksi
D_c = B_c / 10 (B_c / E)^(1/2) = ?? ksi
C_c = 0.41 B_c / D_c = ??
> Flat elements in uniform compression
B_p = F_cy (1 + (F_cy / (1500 κ))^(1/3)) = ?? ksi
D_p = B_p / 10 (B_p / E)^(1/2) = ?? ksi
> Flat elements in flexure
B_br = 1.3 F_cy (1 + (F_cy / (340 κ))^(1/3)) = ?? ksi
D_br = B_br / 20 (6 B_br / E)^(1/2) = ?? ksi
> Flat elements in shear
B_s = F_sy (1 + (F_sy / (800 κ))^(1/3)) = ?? ksi
D_s = B_s / 10 (B_s / E)^(1/2) = ?? ksi
C_s = 0.41 B_s / D_s = ??
```

### Section

```calc {description="Plastic section modulus about x"}
Z_x = 6.14 in³_
```

```calc {description="Elastic section modulus about x"}
S_x = 5.58 in³_
```

```calc {description="Second moment of area about x"}
I_x = 13.9 in⁴_
```

```calc {description="Overall depth"}
d = 5 in_
```

```calc {description="Flange thickness"}
t_f = 0.32 in_
```

```calc {description="Web thickness"}
t_w = 0.19 in_
```

```calc {description="Flange outstand slenderness b/t, tip to fillet toe (B.5.1)"}
λ_f = 5.2_
```

```calc {description="Web slenderness b/t, clear of the fillets (B.5.1)"}
λ_w = 22.9_
```

```calc {description="Post-buckling constant k₁, Table B.4.3 (flat elements in compression, T5 to T9)"}
k_1 = 0.35_
```

```calc {description="Post-buckling constant k₂, Table B.4.3 (flat elements in compression, T5 to T9)"}
k_2 = 2.27_
```

```calc {description="Post-buckling constant k₁, Table B.4.3 (flat elements in flexure)"}
k_1b = 0.50_
```

```calc {description="Post-buckling constant k₂, Table B.4.3 (flat elements in flexure)"}
k_2b = 2.04_
```

```calc {description="Effective radius of gyration for LTB: Eq. F.4-4, or as F.4.2.1 allows"}
r_ye = 1.0236 in_
```

### Bracing and loads

```calc {description="Unbraced length of the compression flange"}
L_b = 48 in_
```

```calc {description="Moment gradient factor (1.0 is conservative)"}
C_b = 1_
```

```calc {description="Required moment, LRFD (factored)"}
M_u = 100 kip·in_
```

```calc {description="Required moment, ASD (service)"}
M_a = 90 kip·in_
```

```calc {description="Required shear, LRFD (factored)"}
V_u = 10 kips_
```

```calc {description="Required shear, ASD (service)"}
V_a = 12 kips_
```

### Yielding and rupture (F.2)

```calc
> Yielding
M_np = min(Z_x F_cy, 1.5 S_x F_ty) = ?? kip·in
> Rupture, Eq. F.2-1
M_(n,rup) = Z_x F_tu / k_t = ?? kip·in
```

### Local buckling (F.3.1)

```calc
> Flange outstands in uniform compression, B.5.4.1 (post-buckling)
F_cf = { F_cy if λ_f ≤ (B_p - F_cy) / (5.0 D_p); B_p - 5.0 D_p λ_f if λ_f < k_1 B_p / (5.0 D_p); k_2 sqrt(B_p E) / (5.0 λ_f) otherwise } = ?? ksi
> Web in flexure, B.5.5.1, symmetric web (m = 0.65)
F_bw = { 1.5 F_cy if λ_w ≤ (B_br - 1.5 F_cy) / (0.65 D_br); B_br - 0.65 D_br λ_w if λ_w < k_1b B_br / (0.65 D_br); k_2b sqrt(B_br E) / (0.65 λ_w) otherwise } = ?? ksi
> Web depth between flanges, and the shares of I_x
h = d - 2 t_f = ?? in
I_w = t_w h^3 / 12 = ?? in⁴
I_fl = I_x - I_w = ?? in⁴
```

```calc {description="Nominal strength for local buckling, Eq. F.3-1"}
M_(n,lb) = F_cf I_fl / ((d - t_f) / 2) + F_bw I_w / (h / 2) = ?? kip·in
```

### Lateral-torsional buckling (F.4)

```calc
> Slenderness for lateral-torsional buckling, Eq. F.4-3
λ_b = L_b / (r_ye sqrt(C_b)) = ??
> Nominal strength for member buckling, Eqs. F.4-1 and F.4-2
M_(n,mb) = min({ M_np (1 - λ_b / C_c) + π^2 E λ_b S_x / C_c^3 if λ_b < C_c; π^2 E S_x / λ_b^2 otherwise }, M_np) = ?? kip·in
```

### Flexural strength

```calc {description="Nominal flexural strength, the least of yielding, local and lateral-torsional buckling"}
M_n = min(M_np, M_(n,lb), M_(n,mb)) = ?? kip·in
```

```calc
> Design flexural strength (LRFD: ϕb 0.90; rupture ϕ 0.75)
ϕM_n = min(0.90 M_n, 0.75 M_(n,rup)) = ?? kip·in
> Allowable flexural strength (ASD: Ωb 1.65; rupture Ω 1.95)
M_(n,ASD) = min(M_n / 1.65, M_(n,rup) / 1.95) = ?? kip·in
```

### Web shear (G.2)

```calc
> Shear buckling stress of the web, G.2 (flat element supported on both edges)
F_s = { F_sy if λ_w ≤ (B_s - F_sy) / (1.25 D_s); B_s - 1.25 D_s λ_w if λ_w < C_s / 1.25; π^2 E / (1.25 λ_w)^2 otherwise } = ?? ksi
> Nominal shear strength, Eq. G.1-1, shear area d t_w
V_n = F_s d t_w = ?? kips
> Design and allowable shear strength (ϕv 0.90, Ωv 1.65)
ϕV_n = 0.90 V_n = ?? kips
V_(n,ASD) = V_n / 1.65 = ?? kips
```

### Checks

```calc
@test M_u ≤ ϕM_n
```

```calc
@test M_a ≤ M_(n,ASD)
```

```calc
@test V_u ≤ ϕV_n
```

```calc
@test V_a ≤ V_(n,ASD)
```
