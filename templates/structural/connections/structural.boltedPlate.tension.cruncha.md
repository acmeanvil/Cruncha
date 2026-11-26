+++
cruncha-template = 1
id = "structural.boltedPlate.tension"
version = "1.0.0"
title = "Bolted plate in tension, AISC 360 J3 and J4 (LRFD or ASD)"
summary = "Available tensile strength of a flat plate pulled through a rectangular bolt pattern: bolt shear with bearing and tearout at each hole, gross yielding, net rupture and block shear, the least governing, by LRFD or ASD, checked against the required force and the spacing and edge distances."
category = ["Structural engineering", "Connections"]
tags = ["bolted plate", "bolt", "bearing", "tearout", "block shear", "net section", "gross yielding", "splice plate", "lap joint", "AISC 360", "J3", "J4", "LRFD", "ASD", "A325", "Group 120"]
from = "lacewing:structural.boltedPlate"
license = "GPL-3.0-or-later"
inputs = ["LRFD", "d_b", "n_s", "F_nv", "d_h", "l_(e,min)", "deform", "P_r", "t", "F_y", "F_u", "n_r", "n_l", "s", "l_e", "g", "e_g"]
outputs = ["r_(n,end)", "r_(n,in)", "R_(a,bolts)", "R_(a,y)", "R_(a,r)", "R_(a,bs)", "R_a"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. B4.3b (net area); D3, Table D3.1 case 1 (U = 1); J3.4 (minimum spacing); J3.5 and Table J3.4 (minimum edge distance); J3.7 (Eq. J3-1) and Table J3.2; Table J3.3 (standard holes); J3.11a (Eqs. J3-6a to J3-6d); J4.1 (Eqs. J4-1, J4-2); J4.3 (Eq. J4-5)"

[[tests]]
name = "1/2 × 8 in. A36 plate, 2 lines of 3 bolts 3/4 in. Group 120-N, 150 kips (LRFD): bolt shear governs and is exceeded"
source = "Lacewing Lab connections_test and help (its opening case: bolt shear about 107 kips for the six, ratio about 1.40), redone by hand with the AISC 360-22 equations (verified): shear 0.4418 × 54 = 23.86 kips per bolt, below bearing 52.2 and tearout 38.1 (end) and 76.1 (inner); 0.75 × 6 × 23.86 = 107.35 kips; yielding 0.9 × 36 × 4 = 129.6; net 0.75 × 58 × 3.125 = 135.9; block shear 0.75 × (162 + 90.6) = 189.5"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", d_h = "0.8125 in", "l_(e,min)" = "1 in", deform = "1", P_r = "150 kips", t = "0.5 in", F_y = "36 ksi", F_u = "58 ksi", n_r = "3", n_l = "2", s = "3 in", l_e = "1.5 in", g = "4 in", e_g = "2 in" }
expect = { "R_(a,bolts)" = "107.35 kips", "R_(a,y)" = "129.6 kips", "R_(a,r)" = "135.94 kips", "R_(a,bs)" = "189.47 kips", R_a = "107.35 kips" }
tolerance = "0.05%"
checks = "fail"

[[tests]]
name = "Same pattern at 100 kips (LRFD): passes"
source = "As above: 100 kips against 107.35 kips"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", d_h = "0.8125 in", "l_(e,min)" = "1 in", deform = "1", P_r = "100 kips", t = "0.5 in", F_y = "36 ksi", F_u = "58 ksi", n_r = "3", n_l = "2", s = "3 in", l_e = "1.5 in", g = "4 in", e_g = "2 in" }
expect = { R_a = "107.35 kips" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Same pattern, ASD, 70 kips"
source = "By hand, AISC 360-22 (Ω verified: 2.00 for bolts, J3.11, rupture and block shear; 1.67 for yielding): bolts 143.14 / 2.00 = 71.57 kips; yielding 144 / 1.67 = 86.23; net 181.25 / 2.00 = 90.63; block shear 252.63 / 2.00 = 126.31"
inputs = { LRFD = "0", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", d_h = "0.8125 in", "l_(e,min)" = "1 in", deform = "1", P_r = "70 kips", t = "0.5 in", F_y = "36 ksi", F_u = "58 ksi", n_r = "3", n_l = "2", s = "3 in", l_e = "1.5 in", g = "4 in", e_g = "2 in" }
expect = { "R_(a,bolts)" = "71.570 kips", "R_(a,y)" = "86.228 kips", "R_(a,r)" = "90.625 kips", "R_(a,bs)" = "126.31 kips", R_a = "71.570 kips" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "1/4 in. plate, 1 1/4 in. end distance (LRFD): end bolts tear out, gross yielding governs"
source = "Lacewing Lab connections_test: end bolt 1.2 × (1.25 − 13/32) × 0.25 × 58 = 14.68 kips nominal (Eq. J3-6c, verified). By hand: inner bolts 23.86 (shear); bolts 0.75 × 2 × (14.68 + 2 × 23.86) = 93.59 kips; yielding 0.9 × 36 × 2 = 64.8 kips; net 0.75 × 58 × 1.5625 = 67.97; block shear 0.75 × (78.3 + 45.31) = 92.71"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", d_h = "0.8125 in", "l_(e,min)" = "1 in", deform = "1", P_r = "60 kips", t = "0.25 in", F_y = "36 ksi", F_u = "58 ksi", n_r = "3", n_l = "2", s = "3 in", l_e = "1.25 in", g = "4 in", e_g = "2 in" }
expect = { "r_(n,end)" = "14.68 kips", "r_(n,in)" = "23.856 kips", "R_(a,bolts)" = "93.59 kips", "R_(a,r)" = "67.97 kips", "R_(a,bs)" = "92.71 kips", R_a = "64.8 kips" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "1/4 in. plate, deformation not a consideration: 1.5 l_c t F_u"
source = "By hand, Eq. J3-6d (verified): end bolt 1.5 × 0.84375 × 0.25 × 58 = 18.35 kips nominal; inner bolts still 23.86 (shear, under 3.0 × 0.75 × 0.25 × 58 = 32.6)"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", d_h = "0.8125 in", "l_(e,min)" = "1 in", deform = "0", P_r = "60 kips", t = "0.25 in", F_y = "36 ksi", F_u = "58 ksi", n_r = "3", n_l = "2", s = "3 in", l_e = "1.25 in", g = "4 in", e_g = "2 in" }
expect = { "r_(n,end)" = "18.352 kips", "r_(n,in)" = "23.856 kips" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "End distance under Table J3.4"
source = "By hand: l_e = 0.875 in against 1 in for a 3/4 in. bolt (AISC 360-22 Table J3.4, verified); end bolt tearout 1.2 × (0.875 − 0.40625) × 0.5 × 58 = 16.31 kips"
inputs = { LRFD = "1", d_b = "0.75 in", n_s = "1", F_nv = "54 ksi", d_h = "0.8125 in", "l_(e,min)" = "1 in", deform = "1", P_r = "50 kips", t = "0.5 in", F_y = "36 ksi", F_u = "58 ksi", n_r = "3", n_l = "2", s = "3 in", l_e = "0.875 in", g = "4 in", e_g = "2 in" }
expect = { "r_(n,end)" = "16.3125 kips" }
tolerance = "0.05%"
checks = "fail"
+++

## Bolted plate in tension (AISC 360 J3 and J4)

Available strength of **one** flat plate pulled along its length through a rectangular pattern of bolts in standard holes, under a concentric axial force: n_l lines of bolts along the force, each with n_r bolts. The limit states are the bolts (each the lesser of bolt shear and bearing or tearout at its hole), gross yielding and net rupture of the plate, and block shear of the end pattern. The least governs. LRFD compares ϕ R_n with the factored force; ASD compares R_n / Ω with the service force.

Scope: a flat plate loaded through every element (shear lag factor U = 1, Table D3.1 case 1), bolts not staggered, standard holes, at least two lines. The other plate of the joint, or the gusset, needs its own check with its own thickness and distances; so does the member the plate connects. For end-loaded patterns longer than 38 in., Table J3.2 reduces F_nv to 83.3 %; enter the reduced value.

### Method

```calc {description="Design method: 1 LRFD, 0 ASD"}
LRFD = 1_
```

```calc
function avail(R, ϕ, Ω, m)
    return { ϕ R if m == 1; R / Ω otherwise }
end
```

### Bolts

```calc {description="Nominal bolt diameter"}
d_b = 0.75 in_
```

```calc {description="Shear planes through each bolt (1 lap, 2 double-shear splice)"}
n_s = 1_
```

```calc {description="Nominal shear stress of the bolt, AISC 360 Table J3.2 (by bolt group, threads N or X)"}
F_nv = 54 ksi_
```

```calc {description="Standard hole diameter, AISC 360 Table J3.3"}
d_h = 0.8125 in_
```

```calc {description="Minimum edge distance for the bolt, AISC 360 Table J3.4"}
l_(e,min) = 1 in_
```

When deformation at the hole under service load is a design consideration, bearing is 2.4 d t F_u and tearout 1.2 l_c t F_u (Eqs. J3-6a, J3-6c); when it isn't, 3.0 d t F_u and 1.5 l_c t F_u (J3-6b, J3-6d).

```calc {description="1 if deformation at the holes is a design consideration, 0 if not"}
deform = 1_
```

### Plate, force and pattern

```calc {description="Required tension in the plate (factored for LRFD, service for ASD)"}
P_r = 150 kips_
```

```calc {description="Plate thickness"}
t = 0.5 in_
```

```calc {description="Yield stress of the plate"}
F_y = 36 ksi_
```

```calc {description="Tensile strength of the plate"}
F_u = 58 ksi_
```

```calc {description="Bolts in each line, along the force"}
n_r = 3_
```

```calc {description="Lines of bolts across the plate (2 or more)"}
n_l = 2_
```

```calc {description="Pitch: spacing of the bolts along the force"}
s = 3 in_
```

```calc {description="End distance: end bolts' centres to the end of the plate"}
l_e = 1.5 in_
```

```calc {description="Gauge: spacing of the lines across the plate"}
g = 4 in_
```

```calc {description="Side edge distance: outer lines to the plate's edges"}
e_g = 2 in_
```

### The bolts' strength

The clear distance l_c runs from the hole's edge to the plate's end (end bolts) or to the next hole (inner bolts). Each bolt's strength is the least of its shear and the bearing or tearout at its hole; the group's is their sum (J3.7 user note). ϕ = 0.75 and Ω = 2.00 for all three.

```calc
> Nominal bolt area
A_b = π d_b^2 / 4 = ?? in²
> Nominal shear strength of one bolt, Eq. J3-1
r_nv = F_nv A_b n_s = ?? kips
> Bearing coefficient, Eq. J3-6a or J3-6b
c_br = { 2.4 if deform == 1; 3.0 otherwise } = ??
> Tearout coefficient, Eq. J3-6c or J3-6d
c_to = { 1.2 if deform == 1; 1.5 otherwise } = ??
> Clear distance, end bolts
l_(c,end) = l_e - d_h / 2 = ?? in
> Clear distance, inner bolts
l_(c,in) = s - d_h = ?? in
> Nominal strength of an end bolt
r_(n,end) = min(r_nv, c_br d_b t F_u, c_to l_(c,end) t F_u) = ?? kips
> Nominal strength of an inner bolt
r_(n,in) = min(r_nv, c_br d_b t F_u, c_to l_(c,in) t F_u) = ?? kips
> Nominal strength of the bolt group
R_(n,bolts) = n_l (r_(n,end) + (n_r - 1) r_(n,in)) = ?? kips
```

```calc {description="Available strength of the bolts (ϕ 0.75, Ω 2.00)"}
R_(a,bolts) = avail(R_(n,bolts), 0.75, 2.00, LRFD) = ?? kips
```

### Plate: yielding and net rupture (J4.1)

Each hole's width in the net area is the hole plus 1/16 in. (B4.3b).

```calc
> Plate width
b_p = (n_l - 1) g + 2 e_g = ?? in
> Gross area
A_g = b_p t = ?? in²
> Width deducted per hole (B4.3b)
d_n = d_h + 0.0625 'in' = ?? in
> Net area across one row of holes
A_n = (b_p - n_l d_n) t = ?? in²
> Effective net area (U = 1)
A_e = A_n = ?? in²
```

```calc {description="Available strength in gross yielding, Eq. J4-1 (ϕ 0.90, Ω 1.67)"}
R_(a,y) = avail(F_y A_g, 0.90, 1.67, LRFD) = ?? kips
```

```calc {description="Available strength in net rupture, Eq. J4-2 (ϕ 0.75, Ω 2.00)"}
R_(a,r) = avail(F_u A_e, 0.75, 2.00, LRFD) = ?? kips
```

### Block shear (J4.3)

The end pattern tears out with shear along the two outer lines, from the plate's end past the last bolt, and tension either across between the outer lines (the middle block) or out from them to both side edges (the outer blocks). U_bs = 1, since the tension is uniform. The smaller block governs.

```calc
> Gross area in shear, two planes
A_gv = 2 (l_e + (n_r - 1) s) t = ?? in²
> Net area in shear
A_nv = A_gv - 2 (n_r - 0.5) d_n t = ?? in²
> Net area in tension, middle block
A_(nt,mid) = (n_l - 1) (g - d_n) t = ?? in²
> Net area in tension, outer blocks
A_(nt,out) = 2 (e_g - d_n / 2) t = ?? in²
> Nominal block shear strength, Eq. J4-5, the smaller block
R_(n,bs) = min(0.60 F_u A_nv, 0.60 F_y A_gv) + F_u min(A_(nt,mid), A_(nt,out)) = ?? kips
```

```calc {description="Available strength in block shear (ϕ 0.75, Ω 2.00)"}
R_(a,bs) = avail(R_(n,bs), 0.75, 2.00, LRFD) = ?? kips
```

### Governing strength

```calc {description="Available strength of the plate and its bolts"}
R_a = min(R_(a,bolts), R_(a,y), R_(a,r), R_(a,bs)) = ?? kips
```

### Checks

```calc
@test P_r ≤ R_a
```

Edge distances at least Table J3.4's (J3.5). Its note permits less where J3.11 and J4 are met, which the strengths above check, but never under d_b without the engineer of record's approval.

```calc
@test l_e ≥ l_(e,min) and e_g ≥ l_(e,min)
```

Spacing at least 2⅔ d_b, with at least d_b clear between holes (J3.4); 3 d_b is preferred.

```calc
@test s ≥ 8 / 3 d_b and g ≥ 8 / 3 d_b
```

```calc
@test s - d_h ≥ d_b and g - d_h ≥ d_b
```

```calc
@test n_l ≥ 2
```
