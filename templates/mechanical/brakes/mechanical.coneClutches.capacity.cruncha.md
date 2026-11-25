+++
cruncha-template = 1
id = "mechanical.coneClutches.capacity"
version = "1.0.0"
title = "Cone clutch or brake, uniform wear or uniform pressure"
summary = "A cone clutch or brake: face width, friction radius, axial clamping force and torque at the allowable pressure by uniform wear or uniform pressure, the clamping force a required torque needs, and the torque, pressure and safety factor against slipping of a given clamping force, with the cone angle checked against the usual range."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["clutch", "brake", "cone clutch", "cone brake", "uniform wear", "uniform pressure", "cone angle", "clamping force", "torque capacity", "Juvinall", "Shigley"]
from = "lacewing:mechanical.coneClutches"
license = "GPL-3.0-or-later"
inputs = ["model", "r_o", "r_i", "α", "f", "p_a", "T_req", "F"]
outputs = ["b", "r_e", "F_max", "T_cap", "k_α", "n", "F_T", "T_F", "p_F", "n_F"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.5: Fig. 18.5, Eqs. 18.2a, 18.3a, 18.6a, 18.7a (with Eqs. 18.1 and 18.5 for the clamping force); Tables 18.1 and 18.2"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-7: Figs. 16-21 and 16-22, Eqs. 16-44 to 16-49; Table 16-3"

[[tests]]
name = "Hand check: 250 mm cone of 12°, 15 mm deep face, uniform wear"
source = "By hand (neither book works a cone example; this is the Lacewing Lab page's opening illustration): ro 125, ri 110 mm, f 0.30, pa 400 kPa: b = 15/sin 12° = 72.146015 mm, re = 117.5 mm, Fmax = 2π × 400e3 × 0.110 × 0.015 = 4146.9023 N, T = 4146.9023 × 0.30 × 0.1175/sin 12° = 703.07882 N·m; for 500 N·m, F = 500 sin 12°/(0.3 × 0.1175) = 2949.1020 N; at 3000 N, T = 508.62941 N·m, p inner = 289.37262 kPa, n = 1.0172588"
inputs = { model = "1", r_o = "125 mm", r_i = "110 mm", "α" = "12°", f = "0.30", p_a = "400 kPa", T_req = "500 N·m", F = "3000 N" }
expect = { b = "72.146015 mm", r_e = "117.5 mm", F_max = "4146.9023 N", T_cap = "703.07882 N·m", "k_α" = "4.8097343", F_T = "2949.1020 N", T_F = "508.62941 N·m", p_F = "289.37262 kPa", n_F = "1.0172588" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Hand check: the same cone, uniform pressure"
source = "By hand: re = 2(0.125³ − 0.110³)/(3(0.125² − 0.110²)) = 117.65957 mm, Fmax = π × 400e3 × (0.125² − 0.110²) = 4429.6456 N, T = 752.03595 N·m; at 3000 N, p = 270.90203 kPa, T = 509.32017 N·m"
inputs = { model = "2", r_o = "125 mm", r_i = "110 mm", "α" = "12°", f = "0.30", p_a = "400 kPa", T_req = "500 N·m", F = "3000 N" }
expect = { r_e = "117.65957 mm", F_max = "4429.6456 N", T_cap = "752.03595 N·m", p_F = "270.90203 kPa", T_F = "509.32017 N·m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "At α = 90° a cone is a flat disk"
source = "Lacewing Lab brakes_test (a cone at 90° carries the disk's torque): ro 100, ri 60 mm, f 0.3, pa 1 MPa, uniform pressure gives the plate clutch's 492.60172 N·m; 90° is outside the usual 8° to 15°"
inputs = { model = "2", r_o = "100 mm", r_i = "60 mm", "α" = "90°", f = "0.3", p_a = "1 MPa", T_req = "400 N·m", F = "15 kN" }
expect = { T_cap = "492.60172 N·m", "k_α" = "1" }
tolerance = "1e-7"
checks = "fail"
+++

## Cone clutch or brake

A cone clutch has one conical friction face between radii r_i and r_o at its two ends, with half-angle α from the axis to the face (Juvinall Sec. 18.5, Shigley Sec. 16-7). A ring of the face of radial depth dr has area 2πr dr / sin α; the axial component of its normal force is that of a flat ring, while its friction force acts at r. So the clamping force is that of a disk of the same radii, and the torque is a disk's divided by sin α: the wedge gains 1/sin α, about five times at 12°. Uniform wear (p·r constant, a worn-in clutch, the design basis) and uniform pressure (a new one) are taken as for disks. The disengaging force is not computed.

Below about 8° the cone may grab and be hard to disengage; 8° to 15° is usual and about 12° is the optimum (Juvinall; Shigley gives 10° to 15°).

### Inputs

Enter the model as a number: 1 uniform wear, 2 uniform pressure. The friction coefficient and allowable pressure come from the lining: Juvinall Tables 18.1 and 18.2, Shigley Table 16-3.

```calc {description="Pressure model: 1 uniform wear (design basis), 2 uniform pressure (new clutch)"}
model = 1_
```

```calc {description="Radius at the large end of the friction face"}
r_o = 125 mm_
```

```calc {description="Radius at the small end of the friction face"}
r_i = 110 mm_
```

```calc {description="Cone half-angle, axis to friction face"}
α = 12°_
```

```calc {description="Dynamic friction coefficient of the lining, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
f = 0.30_
```

```calc {description="Allowable lining pressure, Juvinall Tables 18.1 and 18.2 or Shigley Table 16-3"}
p_a = 400 kPa_
```

```calc {description="Torque to carry, with any service or safety factor"}
T_req = 500 N·m_
```

```calc {description="Axial clamping force applied"}
F = 3000 N_
```

### Capacity at the allowable pressure

```calc
> Face width along the cone
b = (r_o - r_i) / sin(α) = ?? mm
> Friction radius, as for a disk
r_e = {(r_o + r_i) / 2 if model == 1; 2 (r_o^3 - r_i^3) / (3 (r_o^2 - r_i^2)) if model == 2} = ?? mm
> Axial clamping force at p_a, Eqs. 16-44 and 16-47
F_max = {2 π p_a r_i (r_o - r_i) if model == 1; π p_a (r_o^2 - r_i^2) if model == 2} = ?? N
> Torque capacity, Eqs. 18.2a and 18.6a (Shigley 16-45, 16-48)
T_cap = F_max f r_e / sin(α) = ?? N·m
> Torque gain of the wedge over a flat disk
k_α = 1 / sin(α) = ??
> Margin of the capacity over the torque required
n = T_cap / T_req = ??
```

### Clamping force for the torque required

```calc {description="Axial force that just carries T_req, Eqs. 18.3a and 18.7a (Shigley 16-46, 16-49)"}
F_T = T_req sin(α) / (f r_e) = ?? N
```

### At the given clamping force

```calc
> Torque the clamping force carries
T_F = F f r_e / sin(α) = ?? N·m
> Largest pressure it makes: at the inner radius by uniform wear, everywhere by uniform pressure
p_F = F / F_max p_a = ?? kPa
> Safety factor against slipping
n_F = T_F / T_req = ??
```

### Checks

```calc
@test T_cap ≥ T_req
```

```calc
@test p_F ≤ p_a
```

```calc
@test n_F ≥ 1
```

```calc
@test 8 '°' ≤ α ≤ 15 '°'
```
