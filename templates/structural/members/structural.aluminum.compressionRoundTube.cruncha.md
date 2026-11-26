+++
cruncha-template = 1
id = "structural.aluminum.compressionRoundTube"
version = "1.0.0"
title = "Aluminum round tube column, ADM 2015 E.2, E.3 and E.4 (LRFD and ASD)"
summary = "Compressive strength of an unwelded round aluminum tube or pipe (artificially aged temper): member buckling, local buckling of the tube wall and their interaction, with the buckling constants of Table B.4.2, checked against the required axial load by LRFD and by ASD."
category = ["Structural engineering", "Members"]
tags = ["Aluminum Design Manual", "ADM 2015", "aluminum", "aluminium", "compression", "column", "round tube", "pipe", "member buckling", "local buckling", "buckling constants", "6061-T6", "LRFD", "ASD"]
from = "lacewing:structural.aluminumMember"
license = "GPL-3.0-or-later"
inputs = ["F_ty", "F_cy", "E", "A_g", "R_bt", "L_cx", "r_x", "L_cy", "r_y", "P_u", "P_a"]
outputs = ["B_c", "D_c", "C_c", "B_t", "D_t", "C_t", "λ", "F_c", "F_cl", "P_n", "ϕP_n", "P_(n,ASD)"]

[[references]]
work = "Aluminum Design Manual 2015, Part I Specification for Aluminum Structures (Aluminum Association)"
edition = "2015"
locator = "Table B.4.2 (buckling constants, tempers T5 to T9); Sec. B.5.4.5 (round elements in uniform compression); Sec. E.2 (Eq. E.2-1), E.3.1, E.4 (Eq. E.4-1); Sec. E.1 (ϕc 0.90, Ωc 1.65)"

[[references]]
work = "Aluminum Design Manual 2015, Parts VI and VII"
edition = "2015"
locator = "Part VI Table 1-1 (buckling constants); Part VII Example 14 (round tube column), p. VII-27"

