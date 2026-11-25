+++
cruncha-template = 1
id = "mechanical.toothThickness.span"
version = "1.0.0"
title = "Gear tooth thickness: span (base tangent length)"
summary = "The span measurement (base tangent length, Wildhaber) of an external spur or helical gear over a number of teeth, AGMA 2002 Sec. 7: the span in the normal plane, the best number of teeth to span, and the radius at which the anvils touch the flanks, checked to lie below the tips."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "spur gear", "helical gear", "tooth thickness", "span measurement", "base tangent length", "Wildhaber", "chordal", "inspection", "AGMA 2002", "Machinery's Handbook"]
from = "lacewing:mechanical.toothThickness"
license = "GPL-3.0-or-later"
inputs = ["m_n", "φ_n", "ψ", "N", "x", "k_a", "S"]
outputs = ["m_t", "φ_t", "ψ_b", "d", "d_b", "d_a", "s_n", "t_b", "p_b", "S_best", "M_s", "r_c"]

[[references]]
work = "ANSI/AGMA 2002-B88, Tooth Thickness Specification and Measurement"
edition = "1988"
locator = "Sec. 4 (Eq. 4.11, base tooth thickness); Sec. 7, Eqs. 7.7, 7.8 and 7.10 (best number of teeth, span of external gears)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 13-5 and 13-10: Eqs. 13-2, 13-6, 13-18, 13-19; Table 13-1"

[[references]]
work = "Machinery's Handbook (Oberg, Jones, Horton, Ryffel)"
edition = "26th"
locator = "MSec. 4 (Eq. 4.11, base tooth thickness); Sec. 7, Eqs. 7.7, 7.8 and 7.10 (best number of teeth, span of external gears)"

[[tests]]
name = "Machinery's Table 10: 12 teeth over 2, 1 diametral pitch, 20°"
source = "Machinery's Handbook 26th, span measurement of 20° spur teeth (Table 10), also Lacewing Lab geometry_test: 4.5963 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "12", x = "0", k_a = "1", S = "2" }
expect = { M_s = "4.5963 in", S_best = "2" }
tolerance = "2e-5"
checks = "pass"

[[tests]]
name = "Machinery's Table 10: 18 teeth over 2"
source = "Machinery's Handbook 26th, Table 10 (also Lacewing Lab geometry_test): 4.6803 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "18", x = "0", k_a = "1", S = "2" }
expect = { M_s = "4.6803 in" }
tolerance = "2e-5"
checks = "pass"

[[tests]]
name = "Machinery's Table 10: 19 teeth over 3"
source = "Machinery's Handbook 26th, Table 10 (also Lacewing Lab geometry_test): 7.6464 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "19", x = "0", k_a = "1", S = "3" }
expect = { M_s = "7.6464 in", S_best = "3" }
tolerance = "2e-5"
checks = "pass"

[[tests]]
name = "Machinery's Table 10: 28 teeth over 4"
source = "Machinery's Handbook 26th, Table 10 (also Lacewing Lab geometry_test): 10.7246 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "28", x = "0", k_a = "1", S = "4" }
expect = { M_s = "10.7246 in", S_best = "4" }
tolerance = "2e-5"
checks = "pass"

[[tests]]
name = "Machinery's Table 10: 81 teeth over 9"
source = "Machinery's Handbook 26th, Table 10 (also Lacewing Lab geometry_test): 26.2275 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "81", x = "0", k_a = "1", S = "9" }
expect = { M_s = "26.2275 in" }
tolerance = "2e-5"
checks = "pass"

