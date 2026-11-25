+++
cruncha-template = 1
id = "mechanical.boundaryBearings.temperature"
version = "1.0.0"
title = "Boundary-lubricated bushing: film temperature and least length"
summary = "The heat balance of a boundary-lubricated bushing in a pillow block: friction heat fs F V, the film temperature at which the housing loses it to the air, and Shigley's least bushing length that keeps the film under an allowed temperature with a design factor on the load."
category = ["Mechanical engineering", "Bearings"]
tags = ["bushing", "plain bearing", "boundary lubrication", "self-lubricating", "film temperature", "heat balance", "friction heat", "pillow block", "bushing length", "Oiles", "Shigley"]
from = "lacewing:mechanical.boundaryBearings"
license = "GPL-3.0-or-later"
inputs = ["F", "D", "L", "N", "n_d", "f_s", "h_CR", "T_a", "T_lim"]
outputs = ["V", "H_gen", "A", "T_f", "L_min"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 12-15: Eqs. 12-34 to 12-38 (friction heat, housing loss, film temperature, pillow-block area, least length); Eq. 12-18 (h̄CR); Table 12-9 (friction coefficients); Example 12-8"

[[tests]]
name = "Shigley Ex. 12-8: Oiles 500, ¾ in bore, 50 lbf at 300 rpm, nd 2, fs 0.03, 2.7 Btu/(h·ft²·°F), 70 °F air, 300 °F allowed"
source = "Lacewing Lab bushings_test, after Shigley 10th Example 12-8: L ≥ 1.34 in; the 1½ in bushing chosen"
inputs = { F = "50 lbf", D = "0.75 in", L = "1.5 in", N = "300 rpm", n_d = "2", f_s = "0.03", h_CR = "2.7 Btu/(h·ft²·°F)", T_a = "70 °F", T_lim = "300 °F" }
expect = { L_min = "1.34 in" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 12-8's bushing"
source = "By hand: V = π × 0.75 in × 5 rev/s = 0.2992367 m/s; Hgen = 0.03 × 222.41108 N × 0.2992367 m/s = 1.9966067 W; A = 2π × 0.75 × 1.5 = 7.0685835 in²; Tf = 70 °F + 2 × 1.9966067/(15.331311 W/(m²·K) × 0.0045603673 m²) = 172.80540 °F; Lmin = 0.03 × 2 × 222.41108 × 5/(15.331311 × 127.77778 K) = 1.3409400 in"
inputs = { F = "50 lbf", D = "0.75 in", L = "1.5 in", N = "300 rpm", n_d = "2", f_s = "0.03", h_CR = "2.7 Btu/(h·ft²·°F)", T_a = "70 °F", T_lim = "300 °F" }
expect = { V = "0.2992367 m/s", H_gen = "1.99660674 W", A = "7.06858347 in²", T_f = "172.805397 °F", L_min = "1.34093996 in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 12-8 with a 1 in bushing: shorter than the least length"
source = "By hand: A = 2π × 0.75 × 1.0 = 4.712389 in², Tf = 224.20810 °F (under 300 °F at the running load), but L = 1 in is under Lmin = 1.34094 in for the design factor of 2"
inputs = { F = "50 lbf", D = "0.75 in", L = "1.0 in", N = "300 rpm", n_d = "2", f_s = "0.03", h_CR = "2.7 Btu/(h·ft²·°F)", T_a = "70 °F", T_lim = "300 °F" }
expect = { T_f = "224.208096 °F", L_min = "1.34093996 in" }
tolerance = "1e-6"
checks = "fail"
+++

## Boundary-lubricated bushing: film temperature and least length

Friction heat in a bushing without a full oil film goes to the housing and from there to the air. After Shigley Sec. 12-15, the housing surface is taken midway between the film and the air, so the loss is h̄CR A (T_f − T_a)/2; for a pillow block the area is about 2πDL. Setting the loss equal to the friction heat gives the film temperature, and, with the area 2πDL, the least length whatever the diameter. Pressure, PV and wear are the boundary bushing wear template.

### Bushing and load

```calc {description="Radial (running) load"}
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

```calc {description="Design factor on the load for the least length (Ex. 12-8)"}
n_d = 2_
```

### Heat

The friction coefficient is the material's (Shigley Table 12-9 gives 0.03 to 0.05 for Oiles' materials). h̄CR combines radiation and convection from the housing: Shigley's Eq. 12-18 gives 2 Btu/(h·ft²·°F) in still air, 2.7 shaft-stirred and 5.9 in air at 500 ft/min. The allowed film temperature is the material's limit (Shigley Table 12-7 or 12-12) or lower.

```calc {description="Coefficient of friction, Shigley Table 12-9"}
f_s = 0.03_
```

```calc {description="Combined heat transfer coefficient of the housing, Shigley Eq. 12-18"}
h_CR = 2.7 Btu/(h·ft²·°F)_
```

```calc {description="Ambient air temperature"}
T_a = 70 °F_
```

```calc {description="Allowed film temperature"}
T_lim = 300 °F_
```

```calc
> Speed in revolutions per second
n = N / (1 'rev') = ?? 1/s
> Sliding velocity
V = π D n = ?? ft/min
> Friction heat at the running load, Eq. 12-34
H_gen = f_s F V = ?? W
> Housing area of a pillow block, Eq. 12-37 (replace with the housing's own if known)
A = 2 π D L = ?? in²
> Film temperature, Eq. 12-36
T_f = T_a + 2 H_gen / (h_CR A) = ?? °F
> Least length for the allowed temperature, Eq. 12-38
L_min = f_s n_d F n / (h_CR (T_lim - T_a)) = ?? in
```

### Checks

```calc
@test T_f ≤ T_lim
```

```calc
@test L ≥ L_min
```

```calc
@test 0.5 ≤ L / D ≤ 2
```
