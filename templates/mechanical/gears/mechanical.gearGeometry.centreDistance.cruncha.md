+++
cruncha-template = 1
id = "mechanical.gearGeometry.centreDistance"
version = "1.0.0"
title = "Spur and helical gear pair at a given centre distance: backlash"
summary = "An external spur or helical pair of involute gears, with or without profile shift, running at a centre distance you set: the operating pressure angle and pitch diameters, the tooth thicknesses carried to the operating pitch circles, the transverse circular backlash (negative when the teeth are too thick for the centres), the contact ratios and interference."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "spur gear", "helical gear", "involute", "backlash", "centre distance", "center distance", "operating pressure angle", "profile shift", "contact ratio", "interference", "AGMA 2002", "Juvinall", "Shigley"]
from = "lacewing:mechanical.gearGeometry"
license = "GPL-3.0-or-later"
inputs = ["m_n", "φ_n", "ψ", "N_P", "N_G", "x_P", "x_G", "k_a", "k_f", "F", "a"]
outputs = ["m_t", "φ_t", "p_b", "ψ_b", "m_G", "d_P", "d_G", "d_(b,P)", "d_(b,G)", "d_(a,P)", "d_(a,G)", "d_(f,P)", "d_(f,G)", "s_(n,P)", "s_(n,G)", "a_0", "φ_w", "d_(w,P)", "d_(w,G)", "t_(w,P)", "t_(w,G)", "B_t", "Z", "m_p", "m_F", "m_N", "r_(a,max,P)", "r_(a,max,G)", "N_(P,min)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 13-5 to 13-10: Eqs. 13-2, 13-6, 13-7, 13-9, 13-11, 13-18, 13-19, 13-22, Table 13-1; Eqs. 14-19, 14-21, 14-24, 14-25"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 15.3, Eqs. 15.8 and 15.9; Appendix J, Tables J.1 and J.2"

[[references]]
work = "ANSI/AGMA 2002-B88, Tooth Thickness Specification and Measurement"
edition = "1988"
locator = "Sec. 4: Eq. 4.8 (tooth thickness at another diameter), Eq. 4.3 (backlash)"

[[references]]
work = "Machinery's Handbook (Oberg, Jones, Horton, Ryffel)"
edition = "26th"
locator = "p. 2015 (tooth thickness of enlarged teeth)"

[[tests]]
name = "Juvinall Table J.2: 16/32 teeth, P 2, 20°, centres opened 0.05 in to 12.05 in"
source = "Juvinall 5th Appendix J, Table J.2 (also Lacewing Lab geometry_test): operating pressure angle 20.64°, contact ratio 1.49, largest gear tip radius 8.63 in. The table's backlash 2(c̄ − c) tan φ̄ = 0.0377 in is the small-opening approximation; the exact circular backlash on the operating pitch circles (AGMA 2002 Eq. 4.8) is 0.0371 in (Lacewing Lab)"
inputs = { m_n = "0.5 in", "φ_n" = "20°", "ψ" = "0°", N_P = "16", N_G = "32", F = "4 in", a = "12.05 in" }
expect = { "φ_w" = "20.64°", m_p = "1.49", B_t = "0.0371 in", "r_(a,max,G)" = "8.63 in" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Standard 2 mm spur pair 17/52 at its standard centre distance, 69 mm"
source = "Lacewing Lab geometry_test: standard teeth at the standard centre distance run at the standard pressure angle with no backlash"
inputs = { m_n = "2 mm", "φ_n" = "20°", "ψ" = "0°", N_P = "17", N_G = "52", F = "20 mm", a = "69 mm" }
expect = { "φ_w" = "20°", "d_(w,P)" = "34 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: the 2 mm pair opened to 69.1 mm"
source = "By hand: φw = acos(69 cos 20°/69.1) = 20.226583°, dwP = 2 × 69.1 × 17/69 = 34.049275 mm, twP = dwP(π/34 + inv 20° − inv φw) = 3.1280872 mm, twG = 3.0909079 mm, Bt = π dwP/17 − twP − twG = 0.0732963 mm (about 2Δa tan φw = 0.0737 mm), Z = 9.3809916 mm, mp = 1.5888506"
inputs = { m_n = "2 mm", "φ_n" = "20°", "ψ" = "0°", N_P = "17", N_G = "52", F = "20 mm", a = "69.1 mm" }
expect = { "φ_w" = "20.226583°", "d_(w,P)" = "34.049275 mm", "t_(w,P)" = "3.1280872 mm", "t_(w,G)" = "3.0909079 mm", B_t = "0.0732963 mm", Z = "9.3809916 mm", m_p = "1.5888506" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: the 2 mm pair closed to 68.95 mm — the teeth interfere"
source = "By hand: φw = 19.885531°, twP = 3.1482525 mm, twG = 3.1666507 mm, Bt = π × 33.975362/17 − twP − twG = −0.0362709 mm"
inputs = { m_n = "2 mm", "φ_n" = "20°", "ψ" = "0°", N_P = "17", N_G = "52", F = "20 mm", a = "68.95 mm" }
expect = { "φ_w" = "19.885531°", B_t = "-0.0362709 mm" }
tolerance = "1e-5"
checks = "fail"
+++

## Spur and helical gear pair at a given centre distance

An external pair of involute spur or helical gears on parallel shafts, cut by a standard rack and perhaps profile-shifted, mounted at an operating centre distance a that you set (Shigley Secs. 13-5 to 13-10). The base circles don't change with the centres, so the pressure angle and pitch circles the pair runs on do. The tooth thicknesses are carried from the standard pitch circle to the operating pitch circles by the involute (AGMA 2002 Eq. 4.8), and what is left of the pitch is the transverse circular backlash. Opening the centres by Δa adds about 2Δa·tan φ_w; a negative backlash means the teeth are too thick for the centres and will bind. The tooth thicknesses here have no backlash allowance of their own; for the zero-backlash centre distance use the tight-mesh geometry template.

### Tooth size and teeth

The tooth size is the normal module m_n, the size of the hob or rack that cuts the teeth; for a spur gear it is the ordinary module. With a diametral pitch P, enter m_n = 1/P (0.1 in for P = 10). The tooth system sets the addendum and dedendum per module (Shigley Table 13-1): 1 and 1.25 for full depth, 1 and 1.35 for deep full depth, 0.8 and 1 for stub teeth.

```calc {description="Normal module (1/Pn for a diametral pitch)"}
m_n = 0.1 in_
```

```calc {description="Normal pressure angle of the generating rack"}
φ_n = 20°_
```

```calc {description="Helix angle at the pitch cylinder (0 for spur gears)"}
ψ = 30°_
```

```calc {description="Pinion teeth"}
N_P = 17_
```

```calc {description="Gear teeth"}
N_G = 52_
```

```calc {description="Pinion profile shift coefficient (0 for standard teeth)"}
x_P = 0_
```

```calc {description="Gear profile shift coefficient (0 for standard teeth)"}
x_G = 0_
```

```calc {description="Addendum per module, Shigley Table 13-1"}
k_a = 1_
```

```calc {description="Dedendum per module, Shigley Table 13-1"}
k_f = 1.25_
```

```calc {description="Face width"}
F = 1.5 in_
```

### Pitches

```calc
> Transverse module, Eq. 13-18
m_t = m_n / cos(ψ) = ?? in
> Transverse pressure angle, Eq. 13-19
φ_t = atan(tan(φ_n) / cos(ψ)) = ?? °
> Transverse base pitch, Eq. 13-7
p_b = π m_t cos(φ_t) = ?? in
> Normal base pitch, Eq. 14-24
p_N = π m_n cos(φ_n) = ?? in
> Base helix angle
ψ_b = atan(tan(ψ) cos(φ_t)) = ?? °
> Gear ratio
m_G = N_G / N_P = ??
```

### Diameters and tooth thickness

The normal tooth thickness on the standard pitch circle is half the normal pitch plus 2x·m_n·tan φ_n for the shift (Machinery's p. 2015).

```calc
> Pitch diameters, Eq. 13-2
d_P = N_P m_t = ?? in
d_G = N_G m_t = ?? in
> Base diameters, Eq. 13-6
d_(b,P) = d_P cos(φ_t) = ?? in
d_(b,G) = d_G cos(φ_t) = ?? in
> Tip (outside) diameters
d_(a,P) = d_P + 2 (k_a + x_P) m_n = ?? in
d_(a,G) = d_G + 2 (k_a + x_G) m_n = ?? in
> Root diameters
d_(f,P) = d_P - 2 (k_f - x_P) m_n = ?? in
d_(f,G) = d_G - 2 (k_f - x_G) m_n = ?? in
> Normal tooth thicknesses on the standard pitch circle
s_(n,P) = m_n (π / 2 + 2 x_P tan(φ_n)) = ?? in
s_(n,G) = m_n (π / 2 + 2 x_G tan(φ_n)) = ?? in
```

### Operating centre distance

```calc {description="Operating centre distance"}
a = 3.99 in_
```

```calc
> Standard centre distance
a_0 = (N_P + N_G) m_t / 2 = ?? in
> Operating transverse pressure angle
φ_w = acos(a_0 cos(φ_t) / a) = ?? °
> Operating pitch diameters
d_(w,P) = 2 a N_P / (N_P + N_G) = ?? in
d_(w,G) = d_(w,P) m_G = ?? in
> Transverse tooth thicknesses on the operating pitch circles, AGMA 2002 Eq. 4.8
t_(w,P) = d_(w,P) (s_(n,P) / (cos(ψ) d_P) + tan(φ_t) - φ_t - tan(φ_w) + φ_w) = ?? in
t_(w,G) = d_(w,G) (s_(n,G) / (cos(ψ) d_G) + tan(φ_t) - φ_t - tan(φ_w) + φ_w) = ?? in
> Transverse circular backlash on the operating pitch circles
B_t = π d_(w,P) / N_P - t_(w,P) - t_(w,G) = ?? in
```

### Contact ratios and load sharing

The length of the line of action in the transverse plane is Shigley's Eq. 14-25 at the operating centre distance; the transverse contact ratio is that length over the base pitch, and Shigley recommends at least 1.2. A helical pair also has the face contact ratio F/p_x (Eq. 14-19): under 1 it is a low-contact-ratio helical, which Shigley ch. 14 does not rate. The load-sharing ratio m_N (Eq. 14-21) goes into the AGMA pitting factor I; it is 1 for spur gears.

```calc
> Length of the line of action, Eq. 14-25
Z = sqrt((d_(a,P) / 2)^2 - (d_(b,P) / 2)^2) + sqrt((d_(a,G) / 2)^2 - (d_(b,G) / 2)^2) - a sin(φ_w) = ?? in
> Transverse contact ratio, Eq. 13-9
m_p = Z / p_b = ??
> Face contact ratio, Eq. 14-19 (F/p_x)
m_F = F sin(ψ) / (π m_n) = ??
> Load-sharing ratio, Eq. 14-21
m_N = {1 if ψ == 0; p_N / (0.95 Z) otherwise} = ??
```

### Interference

A tip that reaches past the mate's interference point, where the line of action touches the mate's base circle, interferes or is undercut. Juvinall's Eq. 15.8 gives the largest tip radius of each gear that stays inside it at the operating centre distance. Shigley's Eq. 13-22 (13-11 for spur) gives the fewest pinion teeth that mesh with this gear free of interference, for unshifted teeth of addendum k_a·m_n; profile shift moves that limit.

```calc
> Largest pinion tip radius free of interference, Juvinall Eq. 15.8
r_(a,max,P) = sqrt((d_(b,P) / 2)^2 + (a sin(φ_w))^2) = ?? in
> Largest gear tip radius free of interference
r_(a,max,G) = sqrt((d_(b,G) / 2)^2 + (a sin(φ_w))^2) = ?? in
> Fewest pinion teeth without interference, unshifted, Eq. 13-22
N_(P,min) = 2 k_a cos(ψ) / ((1 + 2 m_G) sin(φ_t)^2) (m_G + sqrt(m_G^2 + (1 + 2 m_G) sin(φ_t)^2)) = ??
```

### Checks

The teeth must not bind, must not interfere, and must keep enough of them in contact.

```calc
@test B_t ≥ 0 'in'
```

```calc
@test d_(a,P) / 2 ≤ r_(a,max,P)
```

```calc
@test d_(a,G) / 2 ≤ r_(a,max,G)
```

```calc
@test m_p ≥ 1.2
```

```calc
@test ψ == 0 or m_F ≥ 1
```
