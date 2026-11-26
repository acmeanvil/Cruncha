+++
cruncha-template = 1
id = "structural.stainless.compressionRectHSS"
version = "1.0.0"
title = "Stainless rectangular HSS column with slender walls, AISC 370 E3 and E7 (LRFD and ASD)"
summary = "Compressive strength of a square or rectangular stainless steel HSS by flexural buckling on curve C, with the effective width of each pair of slender walls at the critical stress, checked against the required axial load by LRFD and by ASD."
category = ["Structural engineering", "Members"]
tags = ["AISC 370", "stainless steel", "Design Guide 27", "compression", "column", "HSS", "rectangular HSS", "slender element", "effective width", "E7", "curve C", "duplex", "LRFD", "ASD"]
from = "lacewing:structural.stainlessMember"
license = "GPL-3.0-or-later"
inputs = ["F_y", "E", "A_g", "t", "λ_b", "λ_h", "L_cx", "r_x", "L_cy", "r_y", "α", "β_0", "β_1", "β_2", "P_u", "P_a"]
outputs = ["F_cr", "b_e", "h_e", "A_e", "P_n", "ϕP_n", "P_(n,ASD)"]

[[references]]
work = "ANSI/AISC 370-21 Specification for Structural Stainless Steel Buildings"
edition = "2021"
locator = "Sec. E3 (Eqs. E3-1 to E3-4; Table E3.1, curve C for rectangular HSS); Sec. E7 (Eqs. E7-1 to E7-4); Table B4.1a (walls of rectangular HSS, λr = 1.24√(E/Fy))"

[[references]]
work = "AISC Design Guide 27, Structural Stainless Steel"
edition = "2nd"
locator = "Sec. 12.2, Design Example 2 (square HSS column with slender walls), pp. 377-380"

