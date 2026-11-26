+++
cruncha-template = 1
id = "structural.stainless.flexureF2"
version = "1.0.0"
title = "Stainless I-shape or channel flexure with lateral-torsional buckling, AISC 370 F2 and F3 (LRFD and ASD)"
summary = "Major-axis flexural strength of a stainless steel I-shape or channel with a compact web: yielding, lateral-torsional buckling with the stainless coefficients of Table F2.1 (limits Lp, Ly and Lr), and noncompact flange local buckling, checked against the required moment by LRFD and by ASD."
category = ["Structural engineering", "Members"]
tags = ["AISC 370", "stainless steel", "Design Guide 27", "flexure", "beam", "lateral-torsional buckling", "LTB", "channel", "I-shape", "Cb", "Lp", "Ly", "Lr", "austenitic", "duplex", "LRFD", "ASD"]
from = "lacewing:structural.stainlessMember"
license = "GPL-3.0-or-later"
inputs = ["F_y", "E", "β_LT", "β_pLT", "β_yLT", "Z_x", "S_x", "r_ts", "h_o", "J", "c", "λ_f", "λ_w", "L_b", "C_b", "M_u", "M_a"]
outputs = ["M_p", "M_y", "L_p", "L_y", "L_r", "M_(n,LTB)", "M_(n,FLB)", "M_n", "ϕM_n", "M_(n,ASD)"]

[[references]]
work = "ANSI/AISC 370-21 Specification for Structural Stainless Steel Buildings"
edition = "2021"
locator = "Sec. F2 (Eqs. F2-1 to F2-8; Table F2.1, coefficients by alloy family); Sec. F3.2 (Eq. F3-1); Table B4.1b; Sec. F1 (ϕb 0.90, Ωb 1.67)"

[[references]]
work = "AISC Design Guide 27, Structural Stainless Steel"
edition = "2nd"
locator = "Sec. 12.3, Design Example 3 (C12×30 beam, S30400), pp. 380-384"

