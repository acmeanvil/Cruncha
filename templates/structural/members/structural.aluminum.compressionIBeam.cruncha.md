+++
cruncha-template = 1
id = "structural.aluminum.compressionIBeam"
version = "1.0.0"
title = "Aluminum I-beam column, ADM 2015 E.2, E.3.1 and E.4 (LRFD and ASD)"
summary = "Compressive strength of an unwelded doubly symmetric aluminum I-beam (artificially aged temper): flexural and torsional member buckling, weighted-average local buckling of the flanges and web, and their interaction, checked against the required axial load by LRFD and by ASD."
category = ["Structural engineering", "Members"]
tags = ["Aluminum Design Manual", "ADM 2015", "aluminum", "aluminium", "compression", "column", "I-beam", "torsional buckling", "member buckling", "local buckling", "weighted average", "buckling constants", "6061-T6", "LRFD", "ASD"]
from = "lacewing:structural.aluminumMember"
license = "GPL-3.0-or-later"
inputs = ["F_cy", "E", "G", "A_g", "I_x", "I_y", "J", "C_w", "r_x", "r_y", "t_f", "t_w", "λ_f", "λ_w", "k_1", "k_2", "L_cx", "L_cy", "L_cz", "P_u", "P_a"]
outputs = ["λ_t", "λ", "F_c", "F_cf", "F_cw", "P_(n,mb)", "P_(n,lb)", "P_n", "ϕP_n", "P_(n,ASD)"]

[[references]]
work = "Aluminum Design Manual 2015, Part I Specification for Aluminum Structures (Aluminum Association)"
edition = "2015"
locator = "Table B.4.2 (buckling constants, T5 to T9), Table B.4.3 (k1 0.35, k2 2.27); Secs. B.5.1, B.5.4.1, B.5.4.2; Sec. E.2 (Eqs. E.2-1, E.2-3, E.2-4); E.3.1 (Eq. E.3-1); E.4 (Eq. E.4-1); Table A.3.1"

[[references]]
work = "Aluminum Design Manual 2015, Part VII"
edition = "2015"
locator = "Example 9 (I 8 × 6.18 column, torsional buckling), pp. VII-18 to VII-19"

