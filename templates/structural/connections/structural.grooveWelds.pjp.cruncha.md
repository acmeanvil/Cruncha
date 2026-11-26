+++
cruncha-template = 1
id = "structural.grooveWelds.pjp"
version = "1.0.0"
title = "PJP groove weld strength, AISC 360 J2 (LRFD or ASD)"
summary = "Available strength of a partial-joint-penetration groove weld, single or double: effective throat from the depth of groove (Table J2.1), less underfill or with a reinforcing fillet, the minimum throat of Table J2.3, and the weld metal and base metal at the fusion zone by Table J2.5 for tension, compression or shear, by LRFD or ASD."
category = ["Structural engineering", "Connections"]
tags = ["weld", "groove weld", "PJP", "partial joint penetration", "bevel", "J groove", "U groove", "V groove", "effective throat", "AISC 360", "J2", "Table J2.1", "Table J2.3", "Table J2.5", "AWS D1.1", "LRFD", "ASD", "E70"]
from = "lacewing:structural.grooveWelds"
license = "GPL-3.0-or-later"
inputs = ["LRFD", "S_1", "S_2", "ΔS", "u", "w_r", "t", "E_min", "l", "F_y", "F_u", "F_EXX", "F_(EXX,m)", "load", "P_r"]
outputs = ["E_1", "E_2", "E", "A_we", "R_(a,w)", "R_(a,b)", "R_a"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J2.1a (effective area, underfill), J2.1b, Tables J2.1 and J2.3 (pp. 16.1-125, 16.1-126), J2.4 (Eqs. J2-2, J2-3), Table J2.5 and its notes a and b (pp. 16.1-131, 16.1-132), J2.6, J4.1, J4.2, J4.4; Commentary J2.1a (reinforcing fillets count in the throat), J2.2a (throat as the shortest distance from the root to the weld face)"

[[references]]
work = "AISC Design Guide 21, Welded Connections: A Primer for Engineers (Miller)"
edition = "2nd (2017)"
locator = "Sec. 3.4.8 (PJP fusion zone check), 3.10.4 and Fig. 3-18 (PJP throat versus fusion plane)"

[[references]]
work = "AWS D1.1/D1.1M Structural Welding Code - Steel"
edition = "2010"
locator = "Figure 3.3 (prequalified PJP groove welded joint details, from p. 78), Table 3.1 (base metal and matching filler metal, from p. 64). AISC 360-22 cites the 2020 edition's Figure 5.2 and Tables 5.3 and 5.4 for these"

[[tests]]
name = "Double 45° bevel by SMAW, 3/8 in. each side, 1 in. A572 Gr 50, 10 in., E70, tension (LRFD; each side under the minimum throat)"
source = "Lacewing Lab groove_welds_test and help, its opening case, worked from AISC 360-22 Tables J2.1, J2.3 and J2.5 (verified; no published example was found): E = 2 × (3/8 − 1/8) = 1/2 in.; weld 0.80 × 0.60 × 70 × 5 = 168 kips; fusion zone 0.75 × 65 × 5 = 243.75 kips. The strength passes, but each side's 1/4 in. is under Table J2.3's 5/16 in. for a 1 in. part (J2.1b), so the checks fail"
inputs = { LRFD = "1", S_1 = "0.375 in", S_2 = "0.375 in", "ΔS" = "0.125 in", u = "0 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "1", P_r = "150 kips" }
expect = { E = "0.5 in", A_we = "5 in²", "R_(a,w)" = "168 kips", "R_(a,b)" = "243.75 kips", R_a = "168 kips" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "Same joint, compression not designed to bear (LRFD): the fusion zone governs"
source = "Lacewing Lab groove_welds_test, Table J2.5 (verified): weld 0.80 × 0.90 × 70 × 5 = 252 kips; base metal at the fusion zone 0.90 × 50 × 5 = 225 kips (checks fail on the minimum throat, as above)"
inputs = { LRFD = "1", S_1 = "0.375 in", S_2 = "0.375 in", "ΔS" = "0.125 in", u = "0 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "2", P_r = "200 kips" }
expect = { "R_(a,w)" = "252 kips", "R_(a,b)" = "225 kips", R_a = "225 kips" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "Same joint in shear (LRFD): shear rupture at the fusion zone governs"
source = "Lacewing Lab groove_welds_test, Table J2.5 and J4.2 (verified): weld 0.75 × 0.60 × 70 × 5 = 157.5 kips; fusion zone 1.00 × 0.60 × 50 × 5 = 150 and 0.75 × 0.60 × 65 × 5 = 146.25 kips"
inputs = { LRFD = "1", S_1 = "0.375 in", S_2 = "0.375 in", "ΔS" = "0.125 in", u = "0 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "160 kips" }
expect = { "R_(a,w)" = "157.5 kips", "R_(a,b)" = "146.25 kips", R_a = "146.25 kips" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "Double 45° bevel by SMAW, 7/16 in. each side, 1 in. A572 Gr 50, 10 in., E70, tension (LRFD)"
source = "By hand, AISC 360-22 Tables J2.1, J2.3 and J2.5 (verified): E = 2 × (7/16 − 1/8) = 5/8 in., each side 5/16 in., the minimum for a 1 in. part; weld 0.80 × 0.60 × 70 × 6.25 = 210 kips; fusion zone 0.75 × 65 × 6.25 = 304.7 kips"
inputs = { LRFD = "1", S_1 = "0.4375 in", S_2 = "0.4375 in", "ΔS" = "0.125 in", u = "0 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "1", P_r = "180 kips" }
expect = { E = "0.625 in", "R_(a,w)" = "210 kips", "R_(a,b)" = "304.6875 kips", R_a = "210 kips" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Same 7/16 in. double bevel, tension, ASD"
source = "By hand, Table J2.5 (verified: weld Ω 1.88, base Ω 2.00): weld 0.60 × 70 × 6.25 / 1.88 = 139.63 kips; fusion zone 65 × 6.25 / 2.00 = 203.13 kips"
inputs = { LRFD = "0", S_1 = "0.4375 in", S_2 = "0.4375 in", "ΔS" = "0.125 in", u = "0 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "1", P_r = "120 kips" }
expect = { "R_(a,w)" = "139.628 kips", "R_(a,b)" = "203.125 kips", R_a = "139.628 kips" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Same 7/16 in. double bevel in shear, ASD"
source = "By hand, Table J2.5 and J4.2 (verified): weld 0.60 × 70 × 6.25 / 2.00 = 131.25 kips; fusion zone 0.60 × 50 × 6.25 / 1.50 = 125 and 0.60 × 65 × 6.25 / 2.00 = 121.875 kips"
inputs = { LRFD = "0", S_1 = "0.4375 in", S_2 = "0.4375 in", "ΔS" = "0.125 in", u = "0 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "100 kips" }
expect = { "R_(a,w)" = "131.25 kips", R_a = "121.875 kips" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "J groove by GMAW, 1/2 in. deep, filled 1/16 in. short of flush (LRFD)"
source = "Lacewing Lab groove_welds_test: E = 0.5 − 0.0625 = 0.4375 in (J2.1a, verified). By hand, tension: 0.80 × 0.60 × 70 × 0.4375 × 10 = 147 kips"
inputs = { LRFD = "1", S_1 = "0.5 in", S_2 = "0 in", "ΔS" = "0 in", u = "0.0625 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "1", P_r = "100 kips" }
expect = { E = "0.4375 in", R_a = "147 kips" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "45° bevel by GMAW (flat), 3/8 in. deep, with a 1/2 in. reinforcing fillet"
source = "Lacewing Lab groove_welds_test (AISC 360-22 Commentary J2.1a, verified: reinforcing fillets on T- and corner joints count in the effective throat; the shortest root-to-face distance follows J2.2a's definition, by analogy): w ≥ E, so E = (0.375 + 0.5) / √2 = 0.61872 in"
inputs = { LRFD = "1", S_1 = "0.375 in", S_2 = "0 in", "ΔS" = "0 in", u = "0 in", w_r = "0.5 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "1", P_r = "100 kips" }
expect = { E = "0.618718 in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Single 45° bevel by SMAW, 3/8 in. deep in 1 in. plate: under the minimum throat"
source = "By hand: E = 3/8 − 1/8 = 1/4 in. (Table J2.1, verified) < 5/16 in. for a 1 in. part (Table J2.3, verified)"
inputs = { LRFD = "1", S_1 = "0.375 in", S_2 = "0 in", "ΔS" = "0.125 in", u = "0 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "1", P_r = "50 kips" }
expect = { E = "0.25 in", R_a = "84 kips" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "E90 filler on 50 ksi steel: more than one level over matching"
source = "Lacewing Lab groove_welds_test; Table J2.5 (verified): PJP filler equal to or less than matching, and note b permits one strength level greater; matching for 50 ksi steel is 70 ksi (J2.6 User Note)"
inputs = { LRFD = "1", S_1 = "0.4375 in", S_2 = "0.4375 in", "ΔS" = "0.125 in", u = "0 in", w_r = "0 in", t = "1 in", E_min = "0.3125 in", l = "10 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "90 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "100 kips" }
expect = { "R_(a,w)" = "253.125 kips" }
tolerance = "1e-9"
checks = "fail"
+++

## PJP groove weld strength (AISC 360 J2)

Available strength of a **partial-joint-penetration** groove weld of length l joining parts whose thinner is t: a bevel, V, J or U groove from one side, or from both. Each side's effective throat comes from its depth of groove by Table J2.1. The weld metal works on that throat, and the base metal is checked at the fusion zone beside it, each by Table J2.5 for the direction of the load. LRFD compares ϕ R_n with the factored force; ASD compares R_n / Ω with the service force.

Not covered: CJP and flare groove welds (their own templates); whether the joint is a prequalified detail of AWS D1.1 (root opening, root face, groove angle, bevel radius and their tolerances, by joint designation: check the drawing against D1.1); combined loads on one weld; groove welds acting with fillets; fatigue; preheat. Tension or compression parallel to the weld axis, and compression in a joint designed to bear (J1.4(b)), may be neglected in the design of the weld (Table J2.5), so they have no case here.

### Method

```calc {description="Design method: 1 LRFD, 0 ASD"}
LRFD = 1_
```

```calc
function avail(R, ϕ, Ω, m)
    return { ϕ R if m == 1; R / Ω otherwise }
end
```

### Groove

Table J2.1 gives each side's effective throat from its depth of groove S, by process, welding position and groove type: either S itself, or S less 1/8 in. Combinations the table doesn't list must be qualified by test.

```calc {description="Depth of groove, first side"}
S_1 = 0.4375 in_
```

```calc {description="Depth of groove, second side (0 for a single groove)"}
S_2 = 0.4375 in_
```

```calc {description="Throat reduction per side, AISC 360 Table J2.1 (0 or 1/8 in.)"}
ΔS = 0.125 in_
```

A weld filled short of flush loses the greatest distance from a line flush with the base metal to the weld's surface (J2.1a). In a T- or corner joint, an equal-leg reinforcing fillet over the groove counts in the throat (Commentary J2.1a), taken here as the shortest distance from the root to the face of the final weld, as J2.2a defines a fillet's throat: (E + w)/√2 while the leg w is at least E, else √(E² + w²). Underfill is ignored where there is a reinforcing fillet.

```calc {description="Underfill below flush, each side"}
u = 0 in_
```

```calc {description="Reinforcing fillet leg over each groove (0 for none; T- and corner joints only)"}
w_r = 0 in_
```

### Joint, materials and load

```calc {description="Thickness of the thinner part joined"}
t = 1 in_
```

```calc {description="Minimum effective throat for the thinner part, AISC 360 Table J2.3"}
E_min = 0.3125 in_
```

```calc {description="Effective length of the weld"}
l = 10 in_
```

```calc {description="Yield stress of the base metal (thinner part)"}
F_y = 50 ksi_
```

```calc {description="Tensile strength of the base metal (thinner part)"}
F_u = 65 ksi_
```

```calc {description="Filler metal classification strength"}
F_EXX = 70 ksi_
```

```calc {description="Matching filler metal strength for the base metal (AISC 360 J2.6 User Note; AWS D1.1)"}
F_(EXX,m) = 70 ksi_
```

```calc {description="Load: 1 tension normal to the weld axis, 2 compression normal (not designed to bear), 3 shear"}
load = 1_
```

```calc {description="Required force on the joint (factored for LRFD, service for ASD)"}
P_r = 180 kips_
```

### Effective throat

```calc
function pjpThroat(S, Δ, u, w)
    E = S - Δ
    return { 0 'in' if S ≤ 0 'in'; (E + w) / sqrt(2) if w > 0 'in' and w ≥ E; sqrt(E^2 + w^2) if w > 0 'in'; max(E - u, 0 'in') otherwise }
end
```

```calc
> Effective throat, first side
E_1 = pjpThroat(S_1, ΔS, u, w_r) = ?? in
> Effective throat, second side
E_2 = pjpThroat(S_2, ΔS, u, w_r) = ?? in
> Effective throat of the joint
E = E_1 + E_2 = ?? in
> Effective area of the weld (J2.1a)
A_we = E l = ?? in²
```

### Strength (Table J2.5)

The weld metal works on its effective area. Table J2.5's base metal check for a PJP weld is taken at the fusion zone beside the weld, which Design Guide 21 (Sec. 3.4.8) takes as about the size of the weld's throat, so its area here is also E l. The thinner part's whole section, t l, is larger and never governs, so it isn't checked.

```calc
> Weld metal, Eq. J2-3: 0.60 F_EXX in tension (ϕ 0.80, Ω 1.88), 0.90 F_EXX in compression (ϕ 0.80, Ω 1.88), 0.60 F_EXX in shear (ϕ 0.75, Ω 2.00)
R_(a,w) = { avail(0.60 F_EXX A_we, 0.80, 1.88, LRFD) if load == 1; avail(0.90 F_EXX A_we, 0.80, 1.88, LRFD) if load == 2; avail(0.60 F_EXX A_we, 0.75, 2.00, LRFD) if load == 3 } = ?? kips
> Base metal at the fusion zone, Eq. J2-2 with J4: F_u in tension (ϕ 0.75, Ω 2.00), F_y in compression (ϕ 0.90, Ω 1.67), the lesser of 0.60 F_y (ϕ 1.00, Ω 1.50) and 0.60 F_u (ϕ 0.75, Ω 2.00) in shear
R_(a,b) = { avail(F_u A_we, 0.75, 2.00, LRFD) if load == 1; avail(F_y A_we, 0.90, 1.67, LRFD) if load == 2; min(avail(0.60 F_y A_we, 1.00, 1.50, LRFD), avail(0.60 F_u A_we, 0.75, 2.00, LRFD)) if load == 3 } = ?? kips
```

```calc {description="Available strength of the joint"}
R_a = min(R_(a,w), R_(a,b)) = ?? kips
```

### Checks

```calc
@test P_r ≤ R_a
```

A PJP groove leaves a root face: the depths of groove together must stay short of the thickness.

```calc
@test S_1 + S_2 < t
```

Each weld's throat at least Table J2.3's minimum (J2.1b).

```calc
@test E_1 ≥ E_min
```

```calc
@test S_2 ≤ 0 'in' or E_2 ≥ E_min
```

A PJP weld may use filler metal equal to or weaker than matching, or one strength level (10 ksi) stronger (Table J2.5 and its note b).

```calc
@test F_EXX ≤ F_(EXX,m) + 10 'ksi'
```
