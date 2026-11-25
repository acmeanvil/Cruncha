+++
cruncha-template = 1
id = "mechanical.couplings.twoJoints"
version = "1.0.0"
title = "Two Hooke's joints on an intermediate shaft: output speed fluctuation"
summary = "Two Hooke's (Cardan) universal joints on an intermediate shaft, the three shafts in one plane: the greatest and least output speed over a turn of the input, sampled every half degree, for any joint angles and any phase error of the intermediate shaft's yokes. Equal angles with aligned yokes give a uniform output speed."
category = ["Mechanical engineering", "Shafts"]
tags = ["universal joint", "Hooke's joint", "Cardan joint", "double Cardan", "intermediate shaft", "propeller shaft", "drive shaft", "coupling", "speed fluctuation", "phasing", "Marks", "Machinery's Handbook", "Juvinall"]
from = "lacewing:mechanical.couplings"
license = "GPL-3.0-or-later"
inputs = ["u_1", "u_2", "φ", "ω_1"]
outputs = ["r_max", "r_min", "ω_(3,max)", "ω_(3,min)", "Δω_3", "Δ_r"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "p. 8-36 (Hooke's coupling velocity ratio)"

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "Universal joints, pp. 2327-2328 (two joints on an intermediate shaft)"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.7 (two joints at equal angles with the intermediate yokes aligned cancel the fluctuation)"

[[tests]]
name = "Equal 23° angles, yokes aligned: uniform output"
source = "Lacewing Lab fits_couplings_test, after Juvinall 5th Sec. 17.7 and Machinery's 26th p. 2328: equal joint angles in one plane with the intermediate yokes aligned give ω3 = ω1 throughout"
inputs = { u_1 = "23°", u_2 = "23°", "φ" = "0°", "ω_1" = "1000 rpm" }
expect = { r_max = "1", r_min = "1", "ω_(3,max)" = "1000 rpm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Equal 20° angles with the yokes 90° out of phase: the fluctuations add"
source = "Lacewing Lab fits_couplings_test (phased yokes add the fluctuations). By hand: at a = 0 the first joint gives sec 20° and drives the second at 90° from its plane, which adds another sec 20°: greatest ratio sec²20° = 1.1324743, least cos²20° = 0.8830222, twice the single joint's range (0.2494521)"
inputs = { u_1 = "20°", u_2 = "20°", "φ" = "90°", "ω_1" = "1000 rpm" }
expect = { r_max = "1.1324743", r_min = "0.8830222", "Δ_r" = "0.2494521", "ω_(3,max)" = "1132.4743 rpm" }
tolerance = "1e-7"
checks = "fail"

[[tests]]
name = "Unequal angles 20° and 12°, yokes aligned"
source = "By hand: at a = 0 the ratio is sec 20° cos 12° = 1.0409229, at a = 90° cos 20° sec 12° = 0.9606859; a second joint of 0° leaves the single joint's sec 20° = 1.0641778 and cos 20° = 0.9396926 (checked in the same sampling); at 600 rev/min the output swings 48.1422 rev/min"
inputs = { u_1 = "20°", u_2 = "12°", "φ" = "0°", "ω_1" = "600 rpm" }
expect = { r_max = "1.0409229", r_min = "0.9606859", "Δω_3" = "48.142213 rpm" }
tolerance = "1e-6"
checks = "fail"
+++

## Two Hooke's joints on an intermediate shaft

A single Hooke's joint makes the driven shaft's speed fluctuate twice per turn. Two joints on an intermediate shaft, with the input, intermediate and output shafts in one plane, cancel each other when the two joint angles are equal and the yokes on the intermediate shaft lie in the same plane (Juvinall Sec. 17.7, Machinery's). Unequal angles, or yokes turned out of phase on the intermediate shaft, leave a fluctuation; yokes 90° out of phase add the two joints' fluctuations.

Each joint follows Marks' ratio cos u/(1 − sin²u cos²a), with a measured from the position where the driving yoke's pins lie in the plane of the shafts. The intermediate shaft turns to θ (tan θ = tan a/cos u₁); its second yoke is at 90° plus the phase error φ from the first, so the second joint sees the angle θ + 90° + φ. The greatest and least output speeds are found by sampling a turn of the input every half degree: exact for aligned or 90° yokes, whose extremes fall at multiples of 90°, and within a small fraction of the fluctuation otherwise.

```calc {description="Angle of the first joint (input to intermediate shaft)"}
u_1 = 20°_
```

```calc {description="Angle of the second joint (intermediate to output shaft)"}
u_2 = 20°_
```

```calc {description="Phase error of the intermediate shaft's yokes (0 when aligned)"}
φ = 0°_
```

```calc {description="Speed of the input shaft"}
ω_1 = 1000 rpm_
```

```calc
function hookeRatio(u, a)
    return cos(u) / (1 - sin(u)^2 cos(a)^2)
end
```

```calc
function hookeAngle(u, a)
    t = atan2(cos(a), sin(a) / cos(u))
    return t + 2 π round((a - t) / (2 π))
end
```

```calc
function twoJointExtremes(v, w, ph)
    hi = 0
    lo = 1e9
    for k in 0:719
        a = π k / 360
        r = hookeRatio(v, a) hookeRatio(w, hookeAngle(v, a) + π / 2 + ph)
        hi = max(hi, r)
        lo = min(lo, r)
    end
    return [hi, lo]
end
```

```calc
> Greatest and least ratio over a turn, sampled
r_ext = twoJointExtremes(u_1, u_2, φ) = ??
> Greatest output-to-input speed ratio
r_max = r_ext[1] = ??
> Least output-to-input speed ratio
r_min = r_ext[2] = ??
> Spread of the ratio
Δ_r = r_max - r_min = ??
> Greatest output speed
ω_(3,max) = r_max ω_1 = ?? rpm
> Least output speed
ω_(3,min) = r_min ω_1 = ?? rpm
> Fluctuation of the output speed
Δω_3 = ω_(3,max) - ω_(3,min) = ?? rpm
```

### Check

A uniform output needs equal joint angles and aligned yokes; the check allows a ratio spread of 0.1 %.

```calc
@test Δ_r ≤ 0.001
```
