+++
cruncha-template = 1
id = "mechanical.diskClutches.plate"
version = "1.0.0"
title = "Plate (disk) clutch or brake, uniform wear or uniform pressure"
summary = "A plate clutch or brake with N friction interfaces: friction radius, clamping force and torque at the allowable pressure by uniform wear or uniform pressure, the interfaces a required torque needs, the clamping force that just carries it, and the torque, pressures and safety factor against slipping of a given clamping force."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["clutch", "brake", "disk clutch", "plate clutch", "multiple-disk clutch", "uniform wear", "uniform pressure", "friction radius", "clamping force", "torque capacity", "Juvinall", "Shigley"]
from = "lacewing:mechanical.diskClutches"
license = "GPL-3.0-or-later"
inputs = ["model", "r_o", "r_i", "f", "p_a", "T_req", "N", "F"]
outputs = ["r_e", "F_max", "T_1", "N_req", "N_even", "T_cap", "n", "F_T", "T_F", "p_F", "p_(F,o)", "n_F", "r_(i,opt)"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.2: Eqs. 18.1 to 18.3 (uniform pressure), 18.4 to 18.7 (uniform wear), 18.8 (optimum inner radius); Tables 18.1 and 18.2 (friction materials); Sample Problem 18.1; Probs. 18.6, 18.9, 18.10"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-5: Eqs. 16-23 to 16-28 (the same equations in diameters); Table 16-3 (friction materials)"

[[tests]]
name = "Juvinall Sample Problem 18.1: 85 N·m wet clutch, 100 mm OD, ri 29 mm"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Sample Problem 18.1: f 0.06, pmax 1400 kPa, 6.69 interfaces needed, 8 used (nine disks), clamping force lowered to 4483 N to carry exactly 85 N·m; ri optimum = 0.577 ro"
inputs = { model = "1", r_o = "50 mm", r_i = "29 mm", f = "0.06", p_a = "1400 kPa", T_req = "85 N·m", N = "8", F = "4484 N" }
expect = { N_req = "6.69", N_even = "8", F_T = "4483 N", n = "1.195", "r_(i,opt)" = "28.87 mm", T_F = "85 N·m" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Juvinall Prob. 18.6: 800 N·m on 90/150 mm disks"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Prob. 18.6 (answer): pa 1 MPa, f 0.10, uniform wear: 16 interfaces, 17 disks"
inputs = { model = "1", r_o = "75 mm", r_i = "45 mm", f = "0.10", p_a = "1 MPa", T_req = "800 N·m", N = "16", F = "8400 N" }
expect = { N_even = "16" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Juvinall Prob. 18.9: 6 hp at 200 rpm with a safety factor of 2, 5 and 7 in disks"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Prob. 18.9 (answer): f 0.20, pa 225 psi, uniform wear: two interfaces (three disks), F 3150 lbf, pressures 200 psi inner and 143 psi outer"
inputs = { model = "1", r_o = "3.5 in", r_i = "2.5 in", f = "0.20", p_a = "225 psi", T_req = "3781.6 lbf·in", N = "2", F = "3152 lbf" }
expect = { N_even = "2", F_T = "3151 lbf", p_F = "200.6 psi", "p_(F,o)" = "143.3 psi" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Juvinall Prob. 18.10: nine 900 N springs, 160/240 mm, new (uniform pressure)"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Prob. 18.10 (answer): f 0.40, two interfaces, 280 N·m: safety factor against slipping 2.34 new"
inputs = { model = "2", r_o = "120 mm", r_i = "80 mm", f = "0.40", p_a = "1 MPa", T_req = "280 N·m", N = "2", F = "8100 N" }
expect = { n_F = "2.345" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Juvinall Prob. 18.10, after initial wear (uniform wear)"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Prob. 18.10 (answer): 2.31 after initial wear"
inputs = { model = "1", r_o = "120 mm", r_i = "80 mm", f = "0.40", p_a = "1 MPa", T_req = "280 N·m", N = "2", F = "8100 N" }
expect = { n_F = "2.314" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: uniform pressure, Eqs. 18.1 to 18.3"
source = "By hand: ro 100, ri 60 mm, f 0.3, pa 1 MPa, one interface: re = 2(0.1³ − 0.06³)/(3(0.1² − 0.06²)) = 81.666667 mm, Fmax = π × 1e6 × (0.01 − 0.0036) = 20106.193 N, T = 20106.193 × 0.3 × 0.081666667 = 492.60172 N·m; at 30 kN the pressure is 1.4920775 MPa, over pa"
inputs = { model = "2", r_o = "100 mm", r_i = "60 mm", f = "0.3", p_a = "1 MPa", T_req = "400 N·m", N = "1", F = "30 kN" }
expect = { r_e = "81.666667 mm", F_max = "20106.193 N", T_cap = "492.60172 N·m", p_F = "1.4920775 MPa" }
tolerance = "1e-7"
checks = "fail"
+++

## Plate clutch or brake

A clutch or brake of flat annular friction faces between radii r_i and r_o, clamped by an axial force F, with N friction interfaces in parallel: one for a single face, two for a single-plate automotive clutch, six for a stack of seven disks (Juvinall Sec. 18.2, Shigley Sec. 16-5). The same clamping force passes through every interface, so each carries the same torque. The friction coefficient is taken as constant.

Two pressure distributions bound the clutch. A new, rigid, accurately made clutch has **uniform pressure**. Once worn in, wear proportional to the friction work makes p·r constant: **uniform wear**, with the largest pressure p_a at the inner radius. Uniform wear gives the smaller torque (about 2 % less at r_i/r_o = 0.6), so it is the usual design basis.

### Inputs

Enter the model as a number: 1 uniform wear, 2 uniform pressure. The friction coefficient and allowable pressure come from the lining material: Juvinall Table 18.1 (dry) and Table 18.2 (in oil), Shigley Table 16-3. Juvinall advises designing with 50 to 75 % of tested friction values, and lower pressures give longer life.

```calc {description="Pressure model: 1 uniform wear (design basis), 2 uniform pressure (new clutch)"}
model = 1_
```

```calc {description="Outer radius of the friction faces"}
r_o = 50 mm_
```

```calc {description="Inner radius of the friction faces"}
r_i = 29 mm_
```

```calc {description="Dynamic friction coefficient of the lining, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
f = 0.06_
```

```calc {description="Allowable lining pressure, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
p_a = 1400 kPa_
```

```calc {description="Torque to carry, with any service or safety factor"}
T_req = 85 N·m_
```

```calc {description="Number of friction interfaces in use (disks − 1)"}
N = 8_
```

```calc {description="Axial clamping force applied (springs or a cylinder)"}
F = 4484 N_
```

### Capacity at the allowable pressure

```calc
> Friction radius, so that each interface carries f F r_e, Eqs. 18.3 and 18.7
r_e = {(r_o + r_i) / 2 if model == 1; 2 (r_o^3 - r_i^3) / (3 (r_o^2 - r_i^2)) if model == 2} = ?? mm
> Clamping force at the allowable pressure, Eqs. 18.1 and 18.5
F_max = {2 π p_a r_i (r_o - r_i) if model == 1; π p_a (r_o^2 - r_i^2) if model == 2} = ?? N
> Torque per interface at p_a
T_1 = F_max f r_e = ?? N·m
> Torque capacity of N interfaces, Eqs. 18.2 and 18.6
T_cap = N T_1 = ?? N·m
> Margin of the capacity over the torque required
n = T_cap / T_req = ??
```

### Interfaces and clamping force for the torque required

Rounding the interfaces needed up to an even number puts the two end disks in the same set, so both are splined to the same shaft (Juvinall Fig. 18.3, Sample Problem 18.1). With more capacity than needed, the clamping force can be lowered to the one that just carries the torque.

```calc
> Interfaces needed at p_a
N_req = T_req / T_1 = ??
> Next even number of interfaces
N_even = max(2, 2 ceil(N_req / 2)) = ??
> Clamping force that just carries T_req on N interfaces, Eqs. 18.3 and 18.7
F_T = T_req / (f r_e N) = ?? N
```

### At the given clamping force

```calc
> Torque the clamping force carries
T_F = F f r_e N = ?? N·m
> Largest pressure it makes: at the inner radius by uniform wear, everywhere by uniform pressure
p_F = {F / (2 π r_i (r_o - r_i)) if model == 1; F / (π (r_o^2 - r_i^2)) if model == 2} = ?? kPa
> Pressure at the outer radius, Eq. 18.4
p_(F,o) = {p_F r_i / r_o if model == 1; p_F if model == 2} = ?? kPa
> Safety factor against slipping
n_F = T_F / T_req = ??
```

### Proportions

By uniform wear the torque at a given r_o and p_a is largest at r_i = r_o/√3 (Eq. 18.8). Juvinall gives r_i = 0.45 to 0.80 r_o as common proportions.

```calc {description="Inner radius for the most uniform-wear torque at this r_o, Eq. 18.8"}
r_(i,opt) = r_o / sqrt(3) = ?? mm
```

### Checks

```calc
@test T_cap ≥ T_req
```

```calc
@test p_F ≤ p_a
```

```calc
@test n_F ≥ 1
```
