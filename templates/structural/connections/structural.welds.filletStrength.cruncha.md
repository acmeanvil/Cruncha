+++
cruncha-template = 1
id = "structural.welds.filletStrength"
version = "1.0.0"
title = "Fillet weld strength, AISC 360 J2 (LRFD or ASD)"
summary = "Available strength of one line of equal-leg fillet weld: 0.60 F_EXX on the effective throat with the directional factor k_ds and the long-weld reduction β for end-loaded welds, by LRFD or ASD, checked against the required force and the size and length limits."
category = ["Structural engineering", "Connections"]
tags = ["weld", "fillet weld", "AISC 360", "J2", "LRFD", "ASD", "E70", "F_EXX", "directional strength", "k_ds", "long weld", "β", "minimum fillet size", "Table J2.4", "Table J2.5"]
from = "lacewing:structural.filletWelds"
license = "GPL-3.0-or-later"
inputs = ["LRFD", "w", "l", "F_EXX", "θ", "endLoaded", "w_min", "t_edge", "P_r"]
outputs = ["t_e", "k_ds", "β", "l_e", "A_we", "R_n", "R_a", "r_a", "w_max"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J2.2a (effective throat), J2.2b(a) to (d) (minimum size, maximum size along edges, minimum length, end-loaded welds and Eq. J2-1), J2.4(a) (Eqs. J2-4, J2-5); Tables J2.4 and J2.5 (pp. 16.1-126 to 16.1-132)"

[[references]]
work = "AISC Steel Construction Manual"
edition = "15th"
locator = "Part 8, p. 8-8: Eq. 8-1, Eqs. 8-2a and 8-2b (E70 fillet strength per sixteenth of leg: LRFD 1.392 kip/in, ASD 0.928 kip/in)"

[[tests]]
name = "1/4 in. E70 fillet, 10 in., longitudinal load, LRFD"
source = "AISC Manual 15th, p. 8-8, Eq. 8-2a (verified): ϕR_n = 1.392 kip/in × D × l = 1.392 × 4 × 10 = 55.68 kips; also Lacewing Lab connections_test. Eqs. J2-4 and J2-5 and Table J2.5 checked in AISC 360-22 (pp. 16.1-130, 16.1-132)"
inputs = { LRFD = "1", w = "0.25 in", l = "10 in", F_EXX = "70 ksi", "θ" = "0 deg", endLoaded = "0", w_min = "0.1875 in", t_edge = "0.5 in", P_r = "40 kips" }
expect = { t_e = "0.176777 in", r_a = "5.568 kips/in", R_a = "55.68 kips", w_max = "0.4375 in" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Same weld, ASD"
source = "AISC Manual 15th, p. 8-8, Eq. 8-2b (verified): R_n/Ω = 0.928 kip/in × 4 × 10 = 37.12 kips, 3.712 kips/in"
inputs = { LRFD = "0", w = "0.25 in", l = "10 in", F_EXX = "70 ksi", "θ" = "0 deg", endLoaded = "0", w_min = "0.1875 in", t_edge = "0.5 in", P_r = "30 kips" }
expect = { R_a = "37.12 kips", r_a = "3.712 kips/in" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Same weld, transverse load: k_ds = 1.5 (LRFD)"
source = "AISC 360-22 Eq. J2-5 (verified) at θ = 90°: k_ds = 1.5; by hand 1.5 × 55.68 = 83.52 kips (Lacewing Lab connections_test has k_ds = 1.5)"
inputs = { LRFD = "1", w = "0.25 in", l = "10 in", F_EXX = "70 ksi", "θ" = "90 deg", endLoaded = "0", w_min = "0.1875 in", t_edge = "0.5 in", P_r = "80 kips" }
expect = { k_ds = "1.5", R_a = "83.52 kips" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Load at 45°, ASD"
source = "By hand, Eq. J2-5: k_ds = 1 + 0.5 × 0.70711^1.5 = 1.29730; R_n/Ω = 37.123 × 1.29730 = 48.16 kips"
inputs = { LRFD = "0", w = "0.25 in", l = "10 in", F_EXX = "70 ksi", "θ" = "45 deg", endLoaded = "0", w_min = "0.1875 in", t_edge = "0.5 in", P_r = "40 kips" }
expect = { k_ds = "1.29730", R_a = "48.16 kips" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "End-loaded 1/4 in. weld 50 in. long (200w): β = 0.8 (LRFD)"
source = "AISC 360-22 J2.2b(d)(2), Eq. J2-1 (verified, p. 16.1-127): β = 1.2 − 0.002 × 200 = 0.8; by hand l_e = 40 in, R_a = 5.568 × 40 = 222.7 kips (Lacewing Lab connections_test has β = 0.8)"
inputs = { LRFD = "1", w = "0.25 in", l = "50 in", F_EXX = "70 ksi", "θ" = "0 deg", endLoaded = "1", w_min = "0.1875 in", t_edge = "0.5 in", P_r = "200 kips" }
expect = { "β" = "0.8", l_e = "40 in", R_a = "222.7 kips" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "End-loaded 1/4 in. weld 100 in. long (400w): l_e = 180w (LRFD)"
source = "AISC 360-22 J2.2b(d)(3) (verified): beyond 300w the effective length is 180w = 45 in; by hand R_a = 5.568 × 45 = 250.6 kips (Lacewing Lab connections_test has 45 in)"
inputs = { LRFD = "1", w = "0.25 in", l = "100 in", F_EXX = "70 ksi", "θ" = "0 deg", endLoaded = "1", w_min = "0.1875 in", t_edge = "0.5 in", P_r = "200 kips" }
expect = { l_e = "45 in", R_a = "250.6 kips" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "3/16 in. fillet where Table J2.4 asks 1/4 in.: under the minimum size"
source = "By hand: w = 3/16 in. < w_min = 1/4 in. (AISC 360-22 Table J2.4, verified, for a thinner part over 1/2 to 3/4 in.); the strength itself suffices"
inputs = { LRFD = "1", w = "0.1875 in", l = "10 in", F_EXX = "70 ksi", "θ" = "0 deg", endLoaded = "0", w_min = "0.25 in", t_edge = "0.75 in", P_r = "20 kips" }
expect = { R_a = "41.76 kips" }
tolerance = "0.05%"
checks = "fail"

[[tests]]
name = "1/4 in. E70, 10 in., 60 kips: overloaded (LRFD)"
source = "As the first case: 60 kips against 55.68 kips"
inputs = { LRFD = "1", w = "0.25 in", l = "10 in", F_EXX = "70 ksi", "θ" = "0 deg", endLoaded = "0", w_min = "0.1875 in", t_edge = "0.5 in", P_r = "60 kips" }
expect = { R_a = "55.68 kips" }
tolerance = "0.05%"
checks = "fail"
+++

## Fillet weld strength (AISC 360 J2)

Available strength of **one line** of equal-leg fillet weld under a force through it, to AISC 360-22 Section J2, by LRFD (ϕ R_n against factored loads) or ASD (R_n / Ω against service loads). The nominal stress on the effective throat is 0.60 F_EXX, raised for a load at an angle to the weld's axis and, for long end-loaded welds, reduced along the length. No credit is taken for penetration beyond the root.

Not covered: groups of welds under an eccentric load (use a weld group template to find the largest force per length, then compare it with r_a here); concentric groups of longitudinal and transverse lines (Eq. J2-6); groove welds (the groove weld templates). The Specification's user note says the base metal check need not be made for fillet welds.

### Method

```calc {description="Design method: 1 LRFD, 0 ASD"}
LRFD = 1_
```

```calc
function avail(R, ϕ, Ω, m)
    return { ϕ R if m == 1; R / Ω otherwise }
end
```

### Weld and load

```calc {description="Fillet leg size (equal legs)"}
w = 0.25 in_
```

```calc {description="Length of the line of weld"}
l = 10 in_
```

```calc {description="Filler metal classification strength (E70XX: 70 ksi)"}
F_EXX = 70 ksi_
```

The directional increase k_ds holds only where the weld's strain compatibility is considered (J2.4(a)): a linear weld group, one leg size, loaded through its centre of gravity. For fillet welds to the ends of rectangular HSS in tension, and in all other cases, k_ds = 1: enter 0°.

```calc {description="Angle of the load to the weld axis (0° longitudinal, 90° transverse)"}
θ = 0 deg_
```

An end-loaded weld is a longitudinal weld at the end of an axially loaded member, such as the side welds of a lap splice. Longer than 100 times its leg, it takes the reduction of J2.2b(d).

```calc {description="1 if the weld is end-loaded, 0 if not"}
endLoaded = 0_
```

```calc {description="Minimum fillet size for the thinner part joined, AISC 360 Table J2.4"}
w_min = 0.1875 in_
```

```calc {description="Thickness of the part whose edge the weld runs along"}
t_edge = 0.5 in_
```

```calc {description="Required force on the weld (factored for LRFD, service for ASD)"}
P_r = 40 kips_
```

### Strength

```calc {description="Effective throat of an equal-leg fillet (J2.2a)"}
t_e = w / sqrt(2) = ?? in
```

```calc {description="Directional strength increase factor, Eq. J2-5"}
k_ds = 1 + 0.50 sin(θ)^1.5 = ??
```

```calc
> Long-weld reduction factor, Eq. J2-1 (end-loaded, over 100w)
β = { min(1.2 - 0.002 l / w, 1) if endLoaded == 1 and l > 100 w; 1 otherwise } = ??
> Effective length: β l, or 180w over 300w, J2.2b(d)
l_e = { 180 w if endLoaded == 1 and l > 300 w; β l otherwise } = ?? in
```

```calc
> Nominal stress of the weld metal, Table J2.5
F_nw = 0.60 F_EXX = ?? ksi
> Effective area of the weld (J2.2a)
A_we = t_e l_e = ?? in²
> Nominal strength, Eq. J2-4
R_n = F_nw A_we k_ds = ?? kips
```

ϕ = 0.75 and Ω = 2.00 for fillet welds (Table J2.5).

```calc {description="Available strength of the line of weld (ϕ R_n or R_n / Ω)"}
R_a = avail(R_n, 0.75, 2.00, LRFD) = ?? kips
```

```calc {description="Available strength per length of weld"}
r_a = avail(F_nw t_e k_ds, 0.75, 2.00, LRFD) = ?? kips/in
```

### Size and length limits

Along an edge the leg may be at most the thickness for material under 1/4 in., else the thickness less 1/16 in., unless the weld is designated to be built out to full throat (J2.2b(b)). A fillet designed for strength must be at least four times its leg long (J2.2b(c)).

```calc {description="Maximum fillet size along an edge, J2.2b(b)"}
w_max = { t_edge if t_edge < 0.25 'in'; t_edge - 0.0625 'in' otherwise } = ?? in
```

### Checks

```calc
@test P_r ≤ R_a
```

```calc
@test w_min ≤ w ≤ w_max
```

```calc
@test l ≥ 4 w
```