[[tests]]
name = "DG27 Example 2: HSS 6 × 6 × 0.180, S32101 lean duplex, 11 ft"
source = "AISC Design Guide 27, 2nd ed., Design Example 2, pp. 377-380 (checked against the PDF): b = 6.00 − 3 × 0.171 = 5.49 in., b/t 32.1, Fe 92.3 ksi, Fcr 50.2 ksi, Fel 102 ksi, be 5.18 in., Ae 3.70 in², Pn 186 kips, ϕcPn 167 kips, Pn/Ωc 111 kips (the book rounds b − be to 0.310 in.). Hand check with E 29 000 ksi: Fe = π² × 29 000/(132/2.37)² = 92.267 ksi; Fcr (curve C) = 50.203 ksi; λr√(Fy/Fcr) = 24.06 × 1.2385 = 29.80 < 32.1; Fel = 4π² × 29 000/(12 × 0.91)/32.1² = 101.748 ksi; b = 5.4891 in., be = 5.1739 in.; Ae = 3.91 − 4 × 0.3152 × 0.171 = 3.6944 in²; Pn = 185.47 kips"
inputs = { F_y = "77 ksi", E = "29000 ksi", A_g = "3.91 in^2", t = "0.171 in", "λ_b" = "32.1", "λ_h" = "32.1", L_cx = "11 ft", r_x = "2.37 in", L_cy = "11 ft", r_y = "2.37 in", "α" = "0.69", "β_0" = "1.195", "β_1" = "0.501", "β_2" = "0.820", P_u = "150 kips", P_a = "100 kips" }
expect = { A_e = "3.70 in^2", P_n = "186 kips", "ϕP_n" = "167 kips", "P_(n,ASD)" = "111 kips" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "DG27 Example 2, exactly"
source = "By hand, as above"
inputs = { F_y = "77 ksi", E = "29000 ksi", A_g = "3.91 in^2", t = "0.171 in", "λ_b" = "32.1", "λ_h" = "32.1", L_cx = "11 ft", r_x = "2.37 in", L_cy = "11 ft", r_y = "2.37 in", "α" = "0.69", "β_0" = "1.195", "β_1" = "0.501", "β_2" = "0.820", P_u = "150 kips", P_a = "100 kips" }
expect = { F_cr = "50.20291 ksi", b_e = "5.173922 in", A_e = "3.694418 in^2", P_n = "185.4705 kips", "ϕP_n" = "166.9235 kips", "P_(n,ASD)" = "111.0602 kips" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Stockier walls are fully effective; too much load"
source = "By hand: λ = 20 below λr√(Fy/Fcr) = 29.80 for both pairs, so Ae = Ag = 3.91 in²; Pn = 50.20291 × 3.91 = 196.2934 kips; ϕPn = 176.664 < 180; Pn/Ω = 117.54 ≥ 110"
inputs = { F_y = "77 ksi", E = "29000 ksi", A_g = "3.91 in^2", t = "0.171 in", "λ_b" = "20", "λ_h" = "20", L_cx = "11 ft", r_x = "2.37 in", L_cy = "11 ft", r_y = "2.37 in", "α" = "0.69", "β_0" = "1.195", "β_1" = "0.501", "β_2" = "0.820", P_u = "180 kips", P_a = "110 kips" }
expect = { A_e = "3.91 in^2", P_n = "196.2934 kips" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Example 2 with 112 kips by ASD: the ASD check fails"
source = "Pn/Ω = 185.47/1.67 = 111.06 kips < 112 kips, while 160 kips by LRFD is below ϕPn 166.92 kips"
inputs = { F_y = "77 ksi", E = "29000 ksi", A_g = "3.91 in^2", t = "0.171 in", "λ_b" = "32.1", "λ_h" = "32.1", L_cx = "11 ft", r_x = "2.37 in", L_cy = "11 ft", r_y = "2.37 in", "α" = "0.69", "β_0" = "1.195", "β_1" = "0.501", "β_2" = "0.820", P_u = "160 kips", P_a = "112 kips" }
expect = { "P_(n,ASD)" = "111.0602 kips" }
tolerance = "1e-6"
checks = "fail"
+++

## Stainless rectangular HSS column (AISC 370 E3, E7)

The compressive strength of a square or rectangular stainless steel hollow section. Flexural buckling about each axis uses curve C of Table E3.1 (enter its coefficients α 0.69, β₀ 1.195, β₁ 0.501, β₂ 0.820 from the table; Eqs. E3-1 to E3-5). Walls whose width-to-thickness ratio b/t is too large for the stress they reach buckle locally first: E7 then replaces each slender wall's width b by an effective width b_e found at the critical stress, and the strength is F_cr times the effective area.

λ_b and λ_h are the walls' b/t as AISC defines them for HSS (flat width over the design thickness, tabulated with the section's properties). t is the design wall thickness. The E7 elastic local buckling stress uses k = 4 (a wall supported on both edges) and ν = 0.3.

Both the LRFD required strength P_u (ϕc = 0.90) and the ASD required strength P_a (Ωc = 1.67) are checked; use the one you design by.

Not covered: round HSS (see the E3 template; a slender round HSS falls under E7.2), and second-order effects, which belong in P_u.

### Material and section

```calc {description="Specified minimum yield stress (0.2 % proof stress)"}
F_y = 77 ksi_
```

```calc {description="Modulus of elasticity of the alloy family"}
E = 29000 ksi_
```

```calc {description="Gross area"}
A_g = 3.91 in²_
```

```calc {description="Design wall thickness"}
t = 0.171 in_
```

```calc {description="Width-to-thickness ratio b/t of the walls on the B side"}
λ_b = 32.1_
```

```calc {description="Width-to-thickness ratio h/t of the walls on the H side"}
λ_h = 32.1_
```

```calc {description="Effective length for buckling about x"}
L_cx = 11 ft_
```

```calc {description="Radius of gyration about x"}
r_x = 2.37 in_
```

```calc {description="Effective length for buckling about y"}
L_cy = 11 ft_
```

```calc {description="Radius of gyration about y"}
r_y = 2.37 in_
```

```calc {description="Curve coefficient α, Table E3.1 (curve C for rectangular HSS)"}
α = 0.69_
```

```calc {description="Curve coefficient β₀, Table E3.1"}
β_0 = 1.195_
```

```calc {description="Curve coefficient β₁, Table E3.1"}
β_1 = 0.501_
```

```calc {description="Curve coefficient β₂, Table E3.1"}
β_2 = 0.820_
```

```calc {description="Required axial compression, LRFD (factored)"}
P_u = 150 kips_
```

```calc {description="Required axial compression, ASD (service)"}
P_a = 100 kips_
```

### Flexural buckling

```calc
> Elastic buckling stress, the more slender axis, Eq. E3-5
F_e = π^2 E / max(L_cx / r_x, L_cy / r_y)^2 = ?? ksi
> Fy over Fe
ρ = F_y / F_e = ??
> Critical stress on the curve, Eqs. E3-2 to E3-4
F_cr = { F_y if ρ ≤ (β_0 / π)^2; 1.2 β_1^(ρ^α) F_y if ρ ≤ 3.20; β_2 F_e otherwise } = ?? ksi
```

### Effective area, E7

A wall is fully effective while λ ≤ λ_r √(F_y/F_cr) (Eq. E7-2); otherwise its effective width is from Eq. E7-3 at its elastic local buckling stress F_el (Eq. E7-4).

```calc
function effectiveWidth(λ, t, λ_r, F_y, F_cr, E)
    b = λ t
    if λ ≤ λ_r sqrt(F_y / F_cr)
        return b
    end
    F_el = 4 π^2 E / (12 (1 - 0.3^2)) / λ^2
    return min(b, 0.772 b (1 - 0.10 sqrt(F_el / F_cr)) sqrt(F_el / F_cr))
end
```

```calc
> Limiting width-to-thickness ratio of a wall, Table B4.1a
λ_r = 1.24 sqrt(E / F_y) = ??
> Effective width of the B walls, Eq. E7-3
b_e = effectiveWidth(λ_b, t, λ_r, F_y, F_cr, E) = ?? in
> Effective width of the H walls, Eq. E7-3
h_e = effectiveWidth(λ_h, t, λ_r, F_y, F_cr, E) = ?? in
> Effective area, two walls of each pair
A_e = A_g - 2 (λ_b t - b_e) t - 2 (λ_h t - h_e) t = ?? in²
```

### Strength

```calc {description="Nominal compressive strength, Eq. E7-1"}
P_n = F_cr A_e = ?? kips
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

```calc
@test max(L_cx / r_x, L_cy / r_y) ≤ 200
```
