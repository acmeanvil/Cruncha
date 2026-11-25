+++
cruncha-template = 1
id = "mechanical.toothThickness.chordal"
version = "1.0.0"
title = "Gear tooth thickness: chordal thickness and addendum"
summary = "The chordal tooth thickness and chordal addendum of an external spur or helical gear for a gear tooth vernier caliper, AGMA 2002 Sec. 5: the arc thickness at the measuring circle a standard addendum below the tip, its helix angle there, and the straight-line thickness and the caliper's depth setting from the tip."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "spur gear", "helical gear", "tooth thickness", "chordal thickness", "chordal addendum", "gear tooth caliper", "vernier", "inspection", "AGMA 2002", "Machinery's Handbook"]
from = "lacewing:mechanical.toothThickness"
license = "GPL-3.0-or-later"
inputs = ["m_n", "φ_n", "ψ", "N", "x", "k_a"]
outputs = ["m_t", "φ_t", "ψ_b", "d", "d_b", "d_a", "s_n", "t_b", "r_m", "ψ_m", "t_m", "t_c", "a_c"]

[[references]]
work = "ANSI/AGMA 2002-B88, Tooth Thickness Specification and Measurement"
edition = "1988"
locator = "Sec. 4 (Eq. 4.11, base tooth thickness); Sec. 5, Eqs. 5.3 to 5.10 (chordal tooth thickness and chordal addendum)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 13-5 and 13-10: Eqs. 13-2, 13-6, 13-18, 13-19; Table 13-1"

[[references]]
work = "Machinery's Handbook (Oberg, Jones, Horton, Ryffel)"
edition = "26th"
locator = "MSec. 4 (Eq. 4.11, base tooth thickness); Sec. 5, Eqs. 5.3 to 5.10 (chordal tooth thickness and chordal addendum)"

[[tests]]
name = "Machinery's chordal table: 12 teeth, 1 diametral pitch, 20°"
source = "Machinery's Handbook 26th, chordal thickness T and chordal addendum H of gear teeth of 1 diametral pitch (also Lacewing Lab geometry_test): T = 1.5663 in, H = 1.0514 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "12", x = "0", k_a = "1" }
expect = { t_c = "1.5663 in", a_c = "1.0514 in" }
tolerance = "1e-4"

[[tests]]
name = "Machinery's chordal table: 21 teeth"
source = "Machinery's Handbook 26th, chordal thickness table (also Lacewing Lab geometry_test): T = 1.5694 in, H = 1.0294 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "21", x = "0", k_a = "1" }
expect = { t_c = "1.5694 in", a_c = "1.0294 in" }
tolerance = "1e-4"

[[tests]]
name = "Machinery's chordal table: 55 teeth"
source = "Machinery's Handbook 26th, chordal thickness table (also Lacewing Lab geometry_test): T = 1.5706 in, H = 1.0112 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "55", x = "0", k_a = "1" }
expect = { t_c = "1.5706 in", a_c = "1.0112 in" }
tolerance = "1e-4"

[[tests]]
name = "Hand check: helical 40 teeth, mn 2 mm, ψ 15°, shift +0.2"
source = "By hand: da = 87.622094 mm, rm = da/2 − 2 = 41.811047 mm, ψm = atan(2 rm sin 15°/(40 × 2)) = 15.138269°, tm = 2 rm(tb/db − inv(acos(db/(2 rm)))) = 3.2737971 mm, tc = 2 rm cos ψm sin(tm/(2 rm)) = 3.1593840 mm, ac = da/2 − rm cos(tm cos²ψm/(2 rm)) = 2.0278177 mm"
inputs = { m_n = "2 mm", "φ_n" = "20°", "ψ" = "15°", N = "40", x = "0.2", k_a = "1" }
expect = { r_m = "41.811047 mm", "ψ_m" = "15.138269°", t_m = "3.2737971 mm", t_c = "3.1593840 mm", a_c = "2.0278177 mm" }
tolerance = "1e-6"
+++

## Gear tooth thickness: chordal thickness and addendum

A gear tooth vernier caliper measures the straight chord across one tooth at a set depth below the tip (AGMA 2002 Sec. 5). The template takes the measuring circle a standard addendum k_a·m_n below the tip, finds the arc tooth thickness there from the base thickness, and gives the chord (normal to the helix for a helical gear) and the chordal addendum, the depth to set the caliper's tongue from the tip. The reading depends on the actual outside diameter: on a gear whose tips are turned under or over size, or that runs out, use the measured tip diameter for d_a. The caliper is less accurate than pins or span; prefer those where you can.

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

### Chordal thickness

```calc
> Radius of the measuring circle, a standard addendum below the tip
r_m = d_a / 2 - k_a m_n = ?? mm
> Helix angle at the measuring circle
ψ_m = atan(2 r_m sin(ψ) / (N m_n)) = ?? °
> Transverse arc tooth thickness at the measuring circle, AGMA 2002 Eq. 4.8
t_m = 2 r_m (t_b / d_b - tan(acos(d_b / (2 r_m))) + acos(d_b / (2 r_m))) = ?? mm
> Chordal tooth thickness, normal to the helix, AGMA 2002 Sec. 5
t_c = 2 r_m cos(ψ_m) sin(t_m / (2 r_m)) = ?? mm
> Chordal addendum (caliper depth setting from the tip), AGMA 2002 Sec. 5
a_c = d_a / 2 - r_m cos(t_m cos(ψ_m)^2 / (2 r_m)) = ?? mm
```
