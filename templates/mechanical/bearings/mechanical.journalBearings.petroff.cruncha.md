+++
cruncha-template = 1
id = "mechanical.journalBearings.petroff"
version = "1.0.0"
title = "Journal bearing friction, Petroff's equation"
summary = "Petroff's light-load friction of a journal bearing, the journal concentric in its bearing: unit load, the bearing characteristic μN/P and Sommerfeld number, the coefficient of friction, the friction torque and the power lost."
category = ["Mechanical engineering", "Bearings"]
tags = ["journal bearing", "Petroff", "friction", "viscous friction", "friction torque", "power loss", "sleeve bearing", "lightly loaded", "Sommerfeld number", "Shigley", "Juvinall"]
from = "lacewing:mechanical.journalBearings"
license = "GPL-3.0-or-later"
inputs = ["d", "l", "c", "N", "W", "μ"]
outputs = ["r", "P", "μNP", "S", "f_P", "T_q", "H"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 12-3: Eq. 12-6 (Petroff's equation), Eq. 12-7 (Sommerfeld number) and the no-load torque before it"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 13.7, Eq. 13.7; Sample Problem 13.2"

[[tests]]
name = "Juvinall Sample Problem 13.2: 100 mm journal, 80 mm long, c 0.05 mm, 600 rpm, 5000 N, 50 mPa·s"
source = "Lacewing Lab journal_bearings_test, after Juvinall & Marshek 5th Sample Problem 13.2: f = 0.0158, T = 3.95 N·m, power 248 W"
inputs = { d = "100 mm", l = "80 mm", c = "0.05 mm", N = "600 rpm", W = "5000 N", "μ" = "50 mPa·s" }
expect = { f_P = "0.0158", T_q = "3.95 N·m", H = "248 W" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: Shigley's Ex. 12-1 bearing by Petroff"
source = "By hand: d = l = 1.5 in, c 0.0015 in, 1800 rpm, 500 lbf, 4 μreyn: P = 222.222 psi, μN/P = 4e-6 × 30/222.222 = 5.4e-7, S = 500² × 5.4e-7 = 0.135, f = 2π² × 0.135/500 = 0.00532959, T = 4π² × 0.75³ × 1.5 × 4e-6 × 30/0.0015 = 1.9985949 lbf·in, H = 2π × 1.9985949 × 30 lbf·in/s = 0.0570797 hp"
inputs = { d = "1.5 in", l = "1.5 in", c = "0.0015 in", N = "1800 rpm", W = "500 lbf", "μ" = "4e-6 lbf·s/in²" }
expect = { P = "222.222222 psi", "μNP" = "5.4e-7", S = "0.135", f_P = "0.00532958638", T_q = "1.99859489 lbf·in", H = "0.0570797366 hp" }
tolerance = "1e-6"
checks = "pass"
+++

## Journal bearing friction, Petroff's equation

Petroff's equation (1883) gives the viscous friction of a journal running concentric in its bearing: the film is a uniform annulus of thickness c, sheared at the journal's surface speed. It is exact only with no load. As the load rises the journal runs eccentric and the friction climbs above Petroff's value, so use it for lightly loaded bearings, as a first estimate, or as the light-load limit of Raimondi and Boyd's friction variable (the Raimondi-Boyd template). The torque does not depend on the load; the coefficient of friction is that torque over W r.

### Bearing

```calc {description="Journal diameter"}
d = 100 mm_
```

```calc {description="Bearing length"}
l = 80 mm_
```

```calc {description="Radial clearance"}
c = 0.05 mm_
```

```calc {description="Journal speed"}
N = 600 rpm_
```

```calc {description="Radial load"}
W = 5000 N_
```

```calc {description="Absolute viscosity of the oil at its operating temperature"}
μ = 50 mPa·s_
```

### Friction

```calc
> Journal radius
r = d / 2 = ?? mm
> Speed in revolutions per second
n = N / (1 'rev') = ?? 1/s
> Unit load on the projected area
P = W / (l d) = ?? MPa
> Bearing characteristic μN/P
μNP = μ n / P = ??
> Sommerfeld number, Eq. 12-7
S = (r / c)^2 μNP = ??
> Petroff's coefficient of friction, Eq. 12-6
f_P = 2 π^2 μNP r / c = ??
> Friction torque, 4π² r³ l μ N / c
T_q = 4 π^2 r^3 l μ n / c = ?? N·m
> Power lost in friction
H = 2 π T_q n = ?? W
```
