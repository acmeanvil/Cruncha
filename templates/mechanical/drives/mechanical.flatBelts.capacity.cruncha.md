+++
cruncha-template = 1
id = "mechanical.flatBelts.capacity"
version = "1.0.0"
title = "Belt torque capacity from one known tension (Juvinall)"
summary = "The torque and power a belt on one pulley carries at the point of slip when the tension of one side is known: the Euler-Eytelwein (capstan) ratio exp(fφ) with the centrifugal tension added to both sides, the other side's tension, the initial tension and the torque. A V-belt takes the wedged coefficient f/sin β."
category = ["Mechanical engineering", "Belts and chains"]
tags = ["belt", "flat belt", "V-belt", "capstan", "Euler-Eytelwein", "tension ratio", "torque capacity", "slip", "wrap angle", "centrifugal tension", "Juvinall"]
from = "lacewing:mechanical.flatBelts"
license = "GPL-3.0-or-later"
inputs = ["known", "P", "f_e", "φ", "d", "P_c", "n"]
outputs = ["r_t", "P_1", "P_2", "P_i", "T", "H"]

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 18.9, Eqs. 18.24 to 18.26 (band and capstan); Sec. 19.2, Eqs. 19.1 to 19.3; Sec. 19.3, Eq. 19.3a (V-belt, f/sin β)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 17-2, Eq. 17-7 (the belting equation)"

[[tests]]
name = "Hand check: slack side 40 N known, f = 0.33, 150° wrap, 100 mm pulley"
source = "Lacewing Lab flat_belts_test (Juvinall Eq. 19.3 case); by hand: exp(0.33 × 150π/180) = 2.3724851, P1 = 94.899405 N, T = (P1 − P2) × 0.05 = 2.7449702 N·m, Pi = 67.449702 N; at 1000 rev/min H = 287.45261 W"
inputs = { known = "1", P = "40 N", f_e = "0.33", "φ" = "150 deg", d = "100 mm", P_c = "0 N", n = "1000 rpm" }
expect = { r_t = "2.3724851", P_1 = "94.899405 N", P_2 = "40 N", T = "2.7449702 N·m", P_i = "67.449702 N", H = "287.45261 W" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: tight side 1000 N known, f = 0.5, 3 rad wrap, 100 N centrifugal tension"
source = "Lacewing Lab flat_belts_test (Juvinall Eq. 19.3 with Pc); by hand: exp(1.5) = 4.4816891, P2 = 100 + 900/4.4816891 = 300.81714 N, T = (1000 − 300.81714) × 0.1 = 69.918286 N·m"
inputs = { known = "2", P = "1000 N", f_e = "0.5", "φ" = "3 rad", d = "200 mm", P_c = "100 N", n = "1000 rpm" }
expect = { r_t = "4.4816891", P_1 = "1000 N", P_2 = "300.81714 N", T = "69.918286 N·m", P_i = "650.40857 N" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 19.1: 5V belt limited to 150 lb, f = 0.2, β = 18°, 165° wrap, 3.7 in sheave at 1750 rev/min"
source = "Lacewing Lab v_belts_test, after Juvinall 5th Sample Problem 19.1 (pp. 787-788): f/sin β = 0.2/sin 18° = 0.64721, exp(f′φ) = 6.45, Pc = 3.57 lb (w = 0.012 lb/in at 339 in/s), P2 = 26.3 lb, T = 229 lb·in, 6.36 hp per belt"
inputs = { known = "2", P = "150 lbf", f_e = "0.6472136", "φ" = "165 deg", d = "3.7 in", P_c = "3.57 lbf", n = "1750 rpm" }
expect = { r_t = "6.45", P_2 = "26.3 lbf", T = "229 lbf·in", H = "6.36 hp" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "Tight side under the centrifugal tension: no capacity"
source = "By hand: P1 = 50 N with Pc = 60 N, exp(1.5) = 4.4816891: P2 = 60 − 10/4.4816891 = 57.768657 N, below Pc"
inputs = { known = "2", P = "50 N", f_e = "0.5", "φ" = "3 rad", d = "200 mm", P_c = "60 N" }
expect = { P_2 = "57.768657 N" }
tolerance = "1e-6"
checks = "fail"
+++

## Belt torque capacity from one known tension

The most torque a belt can carry on one pulley before it slips, when the tension in one side is known: the slack side (from a weighted or spring tensioner, say) or the tight side (the belt's largest allowable tension). At the point of slip the two tensions less the centrifugal tension are in the ratio exp(fφ), the capstan equation of a band on a drum (Juvinall Eqs. 18.26 and 19.3; Shigley's Eq. 17-7 is the same). The torque is the difference of the tensions times the pulley radius.

For a flat belt f_e is the belt's coefficient of friction on the pulley. For a V-belt it is the wedged coefficient f/sin β, with β half the groove angle (Juvinall Eq. 19.3a: about 3.2 f at β = 18°). Use the wrap on the pulley that limits the drive, usually the smaller one.

### Inputs

```calc {description="Which side's tension is known: 1 slack side P_2, 2 tight side P_1"}
known = 1_
```

```calc {description="The known tension"}
P = 40 N_
```

```calc {description="Effective coefficient of friction: f for a flat belt, f/sin β for a V-belt"}
f_e = 0.33_
```

```calc {description="Angle of wrap on the pulley"}
φ = 150 deg_
```

```calc {description="Pulley diameter"}
d = 100 mm_
```

```calc {description="Centrifugal tension m′V², Juvinall Eq. 19.2 (0 at low speed)"}
P_c = 0 N_
```

```calc {description="Pulley speed, to turn the torque into a power"}
n = 1000 rpm_
```

### Capacity

```calc
> Tension ratio at slip, Juvinall Eq. 18.26
r_t = exp(f_e φ) = ??
> Tight-side tension, Eq. 19.3
P_1 = {P_c + (P - P_c) r_t if known == 1; P if known == 2} = ?? N
> Slack-side tension, Eq. 19.3
P_2 = {P if known == 1; P_c + (P - P_c) / r_t if known == 2} = ?? N
> Initial tension, Juvinall Eq. 19.1 (leaves out the centrifugal tension)
P_i = (P_1 + P_2) / 2 = ?? N
> Torque capacity at slip, Eq. 18.24
T = (P_1 - P_2) d / 2 = ?? N·m
> Power capacity at the pulley speed
H = T n = ?? W
```

### Check

The slack side must stay in tension above the centrifugal tension, or the belt cannot drive at all.

```calc
@test P_2 > P_c
```
