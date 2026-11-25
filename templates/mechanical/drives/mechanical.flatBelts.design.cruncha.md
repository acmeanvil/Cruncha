+++
cruncha-template = 1
id = "mechanical.flatBelts.design"
version = "1.0.0"
title = "Flat belt drive at its tension limit (Shigley)"
summary = "A two-pulley flat belt drive analysed by Shigley's procedure: wrap angles and belt length for an open or crossed drive, Euler-Eytelwein tension ratio, centrifugal tension, torque, the largest allowable tension from the belt's rating with the pulley and velocity factors, slack-side and initial tensions, the friction the drive develops, the factor of safety, the narrowest belt that does not slip, the most power the belt carries, the catenary dip and the shaft load."
category = ["Mechanical engineering", "Belts and chains"]
tags = ["belt", "flat belt", "belt drive", "polyamide belt", "leather belt", "Euler-Eytelwein", "capstan", "tension ratio", "initial tension", "centrifugal tension", "wrap angle", "belt length", "catenary dip", "shaft load", "Shigley"]
from = "lacewing:mechanical.flatBelts"
license = "GPL-3.0-or-later"
inputs = ["H_nom", "n", "d", "D", "C", "crossed", "b", "t", "γ", "f", "F_a", "d_min", "C_p", "C_v", "K_s", "n_d"]
outputs = ["φ", "θ_D", "L", "r_t", "V", "w", "F_c", "H_d", "T", "ΔF", "F_(1,a)", "F_2", "F_i", "f′", "n_fs", "b_min", "H_max", "y_dip", "F_shaft"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 17-2: Eqs. 17-1 to 17-4 (wrap and length), 17-7, 17-8 (belting equation and power), 17-12 (allowable tension), 17-13 (dip), Eqs. (e), (h), (i) of p. 877-878, the analysis procedure of p. 880, Tables 17-2 (belts), 17-4 (Cp), Fig. 17-9 (Cv), Table 17-15 (Ks); Examples 17-1 and 17-2, Prob. 17-5"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 19.2, Eqs. 19.1 to 19.3"

[[tests]]
name = "Shigley Ex. 17-1: polyamide A-3, 6 in wide, 15 hp at 1750 rev/min, 6 and 18 in pulleys 8 ft apart"
source = "Lacewing Lab flat_belts_test, after Shigley 10th Example 17-1: φ = 3.0165 rad, exp(fφ) = 11.17, V = 2749 ft/min, w = 0.393 lbf/ft, Fc = 25.6 lbf, T = 742.8 lbf·in, ΔF = 247.6 lbf, Cp = 0.70 (Table 17-4), (F1)a = 420 lbf, F2 = 172.4 lbf, Fi = 270.6 lbf, Hd = 20.6 hp, f′ = 0.328, nfs = 1.1, dip 0.14 in"
inputs = { H_nom = "15 hp", n = "1750 rpm", d = "6 in", D = "18 in", C = "96 in", crossed = "0", b = "6 in", t = "0.13 in", "γ" = "0.042 lbf/in³", f = "0.8", F_a = "100 lbf/in", d_min = "4.3 in", C_p = "0.70", C_v = "1", K_s = "1.25", n_d = "1.1" }
expect = { "φ" = "3.0165", r_t = "11.17", V = "2749 ft/min", w = "0.393 lbf/ft", F_c = "25.6 lbf", T = "742.8 lbf·in", "ΔF" = "247.6 lbf", "F_(1,a)" = "420 lbf", F_2 = "172.4 lbf", F_i = "270.6 lbf", H_d = "20.6 hp", "f′" = "0.328", n_fs = "1.1", y_dip = "0.14 in" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 17-2: polyamide A-3, 10 in wide, 60 hp at 860 rev/min, 16 and 36 in pulleys 16 ft apart"
source = "Lacewing Lab flat_belts_test, after Shigley 10th Example 17-2: Hd = 72.5 hp, T = 5310 lbf·in, Cp = 0.94, φ = 3.037 rad, V = 3602 ft/min, ΔF = 664 lbf, bmin = 8.40 in, Fc = 73.4 lbf, (F1)a = 940 lbf, F2 = 276 lbf, Fi = 535 lbf, f′ = 0.478, dip 0.470 in"
inputs = { H_nom = "60 hp", n = "860 rpm", d = "16 in", D = "36 in", C = "192 in", crossed = "0", b = "10 in", t = "0.13 in", "γ" = "0.042 lbf/in³", f = "0.8", F_a = "100 lbf/in", d_min = "4.3 in", C_p = "0.94", C_v = "1", K_s = "1.15", n_d = "1.05" }
expect = { H_d = "72.45 hp", T = "5310 lbf·in", "φ" = "3.037", V = "3602 ft/min", "ΔF" = "664 lbf", b_min = "8.40 in", F_c = "73.4 lbf", "F_(1,a)" = "940 lbf", F_2 = "276 lbf", F_i = "535 lbf", "f′" = "0.478", y_dip = "0.470 in" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Ex. 17-2 with an 8 in belt: narrower than bmin, it slips"
source = "Lacewing Lab flat_belts_test ('a belt narrower than bmin slips'); by hand: (F1)a = 8 × 100 × 0.94 = 752 lbf, Fc = 58.726 lbf, F2 = 88.310 lbf, f′ = ln(693.27/29.584)/3.03738 = 1.0385, over f = 0.8"
inputs = { H_nom = "60 hp", n = "860 rpm", d = "16 in", D = "36 in", C = "192 in", crossed = "0", b = "8 in", t = "0.13 in", "γ" = "0.042 lbf/in³", f = "0.8", F_a = "100 lbf/in", d_min = "4.3 in", C_p = "0.94", C_v = "1", K_s = "1.15", n_d = "1.05" }
expect = { "F_(1,a)" = "752 lbf", F_2 = "88.310008 lbf", "f′" = "1.0384574" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: Ex. 17-1 exactly"
source = "By hand (open): α = asin(12/192) = 0.062540762, φ = 3.0165111, θD = 3.2666742, L = √(4·96² − 12²) + (18·3.2666742 + 6·3.0165111)/2 = 230.07423 in, exp(0.8φ) = 11.169746, V = π·6·1750/12 = 2748.8936 ft/min, Fc = 25.646776 lbf, T = 742.79886 lbf·in, F2 = 172.40038 lbf, Fi = 270.55341 lbf, f′ = 0.3276938, bmin = 247.59962·11.169746/((70 − 25.646776/6)·10.169746) = 4.1376049 in, Hmax = 29.908607 hp, dip = 0.13949009 in, shaft load √(420² + 172.40² + 2·420·172.40·cos 2α) = 591.4447 lbf"
inputs = { H_nom = "15 hp", n = "1750 rpm", d = "6 in", D = "18 in", C = "96 in", crossed = "0", b = "6 in", t = "0.13 in", "γ" = "0.042 lbf/in³", f = "0.8", F_a = "100 lbf/in", d_min = "4.3 in", C_p = "0.70", C_v = "1", K_s = "1.25", n_d = "1.1" }
expect = { "φ" = "3.0165111", "θ_D" = "3.2666742", L = "230.07423 in", r_t = "11.169746", V = "2748.8936 ft/min", F_c = "25.646776 lbf", T = "742.79886 lbf·in", F_2 = "172.40038 lbf", F_i = "270.55341 lbf", "f′" = "0.3276938", b_min = "4.1376049 in", H_max = "29.908607 hp", y_dip = "0.13949009 in", F_shaft = "591.4447 lbf" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 17-1 as a crossed drive"
source = "By hand (crossed, Eqs. 17-3, 17-4): α = asin(24/192) = 0.12532783, φ = θD = 3.3922483, L = √(4·96² − 24²) + 24·3.3922483/2 = 231.20107 in, exp(0.8φ) = 15.086475, f′ = ln(394.35322/146.75360)/3.3922483 = 0.2913973, bmin = 4.0346073 in, shaft load 588.56835 lbf"
inputs = { H_nom = "15 hp", n = "1750 rpm", d = "6 in", D = "18 in", C = "96 in", crossed = "1", b = "6 in", t = "0.13 in", "γ" = "0.042 lbf/in³", f = "0.8", F_a = "100 lbf/in", d_min = "4.3 in", C_p = "0.70", C_v = "1", K_s = "1.25", n_d = "1.1" }
expect = { "φ" = "3.3922483", "θ_D" = "3.3922483", L = "231.20107 in", r_t = "15.086475", "f′" = "0.2913973", b_min = "4.0346073 in", F_shaft = "588.56835 lbf" }
tolerance = "1e-6"
checks = "pass"
+++

## Flat belt drive at its tension limit

A flat belt on two pulleys, the small one driving, analysed by Shigley's procedure (Sec. 17-2). The belt is run at its largest allowable tension, set by the maker's rating per width; the torque fixes the difference of the tight- and slack-side tensions, and with the centrifugal tension that gives the slack-side and initial tensions. The friction the drive then develops, f′, from the belting (Euler-Eytelwein) equation, must stay under the belt's coefficient f, or the belt slips. Set the initial tension F_i at installation and keep it: a belt without initial tension carries no torque.

Shigley and Juvinall use the same belting equation, (F_1 − F_c)/(F_2 − F_c) = exp(fφ). Juvinall's initial tension (Eq. 19.1) is the plain mean of the two tensions and leaves out F_c; this template uses Shigley's.

### Drive

```calc {description="Nominal power transmitted"}
H_nom = 15 hp_
```

```calc {description="Speed of the small (driving) pulley"}
n = 1750 rpm_
```

```calc {description="Small pulley diameter"}
d = 6 in_
```

```calc {description="Large pulley diameter"}
D = 18 in_
```

```calc {description="Centre distance between the pulley axes"}
C = 96 in_
```

```calc {description="Drive: 0 open, 1 crossed (reversing)"}
crossed = 0_
```

```calc {description="Service factor, Shigley Table 17-15"}
K_s = 1.25_
```

```calc {description="Design factor for exigencies"}
n_d = 1.1_
```

### Belt

The belt's thickness, specific weight, coefficient of friction on a metal pulley, allowable tension per width at 600 ft/min and smallest pulley are in Shigley Table 17-2 (leather and polyamide belts; for urethane belts the table's allowable is the whole belt's, so enter it divided by the width). The pulley correction factor C_p is from Table 17-4, by the small pulley's diameter; the velocity correction C_v is 1 for polyamide and urethane, and from Fig. 17-9 for leather.

```calc {description="Belt width"}
b = 6 in_
```

```calc {description="Belt thickness, Shigley Table 17-2"}
t = 0.13 in_
```

```calc {description="Specific weight of the belt, Shigley Table 17-2"}
γ = 0.042 lbf/in³_
```

```calc {description="Coefficient of friction on the pulley, Shigley Table 17-2"}
f = 0.8_
```

```calc {description="Allowable tension per width at 600 ft/min, Shigley Table 17-2"}
F_a = 100 lbf/in_
```

```calc {description="Smallest pulley for the belt, Shigley Table 17-2"}
d_min = 4.3 in_
```

```calc {description="Pulley correction factor, Shigley Table 17-4"}
C_p = 0.70_
```

```calc {description="Velocity correction factor, Shigley Fig. 17-9 (1 for polyamide and urethane)"}
C_v = 1_
```

### Geometry

The small pulley's wrap φ limits the drive. An open drive wraps π less twice the strand angle α on the small pulley and π plus twice α on the large one (Eqs. 17-1, 17-2); a crossed drive wraps π plus twice α on both (Eqs. 17-3, 17-4).

```calc
> Angle of each strand to the line of centres
α = {asin((D - d) / (2 C)) if crossed == 0; asin((D + d) / (2 C)) if crossed == 1} = ??
> Wrap on the small pulley, Eq. 17-1 or 17-3
φ = {π - 2 α if crossed == 0; π + 2 α if crossed == 1} = ??
> Wrap on the large pulley
θ_D = π + 2 α = ??
> Belt length, Eq. 17-2 or 17-4
L = {sqrt(4 C^2 - (D - d)^2) + (D θ_D + d φ) / 2 if crossed == 0; sqrt(4 C^2 - (D + d)^2) + (D + d) φ / 2 if crossed == 1} = ?? in
> Tension ratio at full friction, exp(fφ)
r_t = exp(f φ) = ??
```

### Tensions

The speed n is an angular speed, so the belt speed π d n (n in revolutions) is n d / 2 here. The belt's weight per length is turned into a mass with standard gravity, 9.80665 m/s².

```calc
> Belt speed
V = n d / 2 = ?? ft/min
> Weight per length of belt
w = γ b t = ?? lbf/ft
> Centrifugal tension, Eq. (e)
F_c = w / (9.80665 'm/s²') V^2 = ?? lbf
> Design power
H_d = H_nom K_s n_d = ?? hp
> Torque on the small pulley
T = H_d / n = ?? lbf·in
> Tension difference, Eq. (h)
ΔF = 2 T / d = ?? lbf
> Largest allowable tension, Eq. 17-12
F_(1,a) = b F_a C_p C_v = ?? lbf
> Slack-side tension
F_2 = F_(1,a) - ΔF = ?? lbf
> Initial tension, Eq. (i)
F_i = (F_(1,a) + F_2) / 2 - F_c = ?? lbf
```

### Friction, safety and width

The friction developed f′ is what the belting equation needs at these tensions (Eq. 17-7). The factor of safety is the design power the drive carries over the nominal power times the service factor (p. 880, step 9): it is the design factor when the belt does not slip. b_min is the width at which the friction is just fully developed at the tension limit (Prob. 17-5); a narrower belt slips. H_max is the most the belt carries at its tension limit with friction fully developed and the initial tension raised to suit (Eq. 17-8).

```calc
> Friction developed, Eq. 17-7
f′ = ln((F_(1,a) - F_c) / (F_2 - F_c)) / φ = ??
> Factor of safety, p. 880 step 9
n_fs = H_d / (H_nom K_s) = ??
> Width at which friction is just fully developed, Prob. 17-5
b_min = ΔF r_t / ((F_a C_p C_v - F_c / b) (r_t - 1)) = ?? in
> Largest power at the tension limit, Eq. 17-8
H_max = (F_(1,a) - F_c) (r_t - 1) / r_t V = ?? hp
```

### Dip and shaft load

The catenary dip of a horizontal drive at rest measures the initial tension (Eq. 17-13). The shaft load is the resultant of the two strand tensions, which meet at twice the strand angle α (Fig. 17-7).

```calc
> Catenary dip at rest, Eq. 17-13
y_dip = w C^2 / (8 F_i) = ?? in
> Belt pull on each shaft
F_shaft = sqrt(F_(1,a)^2 + F_2^2 + 2 F_(1,a) F_2 cos(2 α)) = ?? lbf
```

### Checks

The belt must not slip at its tension limit, and the small pulley must be at least the belt's smallest.

```calc
@test f′ < f
```

```calc
@test d ≥ d_min
```
