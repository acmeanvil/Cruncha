+++
cruncha-template = 1
id = "mechanical.drumBrakes.shortShoe"
version = "1.0.0"
title = "Drum brake, short shoe on a lever (Juvinall)"
summary = "A short shoe pressed on a drum by a pivoted lever: the normal force and braking torque for both senses of rotation (self-energizing and deenergizing), the friction coefficient at which the shoe self-locks, the average pressure on the projected area, and the lever moment that brings the energizing shoe to the allowable pressure."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "drum brake", "short shoe", "block brake", "self-energizing", "self-locking", "lever", "Juvinall"]
from = "lacewing:mechanical.drumBrakes"
license = "GPL-3.0-or-later"
inputs = ["f", "r", "a", "b", "F", "c", "w", "θ_s", "p_a"]
outputs = ["N_en", "T_en", "N_de", "T_de", "f_lock", "A_p", "p_en", "p_de", "F_max"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.6: Fig. 18.6, Eqs. 18.13 to 18.15; Tables 18.1 and 18.2; Sample Problem 18.2"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-1 (static analysis of a short shoe); Table 16-3"

[[tests]]
name = "Juvinall Sample Problem 18.2, shoe 3 (self-energizing): 4F at 700 mm, F = 1074 N"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Sample Problem 18.2: f 0.2, r 250, a 170, b 300 mm, shoe 80 mm wide over 90°: N3 = 10.53F, projected area 28 284 mm², 0.40 MPa, the allowable; b/a = 1.765 locks"
inputs = { f = "0.2", r = "250 mm", a = "170 mm", b = "300 mm", F = "4296 N", c = "700 mm", w = "80 mm", "θ_s" = "90°", p_a = "0.4 MPa" }
expect = { N_en = "11309 N", A_p = "28284 mm²", p_en = "0.3999 MPa", F_max = "4297 N", f_lock = "1.7647" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 18.2, shoe 2 (deenergizing): 4F at 600 mm less F at 40 mm"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Sample Problem 18.2: the lever moment (4 × 0.600 − 0.040)F = 2.36F, entered as 4F at 590 mm; N2 = 7.07F (7589 N), T2 = 379.4 N·m, with shoe 3's 565.5 N·m the brake's 945 N·m"
inputs = { f = "0.2", r = "250 mm", a = "170 mm", b = "300 mm", F = "4296 N", c = "590 mm", w = "80 mm", "θ_s" = "90°", p_a = "0.4 MPa" }
expect = { N_de = "7589 N", T_de = "379.4 N·m" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Self-locking shoe: f a ≥ b"
source = "By hand: f 0.4, a 300, b 100 mm, so f a = 120 mm ≥ b: the shoe locks; deenergizing N = 100 × 1/(0.1 + 0.12) = 454.54545 N"
inputs = { f = "0.4", r = "200 mm", a = "300 mm", b = "100 mm", F = "100 N", c = "1 m", w = "50 mm", "θ_s" = "60°", p_a = "1 MPa" }
expect = { N_de = "454.54545 N", f_lock = "0.33333333" }
tolerance = "1e-7"
checks = "fail"
+++

## Drum brake, short shoe

A shoe short enough (contact up to about 45°) that its normal and friction forces can be taken as concentrated at the middle of the contact, pressed on a drum of radius r by a lever pivoted at A (Juvinall Sec. 18.6, Fig. 18.6). The applying force F acts at arm c about A, the normal force N at arm b and the friction force f N at arm a. Summing moments about A gives N for both senses of rotation. When the drum turns so that friction helps apply the shoe, the shoe is **self-energizing**, N = F c / (b − f a); turned the other way it is **deenergizing**, N = F c / (b + f a). Once f a ≥ b the energizing shoe locks the drum with no force at all.

The pressure is taken as the normal force over the projected area, shoe width × chord.

### Inputs

```calc {description="Friction coefficient of the lining, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
f = 0.2_
```

```calc {description="Drum radius"}
r = 250 mm_
```

```calc {description="Arm of the friction force about the lever pivot A"}
a = 170 mm_
```

```calc {description="Arm of the normal force about the lever pivot A"}
b = 300 mm_
```

```calc {description="Applying force on the lever"}
F = 4296 N_
```

```calc {description="Arm of the applying force about the lever pivot A"}
c = 700 mm_
```

```calc {description="Shoe face width"}
w = 80 mm_
```

```calc {description="Angle of contact of the shoe"}
θ_s = 90°_
```

```calc {description="Allowable lining pressure, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
p_a = 0.4 MPa_
```

### Forces and torques

```calc
> Normal force, self-energizing, Eq. 18.13
N_en = F c / (b - f a) = ?? N
> Braking torque, self-energizing
T_en = f N_en r = ?? N·m
> Normal force, deenergizing (drum turning the other way), Eq. 18.14
N_de = F c / (b + f a) = ?? N
> Braking torque, deenergizing
T_de = f N_de r = ?? N·m
> Friction coefficient at which the energizing shoe self-locks, Eq. 18.15
f_lock = b / a = ??
```

### Pressure

```calc
> Projected area of the shoe, width × chord
A_p = w 2 r sin(θ_s / 2) = ?? mm²
> Average pressure, self-energizing
p_en = N_en / A_p = ?? MPa
> Average pressure, deenergizing
p_de = N_de / A_p = ?? MPa
> Applying force that brings the energizing shoe to p_a
F_max = p_a A_p (b - f a) / c = ?? N
```

### Checks

```calc
@test f a < b
```

```calc
@test p_en ≤ p_a
```
