+++
cruncha-template = 1
id = "mechanical.toothThickness.overPins"
version = "1.0.0"
title = "Gear tooth thickness: measurement over pins or balls"
summary = "The measurement over two pins (or balls, for a helical gear) of an external spur or helical gear, AGMA 2002 Sec. 6: the best pin size, which touches the profile half way up the working depth, the pressure angle at the pin centre, the radius over one pin and the measurement over two, with the pins as nearly opposite as they can be for an odd number of teeth."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "spur gear", "helical gear", "tooth thickness", "measurement over pins", "measurement over wires", "over balls", "best pin", "involute", "inspection", "AGMA 2002", "Machinery's Handbook"]
from = "lacewing:mechanical.toothThickness"
license = "GPL-3.0-or-later"
inputs = ["m_n", "φ_n", "ψ", "N", "x", "k_a", "W"]
outputs = ["m_t", "φ_t", "ψ_b", "d", "d_b", "d_a", "s_n", "t_b", "W_best", "inv_2", "φ_2", "r_1", "M"]

[[references]]
work = "ANSI/AGMA 2002-B88, Tooth Thickness Specification and Measurement"
edition = "1988"
locator = "Sec. 4 (Eq. 4.11, base tooth thickness); Sec. 6, Eqs. 6.1 to 6.19 (best pin, measurement over pins and balls)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 13-5 and 13-10: Eqs. 13-2, 13-6, 13-18, 13-19; Table 13-1"

[[references]]
work = "Machinery's Handbook (Oberg, Jones, Horton, Ryffel)"
edition = "26th"
locator = "MSec. 4 (Eq. 4.11, base tooth thickness); Sec. 6, Eqs. 6.1 to 6.19 (best pin, measurement over pins and balls)"

[[tests]]
name = "Machinery's Table 1: 12 teeth, 1 diametral pitch, 20°, over 1.728 in wires"
source = "Machinery's Handbook 26th, measurement over wires of external spur gears, 20° (Table 1), also Lacewing Lab geometry_test: 14.3578 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "12", x = "0", k_a = "1", W = "1.728 in" }
expect = { M = "14.3578 in" }
tolerance = "4e-5"
checks = "pass"

[[tests]]
name = "Machinery's Table 1: 20 teeth over 1.728 in wires"
source = "Machinery's Handbook 26th, Table 1 (also Lacewing Lab geometry_test): 22.3900 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "20", x = "0", k_a = "1", W = "1.728 in" }
expect = { M = "22.3900 in" }
tolerance = "4e-5"
checks = "pass"

[[tests]]
name = "Machinery's Table 1: 40 teeth over 1.728 in wires"
source = "Machinery's Handbook 26th, Table 1 (also Lacewing Lab geometry_test): 42.4217 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "40", x = "0", k_a = "1", W = "1.728 in" }
expect = { M = "42.4217 in" }
tolerance = "4e-5"
checks = "pass"

[[tests]]
name = "Machinery's Table 1: 90 teeth over 1.728 in wires"
source = "Machinery's Handbook 26th, Table 1 (also Lacewing Lab geometry_test): 92.4437 in"
inputs = { m_n = "1 in", "φ_n" = "20°", "ψ" = "0°", N = "90", x = "0", k_a = "1", W = "1.728 in" }
expect = { M = "92.4437 in" }
tolerance = "4e-5"
checks = "pass"

