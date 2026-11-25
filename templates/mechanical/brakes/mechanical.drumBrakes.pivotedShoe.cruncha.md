+++
cruncha-template = 1
id = "mechanical.drumBrakes.pivotedShoe"
version = "1.0.0"
title = "Drum brake, symmetrical pivoted shoe (Shigley)"
summary = "A symmetrical shoe pivoted where the friction forces have no moment about the pivot: the pivot distance, the normal and friction forces and the braking torque at the allowable pressure."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "drum brake", "pivoted shoe", "symmetrical shoe", "external shoe", "Shigley", "Juvinall"]
from = "lacewing:mechanical.drumBrakes"
license = "GPL-3.0-or-later"
inputs = ["r", "θ_2", "b", "f", "p_a"]
outputs = ["a", "N", "R_y", "T"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-3: Fig. 16-12, Eqs. 16-15 to 16-18; Table 16-3"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.8: Fig. 18.11, Eqs. 18.22 and 18.23 (friction radius)"

[[tests]]
name = "Hand check: 150 mm drum, ±50° shoe"
source = "By hand (neither book works a numerical pivoted-shoe example; the Lacewing Lab brakes_test checks the pivot by integration): r 150 mm, θ2 50°, b 40 mm, f 0.3, pa 0.8 MPa: 2θ2 + sin 2θ2 = 2.7301370, a = 4 × 0.15 sin 50°/2.7301370 = 168.35297 mm, N = ½ × 0.8e6 × 0.04 × 0.15 × 2.7301370 = 6552.3288 N, Ry = 1965.6986 N, T = 330.93120 N·m"
inputs = { r = "150 mm", "θ_2" = "50°", b = "40 mm", f = "0.3", p_a = "0.8 MPa" }
expect = { a = "168.35297 mm", N = "6552.3288 N", R_y = "1965.6986 N", T = "330.93120 N·m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "A shoe wider than 180° of contact is out of range"
source = "By hand: θ2 = 95° is more than the 90° half-angle the analysis allows"
inputs = { r = "150 mm", "θ_2" = "95°", b = "40 mm", f = "0.3", p_a = "0.8 MPa" }
expect = {}
checks = "fail"
+++

## Drum brake, symmetrical pivoted shoe

An external shoe whose lining spans ±θ_2 symmetrically about the line through the drum centre and the shoe's pivot (Shigley Fig. 16-12). The shoe is pivoted, so its wear keeps the lining cylindrical and the pressure is p_a cos θ, largest on the centre line. Placing the pivot at the distance a where the friction forces have no moment about it (Eq. 16-15) keeps the shoe from cocking; a is always a little more than r, and is Juvinall's friction radius (Eq. 18.22 with θ = 2θ_2). The normal force then acts through the pivot along the centre line and the friction force across it.

### Inputs

```calc {description="Drum radius"}
r = 150 mm_
```

```calc {description="Half-angle of the lining about the pivot line (at most 90°)"}
θ_2 = 50°_
```

```calc {description="Face width of the shoe"}
b = 40 mm_
```

```calc {description="Friction coefficient of the lining, Shigley Table 16-3"}
f = 0.3_
```

```calc {description="Allowable lining pressure, on the centre line, Shigley Table 16-3"}
p_a = 0.8 MPa_
```

### Pivot, forces and torque

```calc
> Pivot distance from the drum centre, Eq. 16-15
a = 4 r sin(θ_2) / (2 θ_2 + sin(2 θ_2)) = ?? mm
> Normal force, along the pivot line (R_x), Eq. 16-16
N = p_a b r (2 θ_2 + sin(2 θ_2)) / 2 = ?? N
> Friction force, across the pivot line, Eq. 16-17
R_y = f N = ?? N
> Braking torque, Eq. 16-18
T = a f N = ?? N·m
```

### Checks

```calc
@test 0 '°' < θ_2 ≤ 90 '°'
```
