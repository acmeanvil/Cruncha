+++
cruncha-template = 1
id = "mechanical.vBelts.drive"
version = "1.0.0"
title = "V-belt drive: rating, belts, tensions and life (Shigley)"
summary = "A multiple V-belt drive by Shigley's analysis: pitch length and centre distance, wrap and the groove's effective friction, belt speed, the allowable power per belt from the tabulated rating with the wrap (K1) and length (K2) corrections, design power and the number of belts, centrifugal, tight-side, slack-side and initial tensions per belt, the factor of safety, the peak tensions with bending, and the belt life in passes and hours."
category = ["Mechanical engineering", "Belts and chains"]
tags = ["belt", "V-belt", "vee belt", "sheave", "belt drive", "power rating", "service factor", "number of belts", "initial tension", "centrifugal tension", "belt life", "Gates", "Shigley"]
from = "lacewing:mechanical.vBelts"
license = "GPL-3.0-or-later"
inputs = ["H_nom", "n", "d", "D", "L_i", "L_add", "d_min", "N_b", "K_s", "n_d", "H_tab", "K_1", "K_2", "f_e", "K_c", "K_b", "K_life", "b_life", "N_max"]
outputs = ["L_p", "C", "φ", "r_t", "V", "H_a", "H_d", "N_(b,req)", "F_c", "ΔF", "F_1", "F_2", "F_i", "n_fs", "T_1", "T_2", "N_P", "t_life"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 17-3: Eqs. 17-16a, 17-16b (length and centre distance), 17-17 (allowable power), 17-18 (Gates' effective friction), 17-19 to 17-26 (design power, belts, tensions, factor of safety), 17-27, 17-28 (life); Tables 17-9 (sections), 17-10 (lengths), 17-11 (length conversion), 17-12 (ratings), 17-13 (K1), 17-14 (K2), 17-15 (Ks), 17-16 (Kb, Kc), 17-17 (durability constants); the analysis steps of p. 897; Example 17-4"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 19.3"

[[tests]]
name = "Shigley Ex. 17-4: three B112 belts on 7.4 and 11 in sheaves, 10 hp at 1750 rev/min, Ks 1.3"
source = "Lacewing Lab v_belts_test, after Shigley 10th Example 17-4: V = 3390 ft/min, Lp = 113.8 in, C = 42.4 in, φ = 3.057 rad, exp(0.5123φ) = 4.788, Htab = 4.693 hp (Table 17-12, between 3000 and 4000 ft/min), K1 = 0.99, K2 = 1.05, Ha = 4.878 hp, Hd = 13 hp, Nb = 2.67 → 3, Fc = 11.1, ΔF = 42.2, F1 = 64.4, F2 = 22.2, Fi = 32.2 lbf, nfs = 1.13, T1 = 142.2, T2 = 116.8 lbf, Np over 10⁹ (the book finds 11 × 10⁹), t over 46 600 h"
inputs = { H_nom = "10 hp", n = "1750 rpm", d = "7.4 in", D = "11 in", L_i = "112 in", L_add = "1.8 in", d_min = "5.4 in", N_b = "3", K_s = "1.3", n_d = "1", H_tab = "4.693 hp", K_1 = "0.99", K_2 = "1.05", f_e = "0.5123", K_c = "0.965 lbf", K_b = "576 lbf·in", K_life = "1193 lbf", b_life = "10.926", N_max = "1e9" }
expect = { V = "3390 ft/min", L_p = "113.8 in", C = "42.4 in", "φ" = "3.057", r_t = "4.788", H_a = "4.878 hp", H_d = "13 hp", "N_(b,req)" = "3", F_c = "11.1 lbf", "ΔF" = "42.2 lbf", F_1 = "64.4 lbf", F_2 = "22.2 lbf", F_i = "32.2 lbf", n_fs = "1.13", T_1 = "142.2 lbf", T_2 = "116.8 lbf", t_life = "46600 h" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 17-4 exactly"
source = "By hand: C = 0.25{[113.8 − 9.2π] + √([113.8 − 9.2π]² − 2 × 3.6²)} = 42.410476 in, φ = 3.0566825, exp(0.5123φ) = 4.7871652, V = π × 7.4 × 1750/12 = 3390.3021 ft/min, Fc = 0.965 × 3.3903021² = 11.091853 lbf, Ha = 4.8783735 hp, ΔF = (13/3) × 33 000/3390.3021 = 42.179132 lbf, F1 = 64.408374, F2 = 22.229242, Fi = 32.226955 lbf, nfs = 1.1257785, T1 = 142.24621, T2 = 116.77201 lbf, Np = 1.1057921e10, t = 10⁹ × 113.8/12/3390.3021/60 = 46 619.904 h"
inputs = { H_nom = "10 hp", n = "1750 rpm", d = "7.4 in", D = "11 in", L_i = "112 in", L_add = "1.8 in", d_min = "5.4 in", N_b = "3", K_s = "1.3", n_d = "1", H_tab = "4.693 hp", K_1 = "0.99", K_2 = "1.05", f_e = "0.5123", K_c = "0.965 lbf", K_b = "576 lbf·in", K_life = "1193 lbf", b_life = "10.926", N_max = "1e9" }
expect = { C = "42.410476 in", "φ" = "3.0566825", r_t = "4.7871652", V = "3390.3021 ft/min", F_c = "11.091853 lbf", H_a = "4.8783735 hp", "ΔF" = "42.179132 lbf", F_1 = "64.408374 lbf", F_2 = "22.229242 lbf", F_i = "32.226955 lbf", n_fs = "1.1257785", T_1 = "142.24621 lbf", T_2 = "116.77201 lbf", N_P = "1.1057921e10", t_life = "46619.904 h" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 17-4 with two belts: too few"
source = "By hand: Hd/Ha = 2.6648 needs three belts; with two, ΔF = 63.268699 lbf, F1 = 91.066635 lbf, nfs = 4.8783735 × 2/13 = 0.750519, Np = 1.6176165e9"
inputs = { H_nom = "10 hp", n = "1750 rpm", d = "7.4 in", D = "11 in", L_i = "112 in", L_add = "1.8 in", d_min = "5.4 in", N_b = "2", K_s = "1.3", n_d = "1", H_tab = "4.693 hp", K_1 = "0.99", K_2 = "1.05", f_e = "0.5123", K_c = "0.965 lbf", K_b = "576 lbf·in", K_life = "1193 lbf", b_life = "10.926", N_max = "1e9" }
expect = { "ΔF" = "63.268699 lbf", F_1 = "91.066635 lbf", n_fs = "0.750519", N_P = "1.6176165e9" }
tolerance = "1e-6"
checks = "fail"
+++

## V-belt drive

A drive of one or more V-belts of the same section on two grooved sheaves, the small one driving, by Shigley's analysis (Sec. 17-3, p. 897). A belt's rated power from the maker's table is corrected for the wrap on the small sheave (K_1) and for the belt's length (K_2), and the number of belts is the design power over that allowable power, rounded up. The tensions per belt follow from the power each belt carries, the centrifugal tension and the groove's effective friction; the initial tension F_i is what each belt is set to at installation. The life comes from the peak tensions, bending included, at the two sheaves.

The tables are inputs: the rating H_tab (Shigley Table 17-12 by section, small-sheave pitch diameter and belt speed, or a maker's catalogue), K_1 (Table 17-13 by (D − d)/C), K_2 (Table 17-14 by section and length), the section's K_b and K_c (Table 17-16) and its durability constants K and b (Table 17-17). Read H_tab and K_1 once the belt speed and centre distance below are known.

### Drive

```calc {description="Nominal power transmitted"}
H_nom = 10 hp_
```

```calc {description="Speed of the small (driving) sheave"}
n = 1750 rpm_
```

```calc {description="Small sheave pitch diameter"}
d = 7.4 in_
```

```calc {description="Large sheave pitch diameter"}
D = 11 in_
```

```calc {description="Smallest sheave pitch diameter for the section, Shigley Table 17-9"}
d_min = 5.4 in_
```

```calc {description="Service factor, Shigley Table 17-15"}
K_s = 1.3_
```

```calc {description="Design factor"}
n_d = 1_
```

```calc {description="Number of belts"}
N_b = 3_
```

### Belt

A lettered belt (A to E) is named by its section and inside circumference (B112: section B, 112 in, Table 17-10); its pitch length adds Table 17-11's amount for the section. For a narrow section (3V, 5V, 8V), enter the pitch length as L_i with L_add = 0.

```calc {description="Inside circumference of the belt, Shigley Table 17-10"}
L_i = 112 in_
```

```calc {description="Inside-to-pitch length addition for the section, Shigley Table 17-11"}
L_add = 1.8 in_
```

```calc {description="Rated power of one belt, Shigley Table 17-12 or the maker's catalogue"}
H_tab = 4.693 hp_
```

```calc {description="Angle-of-contact correction factor, Shigley Table 17-13"}
K_1 = 0.99_
```

```calc {description="Belt-length correction factor, Shigley Table 17-14"}
K_2 = 1.05_
```

```calc {description="Effective coefficient of friction of the groove (Gates' 0.5123, Shigley Eq. 17-18)"}
f_e = 0.5123_
```

```calc {description="Centrifugal parameter Kc of the section, Shigley Table 17-16 (tension at 1000 ft/min)"}
K_c = 0.965 lbf_
```

```calc {description="Bending parameter Kb of the section, Shigley Table 17-16"}
K_b = 576 lbf·in_
```

```calc {description="Durability constant K of the section, Shigley Table 17-17"}
K_life = 1193 lbf_
```

```calc {description="Durability exponent b of the section, Shigley Table 17-17"}
b_life = 10.926_
```

```calc {description="Upper end of the force-peak range Table 17-17's constants cover"}
N_max = 1e9_
```

### Geometry and speed

Shigley asks the centre distance to be no more than three times the sum of the sheave diameters and no less than the larger one, and the belt speed near 4000 ft/min: below 1000 or above 5000 ft/min V-belts run poorly (p. 892). The speed n is an angular speed, so the belt speed π d n (n in revolutions) is n d / 2 here.

```calc
> Pitch length, Table 17-11
L_p = L_i + L_add = ?? in
> Centre distance, Eq. 17-16b
C = 0.25 ((L_p - π (D + d) / 2) + sqrt((L_p - π (D + d) / 2)^2 - 2 (D - d)^2)) = ?? in
> Wrap on the small sheave, Eq. 17-1
φ = π - 2 asin((D - d) / (2 C)) = ??
> Tension ratio exp(f_e φ), Eq. 17-18
r_t = exp(f_e φ) = ??
> Belt speed
V = n d / 2 = ?? ft/min
```

### Power and belts

```calc
> Allowable power per belt, Eq. 17-17
H_a = K_1 K_2 H_tab = ?? hp
> Design power, Eq. 17-19
H_d = H_nom K_s n_d = ?? hp
> Belts needed, Eq. 17-20
N_(b,req) = ceil(H_d / H_a) = ??
> Factor of safety, Eq. 17-26
n_fs = H_a N_b / (H_nom K_s) = ??
```

### Tensions per belt

```calc
> Centrifugal tension, Eq. 17-21
F_c = K_c (V / (1000 'ft/min'))^2 = ?? lbf
> Power-transmitting tension difference, Eq. 17-22
ΔF = H_d / N_b / V = ?? lbf
> Tight-side tension, Eq. 17-23
F_1 = F_c + ΔF r_t / (r_t - 1) = ?? lbf
> Slack-side tension, Eq. 17-24
F_2 = F_1 - ΔF = ?? lbf
> Initial tension, Eq. 17-25
F_i = (F_1 + F_2) / 2 - F_c = ?? lbf
```

### Life

Each pass the belt sees a peak tension at each sheave: the tight-side tension plus the bending tension K_b over the sheave's diameter. Eq. 17-27 combines the two into the passes to failure; Table 17-17's constants hold up to N_max (10⁹ passes for A to E), and Shigley reports a longer result as "more than" that. The life in hours, Eq. 17-28, is taken at the smaller of N_P and N_max.

```calc
> Peak tension at the small sheave
T_1 = F_1 + K_b / d = ?? lbf
> Peak tension at the large sheave
T_2 = F_1 + K_b / D = ?? lbf
> Passes to failure, Eq. 17-27
N_P = 1 / ((K_life / T_1)^(-b_life) + (K_life / T_2)^(-b_life)) = ??
> Life in hours, Eq. 17-28 (at most N_max passes)
t_life = min(N_P, N_max) L_p / V = ?? h
```

### Checks

Enough belts for the design power, a small sheave no smaller than the section allows, the centre distance and belt speed in Shigley's ranges.

```calc
@test N_b ≥ H_d / H_a
```

```calc
@test d ≥ d_min
```

```calc
@test D ≤ C ≤ 3 (D + d)
```

```calc
@test 1000 'ft/min' ≤ V ≤ 5000 'ft/min'
```
