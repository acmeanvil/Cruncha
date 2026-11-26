+++
cruncha-template = 1
id = "structural.catenary.horizontalTension"
version = "1.0.0"
title = "Catenary from its horizontal tension: end reactions, sag, length and low point"
summary = "A cable hanging under its own weight between supports at the same or different heights, with a given horizontal tension: the vertical and total tension at each end with their angles, the sag below the chord, the cable length, the low point, and the parabola's sag for comparison (Den Hartog, Mechanics §16)."
category = ["Structural engineering", "Analysis"]
tags = ["cable", "catenary", "sag", "tension", "parabola", "overhead line", "guy", "self-weight", "Den Hartog"]
from = "lacewing:structural.catenary"
license = "GPL-3.0-or-later"
inputs = ["q", "l", "d", "H"]
outputs = ["a", "x_0", "V_A", "V_B", "T_A", "T_B", "θ_A", "θ_B", "S", "s", "s_low", "s_par", "e_par"]

[[references]]
work = "Den Hartog, Mechanics"
edition = "1st (McGraw-Hill, 1948)"
locator = "§16, Uniformly loaded cables, pp. 63 to 67: the parabola s = ql²/8H (Fig. 69), the catenary (q/H)y = cosh(qx/H) − 1, and the table comparing them (Fig. 70)"