[[tests]]
name = "Hand check: odd 17-tooth spur gear, P 10, 0.1728 in pins"
source = "By hand: d = 1.7 in, db = 1.5974775 in, tb = db(π/34 + inv 20°) = 0.1714168 in, inv φ2 = (tb + 0.1728 − π db/17)/db = 0.0306751, φ2 = 25.182865°, M = db cos(90°/17)/cos φ2 + W = 1.9305272 in, r1 = db/(2 cos φ2) + W/2 = 0.9690288 in; best pin 0.1734892 in"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N = "17", x = "0", k_a = "1", W = "0.1728 in" }
expect = { inv_2 = "0.030675141", "φ_2" = "25.182865°", M = "1.9305272 in", r_1 = "0.96902876 in", W_best = "0.17348925 in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: helical 17 teeth, Pn 10, ψ 30°, over 0.18 in balls"
source = "By hand: d = 1.9629909 in, db = 1.8096637 in, ψb = 28.024°, tb = 0.2077735 in, Wt = 0.18/cos ψb = 0.2039 in, inv φ2 = 0.0426914, φ2 = 27.933774°, M = db cos(90°/17)/cos φ2 + W = 2.2195757 in; best ball 0.1812326 in"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "30°", N = "17", x = "0", k_a = "1", W = "0.18 in" }
expect = { "φ_2" = "27.933774°", M = "2.2195757 in", W_best = "0.18123264 in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: a 0.13 in pin on the 17-tooth P 10 gear sits below the tips"
source = "By hand: inv φ2 = 0.0038829, φ2 = 12.899781°, r1 = 0.8844192 in; the pin's top at r1 + W/2 = 0.9494192 in is under the 0.95 in tip radius, so a micrometer can't reach it"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N = "17", x = "0", k_a = "1", W = "0.13 in" }
expect = { "φ_2" = "12.899781°", r_1 = "0.88441919 in" }
tolerance = "1e-6"
checks = "fail"
+++

## Gear tooth thickness: measurement over pins or balls

Two pins (or, for a helical gear, two balls) are set in opposite tooth spaces and the distance over them is measured with a micrometer (AGMA 2002 Sec. 6). The measurement over pins reads back the tooth thickness: a thinner tooth lets the pins sink deeper. With an even number of teeth the spaces are diametrically opposite; with an odd number they are as nearly opposite as they can be, half a pitch off, and the measurement is reduced by cos(90°/N). Use the pin sizes you have: the best pin, which touches the profile half way up the working depth, is given for choosing one.

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

### Pin or ball

The best pin (AGMA 2002 Eqs. 6.1 to 6.9) touches the flanks on the circle a standard addendum below the tip. A pin much smaller than that may sit on the root fillet or below the tips; much larger, on the tip corners. The pin's centre lies on the involute of a pressure angle φ_2 whose involute is set by the base thickness, the pin size and the base pitch (AGMA 2002 Eq. 6.10 and on). A helical gear is measured over balls, and the ball diameter is taken in the transverse plane as W/cos ψ_b.

```calc {description="Pin or ball diameter"}
W = 3.5 mm_
```

```calc
> Diameter a standard addendum below the tip
d_W = d_a - 2 k_a m_n = ?? mm
> Pressure angle there
φ_W = acos(d_b / d_W) = ?? °
> Space width there (AGMA 2002 Eq. 4.8)
e_W = π d_W / N - d_W (t_b / d_b - tan(φ_W) + φ_W) = ?? mm
> Best pin or ball diameter, AGMA 2002 Eqs. 6.1 to 6.9 (normal plane)
W_best = d_W sin(e_W / d_W) / cos(e_W / d_W + φ_W) cos(ψ_b) = ?? mm
```

```calc
function invInverse(x)
    φ = (3 x)^(1 / 3)
    for k in 1:40
        s = (tan(φ) - φ - x) / tan(φ)^2
        if abs(s) > 1e-15
            φ = φ - s
        end
    end
    return φ
end
```

```calc
> Involute of the pressure angle at the pin centre
inv_2 = (t_b + W / cos(ψ_b) - π d_b / N) / d_b = ??
> Pressure angle at the pin centre
φ_2 = invInverse(inv_2) = ?? °
> Radius over one pin
r_1 = d_b / (2 cos(φ_2)) + W / 2 = ?? mm
> Measurement over two pins (odd tooth counts: the nearest-opposite spaces)
M = {d_b / cos(φ_2) + W if mod(N, 2) == 0; d_b cos(π / (2 N)) / cos(φ_2) + W otherwise} = ?? mm
```

### Checks

The pin centre must lie outside the base circle, and the pins must stand proud of the tips so the micrometer touches them and not the teeth.

```calc
@test inv_2 > 0
```

```calc
@test r_1 + W / 2 > d_a / 2
```
