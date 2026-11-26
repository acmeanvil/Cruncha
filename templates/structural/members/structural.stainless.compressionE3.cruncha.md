+++
cruncha-template = 1
id = "structural.stainless.compressionE3"
version = "1.0.0"
title = "Stainless steel column, flexural buckling, AISC 370 E3 (LRFD and ASD)"
summary = "Nominal and available compressive strength of a stainless steel member without slender elements, by flexural buckling about both axes on the buckling curves of AISC 370 Table E3.1, checked against the required axial load by LRFD and by ASD."
category = ["Structural engineering", "Members"]
tags = ["AISC 370", "stainless steel", "Design Guide 27", "compression", "column", "flexural buckling", "buckling curve", "E3", "austenitic", "duplex", "HSS", "LRFD", "ASD"]
from = "lacewing:structural.stainlessMember"
license = "GPL-3.0-or-later"
inputs = ["F_y", "E", "A_g", "L_cx", "r_x", "α_x", "β_0x", "β_1x", "β_2x", "L_cy", "r_y", "α_y", "β_0y", "β_1y", "β_2y", "P_u", "P_a"]
outputs = ["F_(e,x)", "F_(cr,x)", "F_(e,y)", "F_(cr,y)", "P_n", "ϕP_n", "P_(n,ASD)"]

[[references]]
work = "ANSI/AISC 370-21 Specification for Structural Stainless Steel Buildings"
edition = "2021"
locator = "Sec. E3 (Eqs. E3-1 to E3-5), Table E3.1 (flexural buckling coefficients); Sec. E1 (ϕc 0.90, Ωc 1.67); Sec. E2 User Note"

[[references]]
work = "AISC Design Guide 27, Structural Stainless Steel"
edition = "2nd"
locator = "Sec. 12.1, Design Example 1 (round HSS column), pp. 375-377"

