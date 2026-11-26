+++
cruncha-template = 1
id = "structural.catenary.lengthGiven"
version = "1.0.0"
title = "Catenary from its length: horizontal tension, end reactions, sag and low point"
summary = "A cable of given length hanging under its own weight between supports at the same or different heights: the horizontal tension (found by bisection), the vertical and total tension at each end with their angles, the sag below the chord and the low point (Den Hartog, Mechanics §16)."
category = ["Structural engineering", "Analysis"]
tags = ["cable", "catenary", "sag", "tension", "parabola", "overhead line", "guy", "self-weight", "Den Hartog"]
from = "lacewing:structural.catenary"
license = "GPL-3.0-or-later"
inputs = ["q", "l", "d", "S_c"]
outputs = ["H", "a", "x_0", "V_A", "V_B", "T_A", "T_B", "θ_A", "θ_B", "S", "s", "s_low", "s_par"]

[[references]]
work = "Den Hartog, Mechanics"
edition = "1st (McGraw-Hill, 1948)"
locator = "§16, Uniformly loaded cables, pp. 63 to 67: the parabola s = ql²/8H (Fig. 69), the catenary (q/H)y = cosh(qx/H) − 1, and the table comparing them (Fig. 70)"

[[tests]]
name = "101 m of cable over a level 100 m span at 10 N/m"
source = "By hand (bisection on the §16 length (2H/q)·sinh(ql/2H)): H = 2044.2962 N, V = 505 N each (half the weight 10 × 101), T = 2105.7474 N, sag 6.1451161 m"
inputs = { q = "10 N/m", l = "100 m", d = "0 m", S_c = "101 m" }
expect = { H = "2044.2962 N", V_A = "505 N", V_B = "505 N", T_A = "2105.7474 N", s = "6.1451161 m", S = "101 m" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "B 15 m above A: the length recovers H = 5 kN"
source = "Lacewing Lab mechanics_test (catenaryTensionForCableLength of the length at q 20, span 100, H 5000, rise 15 gives 5000 back); the length 101.7794 m by hand from the horizontal-tension template's closed forms"
inputs = { q = "20 N/m", l = "100 m", d = "15 m", S_c = "101.7794 m" }
expect = { H = "5000 N", V_A = "257.82058 N", T_B = "5306.6427 N", s = "5.0721969 m" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "A cable shorter than the chord: the check fails"
source = "By hand: the chord of a level 100 m span is 100 m, so 99.9 m of cable can't hang between the supports"
inputs = { q = "10 N/m", l = "100 m", d = "0 m", S_c = "99.9 m" }
expect = { l = "100 m" }
tolerance = "1e-9"
checks = "fail"
+++

## Catenary from its length

A perfectly flexible, inextensible cable hanging between supports A and B under its own weight q per unit length of cable (Den Hartog, Mechanics §16). Its shape is the catenary y = a cosh((x − x₀)/a) + c with a = H/q, where H is the horizontal component of tension, the same all along the cable, and x₀ is the low point. B may be higher or lower than A. Here the cable length is given, and H is found from it; the horizontal-tension template takes H directly.

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

```calc {description="Length of the cable between the supports (longer than the chord)"}
S_c = 101 m_
```

### Horizontal tension

The length decreases steadily as H grows, so H is found by bisection (on a log scale, from H = ql/40, a very deep sag with the half-span 20 times the parameter a, up to H = 10⁷·ql). The functions work in plain SI numbers (N, m). A cable barely longer than the chord is very tight: small errors in the length make large changes in H, so measure it well or give the sag instead.

```calc
function cableLength(q, l, d, H)
    a = H / q
    x0 = l / 2 - a asinh(d / (2 a sinh(l / (2 a))))
    return a (sinh((l - x0) / a) + sinh(x0 / a))
end
```

```calc
function tensionForLength(q, l, d, S)
    lo = q l / 40
    hi = q l 1e7
    for i in 1:100
        m = roundSig(sqrt(lo hi), 15)
        if cableLength(q, l, d, m) > S
            lo = m
        else
            hi = m
        end
    end
    return sqrt(lo hi)
end
```

```calc {description="Horizontal component of tension that gives the length"}
H = tensionForLength(q / (1 'N/m'), l / (1 'm'), d / (1 'm'), S_c / (1 'm')) × 1 'N' = ?? N
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
@test S_c > sqrt(l^2 + d^2) and abs(S - S_c) < 1e-6 S_c
```