[[tests]]
name = "Den Hartog §16 table: qx/H = 1/8 at the support, the parabola 0.1 % low"
source = "Den Hartog, Mechanics §16, the table after Fig. 70 (verified in the PDF): qx/H = 1/8 at the support gives sag/span 1/32 and an error of 0.1 %; here q = 1 N/m, H = 1 N, span 0.25 m: catenary sag cosh(1/8) − 1 = 0.0078226 m, parabola 1/128 m; Lacewing Lab mechanics_test checks the same row"
inputs = { q = "1 N/m", l = "0.25 m", d = "0 m", H = "1 N" }
expect = { s = "0.0078226 m", s_par = "0.0078125 m", e_par = "0.0013028" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Den Hartog §16 table: qx/H = 1/2, the parabola 2 % low"
source = "Den Hartog, Mechanics §16, the table after Fig. 70 (verified in the PDF): qx/H = 1/2 gives sag/span 1/8 and an error of 2 % (the printed digit; cosh(1/2) − 1 = 0.127626 against 0.125, 2.1 %)"
inputs = { q = "1 N/m", l = "1 m", d = "0 m", H = "1 N" }
expect = { s = "0.12762597 m", s_par = "0.125 m", e_par = "0.02100772" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Level supports: 10 N/m over 100 m at H = 2 kN"
source = "Lacewing Lab mechanics_test (catenary at q 10, span 100, H 2000: T end = H + q·s); by hand a = 200 m, s = 200(cosh(0.25) − 1) = 6.28262 m, S = 400 sinh(0.25) = 101.04493 m, T = 2000 cosh(0.25) = 2062.8262 N, V = 505.22463 N each"
inputs = { q = "10 N/m", l = "100 m", d = "0 m", H = "2000 N" }
expect = { a = "200 m", x_0 = "50 m", V_A = "505.22463 N", V_B = "505.22463 N", T_A = "2062.8262 N", T_B = "2062.8262 N", "θ_A" = "14.177027°", S = "101.04493 m", s = "6.28262 m", s_low = "6.28262 m", s_par = "6.25 m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "B 15 m above A: T_B − T_A = q·d and the supports carry the weight"
source = "Lacewing Lab mechanics_test (q 20, span 100, H 5000, rise 15: VA + VB = q × length, TB − TA = 20 × 15 = 300 N, VB > VA); values by hand from the same closed forms: x0 = 12.885323 m, VA = 257.82058 N, VB = 1777.7674 N, S = 101.7794 m, sag below the chord 5.0721969 m"
inputs = { q = "20 N/m", l = "100 m", d = "15 m", H = "5000 N" }
expect = { x_0 = "12.885323 m", V_A = "257.82058 N", V_B = "1777.7674 N", T_A = "5006.6427 N", T_B = "5306.6427 N", "θ_B" = "19.573021°", S = "101.7794 m", s = "5.0721969 m", s_low = "0.33213662 m" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Steep and tight: the low point lies beyond A, which is pulled down"
source = "Lacewing Lab mechanics_test (q 20, span 100, H 50 000, rise 60: low point outside the span, VA < 0); by hand x0 = −1371.98 m, VA = −28 837.75 N"
inputs = { q = "20 N/m", l = "100 m", d = "60 m", H = "50000 N" }
expect = { x_0 = "-1371.9765 m", V_A = "-28837.752 N", V_B = "31170.247 N", T_B = "58920.152 N", s = "0.58310605 m", s_low = "0 m" }
tolerance = "1e-6"
checks = "pass"
+++

## Catenary from its horizontal tension

A perfectly flexible, inextensible cable hanging between supports A and B under its own weight q per unit length of cable (Den Hartog, Mechanics §16). Its shape is the catenary y = a cosh((x − x₀)/a) + c with a = H/q, where H is the horizontal component of tension, the same all along the cable, and x₀ is the low point. B may be higher or lower than A. Here H is given; the sag-given and length-given templates find H first.

Not included: elastic stretch (a loaded cable lengthens and sags more; the tension member template gives its stretch), point loads, wind, ice beyond what you add to q, and temperature changes.

### Cable and supports

```calc {description="Weight per unit length of cable (mass per length × g), with any covering or ice"}
q = 10 N/m_
```

```calc {description="Horizontal distance between supports A and B"}
l = 100 m_
```

```calc {description="Height of B above A (negative when B is lower)"}
d = 0 m_
```

```calc {description="Horizontal component of tension"}
H = 2000 N_
```

### Shape

The low point sits at x₀ = l/2 − a·asinh(d/(2a·sinh(l/2a))) from A; it lies outside the span (x₀ < 0 or x₀ > l) when the cable is tight and the supports differ in height, and then the lower support is the lowest point.

```calc
> Catenary parameter, the radius of curvature at the low point
a = H / q = ?? m
> Low point, horizontally from A
x_0 = l / 2 - a asinh(d / (2 a sinh(l / (2 a)))) = ?? m
```

### End reactions

The support pulls along the cable, so its vertical component is H times the slope there. A negative vertical component means the cable pulls the support down. The higher support has the higher tension: T_B − T_A = q·d.

```calc
> Vertical force at A (up positive)
V_A = H sinh(x_0 / a) = ?? N
> Vertical force at B (up positive)
V_B = H sinh((l - x_0) / a) = ?? N
> Total tension at A
T_A = sqrt(H^2 + V_A^2) = ?? N
> Total tension at B
T_B = sqrt(H^2 + V_B^2) = ?? N
> Angle of the cable below horizontal at A
θ_A = atan(V_A / H) = ?? °
> Angle of the cable below horizontal at B
θ_B = atan(V_B / H) = ?? °
```

### Length and sag

The sag below the chord is the largest depth under the straight line from A to B, where the cable's slope equals the chord's: x_s = x₀ + a·asinh(d/l), kept within the span.

```calc
> Cable length between the supports, (V_A + V_B)/q
S = a (sinh((l - x_0) / a) + sinh(x_0 / a)) = ?? m
> Where the sag below the chord is greatest, from A
x_s = max(0 'm', min(l, x_0 + a asinh(d / l))) = ?? m
> Sag below the chord
s = d x_s / l - a (cosh((x_s - x_0) / a) - cosh(x_0 / a)) = ?? m
> Depth of the low point below the lower support (0 when the low point is outside the span)
s_low = { min(0 'm', d) - a (1 - cosh(x_0 / a)) if 0 'm' ≤ x_0 ≤ l; 0 'm' otherwise } = ?? m
```

For comparison, a load uniform per horizontal length hangs as a parabola with mid-span sag ql²/8H (Den Hartog Fig. 69). For level supports it is a little smaller than the catenary's: about 0.1 % at sag/span 1/32 and 2 % at 1/8, so it serves for ordinary lines.

```calc
> Parabola's sag for level supports at the same H
s_par = q l^2 / (8 H) = ?? m
> How far the parabola's sag is below the catenary's (level supports)
e_par = s / s_par - 1 = ??
```

### Checks

```calc
@test abs(V_A + V_B - q S) < 1e-6 q S
```
