+++
cruncha-template = 1
id = "mechanical.bandBrakes.fromPressure"
version = "1.0.0"
title = "Band brake, simple or differential, at the allowable pressure"
summary = "A simple or differential band brake at the lining's largest pressure: the tension ratio e^(fφ), the tight- and slack-side tensions, the braking torque, the force on the lever, and the arm at which a differential band self-locks."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "band brake", "differential band brake", "capstan", "belt friction", "self-locking", "lever", "Juvinall", "Shigley"]
from = "lacewing:mechanical.bandBrakes"
license = "GPL-3.0-or-later"
inputs = ["r", "w", "φ", "f", "a", "c", "s", "p_a"]
outputs = ["k", "P_1", "P_2", "T", "F", "s_lock"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.9: Figs. 18.15 to 18.17, Eqs. 18.24 to 18.28; Tables 18.1 and 18.2; Sample Problem 18.4"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-4: Eqs. 16-19 to 16-22; Table 16-3"

[[tests]]
name = "Juvinall Sample Problem 18.4: differential band, 500 kPa"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Sample Problem 18.4: r 250, b 80 mm, 240° of wrap, f 0.20, a 150, c 700, s 35 mm: P1 10 000 N, P2 4328 N (the book writes φ as 1.33π), T 1418 N·m, F 427 N, self-locking for s ≥ 64.9 mm"
inputs = { r = "250 mm", w = "80 mm", "φ" = "240°", f = "0.20", a = "150 mm", c = "700 mm", s = "35 mm", p_a = "0.5 MPa" }
expect = { P_1 = "10000 N", P_2 = "4328 N", T = "1418 N·m", F = "427 N", s_lock = "64.9 mm" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: Sample Problem 18.4 unrounded, and as a simple band"
source = "By hand: e^(0.2 × 4.1887902) = 2.3111796, P2 = 10000/2.3111796 = 4326.7949 N, T = 5673.2051 × 0.25 = 1418.3013 N·m; with s = 0, F = 4326.7949 × 0.15/0.7 = 927.17033 N"
inputs = { r = "250 mm", w = "80 mm", "φ" = "240°", f = "0.20", a = "150 mm", c = "700 mm", s = "0 mm", p_a = "0.5 MPa" }
expect = { k = "2.3111796", P_2 = "4326.7949 N", T = "1418.3013 N·m", F = "927.17033 N" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Sample Problem 18.4 with s = 70 mm: self-locking"
source = "Juvinall & Marshek 5th Sample Problem 18.4(c): s at or above 64.9 mm locks; at 70 mm F is negative"
inputs = { r = "250 mm", w = "80 mm", "φ" = "240°", f = "0.20", a = "150 mm", c = "700 mm", s = "70 mm", p_a = "0.5 MPa" }
expect = { s_lock = "64.90192 mm" }
tolerance = "1e-6"
checks = "fail"
+++

## Band brake at the allowable pressure

A flexible band, usually with a woven lining, wrapped through the angle φ round a drum of radius r (Juvinall Sec. 18.9, Shigley Sec. 16-4). An element of band in equilibrium gives dP = f P dθ, so the tight-side tension P_1 and slack-side tension P_2 are in the ratio e^(fφ), and the band presses on the drum with p = P/(w r), largest at the tight end. The tight side is the end the drum surface runs toward; turned the other way, the ends swap.

The band is pulled by a lever pivoted at O, applied by the force F at arm c. The slack end is on the lever at arm a. In a **simple** band brake (Juvinall Fig. 18.15) the tight end is anchored: enter s = 0. In a **differential** band brake (Fig. 18.17) the tight end is also on the lever, at arm s on the other side of the pivot, where its tension helps apply the brake: F = (P_2 a − P_1 s)/c. It self-locks, holding the drum with no force, once s reaches a e^(−fφ). The checks ask for a brake that does not self-lock; drop that check for a backstop meant to lock.

### Inputs

```calc {description="Drum radius"}
r = 250 mm_
```

```calc {description="Band (lining) width"}
w = 80 mm_
```

```calc {description="Wrap angle of the band on the drum"}
φ = 240°_
```

```calc {description="Friction coefficient of the lining, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
f = 0.20_
```

```calc {description="Arm of the slack end about the lever pivot"}
a = 150 mm_
```

```calc {description="Arm of the applying force about the lever pivot"}
c = 700 mm_
```

```calc {description="Arm of the tight end on the other side of the pivot (0 for a simple band brake)"}
s = 35 mm_
```

```calc {description="Largest (allowable) lining pressure, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
p_a = 0.5 MPa_
```

### Tensions, torque and lever force

```calc
> Tension ratio, Eq. 18.26 (Shigley 16-19)
k = exp(f φ) = ??
> Tight-side tension at the largest pressure, Eq. 18.27 (Shigley 16-22)
P_1 = p_a w r = ?? N
> Slack-side tension
P_2 = P_1 / k = ?? N
> Braking torque, Eq. 18.24 (Shigley 16-20)
T = (P_1 - P_2) r = ?? N·m
> Applying force on the lever, Eqs. 18.25 and 18.28
F = (P_2 a - P_1 s) / c = ?? N
> Tight-end arm at which the band self-locks
s_lock = a / k = ?? mm
```

### Checks

```calc
@test s < s_lock
```