[[tests]]
name = "DG27 Example 3: C12×30, S30400, Lb 15 ft, Cb 1.67"
source = "AISC Design Guide 27, 2nd ed., Design Example 3, pp. 380-383 (checked against the PDF): Lp 2.47 ft, Ly 4.53 ft, Lr 33.7 ft, αLT 0.456, Mn 62.6 kip-ft, ϕbMn 56.3 kip-ft, Mn/Ωb 37.5 kip-ft. Welded C12×30 (DG27 Table 4-3A): Zx 33.4, Sx 26.7 in³, rts 1.08 in., ho 11.5 in., J 0.714 in⁴, c = (ho/2)√(Iy/Cw) = 5.75 √(6.35/150) = 1.18307"
inputs = { F_y = "30 ksi", E = "28000 ksi", "β_LT" = "0.82", "β_pLT" = "0.90", "β_yLT" = "0.40", Z_x = "33.4 in^3", S_x = "26.7 in^3", r_ts = "1.08 in", h_o = "11.5 in", J = "0.714 in^4", c = "1.18307", "λ_f" = "6.34", "λ_w" = "22.0", L_b = "15 ft", C_b = "1.67", M_u = "50 kip·ft", M_a = "35 kip·ft" }
expect = { M_n = "62.6 kip·ft", "ϕM_n" = "56.3 kip·ft", "M_(n,ASD)" = "37.5 kip·ft", L_p = "2.47 ft", L_y = "4.53 ft", L_r = "33.7 ft" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "DG27 Example 3, exactly"
source = "By hand (script): Lp = 0.90 × 1.08 √(28 000/30) = 29.695 in.; jc = 0.714 × 1.18307/(26.7 × 11.5) = 0.0027510; Ly = 54.411 in.; Lr = 405.08 in.; α = 0.60 − 0.40 (180 − 54.41)/(405.08 − 54.41) = 0.45675; Mn = 1.67 [801 − 0.7 × 801 × ((180 − 54.41)/(405.08 − 54.41))^0.45675] = 751.854 kip-in = 62.6545 kip-ft"
inputs = { F_y = "30 ksi", E = "28000 ksi", "β_LT" = "0.82", "β_pLT" = "0.90", "β_yLT" = "0.40", Z_x = "33.4 in^3", S_x = "26.7 in^3", r_ts = "1.08 in", h_o = "11.5 in", J = "0.714 in^4", c = "1.18307", "λ_f" = "6.34", "λ_w" = "22.0", L_b = "15 ft", C_b = "1.67", M_u = "50 kip·ft", M_a = "35 kip·ft" }
expect = { L_p = "2.474591 ft", L_y = "4.534225 ft", L_r = "33.75705 ft", M_n = "62.65451 kip·ft" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "The same channel braced at 2 ft: yielding governs"
source = "By hand: Lb 2 ft < Lp 2.475 ft, no LTB; flange compact (6.34 < 0.33√(28 000/30) = 10.08); Mn = Mp = 30 × 33.4 = 1002 kip-in = 83.5 kip-ft"
inputs = { F_y = "30 ksi", E = "28000 ksi", "β_LT" = "0.82", "β_pLT" = "0.90", "β_yLT" = "0.40", Z_x = "33.4 in^3", S_x = "26.7 in^3", r_ts = "1.08 in", h_o = "11.5 in", J = "0.714 in^4", c = "1.18307", "λ_f" = "6.34", "λ_w" = "22.0", L_b = "2 ft", C_b = "1", M_u = "50 kip·ft", M_a = "35 kip·ft" }
expect = { M_n = "83.5 kip·ft", "ϕM_n" = "75.15 kip·ft" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "The same channel over 40 ft, uniform moment: elastic LTB, too weak for 50 kip-ft"
source = "By hand, Eqs. F2-4 and F2-5: Lb/rts = 480/1.08 = 444.44; Fcr = π² × 28 000/444.44² × √(1 + 0.078 × 0.0027510 × 444.44²) = 1.39901 × 6.58687 = 9.21512 ksi; Mn = 0.82 × 9.21512 × 26.7 = 201.756 kip-in = 16.8130 kip-ft"
inputs = { F_y = "30 ksi", E = "28000 ksi", "β_LT" = "0.82", "β_pLT" = "0.90", "β_yLT" = "0.40", Z_x = "33.4 in^3", S_x = "26.7 in^3", r_ts = "1.08 in", h_o = "11.5 in", J = "0.714 in^4", c = "1.18307", "λ_f" = "6.34", "λ_w" = "22.0", L_b = "40 ft", C_b = "1", M_u = "50 kip·ft", M_a = "10 kip·ft" }
expect = { M_n = "16.8130 kip·ft" }
tolerance = "0.05%"
checks = "fail"

[[tests]]
name = "Example 3 with 38 kip-ft by ASD: the ASD check fails"
source = "Mn/Ω = 62.6545/1.67 = 37.518 kip-ft < 38 kip-ft; 50 kip-ft by LRFD is below ϕMn 56.39 kip-ft"
inputs = { F_y = "30 ksi", E = "28000 ksi", "β_LT" = "0.82", "β_pLT" = "0.90", "β_yLT" = "0.40", Z_x = "33.4 in^3", S_x = "26.7 in^3", r_ts = "1.08 in", h_o = "11.5 in", J = "0.714 in^4", c = "1.18307", "λ_f" = "6.34", "λ_w" = "22.0", L_b = "15 ft", C_b = "1.67", M_u = "50 kip·ft", M_a = "38 kip·ft" }
expect = { "M_(n,ASD)" = "37.51768 kip·ft" }
tolerance = "1e-5"
checks = "fail"
+++

## Stainless I-shape or channel flexure (AISC 370 F2, F3)

The design flexural strength about the major axis of a stainless steel doubly symmetric I-shape or channel bent in the plane of its web, with the compression flange braced at intervals L_b. The web must be compact (h/t_w ≤ 2.54√(E/F_y)); the flanges may be compact or noncompact.

Lateral-torsional buckling follows AISC 360's form with coefficients for the alloy family (Table F2.1), and with a third limiting length L_y because stainless steel's strength falls off gradually: up to L_p the section reaches M_p; between L_p and L_y the strength falls linearly to M_y (Eq. F2-2); between L_y and L_r it follows a curve down to 0.30 M_y (Eq. F2-3); beyond L_r buckling is elastic (Eqs. F2-4, F2-5). C_b raises the buckling strength, never above M_p.

Table F2.1's coefficients: austenitic β_LT 0.82, β_p,LT 0.90, β_y,LT 0.40; duplex 0.86, 1.10, 0.50. E is 28 000 ksi for austenitic and 29 000 ksi for duplex grades. c is 1 for a doubly symmetric I-shape and (h_o/2)√(I_y/C_w) for a channel. Stainless rolled and welded shapes differ slightly from carbon steel shapes: take properties for the actual product.

Both the LRFD required moment M_u (ϕb = 0.90) and the ASD required moment M_a (Ωb = 1.67) are checked; use the one you design by.

Not covered: noncompact or slender webs (F4, F5), slender flanges (F3.2(b)), minor-axis bending, and single angles or tees.

### Material

```calc {description="Specified minimum yield stress (0.2 % proof stress)"}
F_y = 30 ksi_
```

```calc {description="Modulus of elasticity of the alloy family"}
E = 28000 ksi_
```

```calc {description="Lateral-torsional buckling coefficient β_LT, Table F2.1"}
β_LT = 0.82_
```

```calc {description="Coefficient β_p,LT for L_p, Table F2.1"}
β_pLT = 0.90_
```

```calc {description="Coefficient β_y,LT for L_y, Table F2.1"}
β_yLT = 0.40_
```

### Section

```calc {description="Plastic section modulus about x"}
Z_x = 33.4 in³_
```

```calc {description="Elastic section modulus about x"}
S_x = 26.7 in³_
```

```calc {description="Effective radius of gyration for LTB"}
r_ts = 1.08 in_
```

```calc {description="Distance between flange centroids"}
h_o = 11.5 in_
```

```calc {description="Torsional constant"}
J = 0.714 in⁴_
```

```calc {description="c: 1 for an I-shape, (h_o/2)√(I_y/C_w) for a channel"}
c = 1.18307_
```

```calc {description="Flange slenderness b/t_f (b_f/2t_f for an I-shape, b_f/t_f for a channel)"}
λ_f = 6.34_
```

```calc {description="Web slenderness h/t_w"}
λ_w = 22.0_
```

### Bracing and load

```calc {description="Unbraced length of the compression flange"}
L_b = 15 ft_
```

```calc {description="Lateral-torsional buckling modification factor (1.0 is conservative)"}
C_b = 1.67_
```

```calc {description="Required moment, LRFD (factored)"}
M_u = 50 kip·ft_
```

```calc {description="Required moment, ASD (service)"}
M_a = 35 kip·ft_
```

### Yielding

```calc
> Plastic moment, Eq. F2-1
M_p = F_y Z_x = ?? kip·ft
> Yield moment
M_y = F_y S_x = ?? kip·ft
```

### Lateral-torsional buckling

```calc
> J c / (S_x h_o)
j_c = J c / (S_x h_o) = ??
> Limiting unbraced length for yielding
L_p = β_pLT r_ts sqrt(E / F_y) = ?? ft
> Limiting unbraced length for the linear range
L_y = 1.95 β_yLT r_ts β_LT E / F_y sqrt(j_c + sqrt(j_c^2 + 6.76 (F_y / (β_LT E))^2)) = ?? ft
> Limiting unbraced length for inelastic LTB
L_r = 1.95 r_ts β_LT E / (0.30 F_y) sqrt(j_c + sqrt(j_c^2 + 6.76 (0.30 F_y / (β_LT E))^2)) = ?? ft
```

```calc
> Exponent of the inelastic curve, Eq. F2-6
α_LT = { 0.60 - 0.40 (L_b - L_y) / (L_r - L_y) if L_y < L_b ≤ L_r; 0.60 otherwise } = ??
> Critical stress for elastic LTB, Eq. F2-5 (beyond L_r only)
F_cr = { C_b π^2 E / (L_b / r_ts)^2 sqrt(1 + 0.078 j_c (L_b / r_ts)^2) if L_b > L_r; 0 'ksi' otherwise } = ?? ksi
> Nominal strength for LTB, Eqs. F2-2 to F2-4
M_(n,LTB) = { M_p if L_b ≤ L_p; min(C_b (M_p - (M_p - M_y) (L_b - L_p) / (L_y - L_p)), M_p) if L_b ≤ L_y; min(C_b (M_y - 0.70 M_y ((L_b - L_y) / (L_r - L_y))^α_LT), M_p) if L_b ≤ L_r; min(β_LT F_cr S_x, M_p) otherwise } = ?? kip·ft
```

### Flange local buckling

```calc
> Compact and noncompact limits of the flange, Table B4.1b
λ_pf = 0.33 sqrt(E / F_y) = ??
λ_rf = 0.41 sqrt(E / F_y) = ??
> Nominal strength for flange local buckling, Eq. F3-1
M_(n,FLB) = { M_p if λ_f ≤ λ_pf; M_p - (M_p - M_y) (λ_f - λ_pf) / (λ_rf - λ_pf) otherwise } = ?? kip·ft
```

### Strength

```calc {description="Nominal flexural strength"}
M_n = min(M_(n,LTB), M_(n,FLB)) = ?? kip·ft
```

```calc {description="Design flexural strength (LRFD, ϕb = 0.90)"}
ϕM_n = 0.90 M_n = ?? kip·ft
```

```calc {description="Allowable flexural strength (ASD, Ωb = 1.67)"}
M_(n,ASD) = M_n / 1.67 = ?? kip·ft
```

### Checks

```calc
@test λ_w ≤ 2.54 sqrt(E / F_y)
```

```calc
@test λ_f ≤ λ_rf
```

```calc
@test M_u ≤ ϕM_n
```

```calc
@test M_a ≤ M_(n,ASD)
```
