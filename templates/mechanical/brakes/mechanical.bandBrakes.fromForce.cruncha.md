+++
cruncha-template = 1
id = "mechanical.bandBrakes.fromForce"
version = "1.0.0"
title = "Band brake, simple or differential, from the lever force"
summary = "A simple or differential band brake applied by a given lever force: the tension ratio e^(fφ), the slack- and tight-side tensions, the braking torque, the largest lining pressure against the allowable, and the arm at which a differential band self-locks."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "band brake", "differential band brake", "capstan", "belt friction", "self-locking", "lever", "Juvinall", "Shigley"]
from = "lacewing:mechanical.bandBrakes"
license = "GPL-3.0-or-later"
inputs = ["r", "w", "φ", "f", "a", "c", "s", "F", "p_a"]
outputs = ["k", "s_lock", "P_2", "P_1", "T", "p_max"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.9: Figs. 18.15 to 18.17, Eqs. 18.24 to 18.28; Tables 18.1 and 18.2; Prob. 18.33"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-4: Eqs. 16-19 to 16-22; Table 16-3"

[[tests]]
name = "Juvinall Prob. 18.33: 300 N on the slack end of a simple band"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Prob. 18.33 (answer): r 500, b 30 mm, f 0.45, 235° of wrap, the slack end pulled with 300 N (a = c): T 800 N·m, pmax 127 kPa"
inputs = { r = "500 mm", w = "30 mm", "φ" = "235°", f = "0.45", a = "1 m", c = "1 m", s = "0 mm", F = "300 N", p_a = "0.5 MPa" }
expect = { T = "800 N·m", p_max = "127 kPa" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 18.4 backwards: 427.17 N on the differential lever"
source = "Lacewing Lab brakes_test (from the force back to the same brake), after Juvinall & Marshek 5th Sample Problem 18.4: 427.17033 N gives P1 10 000 N and the 500 kPa it was designed for"
inputs = { r = "250 mm", w = "80 mm", "φ" = "240°", f = "0.20", a = "150 mm", c = "700 mm", s = "35 mm", F = "427.17033 N", p_a = "0.6 MPa" }
expect = { P_1 = "10000 N", P_2 = "4326.7949 N", T = "1418.3013 N·m", p_max = "0.5 MPa" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Prob. 18.33 band pulled with 600 N: over a 200 kPa lining"
source = "By hand: twice the force, twice the pressure: 253.30 kPa, over 200 kPa"
inputs = { r = "500 mm", w = "30 mm", "φ" = "235°", f = "0.45", a = "1 m", c = "1 m", s = "0 mm", F = "600 N", p_a = "200 kPa" }
expect = { p_max = "253.29761 kPa" }
tolerance = "1e-6"
checks = "fail"
+++

## Band brake from the lever force

A flexible band, usually with a woven lining, wrapped through the angle φ round a drum of radius r (Juvinall Sec. 18.9, Shigley Sec. 16-4). An element of band in equilibrium gives dP = f P dθ, so the tight-side tension P_1 and slack-side tension P_2 are in the ratio e^(fφ), and the band presses on the drum with p = P/(w r), largest at the tight end. The tight side is the end the drum surface runs toward; turned the other way, the ends swap.

The band is pulled by a lever pivoted at O, applied by the force F at arm c. The slack end is on the lever at arm a. In a **simple** band brake (Juvinall Fig. 18.15) the tight end is anchored: enter s = 0. In a **differential** band brake (Fig. 18.17) the tight end is also on the lever, at arm s on the other side of the pivot, where its tension helps apply the brake: F = (P_2 a − P_1 s)/c. It self-locks, holding the drum with no force, once s reaches a e^(−fφ). The checks ask for a brake that does not self-lock; drop that check for a backstop meant to lock.

### Inputs

```calc {description="Drum radius"}
r = 500 mm_
```

```calc {description="Band (lining) width"}
w = 30 mm_
```

```calc {description="Wrap angle of the band on the drum"}
φ = 235°_
```

```calc {description="Friction coefficient of the lining, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
f = 0.45_
```

```calc {description="Arm of the slack end about the lever pivot"}
a = 1 m_
```

```calc {description="Arm of the applying force about the lever pivot"}
c = 1 m_
```

```calc {description="Arm of the tight end on the other side of the pivot (0 for a simple band brake)"}
s = 0 mm_
```

```calc {description="Applying force on the lever"}
F = 300 N_
```

```calc {description="Allowable lining pressure, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
p_a = 0.5 MPa_
```

### Tensions, torque and pressure

```calc
> Tension ratio, Eq. 18.26 (Shigley 16-19)
k = exp(f φ) = ??
> Tight-end arm at which the band self-locks
s_lock = a / k = ?? mm
> Slack-side tension from the lever's moments, Eqs. 18.25 and 18.28
P_2 = F c / (a - s k) = ?? N
> Tight-side tension
P_1 = k P_2 = ?? N
> Braking torque, Eq. 18.24 (Shigley 16-20)
T = (P_1 - P_2) r = ?? N·m
> Largest lining pressure, at the tight end, Eq. 18.27 (Shigley 16-22)
p_max = P_1 / (w r) = ?? kPa
```

### Checks

```calc
@test s < s_lock
```

```calc
@test p_max ≤ p_a
```