[[tests]]
name = "DG27 Example 1: round HSS 6.25 × 0.250, S30400, 11 ft"
source = "AISC Design Guide 27, 2nd ed., Design Example 1, pp. 375-377 (checked against the PDF): E 28 000 ksi, Fy 30 ksi, Ag 4.50 in², r 2.13 in., Lc/r 62.0, Fe 71.9 ksi, curve B (α 0.58, β0 0.891, β1 0.455, β2 0.820), Fcr 22.4 ksi, Pn 101 kips, ϕcPn 90.9 kips, Pn/Ωc 60.5 kips"
inputs = { F_y = "30 ksi", E = "28000 ksi", A_g = "4.50 in^2", L_cx = "11 ft", r_x = "2.13 in", "α_x" = "0.58", "β_0x" = "0.891", "β_1x" = "0.455", "β_2x" = "0.820", L_cy = "11 ft", r_y = "2.13 in", "α_y" = "0.58", "β_0y" = "0.891", "β_1y" = "0.455", "β_2y" = "0.820", P_u = "90 kips", P_a = "60 kips" }
expect = { "F_(e,y)" = "71.9 ksi", "F_(cr,y)" = "22.4 ksi", P_n = "101 kips", "ϕP_n" = "90.9 kips", "P_(n,ASD)" = "60.5 kips" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "DG27 Example 1, exactly"
source = "By hand: Fe = π² × 28 000/(132/2.13)² = 71.95635 ksi; Fy/Fe 0.41692; Fcr = 1.2 × 0.455^(0.41692^0.58) × 30 = 22.40832 ksi; Pn = 100.83746 kips; ϕPn 90.75371 kips; Pn/Ω 60.38171 kips (the book rounds Fcr and Pn first: 90.9 and 60.5)"
inputs = { F_y = "30 ksi", E = "28000 ksi", A_g = "4.50 in^2", L_cx = "11 ft", r_x = "2.13 in", "α_x" = "0.58", "β_0x" = "0.891", "β_1x" = "0.455", "β_2x" = "0.820", L_cy = "11 ft", r_y = "2.13 in", "α_y" = "0.58", "β_0y" = "0.891", "β_1y" = "0.455", "β_2y" = "0.820", P_u = "90 kips", P_a = "60 kips" }
expect = { "F_(e,y)" = "71.95635 ksi", "F_(cr,y)" = "22.40832 ksi", P_n = "100.83746 kips", "ϕP_n" = "90.75371 kips", "P_(n,ASD)" = "60.38171 kips" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Example 1 with 61 kips by ASD: the ASD check fails"
source = "Pn/Ω = 60.38 kips < 61 kips, while 85 kips by LRFD is below ϕPn 90.75 kips"
inputs = { F_y = "30 ksi", E = "28000 ksi", A_g = "4.50 in^2", L_cx = "11 ft", r_x = "2.13 in", "α_x" = "0.58", "β_0x" = "0.891", "β_1x" = "0.455", "β_2x" = "0.820", L_cy = "11 ft", r_y = "2.13 in", "α_y" = "0.58", "β_0y" = "0.891", "β_1y" = "0.455", "β_2y" = "0.820", P_u = "85 kips", P_a = "61 kips" }
expect = { "P_(n,ASD)" = "60.38171 kips" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "W-shape: curve B about x, curve A about y (Eq. E3-3); overloaded by LRFD"
source = "By hand, coefficients of AISC 370 Table E3.1 (checked against the PDF): about y, Fe = π² × 28 000/(240/1.5)² = 10.79488 ksi; Fy/Fe = 2.7791 ≤ 3.20; Fcr = 1.2 × 0.409^(2.7791^0.56) × 30 = 7.38045 ksi; about x the curve-B stress is higher; Pn = 7.38045 × 10 = 73.8045 kips; ϕPn 66.42 < 80; Pn/Ω 44.19 ≥ 40"
inputs = { F_y = "30 ksi", E = "28000 ksi", A_g = "10 in^2", L_cx = "20 ft", r_x = "4 in", "α_x" = "0.58", "β_0x" = "0.891", "β_1x" = "0.455", "β_2x" = "0.820", L_cy = "20 ft", r_y = "1.5 in", "α_y" = "0.56", "β_0y" = "0.759", "β_1y" = "0.409", "β_2y" = "0.690", P_u = "80 kips", P_a = "40 kips" }
expect = { "F_(e,y)" = "10.79488 ksi", "F_(cr,y)" = "7.38045 ksi", P_n = "73.8045 kips" }
tolerance = "1e-5"
checks = "fail"
+++

## Stainless steel column, flexural buckling (AISC 370 E3)

The compressive strength of a stainless steel member whose elements are all nonslender (Table B4.1a), by flexural buckling about each principal axis. Stainless steel has a rounded stress-strain curve with no sharp yield, so AISC 370 uses three buckling curves, A, B and C, rather than AISC 360's single column curve. Table E3.1 gives each curve's coefficients α, β₀, β₁ and β₂, and says which curve applies: curve A for I-shapes buckling about the minor axis and sections not listed otherwise; curve B for I-shapes about the major axis, welded box sections and round HSS; curve C for rectangular HSS. Enter the coefficients for each axis from the table.

F_y is the specified 0.2 % proof stress for the grade and product form; E is 28 000 ksi (193 GPa) for austenitic and 29 000 ksi (200 GPa) for duplex grades (Sec. E3). The template checks both the LRFD required strength P_u (ϕc = 0.90) and the ASD required strength P_a (Ωc = 1.67); give the one you design by and ignore the other check.

Not covered: torsional and flexural-torsional buckling (E4), slender elements (E7: see the rectangular HSS template), and second-order effects, which belong in the required strengths.

### Material and section

```calc {description="Specified minimum yield stress (0.2 % proof stress)"}
F_y = 30 ksi_
```

```calc {description="Modulus of elasticity of the alloy family"}
E = 28000 ksi_
```

```calc {description="Gross area"}
A_g = 4.50 in²_
```

### Buckling about x

```calc {description="Effective length for buckling about x"}
L_cx = 11 ft_
```

```calc {description="Radius of gyration about x"}
r_x = 2.13 in_
```

```calc {description="Curve coefficient α about x, Table E3.1"}
α_x = 0.58_
```

```calc {description="Curve coefficient β₀ about x, Table E3.1"}
β_0x = 0.891_
```

```calc {description="Curve coefficient β₁ about x, Table E3.1"}
β_1x = 0.455_
```

```calc {description="Curve coefficient β₂ about x, Table E3.1"}
β_2x = 0.820_
```

### Buckling about y

```calc {description="Effective length for buckling about y"}
L_cy = 11 ft_
```

```calc {description="Radius of gyration about y"}
r_y = 2.13 in_
```

```calc {description="Curve coefficient α about y, Table E3.1"}
α_y = 0.58_
```

```calc {description="Curve coefficient β₀ about y, Table E3.1"}
β_0y = 0.891_
```

```calc {description="Curve coefficient β₁ about y, Table E3.1"}
β_1y = 0.455_
```

```calc {description="Curve coefficient β₂ about y, Table E3.1"}
β_2y = 0.820_
```

### Required strength

```calc {description="Required axial compression, LRFD (factored)"}
P_u = 90 kips_
```

```calc {description="Required axial compression, ASD (service)"}
P_a = 60 kips_
```

### Critical stress

The critical stress is F_y up to the slenderness β₀√(E/F_y) (Eq. E3-2), then an inelastic curve in F_y/F_e (Eq. E3-3), and beyond 5.62√(E/F_y), where F_y/F_e = 3.20, a fraction β₂ of the elastic buckling stress (Eq. E3-4).

```calc
function stainlessFcr(F_y, F_e, α, β_0, β_1, β_2)
    ρ = F_y / F_e
    if ρ ≤ (β_0 / π)^2
        return F_y
    end
    if ρ ≤ 3.20
        return 1.2 β_1^(ρ^α) F_y
    end
    return β_2 F_e
end
```

```calc
> Elastic buckling stress about x, Eq. E3-5
F_(e,x) = π^2 E / (L_cx / r_x)^2 = ?? ksi
> Critical stress about x, Eqs. E3-2 to E3-4
F_(cr,x) = stainlessFcr(F_y, F_(e,x), α_x, β_0x, β_1x, β_2x) = ?? ksi
```

```calc
> Elastic buckling stress about y, Eq. E3-5
F_(e,y) = π^2 E / (L_cy / r_y)^2 = ?? ksi
> Critical stress about y, Eqs. E3-2 to E3-4
F_(cr,y) = stainlessFcr(F_y, F_(e,y), α_y, β_0y, β_1y, β_2y) = ?? ksi
```

### Strength

```calc {description="Nominal compressive strength, Eq. E3-1"}
P_n = min(F_(cr,x), F_(cr,y)) A_g = ?? kips
```

```calc {description="Design compressive strength (LRFD, ϕc = 0.90)"}
ϕP_n = 0.90 P_n = ?? kips
```

```calc {description="Allowable compressive strength (ASD, Ωc = 1.67)"}
P_(n,ASD) = P_n / 1.67 = ?? kips
```

### Checks

```calc
@test P_u ≤ ϕP_n
```

```calc
@test P_a ≤ P_(n,ASD)
```

The E2 User Note prefers slenderness no greater than 200.

```calc
@test max(L_cx / r_x, L_cy / r_y) ≤ 200
```
