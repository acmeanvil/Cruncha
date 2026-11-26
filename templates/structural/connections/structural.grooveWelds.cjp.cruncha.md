+++
cruncha-template = 1
id = "structural.grooveWelds.cjp"
version = "1.0.0"
title = "CJP groove weld strength, AISC 360 J2 (LRFD or ASD)"
summary = "Available strength of a complete-joint-penetration groove weld by Table J2.5: the base metal of the thinner part governs in tension, compression and shear (J4), with undermatched filler in shear (note c) and the filler strength levels Table J2.5 permits, by LRFD or ASD."
category = ["Structural engineering", "Connections"]
tags = ["weld", "groove weld", "CJP", "complete joint penetration", "butt weld", "splice", "AISC 360", "J2", "Table J2.5", "J4", "AWS D1.1", "matching filler", "LRFD", "ASD", "E70"]
from = "lacewing:structural.grooveWelds"
license = "GPL-3.0-or-later"
inputs = ["LRFD", "t", "l", "F_y", "F_u", "F_EXX", "F_(EXX,m)", "load", "P_r"]
outputs = ["A_g", "R_(a,b)", "R_a"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J2.1a (CJP effective throat), J2.4, Table J2.5 and its notes b and c (pp. 16.1-131, 16.1-132), J2.6 and its User Note (p. 16.1-133), J4.1 (Eqs. J4-1, J4-2), J4.2 (Eqs. J4-3, J4-4), J4.4 (Eq. J4-6)"

[[references]]
work = "AWS D1.1/D1.1M Structural Welding Code - Steel"
edition = "2010"
locator = "Figure 3.4 (prequalified CJP groove welded joint details, from p. 94), Table 3.1 (base metal and matching filler metal, from p. 64). AISC 360-22 cites the 2020 edition's clause 5 and Tables 5.3 and 5.4 for these"

[[tests]]
name = "3/4 in. A36 butt splice, 10 in., E70, tension (LRFD)"
source = "Lacewing Lab groove_welds_test, worked from AISC 360-22 Table J2.5 and J4.1 (verified; no published example was found): yielding 0.90 × 36 × 7.5 = 243 kips, rupture 0.75 × 58 × 7.5 = 326.25 kips. Matching filler for A36 to 3/4 in. is 60 or 70 ksi (J2.6 User Note, verified); 60 is entered, and E70 is one level over, which note b permits"
inputs = { LRFD = "1", t = "0.75 in", l = "10 in", F_y = "36 ksi", F_u = "58 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "60 ksi", load = "1", P_r = "200 kips" }
expect = { A_g = "7.5 in²", R_a = "243 kips" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Same splice, tension, ASD"
source = "By hand, J4.1 (verified: Ω 1.67 yielding, 2.00 rupture): 36 × 7.5 / 1.67 = 161.68 kips; 58 × 7.5 / 2.00 = 217.5 kips"
inputs = { LRFD = "0", t = "0.75 in", l = "10 in", F_y = "36 ksi", F_u = "58 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "60 ksi", load = "1", P_r = "150 kips" }
expect = { R_a = "161.677 kips" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Same splice in compression (LRFD)"
source = "Lacewing Lab groove_welds_test: J4.4, Eq. J4-6 (verified), 0.90 × 36 × 7.5 = 243 kips"
inputs = { LRFD = "1", t = "0.75 in", l = "10 in", F_y = "36 ksi", F_u = "58 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "60 ksi", load = "2", P_r = "200 kips" }
expect = { R_a = "243 kips" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "1/2 in. A572 Gr 50 web, 10 in., E70, shear (LRFD)"
source = "Lacewing Lab groove_welds_test, J4.2 (verified): 1.00 × 0.60 × 50 × 5 = 150 kips; 0.75 × 0.60 × 65 × 5 = 146.25 kips governs"
inputs = { LRFD = "1", t = "0.5 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "140 kips" }
expect = { R_a = "146.25 kips" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Same web with undermatched E60 filler in shear (note c), LRFD"
source = "Lacewing Lab groove_welds_test; Table J2.5 note c (verified: thickness as the throat, ϕ 0.80, Ω 1.88, 0.60 F_EXX): 0.80 × 0.60 × 60 × 5 = 144 kips governs"
inputs = { LRFD = "1", t = "0.5 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "60 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "140 kips" }
expect = { R_a = "144 kips" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Same undermatched web in shear, ASD"
source = "Lacewing Lab groove_welds_test (180 / 1.88 = 95.74 kips); by hand the base metal gives 150 / 1.50 = 100 and 195 / 2.00 = 97.5 kips, so the weld governs"
inputs = { LRFD = "0", t = "0.5 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "60 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "90 kips" }
expect = { "R_(a,b)" = "97.5 kips", R_a = "95.7447 kips" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Undermatched E60 on 50 ksi steel in tension: not permitted"
source = "Lacewing Lab groove_welds_test; Table J2.5 (verified): a CJP weld in tension needs matching filler. The strength itself, 0.90 × 50 × 10 = 450 kips, suffices"
inputs = { LRFD = "1", t = "1 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "60 ksi", "F_(EXX,m)" = "70 ksi", load = "1", P_r = "300 kips" }
expect = { R_a = "450 kips" }
tolerance = "1e-9"
checks = "fail"
+++

## CJP groove weld strength (AISC 360 J2)

Available strength of a **complete-joint-penetration** groove weld of length l through the thinner part joined, t. A CJP weld fuses the whole thickness, so its effective throat is t (J2.1a) and, with matching filler, the base metal governs: Table J2.5 sends tension, compression and shear to the base metal checks of Section J4 on the area t l. LRFD compares ϕ R_n with the factored force; ASD compares R_n / Ω with the service force.

Not covered: whether the joint is a prequalified detail of AWS D1.1 (check the drawing against D1.1's prequalified CJP details); the member's own net section away from the weld; combined loads; fatigue; notch toughness (J2.6 asks notch-tough filler for CJP T- and corner joints with backing left in place, and for splices of heavy shapes, in tension); preheat. Tension or compression parallel to the weld axis may be neglected in the design of the weld (Table J2.5).

### Method

```calc {description="Design method: 1 LRFD, 0 ASD"}
LRFD = 1_
```

```calc
function avail(R, ϕ, Ω, m)
    return { ϕ R if m == 1; R / Ω otherwise }
end
```

### Joint, materials and load

```calc {description="Thickness of the thinner part joined (the effective throat)"}
t = 0.75 in_
```

```calc {description="Effective length of the weld"}
l = 10 in_
```

```calc {description="Yield stress of the base metal (thinner part)"}
F_y = 36 ksi_
```

```calc {description="Tensile strength of the base metal (thinner part)"}
F_u = 58 ksi_
```

```calc {description="Filler metal classification strength"}
F_EXX = 70 ksi_
```

```calc {description="Matching filler metal strength for the base metal (AISC 360 J2.6 User Note; AWS D1.1)"}
F_(EXX,m) = 60 ksi_
```

```calc {description="Load: 1 tension normal to the weld axis, 2 compression normal, 3 shear"}
load = 1_
```

```calc {description="Required force on the joint (factored for LRFD, service for ASD)"}
P_r = 200 kips_
```

### Strength (Table J2.5)

```calc {description="Base metal area along the weld (thinner part)"}
A_g = t l = ?? in²
```

In tension the base metal yields (F_y; ϕ 0.90, Ω 1.67) or ruptures (F_u; ϕ 0.75, Ω 2.00); in compression it yields (F_y; ϕ 0.90, Ω 1.67, J4.4 for a short element); in shear it yields (0.60 F_y; ϕ 1.00, Ω 1.50) or ruptures (0.60 F_u; ϕ 0.75, Ω 2.00). Filler weaker than matching in shear is designed as a weld on the full thickness, 0.60 F_EXX with ϕ 0.80, Ω 1.88 (note c).

```calc
> Base metal, Table J2.5 with J4.1, J4.4 and J4.2
R_(a,b) = { min(avail(F_y A_g, 0.90, 1.67, LRFD), avail(F_u A_g, 0.75, 2.00, LRFD)) if load == 1; avail(F_y A_g, 0.90, 1.67, LRFD) if load == 2; min(avail(0.60 F_y A_g, 1.00, 1.50, LRFD), avail(0.60 F_u A_g, 0.75, 2.00, LRFD)) if load == 3 } = ?? kips
> Available strength of the joint, with an undermatched weld in shear (note c)
R_a = { min(R_(a,b), avail(0.60 F_EXX A_g, 0.80, 1.88, LRFD)) if load == 3 and F_EXX < F_(EXX,m); R_(a,b) otherwise } = ?? kips
```

### Checks

```calc
@test P_r ≤ R_a
```

Filler strength (Table J2.5 and its note b): in tension, matching, or one strength level (10 ksi) over; in compression, from one level under matching to one level over. In shear, filler under matching is permitted only for web-to-flange welds of built-up members transferring shear, and joints under high restraint (note c); the strength above then counts the weld.

```calc
@test load ≠ 1 or F_(EXX,m) ≤ F_EXX ≤ F_(EXX,m) + 10 'ksi'
```

```calc
@test load ≠ 2 or F_(EXX,m) - 10 'ksi' ≤ F_EXX ≤ F_(EXX,m) + 10 'ksi'
```

```calc
@test F_EXX ≤ F_(EXX,m) + 10 'ksi'
```