[[tests]]
name = "ADM Part VII Example 9: 6061-T6 I 8 × 6.18, 8 ft, weak axis braced, twist braced at mid-height"
source = "Aluminum Design Manual 2015 Part VII Example 9, pp. VII-18 and VII-19 (checked against the PDF): flexural kL/r 28.5; torsional Fe 79.8 ksi (kz L = 0.5 × 96 in.), λ 35.3, Fc 29.0 ksi, allowable member buckling 92.6 kips; the web's elastic Fe 38.7 ksi > Fc, so no E.4 interaction. A 5.26 in², Ix 59.7, Iy 7.3 in⁴, J 0.188 in⁴, Cw 107 in⁶, rx 3.37 in., G 3800 ksi, E 10 100 ksi"
inputs = { F_cy = "35 ksi", E = "10100 ksi", G = "3800 ksi", A_g = "5.26 in^2", I_x = "59.7 in^4", I_y = "7.3 in^4", J = "0.188 in^4", C_w = "107 in^6", r_x = "3.37 in", r_y = "1.178 in", t_f = "0.35 in", t_w = "0.23 in", "λ_f" = "6.8", "λ_w" = "31.7", k_1 = "0.35", k_2 = "2.27", L_cx = "96 in", L_cy = "0 in", L_cz = "48 in", P_u = "120 kips", P_a = "90 kips" }
expect = { "λ_t" = "35.3", F_c = "29.0 ksi", "P_(n,ASD)" = "92.6 kips" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Example 9, exactly, with the local buckling weighted average"
source = "By hand (script): Fe = (π² × 10 100 × 107/48² + 3800 × 0.188)/(59.7 + 7.3) = 79.758 ksi; λt = π √(10 100/79.758) = 35.353; Fc = 28.9875 ksi; Pn 152.4745 kips. Local: flanges b/t 6.8 (B.5.4.1, λ1 6.659 < 6.8 < λ2 10.487) Fc = 45.0014 − 5 × 0.300385 × 6.8 = 34.788 ksi; web 31.7 (B.5.4.2, λ1 20.81 < 31.7 < λ2 32.77) Fc = 45.0014 − 1.6 × 0.300385 × 31.7 = 29.766 ksi; Pn = 34.788 × 3.332 + 29.766 × 1.67693 + 35 × 0.25107 = 174.617 kips (the book counts only the flange-web cores, 2 × 0.23 × 0.35 in², at Fcy, and gets 104.3 kips allowable, 172.1 kips nominal; this template puts all of Ag − Af − Aw at Fcy)"
inputs = { F_cy = "35 ksi", E = "10100 ksi", G = "3800 ksi", A_g = "5.26 in^2", I_x = "59.7 in^4", I_y = "7.3 in^4", J = "0.188 in^4", C_w = "107 in^6", r_x = "3.37 in", r_y = "1.178 in", t_f = "0.35 in", t_w = "0.23 in", "λ_f" = "6.8", "λ_w" = "31.7", k_1 = "0.35", k_2 = "2.27", L_cx = "96 in", L_cy = "0 in", L_cz = "48 in", P_u = "120 kips", P_a = "90 kips" }
expect = { "λ_t" = "35.35280", F_c = "28.98755 ksi", F_cf = "34.78830 ksi", F_cw = "29.76586 ksi", "P_(n,lb)" = "174.6173 kips", P_n = "152.4745 kips" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "The same column unbraced over 16 ft about y: flexural buckling, overloaded"
source = "By hand: L/ry = 192/1.178 = 162.99 > Cc 65.67, Fc = 0.85π² × 10 100/162.99² = 3.18954 ksi; the web's Fe = π² × 10 100/(1.6 × 31.7)² = 38.75 ksi > Fc, no interaction; Pn = 3.18954 × 5.26 = 16.7770 kips, ϕPn 15.10 < 20"
inputs = { F_cy = "35 ksi", E = "10100 ksi", G = "3800 ksi", A_g = "5.26 in^2", I_x = "59.7 in^4", I_y = "7.3 in^4", J = "0.188 in^4", C_w = "107 in^6", r_x = "3.37 in", r_y = "1.178 in", t_f = "0.35 in", t_w = "0.23 in", "λ_f" = "6.8", "λ_w" = "31.7", k_1 = "0.35", k_2 = "2.27", L_cx = "192 in", L_cy = "192 in", L_cz = "192 in", P_u = "20 kips", P_a = "10 kips" }
expect = { "λ" = "162.988", F_c = "3.18954 ksi", P_n = "16.7770 kips" }
tolerance = "1e-5"
checks = "fail"

[[tests]]
name = "Example 9 with 93 kips by ASD: the ASD check fails"
source = "Pn/Ω = 152.4745/1.65 = 92.409 kips < 93 kips; 120 kips by LRFD is below ϕPn 137.23 kips"
inputs = { F_cy = "35 ksi", E = "10100 ksi", G = "3800 ksi", A_g = "5.26 in^2", I_x = "59.7 in^4", I_y = "7.3 in^4", J = "0.188 in^4", C_w = "107 in^6", r_x = "3.37 in", r_y = "1.178 in", t_f = "0.35 in", t_w = "0.23 in", "λ_f" = "6.8", "λ_w" = "31.7", k_1 = "0.35", k_2 = "2.27", L_cx = "96 in", L_cy = "0 in", L_cz = "48 in", P_u = "120 kips", P_a = "93 kips" }
expect = { "P_(n,ASD)" = "92.40879 kips" }
tolerance = "1e-5"
checks = "fail"
+++

## Aluminum I-beam column (ADM 2015)

The compressive strength of an unwelded, doubly symmetric aluminum I-beam of an artificially aged temper (T5 to T9), in a building-type structure. The limit states:

- **member buckling** (E.2): the larger of the flexural slenderness about each axis and the equivalent slenderness for torsional buckling (Eqs. E.2-3, E.2-4, over the length L_cz between points where twist is prevented);
- **local buckling** (E.3.1): the weighted average of the elements' strengths, four flange outstands supported on one edge (B.5.4.1) and the web supported on both (B.5.4.2), the rest of the area (the fillets and the flange-web cores) at F_cy;
- their **interaction** (E.4), when an element's elastic local buckling stress is below the member buckling stress.

λ_f and λ_w are the element slendernesses as B.5.1 measures them: a flange outstand from the tip to the toe of the fillet, the web between the toes of the fillets (Example 9 uses (b_f − t_w)/2t_f and (d − 2t_f)/t_w). The buckling constants are Table B.4.2's (tempers T5 to T9) with Table B.4.3's post-buckling constants k₁ and k₂ (0.35 and 2.27 for flat elements in compression, tempers T5 to T9); F_cy is F_ty for these tempers. E is 10 100 ksi and G 3800 ksi (Table A.3.1).

Both the LRFD required strength P_u (ϕc = 0.90) and the ASD required strength P_a (Ωc = 1.65, building-type structures) are checked; use the one you design by.

Not covered: welded members, bridge-type safety factors, channels (flexural-torsional buckling), O, H and T1 to T4 tempers, and second-order effects.

### Material

```calc {description="Compressive yield strength (F_ty for T5 to T9 tempers)"}
F_cy = 35 ksi_
```

```calc {description="Modulus of elasticity"}
E = 10100 ksi_
```

```calc {description="Shear modulus"}
G = 3800 ksi_
```

```calc {description="Unit stress of the buckling formulas"}
κ = 1 ksi
```

```calc
> Member buckling constants, Table B.4.2
B_c = F_cy (1 + (F_cy / (2250 κ))^(1/2)) = ?? ksi
D_c = B_c / 10 (B_c / E)^(1/2) = ?? ksi
C_c = 0.41 B_c / D_c = ??
> Flat elements in uniform compression, Table B.4.2
B_p = F_cy (1 + (F_cy / (1500 κ))^(1/3)) = ?? ksi
D_p = B_p / 10 (B_p / E)^(1/2) = ?? ksi
```

### Section

```calc {description="Gross area"}
A_g = 5.26 in²_
```

```calc {description="Second moment of area about x"}
I_x = 59.7 in⁴_
```

```calc {description="Second moment of area about y"}
I_y = 7.3 in⁴_
```

```calc {description="Torsional constant"}
J = 0.188 in⁴_
```

```calc {description="Warping constant"}
C_w = 107 in⁶_
```

```calc {description="Radius of gyration about x"}
r_x = 3.37 in_
```

```calc {description="Radius of gyration about y"}
r_y = 1.178 in_
```

```calc {description="Flange thickness"}
t_f = 0.35 in_
```

```calc {description="Web thickness"}
t_w = 0.23 in_
```

```calc {description="Flange outstand slenderness b/t, tip to fillet toe (B.5.1)"}
λ_f = 6.8_
```

```calc {description="Web slenderness b/t, clear of the fillets (B.5.1)"}
λ_w = 31.7_
```

```calc {description="Post-buckling constant k₁, Table B.4.3 (flat elements in compression)"}
k_1 = 0.35_
```

```calc {description="Post-buckling constant k₂, Table B.4.3 (flat elements in compression)"}
k_2 = 2.27_
```

### Lengths and load

```calc {description="Effective length for buckling about x"}
L_cx = 96 in_
```

```calc {description="Effective length for buckling about y"}
L_cy = 0 in_
```

```calc {description="Effective length for torsional buckling (between twist restraints)"}
L_cz = 48 in_
```

```calc {description="Required axial compression, LRFD (factored)"}
P_u = 120 kips_
```

```calc {description="Required axial compression, ASD (service)"}
P_a = 90 kips_
```

### Member buckling (E.2)

```calc
> Elastic torsional buckling stress, Eq. E.2-4
F_et = (π^2 E C_w / L_cz^2 + G J) / (I_x + I_y) = ?? ksi
> Equivalent slenderness for torsional buckling, Eq. E.2-3
λ_t = π sqrt(E / F_et) = ??
> Governing slenderness
λ = max(L_cx / r_x, L_cy / r_y, λ_t) = ??
> Slenderness below which the member yields
λ_1 = (B_c - F_cy) / D_c = ??
> Member buckling stress, Eq. E.2-1
F_c = { F_cy if λ ≤ λ_1; (B_c - D_c λ) (0.85 + 0.15 (C_c - λ) / (C_c - λ_1)) if λ < C_c; 0.85 π^2 E / λ^2 otherwise } = ?? ksi
```

```calc {description="Nominal strength for member buckling"}
P_(n,mb) = F_c A_g = ?? kips
```

### Local buckling (B.5.4, E.3.1)

```calc
> Flange outstands: one edge supported, B.5.4.1 (post-buckling)
F_cf = { F_cy if λ_f ≤ (B_p - F_cy) / (5.0 D_p); B_p - 5.0 D_p λ_f if λ_f < k_1 B_p / (5.0 D_p); k_2 sqrt(B_p E) / (5.0 λ_f) otherwise } = ?? ksi
> Web: both edges supported, B.5.4.2
F_cw = { F_cy if λ_w ≤ (B_p - F_cy) / (1.6 D_p); B_p - 1.6 D_p λ_w if λ_w < k_1 B_p / (1.6 D_p); k_2 sqrt(B_p E) / (1.6 λ_w) otherwise } = ?? ksi
> Areas of the four flange outstands and the web
A_f = 4 λ_f t_f^2 = ?? in²
A_w = λ_w t_w^2 = ?? in²
```

```calc {description="Nominal strength for local buckling, weighted average, Eq. E.3-1"}
P_(n,lb) = F_cf A_f + F_cw A_w + F_cy (A_g - A_f - A_w) = ?? kips
```

### Interaction (E.4)

```calc
> Elastic local buckling stresses of the elements, Table B.5.1
F_ef = π^2 E / (5.0 λ_f)^2 = ?? ksi
F_ew = π^2 E / (1.6 λ_w)^2 = ?? ksi
F_el = min(F_ef, F_ew) = ?? ksi
> Interaction of member and local buckling, Eq. E.4-1 (only when F_el < F_c)
P_(n,i) = { (0.85 π^2 E / λ^2)^(1/3) F_el^(2/3) A_g if F_el < F_c; F_cy A_g otherwise } = ?? kips
```

### Strength

```calc {description="Nominal compressive strength, the least limit state"}
P_n = min(P_(n,mb), P_(n,lb), P_(n,i)) = ?? kips
```

```calc {description="Design compressive strength (LRFD, ϕc = 0.90)"}
ϕP_n = 0.90 P_n = ?? kips
```

```calc {description="Allowable compressive strength (ASD, building-type, Ωc = 1.65)"}
P_(n,ASD) = P_n / 1.65 = ?? kips
```

### Check

```calc
@test P_u ≤ ϕP_n
```

```calc
@test P_a ≤ P_(n,ASD)
```