[[tests]]
name = "Hand check: helical 40 teeth, mn 2 mm, ψ 15°, shift +0.2, over 6 teeth"
source = "By hand: d = 82.822094 mm, db = 77.502534 mm, ψb = 14.0761°, tb = 4.6007830 mm, pb = π db/40 = 6.0870697 mm, Ms = (5 pb + tb) cos ψb = 33.983952 mm, contact radius √(rb² + (Ms/(2 cos ψb))²) = 42.526936 mm under the 43.811047 mm tip; best span 6 teeth"
inputs = { m_n = "2 mm", "φ_n" = "20°", "ψ" = "15°", N = "40", x = "0.2", k_a = "1", S = "6" }
expect = { t_b = "4.6007830 mm", M_s = "33.983952 mm", r_c = "42.526936 mm", S_best = "6" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: 20-tooth spur gear, 2 mm, spanned over 6 teeth — the anvils reach past the tips"
source = "By hand: Ms = (5 pb + tb) = 33.033667 mm; the anvils would touch at radius 25.020285 mm, beyond the 22 mm tip radius; the best span is 3 teeth"
inputs = { m_n = "2 mm", "φ_n" = "20°", "ψ" = "0°", N = "20", x = "0", k_a = "1", S = "6" }
expect = { M_s = "33.033667 mm", S_best = "3" }
tolerance = "1e-6"
checks = "fail"
+++

## Gear tooth thickness: span (base tangent length)

A disc micrometer spanning S teeth touches two opposite flanks along a line tangent to the base cylinder, so the span is S − 1 base pitches plus one base tooth thickness, measured in the normal plane for a helical gear (AGMA 2002 Sec. 7). It needs no pins and doesn't depend on the outside diameter. The best number of teeth to span puts the contact near the pitch circle (Eqs. 7.7 and 7.8); for a standard spur gear it is about N·φ_n/180° + 0.5, and where that lands on a half (18 or 81 teeth at 20°) the two neighbouring counts are equally good (Machinery's tables take the lower). A helical gear must be wide enough for the micrometer to reach across the span at the base helix angle, roughly M_s·sin ψ_b plus the anvil width.

### The gear

A spur or helical external gear cut by a standard rack of normal module m_n (1/P_n for a diametral pitch) and normal pressure angle φ_n, with profile shift x (0 for standard teeth). The tooth thickness is the theoretical one on the standard pitch circle, half the normal pitch plus 2x·m_n·tan φ_n (Machinery's p. 2015), carried to the base circle by the involute (AGMA 2002 Eq. 4.11). To put the measurement on a drawing, take the thinned tooth (after the backlash allowance and tolerance) instead.

```calc {description="Normal module (1/Pn for a diametral pitch)"}
m_n = 2 mm_
```

```calc {description="Normal pressure angle of the generating rack"}
φ_n = 20°_
```

```calc {description="Helix angle at the pitch cylinder (0 for spur gears)"}
ψ = 0°_
```

```calc {description="Number of teeth"}
N = 40_
```

```calc {description="Profile shift coefficient (0 for standard teeth)"}
x = 0_
```

```calc {description="Addendum per module, Shigley Table 13-1"}
k_a = 1_
```

```calc
> Transverse module, Eq. 13-18
m_t = m_n / cos(ψ) = ?? mm
> Transverse pressure angle, Eq. 13-19
φ_t = atan(tan(φ_n) / cos(ψ)) = ?? °
> Base helix angle
ψ_b = atan(tan(ψ) cos(φ_t)) = ?? °
> Pitch diameter, Eq. 13-2
d = N m_t = ?? mm
> Base diameter, Eq. 13-6
d_b = d cos(φ_t) = ?? mm
> Tip (outside) diameter
d_a = d + 2 (k_a + x) m_n = ?? mm
> Normal tooth thickness on the pitch circle (Machinery's p. 2015)
s_n = m_n (π / 2 + 2 x tan(φ_n)) = ?? mm
> Transverse tooth thickness at the base circle, AGMA 2002 Eq. 4.11
t_b = d_b (s_n / (cos(ψ) d) + tan(φ_t) - φ_t) = ?? mm
```

### Span

```calc {description="Number of teeth spanned"}
S = 5_
```

```calc
> Transverse base pitch
p_b = π d_b / N = ?? mm
> Best number of teeth to span, AGMA 2002 Eqs. 7.7 and 7.8
S_best = max(2, round((sqrt((d_a - 2 k_a m_n)^2 - d_b^2) / cos(ψ_b) - t_b cos(ψ_b)) / (p_b cos(ψ_b)) + 1)) = ??
> Span over S teeth in the normal plane, AGMA 2002 Eq. 7.10
M_s = ((S - 1) p_b + t_b) cos(ψ_b) = ?? mm
> Radius at which the anvils touch the flanks
r_c = sqrt((d_b / 2)^2 + (M_s / (2 cos(ψ_b)))^2) = ?? mm
```

### Check

The anvils must touch the flanks below the tips (and, though not checked here, above the root fillet's start of the active profile).

```calc
@test r_c < d_a / 2
```
