+++
cruncha-template = 1
id = "mechanical.weldCapacity.fillet"
version = "1.0.0"
title = "Fillet weld allowable load, AWS D1.1 and AISC 360 (ASD)"
summary = "Allowable load of one equal-leg fillet weld from its leg, length and the filler metal's classification strength: 0.30 F_EXX on the effective throat (0.60 F_EXX with Ω = 2.00), with the directional increase for a transverse load, checked against the service load and the size and length limits."
category = ["Mechanical engineering", "Fastening"]
tags = ["weld", "fillet weld", "weld capacity", "allowable load", "throat", "F_EXX", "E70", "electrode", "AWS D1.1", "AISC 360", "ASD", "directional strength", "minimum fillet size"]
from = "lacewing:mechanical.weldCapacity"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "F_EXX", "θ", "F", "w_min", "t_p"]
outputs = ["t_e", "τ_all", "k_ds", "q_all", "F_all", "τ", "w_max"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J2.2a (effective throat), J2.2b (minimum and maximum size, minimum length), J2.4 (Eqs. J2-4, J2-5: directional strength), Table J2.5 (F_nw = 0.60 F_EXX, Ω = 2.00); Table J2.4 (minimum size)"

[[references]]
work = "AWS D1.1/D1.1M Structural Welding Code - Steel"
edition = "2010"
locator = "Table 2.3 (allowable stresses: 0.30 × classification tensile strength on the throat), 2.6.4.2 (linear welds loaded in plane), Table 5.8 (minimum fillet weld sizes)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 9-6 (strength of welded joints), Table 9-6 (allowable stresses by the AISC code for weld metal)"

[[tests]]
name = "E70 electrode, 3/8 in fillet 4 in long, parallel load"
source = "Lacewing Lab weld_codes_test (AWS D1.1 and AISC 360: 0.30 F_EXX; E70 gives 21 ksi on the throat and 14.85 ksi per unit leg), and by hand: F_all = 0.7071 × 0.375 × 4 × 21 = 22.27 kips"
inputs = { w = "0.375 in", L = "4 in", F_EXX = "70 ksi", "θ" = "0 deg", F = "16.5 kips", w_min = "0.25 in", t_p = "0.5 in" }
expect = { "τ_all" = "21 ksi", t_e = "0.265165 in", q_all = "5.56847 kips/in", F_all = "22.2739 kips", w_max = "0.4375 in" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "E70, 3/8 in fillet 4 in long, transverse load: 1.5 times stronger"
source = "Lacewing Lab weld_codes_test: the transverse allowable is 1.5 times the parallel one (AISC 360 Eq. J2-5 at θ = 90°, AWS D1.1 2.6.4.2); by hand F_all = 1.5 × 22.27 = 33.41 kips"
inputs = { w = "0.375 in", L = "4 in", F_EXX = "70 ksi", "θ" = "90 deg", F = "30 kips", w_min = "0.25 in", t_p = "0.5 in" }
expect = { k_ds = "1.5", F_all = "33.4108 kips" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Stainless filler of 75 ksi: 22.5 ksi on the throat"
source = "Lacewing Lab weld_codes_test (AWS D1.6 and AISC 370: 0.30 F_EXX, 75 ksi gives 22.5 ksi); D1.6 takes no directional increase, so θ = 0"
inputs = { w = "0.25 in", L = "3 in", F_EXX = "75 ksi", "θ" = "0 deg", F = "8 kips", w_min = "0.1875 in", t_p = "0.3125 in" }
expect = { "τ_all" = "22.5 ksi", F_all = "11.9324 kips", w_max = "0.25 in" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "E70, 3/8 in × 4 in, 25 kips parallel: overloaded"
source = "As the first case: 25 kips against 22.27 kips allowable"
inputs = { w = "0.375 in", L = "4 in", F_EXX = "70 ksi", "θ" = "0 deg", F = "25 kips", w_min = "0.25 in", t_p = "0.5 in" }
expect = { "τ" = "23.570 ksi" }
tolerance = "1e-4"
checks = "fail"

[[tests]]
name = "3/16 in fillet where the thicker part needs 1/4 in: under the minimum size"
source = "By hand: w = 0.1875 in < w_min = 0.25 in (the minimum size is entered from AWS D1.1 Table 5.8 or AISC 360 Table J2.4); the load itself is carried"
inputs = { w = "0.1875 in", L = "4 in", F_EXX = "70 ksi", "θ" = "0 deg", F = "5 kips", w_min = "0.25 in", t_p = "0.5 in" }
expect = { F_all = "11.1369 kips" }
tolerance = "1e-5"
checks = "fail"
+++

## Fillet weld allowable load (AWS D1.1, AISC 360 ASD)

The allowable load of **one** equal-leg fillet weld under a service load through its centre, by allowable stress design. The steel codes take a fillet's stress as shear on its effective throat whatever the direction of the load, and allow 30 % of the filler metal's classification tensile strength there: AWS D1.1 Table 2.3 directly, AISC 360 as the nominal 0.60 F_EXX with Ω = 2.00. AWS D1.6 and AISC 370 (stainless steel) use the same 0.30 F_EXX. The base metal along the weld needs its own check.

For a group of welds under twist or bending, find the largest force per length with a weld group template and compare it with q_all here.

### Weld and load

```calc {description="Fillet leg size (equal legs)"}
w = 0.375 in_
```

```calc {description="Effective length of the weld, full size (craters excluded)"}
L = 4 in_
```

The classification tensile strength is the filler's: 60, 70 or 80 ksi for E60XX, E70XX and E80XX electrodes, for instance. Use filler metal that matches the base metal (AWS D1.1 Table 3.1, AISC 360 Table J2.5).

```calc {description="Filler metal classification tensile strength"}
F_EXX = 70 ksi_
```

The load's angle to the weld's axis: 0° along the weld, 90° across it. A transverse load may take the directional increase only for a linear weld, or welds all in line, loaded in plane through their centre of gravity; otherwise, or with any doubt, enter 0°. AWS D1.6, D1.5 and D1.2 take no increase.

```calc {description="Angle of the load to the weld's axis (0° parallel, 90° transverse)"}
θ = 0 deg_
```

```calc {description="Service load on the weld"}
F = 16.5 kips_
```

```calc {description="Minimum fillet size for the thicker part joined, AWS D1.1 Table 5.8 or AISC 360 Table J2.4"}
w_min = 0.25 in_
```

```calc {description="Thickness of the part whose edge the fillet runs along"}
t_p = 0.5 in_
```

### Strength

```calc
> Effective throat of an equal-leg fillet, 0.707 w (J2.2a)
t_e = w / sqrt(2) = ?? in
> Allowable shear stress on the throat, 0.60 F_EXX / Ω with Ω = 2.00
τ_all = 0.60 F_EXX / 2.00 = ?? ksi
> Directional strength factor, 1.0 + 0.50 sin^1.5 θ (Eq. J2-5)
k_ds = 1.0 + 0.50 sin(θ)^1.5 = ??
> Allowable load per length of weld
q_all = k_ds τ_all t_e = ?? kips/in
> Allowable load on the weld
F_all = q_all L = ?? kips
> Shear stress on the throat under the service load
τ = F / (t_e L) = ?? ksi
```

### Size and length limits

Along the edge of a part 1/4 in thick or more, the leg may be at most the thickness less 1/16 in (unless the weld is built out to full throat); under 1/4 in, the thickness. For a fillet that does not run along an edge, enter the thickness of the thinner part joined plus 1/16 in, so the limit is the part itself. A fillet's effective length must be at least four times its leg (AISC J2.2b). End-loaded welds longer than 100 times the leg take a reduced effective length (J2.2b), which is not applied here.

```calc {description="Largest leg along the part's edge, AISC 360 J2.2b"}
w_max = { t_p if t_p < 0.25 'in'; t_p - 0.0625 'in' otherwise } = ?? in
```

### Checks

```calc
@test F ≤ F_all
```

```calc
@test w_min ≤ w ≤ w_max
```

```calc
@test L ≥ 4 w
```
