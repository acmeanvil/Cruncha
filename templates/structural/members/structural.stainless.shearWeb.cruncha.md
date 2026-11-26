+++
cruncha-template = 1
id = "structural.stainless.shearWeb"
version = "1.0.0"
title = "Stainless I-shape or channel web shear, AISC 370 G2.1 (LRFD and ASD)"
summary = "Shear strength of the unstiffened web of a stainless steel I-shape or channel, with the web shear strength coefficient Cv1 by web slenderness, checked against the required shear by LRFD and by ASD."
category = ["Structural engineering", "Members"]
tags = ["AISC 370", "stainless steel", "Design Guide 27", "shear", "web shear", "Cv1", "G2", "beam", "channel", "I-shape", "LRFD", "ASD"]
from = "lacewing:structural.stainlessMember"
license = "GPL-3.0-or-later"
inputs = ["F_y", "E", "d", "t_w", "λ_w", "V_u", "V_a"]
outputs = ["A_w", "C_v1", "V_n", "ϕV_n", "V_(n,ASD)"]

[[references]]
work = "ANSI/AISC 370-21 Specification for Structural Stainless Steel Buildings"
edition = "2021"
locator = "Sec. G2.1 (Eqs. G2-1 to G2-3, k_v = 5.34 for unstiffened webs); Sec. G1 (ϕv 0.90, Ωv 1.67)"

[[references]]
work = "AISC Design Guide 27, Structural Stainless Steel"
edition = "2nd"
locator = "Sec. 12.3, Design Example 3 (C12×30 beam, S30400: shear), pp. 383-384"

[[tests]]
name = "DG27 Example 3: C12×30, S30400"
source = "AISC Design Guide 27, 2nd ed., Design Example 3, pp. 383-384 (checked against the PDF): 0.59√(kvE/Fy) = 41.7, Cv1 1.2, Vn 130 kips, ϕvVn 117 kips, Vn/Ωv 77.8 kips (the book rounds Vn to 130 first). By hand: h/tw 22.0 ≤ 0.59 √(5.34 × 28 000/30) = 41.65, Cv1 1.2; Vn = 0.6 × 30 × 12 × 0.5 × 1.2 = 129.6 kips"
inputs = { F_y = "30 ksi", E = "28000 ksi", d = "12 in", t_w = "0.5 in", "λ_w" = "22.0", V_u = "40 kips", V_a = "30 kips" }
expect = { C_v1 = "1.2", V_n = "129.6 kips", "ϕV_n" = "116.64 kips", "V_(n,ASD)" = "77.604790 kips" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Slender welded web, duplex: Cv1 below 1.2"
source = "By hand: √(5.34 × 29 000/65) = 48.811; h/tw 60 > 0.59 × 48.811 = 28.80; Cv1 = 1.55 × 48.811/(0.7 × 48.811 + 60) = 75.657/94.168 = 0.80343; Aw = 18 × 0.3 = 5.4 in²; Vn = 0.6 × 65 × 5.4 × 0.80343 = 169.202 kips; ϕVn 152.28 < 160"
inputs = { F_y = "65 ksi", E = "29000 ksi", d = "18 in", t_w = "0.3 in", "λ_w" = "60", V_u = "160 kips", V_a = "100 kips" }
expect = { C_v1 = "0.803428", V_n = "169.202 kips" }
tolerance = "1e-5"
checks = "fail"

[[tests]]
name = "Example 3 with 78 kips by ASD: the ASD check fails"
source = "Vn/Ω = 129.6/1.67 = 77.605 kips < 78 kips; 100 kips by LRFD is below ϕVn 116.64 kips"
inputs = { F_y = "30 ksi", E = "28000 ksi", d = "12 in", t_w = "0.5 in", "λ_w" = "22.0", V_u = "100 kips", V_a = "78 kips" }
expect = { "V_(n,ASD)" = "77.60479 kips" }
tolerance = "1e-6"
checks = "fail"
+++

## Stainless web shear (AISC 370 G2.1)

The shear strength of the web of a stainless steel I-shape or channel loaded in the plane of the web, without transverse stiffeners (k_v = 5.34). The shear area is the overall depth times the web thickness. A stocky web reaches C_v1 = 1.2, which credits the strain hardening of stainless steel; a slender web's strength falls with h/t_w (Eq. G2-3).

Both the LRFD required shear V_u (ϕv = 0.90) and the ASD required shear V_a (Ωv = 1.67) are checked; use the one you design by.

Stiffened webs, tension field action and shear in the flanges (G5) are not covered.

### Material and web

```calc {description="Specified minimum yield stress (0.2 % proof stress)"}
F_y = 30 ksi_
```

```calc {description="Modulus of elasticity of the alloy family"}
E = 28000 ksi_
```

```calc {description="Overall depth"}
d = 12 in_
```

```calc {description="Web thickness"}
t_w = 0.5 in_
```

```calc {description="Web slenderness h/t_w"}
λ_w = 22.0_
```

```calc {description="Required shear, LRFD (factored)"}
V_u = 40 kips_
```

```calc {description="Required shear, ASD (service)"}
V_a = 30 kips_
```

### Strength

```calc
> Shear area
A_w = d t_w = ?? in²
> √(k_v E / F_y), unstiffened web
r_v = sqrt(5.34 E / F_y) = ??
> Web shear strength coefficient, Eqs. G2-2 and G2-3
C_v1 = { 1.2 if λ_w ≤ 0.59 r_v; 1.55 r_v / (0.7 r_v + λ_w) otherwise } = ??
```

```calc {description="Nominal shear strength, Eq. G2-1"}
V_n = 0.6 F_y A_w C_v1 = ?? kips
```

```calc {description="Design shear strength (LRFD, ϕv = 0.90)"}
ϕV_n = 0.90 V_n = ?? kips
```

```calc {description="Allowable shear strength (ASD, Ωv = 1.67)"}
V_(n,ASD) = V_n / 1.67 = ?? kips
```

### Check

```calc
@test V_u ≤ ϕV_n
```

```calc
@test V_a ≤ V_(n,ASD)
```
