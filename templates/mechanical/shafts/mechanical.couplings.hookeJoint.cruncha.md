+++
cruncha-template = 1
id = "mechanical.couplings.hookeJoint"
version = "1.0.0"
title = "Hooke's (Cardan) universal joint: speed ratio, fluctuation, output angle and torque"
summary = "A single Hooke's (Cardan) universal joint at a shaft angle u: the driven shaft's speed ratio at an input angle (Marks), its greatest and least values sec u and cos u and the fluctuation twice per turn (Machinery's), the output angle and phase error, the angular acceleration of the driven shaft at constant input speed, Marks' efficiency and the torque through the joint."
category = ["Mechanical engineering", "Shafts"]
tags = ["universal joint", "Hooke's joint", "Cardan joint", "U-joint", "coupling", "speed fluctuation", "velocity ratio", "shaft angle", "Marks", "Machinery's Handbook", "Juvinall"]
from = "lacewing:mechanical.couplings"
license = "GPL-3.0-or-later"
inputs = ["u", "ω_1", "a", "T_1", "u_max"]
outputs = ["r_ω", "ω_2", "ω_(2,max)", "ω_(2,min)", "Δω_2", "θ_2", "e_θ", "α_2", "η", "T_2"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "p. 8-36 (Hooke's coupling: velocity ratio, efficiency 100(1 − 0.003u) % to 15°, angles to 30°)"

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "Universal joints, pp. 2327-2328 (extremes sec u and cos u, the 25° example, two joints on an intermediate shaft)"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.7 (couplings and universal joints: the output fluctuates twice per turn)"

[[tests]]
name = "Machinery's example: 100 rev/min through a 25° joint"
source = "Lacewing Lab fits_couplings_test, after Machinery's 26th p. 2327: 100 rev/min at 25° gives 110.34 rev/min at most and 90.63 at least, 19.71 rev/min apart; at a = 0 the ratio is sec 25° = 1.10338. The 25° angle exceeds the 15° to which Marks' efficiency holds (η 0.925) but is within Machinery's 25°"
inputs = { u = "25°", "ω_1" = "100 rpm", a = "0°", T_1 = "0 N·m", u_max = "25°" }
expect = { "ω_(2,max)" = "110.34 rpm", "ω_(2,min)" = "90.63 rpm", "Δω_2" = "19.71 rpm", "r_ω" = "1.1033779", "η" = "0.925" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Hand check: 20° joint at 1500 rev/min, input turned 30°, 200 N·m"
source = "By hand from Marks' ratio and its integral and derivative: r = cos 20°/(1 − sin²20° cos²30°) = 1.0300635, ω2 = 1545.0953 rev/min; θ2 = atan(tan 30°/cos 20°) = 31.566704°, phase error 1.566704°; α2 = −ω1² cos u sin²u sin 2a/(1 − sin²u cos²a)² = −2822.383 rad/s²; η = 1 − 0.003 × 20 = 0.94; T2 = 0.94 × 200/1.0300635 = 182.51301 N·m"
inputs = { u = "20°", "ω_1" = "1500 rpm", a = "30°", T_1 = "200 N·m", u_max = "25°" }
expect = { "r_ω" = "1.0300635", "ω_2" = "1545.0953 rpm", "θ_2" = "31.566704°", "e_θ" = "1.566704°", "α_2" = "-2822.3831 rad/s²", "η" = "0.94", T_2 = "182.51301 N·m" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: 30° joint, input in the third quadrant, over the recommended angle"
source = "By hand: u 30°, a 200°, 1000 rev/min: r = cos 30°/(1 − 0.25 cos²200°) = 1.1113655; θ2 = 202.79588° on the input's turn (atan(tan 200°/cos 30°) + 180°), phase error 2.79588°; Machinery's keeps joints within 20-25° unless slow and lightly loaded, so the angle check fails"
inputs = { u = "30°", "ω_1" = "1000 rpm", a = "200°", T_1 = "50 N·m", u_max = "25°" }
expect = { "r_ω" = "1.1113655", "θ_2" = "202.79588°", "e_θ" = "2.79588°" }
tolerance = "1e-6"
checks = "fail"
+++

## Hooke's (Cardan) universal joint

A single Hooke's joint between two shafts at an angle u does not transmit a uniform speed: with the driving shaft at a constant speed ω₁, the driven shaft speeds up and slows down twice per turn, between ω₁ sec u and ω₁ cos u (Machinery's). Marks gives the ratio at any position as ω₂/ω₁ = cos u/(1 − sin²u sin²(a + 90°)), with a the driving shaft's turn from the position where its yoke pins lie in the plane of the two shafts. The output angle θ₂ (tan θ₂ = tan a/cos u) is the integral of that ratio, and the angular acceleration of the driven shaft its derivative at constant ω₁; both are derived here from Marks' ratio. The acceleration loads the driven inertia and the joint.

Marks gives the efficiency as 100(1 − 0.003u) % with u in degrees, for angles up to 15°, and allows shaft angles up to 30°; Machinery's advises keeping to 20 to 25° unless the joint runs slowly and lightly loaded, and says a joint does not work well beyond 45°. Two joints at equal angles with the intermediate shaft's yokes aligned cancel the fluctuation: see the two-joint template.

```calc {description="Angle between the shafts"}
u = 25°_
```

```calc {description="Speed of the driving shaft"}
ω_1 = 100 rpm_
```

```calc {description="Driving shaft's turn from the position where its yoke pins lie in the plane of the shafts"}
a = 0°_
```

```calc {description="Torque on the driving shaft"}
T_1 = 0 N·m_
```

```calc {description="Largest shaft angle accepted (Machinery's: 20 to 25°; Marks: up to 30°)"}
u_max = 25°_
```

```calc
> Speed ratio at the input angle a, Marks p. 8-36
r_ω = cos(u) / (1 - sin(u)^2 cos(a)^2) = ??
> Speed of the driven shaft
ω_2 = r_ω ω_1 = ?? rpm
> Greatest speed of the driven shaft (a = 0°, 180°), Machinery's p. 2327
ω_(2,max) = ω_1 / cos(u) = ?? rpm
> Least speed of the driven shaft (a = 90°, 270°)
ω_(2,min) = ω_1 cos(u) = ?? rpm
> Fluctuation of the driven speed, twice per turn
Δω_2 = ω_(2,max) - ω_(2,min) = ?? rpm
```

The output angle is found on the same turn as a: the engine's atan2 takes (x, y), so atan2(cos a, sin a/cos u) is the angle whose tangent is tan a/cos u, in the right quadrant.

```calc
> Driven shaft's angle on the input's turn, tan θ₂ = tan a/cos u (derived)
θ_2 = atan2(cos(a), sin(a) / cos(u)) + 2 π round((a - atan2(cos(a), sin(a) / cos(u))) / (2 π)) = ?? deg
> Phase error of the driven shaft
e_θ = θ_2 - a = ?? deg
> Angular acceleration of the driven shaft at constant input speed (derived)
α_2 = -ω_1^2 cos(u) sin(u)^2 sin(2 a) / (1 - sin(u)^2 cos(a)^2)^2 = ?? rad/s²
> Efficiency, Marks p. 8-36 (u in degrees, to 15°)
η = 1 - 0.003 u / 1° = ??
> Torque on the driven shaft at this position, from the power through the joint
T_2 = η T_1 / r_ω = ?? N·m
```

### Check

```calc
@test u ≤ u_max
```