[[tests]]
name = "ADM Part VII Example 14: 6061-T6 tube 6 OD × 0.188, 18 in."
source = "Aluminum Design Manual 2015 Part VII Example 14, p. VII-27 (checked against the PDF): A 3.43 in², r 2.06 in., kL/r 8.7 < λ1 17.8, Fc/Ω 21.2 ksi, Rb/t 15.5 < λ1 27.6, allowable 72.7 kips ≥ 40 kips. E 10 100 ksi. By hand: kL/r 8.74 < λ1, Fc = Fcy = 35 ksi; Rb/t 15.5 < λ1 = ((Bt − Fcy)/Dt)² = 27.62, Fc = 35 ksi; Pn = 120.05 kips, Pn/Ω = 72.76 kips"
inputs = { F_ty = "35 ksi", F_cy = "35 ksi", E = "10100 ksi", A_g = "3.43 in^2", R_bt = "15.5", L_cx = "18 in", r_x = "2.06 in", L_cy = "18 in", r_y = "2.06 in", P_u = "100 kips", P_a = "40 kips" }
expect = { "P_(n,ASD)" = "72.7 kips", P_n = "120.05 kips" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "6061-T6 buckling constants, ADM Part VI Table 1-1"
source = "ADM 2015 Part VI Table 1-1, 6061-T6 row (Fcy 35 ksi; checked against the PDF): Bc 39.4 ksi, Dc 0.246 ksi, Cc 65.7, Bt 43.2 ksi, Dt 1.558 ksi, Ct 141"
inputs = { F_ty = "35 ksi", F_cy = "35 ksi", E = "10100 ksi", P_a = "40 kips" }
expect = { B_c = "39.4 ksi", D_c = "0.246 ksi", C_c = "65.7", B_t = "43.2 ksi", D_t = "1.558 ksi", C_t = "141" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "A 6063-T6 tube, 12 ft: inelastic member buckling governs; overloaded"
source = "By hand (script, the same equations): Fcy 25 ksi gives Bc 27.635 ksi, Dc 0.144555 ksi, Cc 78.381, λ1 18.230; kL/r = 144/2 = 72 < Cc, Fc = (27.635 − 0.144555 × 72)(0.85 + 0.15 × 6.381/60.151) = 14.9173 ksi. Wall: Bt 30.4668 ksi, Dt 0.97825 ksi, λ1 31.23 < Rb/t 60 < Ct 188.68, Fcl = 30.4668 − 0.97825 √60 = 22.8893 ksi; wall Fe 69.61 ksi > Fc, no interaction; Pn = 1.5 × 14.9173 = 22.376 kips, ϕPn 20.14 < 25"
inputs = { F_ty = "25 ksi", F_cy = "25 ksi", E = "10100 ksi", A_g = "1.5 in^2", R_bt = "60", L_cx = "12 ft", r_x = "2.0 in", L_cy = "12 ft", r_y = "2.0 in", P_u = "25 kips", P_a = "10 kips" }
expect = { "λ" = "72", F_c = "14.91731 ksi", F_cl = "22.88932 ksi", C_t = "188.684", P_n = "22.37596 kips" }
tolerance = "1e-5"
checks = "fail"

[[tests]]
name = "Example 14 with 73 kips by ASD: the ASD check fails"
source = "Pn/Ω = 120.05/1.65 = 72.758 kips < 73 kips; 100 kips by LRFD is below ϕPn 108.05 kips"
inputs = { F_ty = "35 ksi", F_cy = "35 ksi", E = "10100 ksi", A_g = "3.43 in^2", R_bt = "15.5", L_cx = "18 in", r_x = "2.06 in", L_cy = "18 in", r_y = "2.06 in", P_u = "100 kips", P_a = "73 kips" }
expect = { "P_(n,ASD)" = "72.75758 kips" }
tolerance = "1e-6"
checks = "fail"
+++

## Aluminum round tube column (ADM 2015)

The compressive strength of an unwelded round tube or pipe of an artificially aged temper (T5 to T9: 6061-T6, 6063-T6, 6005-T5…), in a building-type structure. Three limit states:

- **member buckling** (E.2): the column's slenderness kL/r against the buckling constants B_c, D_c, C_c, with an inelastic range below C_c and 0.85 of Euler's stress above it;
- **local buckling** of the tube wall (B.5.4.5, by R_b/t, the mean radius over the thickness); a tube is a single element, so the weighted average of E.3.1 is the wall's own strength;
- their **interaction** (E.4), when the wall's elastic buckling stress is below the member buckling stress.

The buckling constants are those of Table B.4.2, for tempers T5 to T9; for O, H and T1 to T4 tempers (Table B.4.1, e.g. 5083-H116) the formulas differ and this template does not apply. F_cy is F_ty for these tempers (Table A.3.1). E is 10 100 ksi (70 000 MPa).

Both the LRFD required strength P_u (ϕc = 0.90) and the ASD required strength P_a (Ωc = 1.65, building-type structures) are checked; use the one you design by.

Welded members (weld-affected zones), bridge-type safety factors and second-order effects (they belong in the required strengths) are not covered.

### Material

```calc {description="Tensile yield strength, Table A.3.3"}
F_ty = 35 ksi_
```

```calc {description="Compressive yield strength (F_ty for T5 to T9 tempers)"}
F_cy = 35 ksi_
```

```calc {description="Modulus of elasticity"}
E = 10100 ksi_
```

### Buckling constants (Table B.4.2)

The tables define the constants with stresses in ksi, so the formulas carry κ = 1 ksi.

```calc {description="Unit stress of the buckling formulas"}
κ = 1 ksi
```

```calc
> Member buckling
B_c = F_cy (1 + (F_cy / (2250 κ))^(1/2)) = ?? ksi
D_c = B_c / 10 (B_c / E)^(1/2) = ?? ksi
C_c = 0.41 B_c / D_c = ??
> Round elements in uniform compression
B_t = F_cy (1 + (F_cy / (50000 κ))^(1/5)) = ?? ksi
D_t = B_t / 4.5 (B_t / E)^(1/3) = ?? ksi
```

C_t is where the inelastic line B_t − D_t √(R_b/t) first meets the elastic curve of B.5.4.5 above the yield limit (Table B.4.2, note 2); it is found here by a scan in steps of 1 and then bisection.

```calc
function tubeCt(B_t, D_t, E, F_cy)
    lo = ((B_t - F_cy) / D_t)^2
    hi = lo
    while B_t - D_t sqrt(hi) - π^2 E / (16 hi (1 + sqrt(hi) / 35)^2) < 0 F_cy and hi < 5000
        lo = hi
        hi = hi + 1
    end
    for i in 1:50
        m = (lo + hi) / 2
        if B_t - D_t sqrt(m) - π^2 E / (16 m (1 + sqrt(m) / 35)^2) < 0 F_cy
            lo = m
        else
            hi = m
        end
    end
    return (lo + hi) / 2
end
```

```calc {description="Slenderness limit C_t of round elements"}
C_t = tubeCt(B_t, D_t, E, F_cy) = ??
```

### Section and load

```calc {description="Gross area"}
A_g = 3.43 in²_
```

```calc {description="Wall slenderness R_b/t (mean radius over thickness)"}
R_bt = 15.5_
```

```calc {description="Effective length for buckling about x"}
L_cx = 18 in_
```

```calc {description="Radius of gyration about x"}
r_x = 2.06 in_
```

```calc {description="Effective length for buckling about y"}
L_cy = 18 in_
```

```calc {description="Radius of gyration about y"}
r_y = 2.06 in_
```

```calc {description="Required axial compression, LRFD (factored)"}
P_u = 100 kips_
```

```calc {description="Required axial compression, ASD (service)"}
P_a = 40 kips_
```

### Member buckling (E.2)

```calc
> Slenderness
λ = max(L_cx / r_x, L_cy / r_y) = ??
> Slenderness below which the member yields
λ_1 = (B_c - F_cy) / D_c = ??
> Member buckling stress, Eq. E.2-1
F_c = { F_cy if λ ≤ λ_1; (B_c - D_c λ) (0.85 + 0.15 (C_c - λ) / (C_c - λ_1)) if λ < C_c; 0.85 π^2 E / λ^2 otherwise } = ?? ksi
```

### Local buckling of the wall (B.5.4.5, E.3.1)

```calc
> Elastic local buckling stress of the wall
F_el = π^2 E / (16 R_bt (1 + sqrt(R_bt) / 35)^2) = ?? ksi
> Local buckling stress, B.5.4.5
F_cl = { F_cy if R_bt ≤ ((B_t - F_cy) / D_t)^2; B_t - D_t sqrt(R_bt) if R_bt < C_t; F_el otherwise } = ?? ksi
> Interaction of member and local buckling, Eq. E.4-1 (only when F_el < F_c)
F_ci = { (0.85 π^2 E / λ^2)^(1/3) F_el^(2/3) if F_el < F_c; F_cy otherwise } = ?? ksi
```

### Strength

```calc {description="Nominal compressive strength, the least limit state"}
P_n = min(F_c, F_cl, F_ci) A_g = ?? kips
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
