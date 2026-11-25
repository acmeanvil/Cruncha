+++
cruncha-template = 1
id = "mechanical.shaftDesign.stiffness"
version = "1.0.0"
title = "Shaft design: twist, and the diameter for a deflection, slope or critical speed"
summary = "The angle of twist of a uniform shaft segment and the diameter that holds an allowable twist per length; the diameter that brings a deflection or slope found by a beam analysis to its allowable (Shigley Eqs. 7-17, 7-18, every diameter scaled together); and the diameter for a wanted critical speed (Juvinall Eq. 17.5)."
category = ["Mechanical engineering", "Shafts"]
tags = ["shaft", "shaft design", "stiffness", "twist", "angle of twist", "deflection", "slope", "critical speed", "resize", "Shigley", "Juvinall"]
from = "lacewing:mechanical.shaftDesign"
license = "GPL-3.0-or-later"
inputs = ["d", "d_i", "T", "L", "G", "θ_all′", "n_d", "y", "y_all", "s", "s_all", "ω_c", "ω_req"]
outputs = ["J", "θ", "θ′", "d_θ", "d_y", "d_s", "d_ω"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-5: Eqs. 7-17, 7-18 (resizing for deflection and slope), Eq. 7-20 and Eq. 4-5 (twist), Table 7-2 (typical allowable deflections and slopes); Example 7-4"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Eq. 17.5 (diameter for a critical speed); Problems 17.19 and 17.34D (0.08° per foot)"

[[tests]]
name = "Shigley Ex. 7-4: slope 0.001095 rad at a 1.0 in diameter, allowed 0.0005 rad"
source = "Lacewing Lab shaft_design_test, after Shigley 10th Example 7-4 (Eq. 7-18): d = 1.0 (0.001095/0.0005)^¼ = 1.216 in. The slope found exceeds the allowable, so the slope check fails"
inputs = { d = "1 in", d_i = "0 in", T = "100 lbf·in", n_d = "1", y = "0.001 in", y_all = "0.005 in", s = "0.001095", s_all = "0.0005" }
expect = { d_s = "1.2165 in" }
tolerance = "0.05%"
checks = "fail"

[[tests]]
name = "Juvinall Problem 17.34D: the 0.08° per foot diameter for 1 hp at 1 rev/min, steel"
source = "Lacewing Lab shaft_design_test, after Juvinall 5th Problem 17.34D: d = 4.6(hp/n)^¼ in. By hand with T = 63 025 lbf·in, G = 11.5 Mpsi, θ′ = 0.08°/ft: d = (32T/(πGθ′))^¼ = 4.680 in (Juvinall rounds the constant to 4.6)"
inputs = { d = "5 in", d_i = "0 in", T = "63025 lbf·in", L = "1 ft", G = "11.5 Mpsi", "θ_all′" = "0.08 °/ft" }
expect = { "d_θ" = "4.680 in" }
tolerance = "0.05%"

[[tests]]
name = "Juvinall Problem 17.19: 708 rev/min at 2 in, wanted 1593 rev/min"
source = "Lacewing Lab shaft_design_test, after Juvinall 5th Problem 17.19 (Eq. 17.5): d = 2 √(1593/708) = 3.00 in"
inputs = { d = "2 in", d_i = "0 in", "ω_c" = "708 rpm", "ω_req" = "1593 rpm", T = "100 lbf·in" }
expect = { "d_ω" = "3.0 in" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: hollow steel shaft, 500 N·m over 0.8 m"
source = "By hand: d 40 mm, bore 20 mm, J = π(40⁴ − 20⁴)/32 = 235 619.45 mm⁴; G 79.3 GPa: θ = 500 × 0.8/(79.3e9 × 2.3561945e-7) = 0.021407979 rad (1.2266°), 0.0267600 rad/m over 0.8 m, above 0.08°/ft = 0.00458092 rad/m; d for that twist at k = 0.5: (32 × 500/(π × 79.3e9 × 0.00458092 × (1 − 0.0625)))^¼ = 62.186 mm. Deflection 0.12 mm against 0.10 mm with n_d 1.5: 40 (1.8)^¼ = 46.3317 mm; slope 0.0001 against 0.00008: 40 (1.875)^¼ = 46.8069 mm"
inputs = { d = "40 mm", d_i = "20 mm", T = "500 N·m", L = "0.8 m", G = "79.3 GPa", "θ_all′" = "0.08 °/ft", n_d = "1.5", y = "0.12 mm", y_all = "0.1 mm", s = "0.0001", s_all = "0.00008", "ω_c" = "1000 rpm", "ω_req" = "1000 rpm" }
expect = { J = "235619.45 mm^4", "θ" = "0.021407979", "θ′" = "0.026759974 1/m", "d_θ" = "62.18610 mm", d_y = "46.33169 mm", d_s = "46.80695 mm", "d_ω" = "40 mm" }
tolerance = "1e-6"
checks = "fail"
+++

## Shaft design: stiffness

Stiffness often sizes a shaft before stress does: gears mesh badly and bearings wear when the shaft deflects or tilts, and a shaft that twists too much makes a poor drive. Shigley Sec. 7-5 finds the deflections and slopes with a beam analysis at a trial geometry and then, because they vary as 1/d⁴, scales every diameter of the shaft by the same factor to bring the worst one to its allowable (Eqs. 7-17, 7-18). The twist of a uniform segment is T L/(G J) (Eq. 7-20 for one segment). Juvinall Eq. 17.5 scales a diameter the same way for a wanted first critical speed, which for a uniform shaft varies as d.

Typical allowables (Shigley Table 7-2): slopes of 0.0005 to 0.0012 rad at tapered roller bearings, 0.001 to 0.003 rad at deep-groove ball bearings, under 0.0005 rad at uncrowned spur gears; deflections of 0.003 to 0.010 in at spur gears, finer pitches needing less. Juvinall's historical twist limit is 0.08° per foot.

### Shaft

```calc {description="Shaft diameter (the present, trial diameter)"}
d = 40 mm_
```

```calc {description="Bore of a hollow shaft (0 for solid); its ratio to d is held when resizing"}
d_i = 0 mm_
```

### Twist

```calc {description="Torque carried by the segment"}
T = 500 N·m_
```

```calc {description="Length of the uniform segment"}
L = 0.8 m_
```

```calc {description="Shear modulus (steel 79.3 GPa, 11.5 Mpsi)"}
G = 79.3 GPa_
```

```calc {description="Allowable twist per length (Juvinall: 0.08° per foot)"}
θ_all′ = 0.08 °/ft_
```

```calc
> Polar second moment of area
J = π (d^4 - d_i^4) / 32 = ?? mm^4
> Angle of twist, Eq. 7-20 for one segment
θ = T L / (G J) = ?? deg
> Twist per length
θ′ = θ / L = ?? deg/m
> Diameter for the allowable twist per length, bore ratio held
d_θ = (32 abs(T) / (π G θ_all′ (1 - (d_i / d)^4)))^(1/4) = ?? mm
```

### Deflection and slope

The largest deflection and slope found at the trial diameter d by a beam analysis, and their allowables. The design factor n_d multiplies the deflection (Eq. 7-17).

```calc {description="Design factor on deflection and slope"}
n_d = 1_
```

```calc {description="Deflection found at the trial diameter"}
y = 0.001 in_
```

```calc {description="Allowable deflection, Shigley Table 7-2"}
y_all = 0.005 in_
```

```calc {description="Slope found at the trial diameter (rad)"}
s = 0.0005_
```

```calc {description="Allowable slope (rad), Shigley Table 7-2"}
s_all = 0.001_
```

```calc
> Diameter for the allowable deflection, Eq. 7-17
d_y = d abs(n_d y / y_all)^(1/4) = ?? mm
> Diameter for the allowable slope, Eq. 7-18
d_s = d abs(n_d s / s_all)^(1/4) = ?? mm
```

### Critical speed

```calc {description="First critical speed found at the trial diameter"}
ω_c = 3000 rpm_
```

```calc {description="First critical speed wanted"}
ω_req = 3000 rpm_
```

```calc {description="Diameter for the wanted critical speed, Juvinall Eq. 17.5"}
d_ω = d sqrt(ω_req / ω_c) = ?? mm
```

### Checks

```calc
@test θ′ ≤ θ_all′
```

```calc
@test n_d abs(y) ≤ y_all
```

```calc
@test n_d abs(s) ≤ s_all
```
