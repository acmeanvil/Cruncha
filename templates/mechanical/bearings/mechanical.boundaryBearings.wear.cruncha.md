+++
cruncha-template = 1
id = "mechanical.boundaryBearings.wear"
version = "1.0.0"
title = "Boundary-lubricated bushing: PV, wear and life"
summary = "A dry, greased, oil-impregnated or self-lubricating plain bushing: nominal and peak pressure, sliding velocity and PV against the material's limits, the radial wear over a running time by w = f1 f2 K Pmax V t, the life to a wear allowance and the journal revolutions in it, and Shigley's proportions 0.5 ≤ L/D ≤ 2."
category = ["Mechanical engineering", "Bearings"]
tags = ["bushing", "plain bearing", "boundary lubrication", "dry bearing", "self-lubricating", "porous bronze", "oil-impregnated", "sintered", "PV", "PV limit", "wear", "wear factor", "bearing life", "Oiles", "Shigley"]
from = "lacewing:mechanical.boundaryBearings"
license = "GPL-3.0-or-later"
inputs = ["F", "D", "L", "N", "n_d", "P_lim", "basis", "V_lim", "PV_lim", "K", "f_1", "f_2", "t", "w_a"]
outputs = ["P", "P_max", "V", "PV", "w", "t_w", "N_w"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 12-15, Boundary-lubricated bearings: Eqs. 12-27 to 12-33; Table 12-7 (material limits), Table 12-8 (wear factors), Table 12-10 (motion factor f1), Table 12-11 (environment factor f2), Table 12-12 (Oiles 500 SP); Examples 12-7 and 12-8"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 13.14: Table 13.4 (porous metal bearings), Table 13.5 (nonmetallic bearings); the halved PV for long life without relubrication"

[[tests]]
name = "Shigley Ex. 12-7: Oiles 500 SP, 1 × 1 in, 700 lbf, 33 ft/min, wear allowance 0.005 in"
source = "Lacewing Lab bushings_test, after Shigley 10th Example 12-7: Pmax 891 psi, PV 23 100 psi·ft/min, f1 1.3 (Table 12-10 at 33 ft/min), life 2180 h, 126 rpm, 16.5·10⁶ revolutions; within Oiles 500 SP's limits (Table 12-12). The required running time, 2000 h here, is not part of the example"
inputs = { F = "700 lbf", D = "1 in", L = "1 in", N = "126.0507 rpm", n_d = "1", P_lim = "3560 psi", basis = "2", V_lim = "100 ft/min", PV_lim = "46700 psi·ft/min", K = "0.6e-10 in³·min/(lbf·ft·h)", f_1 = "1.3", f_2 = "1", t = "2000 h", w_a = "0.005 in" }
expect = { P_max = "891 psi", P = "700 psi", PV = "23100 psi·ft/min", t_w = "2180 h", N_w = "16.5e6" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 12-8: Oiles 500, ¾ × 1½ in, 50 lbf at 300 rpm, design factor 2, 800 h"
source = "Lacewing Lab bushings_test, after Shigley 10th Example 12-8: P 88.9 psi, Pmax 113 psi, V 58.9 ft/min, PV 5240 psi·ft/min, f1 1.50 (Table 12-10 interpolated in V), wear 0.000480 in in 800 h. The wear allowance, 0.001 in here, is not part of the example"
inputs = { F = "50 lbf", D = "0.75 in", L = "1.5 in", N = "300 rpm", n_d = "2", P_lim = "3560 psi", basis = "2", V_lim = "100 ft/min", PV_lim = "46700 psi·ft/min", K = "0.6e-10 in³·min/(lbf·ft·h)", f_1 = "1.50", f_2 = "1", t = "800 h", w_a = "0.001 in" }
expect = { P = "88.9 psi", P_max = "113 psi", V = "58.9 ft/min", PV = "5240 psi·ft/min", w = "0.000480 in" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 12-8 exactly"
source = "By hand: nd·F = 100 lbf, P = 100/(0.75 × 1.5) = 88.888889 psi, Pmax = 4 × 100/(π × 1.125) = 113.17685 psi, V = π × 0.75 × 300/12 = 58.904862 ft/min, PV = 5235.9878 psi·ft/min, w = 1.5 × 0.6e-10 × 113.17685 × 58.904862 × 800 = 0.00048 in, life to 0.001 in = 1666.6667 h, 3·10⁷ revolutions"
inputs = { F = "50 lbf", D = "0.75 in", L = "1.5 in", N = "300 rpm", n_d = "2", P_lim = "3560 psi", basis = "2", V_lim = "100 ft/min", PV_lim = "46700 psi·ft/min", K = "0.6e-10 in³·min/(lbf·ft·h)", f_1 = "1.50", f_2 = "1", t = "800 h", w_a = "0.001 in" }
expect = { P = "88.8888889 psi", P_max = "113.176848 psi", V = "58.9048623 ft/min", PV = "5235.98776 psi·ft/min", w = "0.00048 in", t_w = "1666.66667 h", N_w = "3e7" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 12-7's bushing worn for 3000 h: past its allowance"
source = "By hand: the life to 0.005 in is 2179.48 h (Ex. 12-7), so 3000 h wears w = 0.005 × 3000/2179.48 = 0.0068824 in"
inputs = { F = "700 lbf", D = "1 in", L = "1 in", N = "126.0507149 rpm", n_d = "1", P_lim = "3560 psi", basis = "2", V_lim = "100 ft/min", PV_lim = "46700 psi·ft/min", K = "0.6e-10 in³·min/(lbf·ft·h)", f_1 = "1.3", f_2 = "1", t = "3000 h", w_a = "0.005 in" }
expect = { w = "0.00688236904 in", t_w = "2179.48208 h" }
tolerance = "1e-6"
checks = "fail"
+++

## Boundary-lubricated bushing: PV, wear and life

A plain bushing that runs without a full oil film: dry, greased, oil-impregnated (porous metal) or self-lubricating (plastics, PTFE composites, carbon). Its ratings are the nominal pressure P, the sliding velocity V and their product PV, the frictional heat per unit area for a given coefficient of friction. After Shigley Sec. 12-15, its wear is linear in the peak pressure, the velocity and the time, with a wear factor K for the material and factors for the motion and the surroundings. The film temperature is the boundary bushing temperature template. For a full hydrodynamic film, use the journal bearing templates.

### Bushing and load

The design factor multiplies the load in every result here, as Shigley's Ex. 12-8 does.

```calc {description="Radial load"}
F = 50 lbf_
```

```calc {description="Bore (journal diameter)"}
D = 0.75 in_
```

```calc {description="Bushing length"}
L = 1.5 in_
```

```calc {description="Journal speed"}
N = 300 rpm_
```

```calc {description="Design factor on the load"}
n_d = 2_
```

```calc
> Speed in revolutions per second
n = N / (1 'rev') = ?? 1/s
> Nominal (projected-area) pressure, Eq. 12-28
P = n_d F / (D L) = ?? psi
> Peak pressure of the cosine distribution round the loaded half, Eq. 12-31
P_max = 4 n_d F / (π D L) = ?? psi
> Sliding velocity, Eq. 12-29
V = π D n = ?? ft/min
> PV, Eq. 12-30
PV = P V = ?? psi·ft/min
```

### Material limits

The limits of load, speed and PV for the bushing material: Shigley Table 12-7 (twelve common materials), Juvinall Tables 13.4 (porous metals) and 13.5 (nonmetallic), or the maker's data sheet. The defaults are Oiles 500 SP's service range, Shigley Table 12-12. Most tables limit the nominal pressure; Oiles' limit is on the peak pressure, as Ex. 12-7 checks it. For a porous metal bushing that must last without being relubricated, Juvinall halves the PV limit: enter half the table's value.

```calc {description="Maximum load pressure of the material (Table 12-7, Juvinall 13.4/13.5 or the maker)"}
P_lim = 3560 psi_
```

```calc {description="What the load limit applies to: 1 the nominal pressure P, 2 the peak pressure Pmax"}
basis = 2_
```

```calc {description="Maximum sliding velocity of the material"}
V_lim = 100 ft/min_
```

```calc {description="PV limit of the material (halved for porous metal running long without relubrication)"}
PV_lim = 46700 psi·ft/min_
```

```calc {description="The pressure the load limit is compared with"}
P_c = {P if basis == 1; P_max if basis == 2} = ?? psi
```

### Wear

The wear factor K is a property of the material (Shigley Table 12-8 gives one maker's, Oiles America's, with their limiting PV). The motion factor f₁ is Shigley Table 12-10's: rotary, oscillating or reciprocating, by pressure band, interpolated in V within its velocity band as Ex. 12-8 does. The environment factor f₂ is Table 12-11's, from 1 for clean surroundings under 140 °F up to 12 for dirty and warm.

```calc {description="Wear factor K of the material, Shigley Table 12-8"}
K = 0.6e-10 in³·min/(lbf·ft·h)_
```

```calc {description="Motion factor f1, Shigley Table 12-10"}
f_1 = 1.50_
```

```calc {description="Environment factor f2, Shigley Table 12-11"}
f_2 = 1_
```

```calc {description="Running time the bushing must last"}
t = 800 h_
```

```calc {description="Radial wear the bushing can take and still work"}
w_a = 0.001 in_
```

```calc
> Radial wear in the running time, Eq. 12-32
w = f_1 f_2 K P_max V t = ?? in
> Running time to the wear allowance, Ex. 12-7
t_w = w_a / (f_1 f_2 K P_max V) = ?? h
> Journal revolutions in that time
N_w = n t_w = ??
```

### Checks

```calc
@test P_c ≤ P_lim
```

```calc
@test V ≤ V_lim
```

```calc
@test PV ≤ PV_lim
```

```calc
@test w ≤ w_a
```

```calc
@test 0.5 ≤ L / D ≤ 2
```
