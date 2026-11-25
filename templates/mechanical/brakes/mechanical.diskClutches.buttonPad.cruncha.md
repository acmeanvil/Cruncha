+++
cruncha-template = 1
id = "mechanical.diskClutches.buttonPad"
version = "1.0.0"
title = "Caliper disk brake, circular (button) pads (Shigley)"
summary = "A caliper disk brake with circular button or puck pads: the equivalent radius from Fazekas's ratio δ, the average pad pressure, the actuating force and the torque of each pad and of the brake."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "disk brake", "caliper brake", "button pad", "puck pad", "circular pad", "Fazekas", "Shigley"]
from = "lacewing:mechanical.diskClutches"
license = "GPL-3.0-or-later"
inputs = ["R", "e_p", "f", "p_max", "n_p", "δ", "k_p"]
outputs = ["R_e", "r_e", "p_av", "F", "T_pad", "T"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-6: Fig. 16-20, Eqs. 16-41 to 16-43; Table 16-1 (after Fazekas, Trans. ASME J. Eng. Ind. 94(3), 1972); Example 16-4"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.3; Prob. 18.14"

[[tests]]
name = "Shigley Ex. 16-4: one button pad, R 0.5 in at 2 in, 175 psi"
source = "Lacewing Lab brakes_test, after Shigley 10th Example 16-4: R/e = 0.25, δ = 0.963 and pmax/pav = 1.290 from Table 16-1, f 0.31: F 106.6 lbf, T 63.65 lbf·in"
inputs = { R = "0.5 in", e_p = "2 in", f = "0.31", p_max = "175 psi", n_p = "1", "δ" = "0.963", k_p = "1.290" }
expect = { R_e = "0.25", p_av = "135.7 psi", F = "106.6 lbf", T = "63.65 lbf·in" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Juvinall Prob. 18.14: four 60 mm pads at 125 mm, 500 kPa average"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Prob. 18.14 (answer, 1414 N a pad): R/e = 0.24, Table 16-1 read linearly δ = 0.9642, pmax/pav = 1.274, so pmax = 637 kPa for pav 500 kPa, f 0.30; T = 4 × 0.3 × 1414 × 0.9642 × 0.125 = 204.5 N·m (212 N·m with re = e)"
inputs = { R = "30 mm", e_p = "125 mm", f = "0.30", p_max = "637 kPa", n_p = "4", "δ" = "0.9642", k_p = "1.274" }
expect = { R_e = "0.24", p_av = "500 kPa", F = "1414 N", T = "204.5 N·m" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Pad too close to the axis for Table 16-1"
source = "By hand: R = 1.2 in at e = 2 in gives R/e = 0.6, beyond Table 16-1's last row (0.5)"
inputs = { R = "1.2 in", e_p = "2 in", f = "0.31", p_max = "175 psi", n_p = "1", "δ" = "0.938", k_p = "1.875" }
expect = { R_e = "0.6" }
tolerance = "1e-9"
checks = "fail"
+++

## Caliper disk brake, circular pads

A circular pad of radius R whose centre is e_p from the disk's axis (Shigley Fig. 16-20), pressed with the largest pressure p_max. The pressure is not uniform over a round pad: Fazekas's numerical solution, tabulated in Shigley Table 16-1 against R/e, gives the ratio δ of the equivalent friction radius to e and the ratio of the largest to the average pressure. The table runs to R/e = 0.5; read δ and p_max/p_av from it, linearly between rows, at the R_e computed below.

### Inputs

```calc {description="Radius of a pad"}
R = 0.5 in_
```

```calc {description="Distance from the disk axis to the pad's centre"}
e_p = 2 in_
```

```calc {description="Dynamic friction coefficient of the lining, Shigley Table 16-3"}
f = 0.31_
```

```calc {description="Largest pad pressure (the lining's allowable)"}
p_max = 175 psi_
```

```calc {description="Number of pads (sides × calipers)"}
n_p = 1_
```

```calc {description="Ratio δ = r_e/e at R/e, Shigley Table 16-1 (Fazekas)"}
δ = 0.963_
```

```calc {description="Ratio of largest to average pressure p_max/p_av at R/e, Shigley Table 16-1 (Fazekas)"}
k_p = 1.290_
```

### Force and torque

```calc
> Pad radius over its eccentricity, the argument of Table 16-1
R_e = R / e_p = ??
> Equivalent friction radius, Eq. 16-41
r_e = δ e_p = ?? in
> Average pad pressure
p_av = p_max / k_p = ?? psi
> Actuating force on each pad, Eq. 16-42
F = π R^2 p_av = ?? lbf
> Torque of each pad, Eq. 16-43
T_pad = f F r_e = ?? lbf·in
> Brake torque, all pads
T = n_p T_pad = ?? lbf·in
```

### Checks

```calc
@test R_e ≤ 0.5
```
