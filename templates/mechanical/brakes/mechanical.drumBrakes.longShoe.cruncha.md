+++
cruncha-template = 1
id = "mechanical.drumBrakes.longShoe"
version = "1.0.0"
title = "Drum brake, long shoes hinged on a pin, internal or external (Shigley)"
summary = "A long internal (expanding) or external (contracting) shoe hinged on a pin, at the allowable pressure or the pV limit: the moments of the normal and friction forces about the pin, the actuating force and braking torque of the self-energizing shoe, the self-locking friction coefficient, the hinge-pin reactions, the pressure, torque and reactions of a mirror-image deenergizing shoe applied by the same force, the rubbing speed, pV and power."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "drum brake", "long shoe", "internal shoe", "external shoe", "expanding shoe", "contracting shoe", "self-energizing", "self-locking", "hinge pin", "pin reaction", "pV", "Shigley", "Juvinall"]
from = "lacewing:mechanical.drumBrakes"
license = "GPL-3.0-or-later"
inputs = ["kind", "r", "a", "c", "θ_1", "θ_2", "b", "f", "p_a", "φ", "n", "pV_lim"]
outputs = ["V", "p", "θ_a", "A", "B", "M_f", "M_N", "T", "F", "f_lock", "R_x", "R_y", "R", "p_2", "T_2", "R_(x,2)", "R_(y,2)", "R_2", "T_tot", "pV", "P"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 16-2 and 16-3: Figs. 16-6, 16-7, 16-11; Eqs. 16-1 to 16-14 (pressure, moments, actuating force, self-locking, torque, pin reactions internal and external); Table 16-3; Example 16-2"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 18.7 and 18.8: Eqs. 18.16 to 18.21; Table 18.3 (pV limits); Sample Problem 18.3"

[[tests]]
name = "Shigley Ex. 16-2: internal shoes, 150 mm drum, 1000 kPa"
source = "Lacewing Lab brakes_test, after Shigley 10th Example 16-2: a = 122.7 mm (√(112² + 50²)), c 212 mm, θ 0° to 126°, b 32 mm, f 0.32, force at 24°: Mf 304, MN 788 N·m, F 2.28 kN, T 366 N·m; left shoe 443 kPa and 162 N·m, 528 N·m in all; pin reactions Rx −1.410, Ry 4.839, R 5.04 kN and 0.678, 0.538, 0.866 kN. No speed is given; 100 rpm keeps pV under the limit so pa governs"
inputs = { kind = "1", r = "150 mm", a = "122.7 mm", c = "212 mm", "θ_1" = "0°", "θ_2" = "126°", b = "32 mm", f = "0.32", p_a = "1000 kPa", "φ" = "24°", n = "100 rpm", pV_lim = "2.1 MPa·m/s" }
expect = { M_f = "304 N·m", M_N = "788 N·m", F = "2.28 kN", T = "366 N·m", A = "0.3273", B = "1.3373", R_x = "-1.410 kN", R_y = "4.839 kN", R = "5.04 kN", p_2 = "443 kPa", T_2 = "162 N·m", T_tot = "528 N·m", "R_(x,2)" = "0.678 kN", "R_(y,2)" = "0.538 kN", R_2 = "0.866 kN" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 18.3: external shoes at 300 rpm, pV 2.1 MPa·m/s governs"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Sample Problem 18.3: r 150, pin a 250, spring c 500 mm, θ1 = atan(200/150) − 45° = 8.13°, 90° shoes 50 mm wide, f 0.30, pa 600 kPa but pV 2.1 MPa·m/s at 300 rpm: pmax 0.446 MPa, spring force 1448 N (the book rounds pmax), other shoe 0.392 MPa, 320 N·m, 10.1 kW (10.05 unrounded)"
inputs = { kind = "2", r = "150 mm", a = "250 mm", c = "500 mm", "θ_1" = "8.130102 °", "θ_2" = "98.130102 °", b = "50 mm", f = "0.30", p_a = "600 kPa", "φ" = "0°", n = "300 rpm", pV_lim = "2.1 MPa·m/s" }
expect = { p = "0.446 MPa", F = "1448 N", p_2 = "0.392 MPa", T_tot = "320 N·m", P = "10.05 kW" }
tolerance = "0.15%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 16-2 unrounded"
source = "By hand from Shigley Eqs. 16-2 to 16-6 with Ex. 16-2's data: Mf = 304.14903 N·m, MN = 787.62890 N·m, T = 365.82572 N·m, F = 2280.5654 N, f locking = 0.32 × 787.6289/304.14903 = 0.82867681"
inputs = { kind = "1", r = "150 mm", a = "122.7 mm", c = "212 mm", "θ_1" = "0°", "θ_2" = "126°", b = "32 mm", f = "0.32", p_a = "1000 kPa", "φ" = "24°", n = "100 rpm", pV_lim = "2.1 MPa·m/s" }
expect = { M_f = "304.14903 N·m", M_N = "787.62890 N·m", T = "365.82572 N·m", F = "2280.5654 N", f_lock = "0.82867681" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Ex. 16-2 with f = 0.9: the shoe self-locks"
source = "By hand: above f locking = 0.829, Mf exceeds MN and the actuating force is negative"
inputs = { kind = "1", r = "150 mm", a = "122.7 mm", c = "212 mm", "θ_1" = "0°", "θ_2" = "126°", b = "32 mm", f = "0.9", p_a = "1000 kPa", "φ" = "24°", n = "100 rpm", pV_lim = "2.1 MPa·m/s" }
expect = { M_N = "787.62890 N·m" }
tolerance = "1e-7"
checks = "fail"
+++

## Drum brake, long shoes hinged on a pin

A long shoe hinged on a pin at distance a from the drum centre, its lining running from θ_1 to θ_2 measured from the line through the drum centre and the pin, pushed against the drum by a force F at arm c about the pin (Shigley Secs. 16-2 and 16-3, Figs. 16-7 and 16-11). Enter the kind as a number: 1 internal (expanding) shoe, 2 external (contracting) shoe. The shoe and drum are rigid and the lining wears in proportion to the friction work, so the pressure is p sin θ / sin θ_a, largest at θ_a: 90° when the lining spans it, otherwise its end nearer 90° (Fig. 16-6).

The shoe is taken at the largest pressure for the sense of rotation that makes it **self-energizing** (friction helps apply it), F = (M_N − M_f)/c. It **self-locks** once M_f reaches M_N; Juvinall advises keeping the self-locking friction coefficient 25 to 50 % above the design f. A brake usually pairs it with a mirror-image shoe applied by the same F, which is **deenergizing** and so runs at the lower pressure p_2 (Shigley Eqs. 16-7 and 16-11); reversing the drum swaps the two. The pin reactions are on Shigley's axes: origin at the drum centre, x through the pin, y toward the shoe; the actuating force has components F sin φ along x and F cos φ along y.

The largest pressure is the lining's allowable p_a or, where the rubbing speed makes it lower, the pV limit over the speed (Juvinall Table 18.3: about 1.05 MPa·m/s continuous and 2.1 MPa·m/s occasional with poor heat dissipation, 3.0 MPa·m/s in an oil bath).

### Inputs

```calc {description="Shoe: 1 internal (expanding), 2 external (contracting)"}
kind = 1_
```

```calc {description="Drum radius (the lining's contact radius)"}
r = 150 mm_
```

```calc {description="Distance from the drum centre to the hinge pin"}
a = 122.7 mm_
```

```calc {description="Arm of the actuating force about the hinge pin"}
c = 212 mm_
```

```calc {description="Angle from the pin's line to the start of the lining"}
θ_1 = 0°_
```

```calc {description="Angle from the pin's line to the end of the lining"}
θ_2 = 126°_
```

```calc {description="Face width of the lining"}
b = 32 mm_
```

```calc {description="Friction coefficient of the lining, Shigley Table 16-3 or Juvinall Tables 18.1 and 18.2"}
f = 0.32_
```

```calc {description="Allowable lining pressure, Shigley Table 16-3 or Juvinall Tables 18.1 and 18.2"}
p_a = 1000 kPa_
```

```calc {description="Angle of the actuating force from the y axis (Fx = F sin φ, Fy = F cos φ)"}
φ = 24°_
```

```calc {description="Drum speed"}
n = 100 rpm_
```

```calc {description="Limit on pressure × rubbing speed, Juvinall Table 18.3"}
pV_lim = 2.1 MPa·m/s_
```

### Pressure

```calc
> Rubbing speed
V = r n = ?? m/s
> Largest pressure used: the allowable, or the pV limit at this speed
p = min(p_a, pV_lim / V) = ?? kPa
> Angle where the pressure peaks, Fig. 16-6
θ_a = {θ_2 if θ_2 ≤ 90 '°'; θ_1 if θ_1 ≥ 90 '°'; 90 '°' otherwise} = ?? °
```

### Moments, force and torque

```calc
> Integrals of Eq. 16-8
A = ((sin(θ_2))^2 - (sin(θ_1))^2) / 2 = ??
B = (θ_2 - θ_1) / 2 - (sin(2 θ_2) - sin(2 θ_1)) / 4 = ??
> Moment of the friction forces about the pin, Eq. 16-2
M_f = f p b r / sin(θ_a) (r (cos(θ_1) - cos(θ_2)) - a A) = ?? N·m
> Moment of the normal forces about the pin, Eq. 16-3
M_N = p b r a B / sin(θ_a) = ?? N·m
> Actuating force, self-energizing shoe, Eqs. 16-4 and 16-13
F = (M_N - M_f) / c = ?? kN
> Braking torque, Eq. 16-6
T = f p b r^2 (cos(θ_1) - cos(θ_2)) / sin(θ_a) = ?? N·m
> Friction coefficient at which the shoe self-locks, Eq. 16-5
f_lock = f M_N / M_f = ??
```

### Hinge-pin reactions, self-energizing shoe

```calc
> Pin reactions, Eqs. 16-9, 16-10 (internal) and 16-12, 16-14 (external)
R_x = p b r / sin(θ_a) (A - f B) - F sin(φ) = ?? kN
R_y = {p b r / sin(θ_a) (B + f A) - F cos(φ) if kind == 1; -p b r / sin(θ_a) (f A + B) + F cos(φ) if kind == 2} = ?? kN
R = hypot(R_x, R_y) = ?? kN
```

### Mirror-image deenergizing shoe, same force

```calc
> Its largest pressure, Eqs. 16-7 and 16-11
p_2 = p (M_N - M_f) / (M_N + M_f) = ?? kPa
> Its braking torque
T_2 = T p_2 / p = ?? N·m
> Its pin reactions
R_(x,2) = p_2 b r / sin(θ_a) (A + f B) - F sin(φ) = ?? kN
R_(y,2) = {p_2 b r / sin(θ_a) (B - f A) - F cos(φ) if kind == 1; p_2 b r / sin(θ_a) (f A - B) + F cos(φ) if kind == 2} = ?? kN
R_2 = hypot(R_(x,2), R_(y,2)) = ?? kN
> Braking torque of both shoes
T_tot = T + T_2 = ?? N·m
```

### Heat

```calc
> Pressure × rubbing speed on the self-energizing shoe
pV = p V = ?? MPa·m/s
> Braking power, both shoes
P = T_tot n = ?? kW
```

### Checks

```calc
@test M_N > M_f
```

```calc
@test pV ≤ pV_lim
```

```calc
@test 0 '°' ≤ θ_1 < θ_2 ≤ 180 '°'
```
