+++
cruncha-template = 1
id = "mechanical.gearGeometry.external"
version = "1.0.0"
title = "Spur and helical gear pair geometry, tight mesh"
summary = "An external spur or helical pair of involute gears cut by a standard rack, with or without profile shift: transverse module and pressure angle, pitch, base, tip and root diameters, tooth thicknesses, the tight-mesh (zero-backlash) operating pressure angle and centre distance, the length of the line of action, transverse and face contact ratios, the load-sharing ratio, and interference by Juvinall's largest tip radius and Shigley's fewest pinion teeth."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "spur gear", "helical gear", "involute", "profile shift", "addendum modification", "centre distance", "center distance", "pressure angle", "contact ratio", "line of action", "load sharing", "interference", "undercut", "module", "diametral pitch", "Shigley", "Juvinall", "Machinery's Handbook"]
from = "lacewing:mechanical.gearGeometry"
license = "GPL-3.0-or-later"
inputs = ["m_n", "φ_n", "ψ", "N_P", "N_G", "x_P", "x_G", "k_a", "k_f", "F"]
outputs = ["m_t", "φ_t", "p_b", "ψ_b", "m_G", "d_P", "d_G", "d_(b,P)", "d_(b,G)", "d_(a,P)", "d_(a,G)", "d_(f,P)", "d_(f,G)", "s_(n,P)", "s_(n,G)", "inv_w", "φ_w", "a_0", "a", "Z", "m_p", "m_F", "m_N", "r_(a,max,P)", "r_(a,max,G)", "N_(P,min)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 13-5 to 13-10: Eqs. 13-1, 13-2, 13-6, 13-7 (pitch and base circles), 13-9 (contact ratio), 13-11 and 13-22 (fewest pinion teeth), 13-16 to 13-19 (helical relations), Table 13-1 (tooth systems); Eqs. 14-19, 14-21, 14-24, 14-25; Example 14-5"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 15.3, Eqs. 15.8 and 15.9, Sample Problem 15.1D; Appendix J, Table J.1"

[[references]]
work = "Machinery's Handbook (Oberg, Jones, Horton, Ryffel)"
edition = "26th"
locator = "p. 2015 (tooth thickness of enlarged pinions), p. 2028 (centre distance for no backlash)"

[[references]]
work = "ANSI/AGMA 2002-B88, Tooth Thickness Specification and Measurement"
edition = "1988"
locator = "Sec. 4 (tooth thickness at the base circle, Eq. 4.11)"

[[tests]]
name = "Shigley Ex. 14-5: helical 17/52, Pn 10, φn 20°, ψ 30°, 1.5 in face"
source = "Shigley 10th Example 14-5 (also Lacewing Lab geometry_test): φt = 22.80°, dP = 1.963 in, Z = 0.4501 in, mN = 0.6903"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "30°", N_P = "17", N_G = "52", F = "1.5 in" }
expect = { "φ_t" = "22.796°", d_P = "1.963 in", Z = "0.4501 in", m_N = "0.6903" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 15.1D: standard 12/36 teeth, P 6, 20° — the gear interferes"
source = "Lacewing Lab geometry_test, after Juvinall 5th Sample Problem 15.1D: at c = 4 in the largest tip radii free of interference are 1.660 in (pinion) and 3.133 in (gear); the standard gear's 3.167 in tip passes its limit"
inputs = { m_n = "0.16666667 in", "φ_n" = "20°", "ψ" = "0°", N_P = "12", N_G = "36", F = "1 in" }
expect = { a = "4 in", "r_(a,max,P)" = "1.660 in", "r_(a,max,G)" = "3.133 in", "d_(a,G)" = "6.3333333 in" }
tolerance = "0.05%"
checks = "fail"

[[tests]]
name = "Juvinall Table J.1: standard 16/32 teeth, P 2, 20°"
source = "Lacewing Lab geometry_test, after Juvinall 5th Appendix J, Table J.1: contact ratio 1.58; largest tip radii 5.57 and 8.56 in"
inputs = { m_n = "0.5 in", "φ_n" = "20°", "ψ" = "0°", N_P = "16", N_G = "32", F = "4 in" }
expect = { a = "12 in", m_p = "1.58", "r_(a,max,P)" = "5.57 in", "r_(a,max,G)" = "8.56 in" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Machinery's p. 2028: enlarged 10-tooth pinion, P 100, with a standard 30-tooth gear"
source = "Lacewing Lab geometry_test, after Machinery's Handbook 26th p. 2028: a pinion tooth 0.01873 in thick on the pitch circle (shift x = (0.01873 − π·0.01/2)/(2·0.01·tan 20°) = 0.415136) meshes tight at inv φw = 0.022459 (22°49′), centre distance 0.2039 in"
inputs = { m_n = "0.01 in", "φ_n" = "20°", "ψ" = "0°", N_P = "10", N_G = "30", x_P = "0.415136", x_G = "0", F = "0.1 in" }
expect = { inv_w = "0.022459", "φ_w" = "22.8167°", a = "0.2039 in" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Shigley Eq. 13-11: two equal 13-tooth spur gears, 20° full depth"
source = "Lacewing Lab geometry_test, after Shigley 10th Sec. 13-7: the fewest teeth on a pinion meshing with an equal gear without interference is 12.3, so 13 teeth do"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N_P = "13", N_G = "13", F = "1 in" }
expect = { "N_(P,min)" = "12.32" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: 2 mm spur pair 17/52 with shifts +0.3 and −0.1"
source = "By hand: mt = 2 mm, φt = 20°, dP = 34, dG = 104 mm, sP = 2(π/2 + 0.6 tan 20°) = 3.5783569 mm, sG = 2(π/2 − 0.2 tan 20°) = 2.9960046 mm, inv φw = 0.0149044 + (3.5783569 + 2.9960046 − 2π)/(2 × 69) = 0.0170144, φw = 20.870604°, a = 69 cos 20°/cos φw = 69.391767 mm, tips 39.2 and 107.6 mm, Z = 9.1459896 mm, mp = Z/(2π cos 20°) = 1.5490485; largest tip radii 29.433687 and 54.761671 mm"
inputs = { m_n = "2 mm", "φ_n" = "20°", "ψ" = "0°", N_P = "17", N_G = "52", x_P = "0.3", x_G = "-0.1", F = "20 mm" }
expect = { "s_(n,P)" = "3.5783569 mm", "s_(n,G)" = "2.9960046 mm", inv_w = "0.017014356", "φ_w" = "20.870604°", a = "69.391767 mm", "d_(a,P)" = "39.2 mm", "d_(a,G)" = "107.6 mm", Z = "9.1459896 mm", m_p = "1.5490485", "r_(a,max,P)" = "29.433687 mm", "r_(a,max,G)" = "54.761671 mm" }
tolerance = "1e-6"
checks = "pass"
+++

## Spur and helical gear pair geometry, tight mesh

An external pair of involute spur or helical gears on parallel shafts, generated by a standard rack (Shigley Secs. 13-5 to 13-10). The teeth may be profile-shifted: the generating rack is moved x·m_n away from the gear centre, which thickens the tooth and lengthens its addendum (the usual cure for undercut on a small pinion). The pair runs at the **tight-mesh** centre distance, where the two tooth thicknesses exactly fill the pitch on the operating pitch circles (Machinery's Handbook p. 2028); for standard teeth this is the standard centre distance. Tooth thicknesses carry no backlash allowance. For the backlash at a centre distance you set, use the operating centre distance template; for strength, the AGMA rating template.

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

### Tight-mesh centre distance

On the operating pitch circles the two transverse tooth thicknesses fill one pitch, which fixes the involute of the operating pressure angle φ_w; the centre distance follows from the base circles. The inverse of the involute function inv φ = tan φ − φ is found by Newton's method.

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
> Standard centre distance
a_0 = (N_P + N_G) m_t / 2 = ?? in
> Involute of the operating pressure angle, tight mesh (Machinery's p. 2028)
inv_w = tan(φ_t) - φ_t + ((s_(n,P) + s_(n,G)) / cos(ψ) - π m_t) / ((N_P + N_G) m_t) = ??
> Operating transverse pressure angle
φ_w = invInverse(inv_w) = ?? °
> Operating centre distance
a = a_0 cos(φ_t) / cos(φ_w) = ?? in
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
