+++
cruncha-template = 1
id = "mechanical.discSprings.single"
version = "1.0.0"
title = "Disc (Belleville) spring, DIN 2092 formulas"
summary = "One disc spring by the Almen-László formulas of DIN 2092 (now EN 16984): the constants K1 to K4 (with the reduced thickness of group 3 springs with flat bearings), the force, rate and work at a deflection, the stresses at points OM and I to IV, the force and σOM when flat, and the load at EN 16983's test length."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "disc spring", "Belleville", "Belleville washer", "conical washer", "DIN 2092", "EN 16983", "EN 16984", "Almen-László", "Schnorr", "preload"]
from = "lacewing:mechanical.discSprings"
license = "GPL-3.0-or-later"
inputs = ["D_e", "D_i", "t", "l_0", "t′", "E", "ν", "s", "σ_(OM,lim)"]
outputs = ["δ", "D_0", "K_1", "K_2", "K_3", "K_4", "h_0", "h_0′", "F", "R", "W", "σ_OM", "σ_I", "σ_II", "σ_III", "σ_IV", "F_c", "σ_(OM,c)", "l_t", "F_t"]

[[references]]
work = "Schnorr, Handbook for Disc Springs (Adolf Schnorr GmbH)"
locator = "Sec. 1 (DIN 2092 formulas): Formulas 1 to 6 (constants), 7, 8a, 8b (force), 9 to 13 (stresses at OM and I to IV), 14 (rate), 16 (work); Examples 1 to 3"

[[references]]
work = "DIN 2092, Disc springs: Calculation (superseded by DIN EN 16984)"
edition = "2006"
locator = "The Almen-László formulas, as Schnorr gives them"

[[references]]
work = "DIN EN 16983, Disc springs: Quality specifications, dimensions"
edition = "2017"
locator = "Table 2 (groups), Tables 4 to 6 (standard sizes A, B and C), Clause 5 (E and ν), Clause 10.1 (σOM when flat)"

[[tests]]
name = "Schnorr Ex. 1: 45 × 22.4 × 1.75, l0 3.05 mm at s = 0.5 h0"
source = "Lacewing Lab disc_spring_test, after Schnorr's Handbook Example 1 (its table at s/h0 = 0.5): F = 2701 N, largest tensile stress 814 MPa (point III)"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", s = "0.65 mm" }
expect = { F = "2701 N", "σ_III" = "814 MPa", F_c = "4475 N" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Schnorr Ex. 1 at the working points: σu and σo at point III"
source = "Lacewing Lab disc_spring_test, after Schnorr's Handbook Example 1: s2 = 0.64 mm gives σo = 804 MPa at point III (s1 = 0.34 mm gives σu = 450 MPa)"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", s = "0.64 mm" }
expect = { "σ_III" = "804 MPa" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Schnorr Ex. 2: 60 × 30.5 × 1.5, l0 3.5 mm, the constants and σOM flat"
source = "Lacewing Lab disc_spring_test, after Schnorr's Handbook Example 2: K1 = 0.688, K2 = 1.212, K3 = 1.365, σOM = −1048 MPa at s = h0, F = 2058 N at s/h0 = 0.5, Fc = 2469 N"
inputs = { D_e = "60 mm", D_i = "30.5 mm", t = "1.5 mm", l_0 = "3.5 mm", "t′" = "1.5 mm", s = "1.0 mm" }
expect = { K_1 = "0.688", K_2 = "1.212", K_3 = "1.365", "σ_(OM,c)" = "-1048 MPa", F = "2058 N", F_c = "2469 N" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Schnorr Ex. 3: 200 × 82 × 12, flat bearings t′ 11.5 mm, l0 16.6 mm at s = 0.75 h0"
source = "Lacewing Lab disc_spring_test, after Schnorr's Handbook Example 3: K4 = 1.0537, F = 182 737 N, σII = 1421, σIII = 1072 MPa at s/h0 = 0.75; Fc = 235 503 N, σOM = −1595 MPa (with t′) at flat"
inputs = { D_e = "200 mm", D_i = "82 mm", t = "12 mm", l_0 = "16.6 mm", "t′" = "11.5 mm", s = "3.45 mm" }
expect = { K_1 = "0.755", K_2 = "1.315", K_3 = "1.541", K_4 = "1.0537", F = "182737 N", "σ_II" = "1421 MPa", "σ_III" = "1072 MPa", F_c = "235503 N", "σ_(OM,c)" = "-1595 MPa" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "EN 16983 Table 4, size A 45: 45 × 22.4 × 2.5, l0 3.5 mm at the test length"
source = "DIN EN 16983:2017 Table 4, size A 45 (as in Lacewing Lab disc_spring_test): F = 7716 N at l = 2.75 mm (s = 0.75 h0), σII = 1296 MPa, σOM = −1534 MPa at s = h0"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "2.5 mm", l_0 = "3.5 mm", "t′" = "2.5 mm", s = "0.75 mm" }
expect = { l_t = "2.75 mm", F_t = "7716 N", F = "7716 N", "σ_II" = "1296 MPa", "σ_(OM,c)" = "-1534 MPa" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 1 at s = 0.65 mm, every point, rate and work"
source = "By hand, DIN 2092 formulas: δ = 2.0089286, D0 = 32.39672 mm, K1 = 0.6961166, K2 = 1.2218091, K3 = 1.3810976; 4E/(1 − ν²) = 905 494.5 MPa; F = 2700.795 N, σOM = −697.753, σI = −1506.541, σII = 511.754, σIII = 814.480, σIV = −190.182 MPa, R = 3205.178 N/mm, W = 953.007 N·mm"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", s = "0.65 mm" }
expect = { "δ" = "2.0089286", D_0 = "32.39672 mm", K_1 = "0.6961166", K_2 = "1.2218091", K_3 = "1.3810976", K_4 = "1", F = "2700.795 N", "σ_OM" = "-697.753 MPa", "σ_I" = "-1506.541 MPa", "σ_II" = "511.754 MPa", "σ_III" = "814.480 MPa", "σ_IV" = "-190.182 MPa", R = "3205.178 N/mm", W = "953.007 N·mm" }
tolerance = "2e-6"
checks = "pass"

[[tests]]
name = "Ex. 1 pressed past flat"
source = "Schnorr Example 1 spring with s = 1.4 mm, beyond h0 = 1.3 mm: the disc cannot deflect that far"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", s = "1.4 mm" }
expect = { h_0 = "1.3 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Disc (Belleville) spring

One disc spring, a coned annular disc loaded on its upper inner edge and lower outer edge, by the Almen-László theory as DIN 2092 sets it out and Schnorr's Handbook for Disc Springs gives it. The cross-section is taken to turn about a centre of rotation at diameter D₀ without distorting, with sharp corners and a linear elastic material. The stresses are nominal: presetting in manufacture leaves residual stresses that lower them in service. The force is accurate to about 0.75 h₀; nearer flat the formulas overstate it. For stacks, friction and two working points, use the disc spring stack template.

### Dimensions

The standard sizes are in DIN EN 16983 Tables 4 to 6 (series A, B and C, with De, Di, t, l0 and, for group 3, t′). Look the size up there and enter it here. EN 16983 Table 2 sorts springs into groups by thickness: group 1 below 1.25 mm, group 2 up to 6 mm, group 3 from 6 to 14 mm. Group 3 springs have small flat bearing faces at the loaded edges and a reduced thickness t′ that keeps the flat load the same; for groups 1 and 2, enter t′ = t.

```calc {description="Outside diameter (EN 16983 Tables 4 to 6 for standard sizes)"}
D_e = 45 mm_
```

```calc {description="Inside diameter"}
D_i = 22.4 mm_
```

```calc {description="Material thickness"}
t = 1.75 mm_
```

```calc {description="Free height of the disc, unloaded"}
l_0 = 3.05 mm_
```

```calc {description="Reduced thickness of a group 3 spring with flat bearings (t′ = t without)"}
t′ = 1.75 mm_
```

EN 16983 Clause 5 designs steel disc springs with E = 206 000 MPa and ν = 0.3.

```calc {description="Young's modulus"}
E = 206000 MPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

### Constants

```calc
> Diameter ratio, Formula 2
δ = D_e / D_i = ??
> Diameter of the centre of rotation, Formula 1
D_0 = (D_e - D_i) / ln(δ) = ?? mm
> Formula 3
K_1 = (1 / π) ((δ - 1) / δ)^2 / ((δ + 1) / (δ - 1) - 2 / ln(δ)) = ??
> Formula 4
K_2 = (6 / π) ((δ - 1) / ln(δ) - 1) / ln(δ) = ??
> Formula 5
K_3 = (3 / π) (δ - 1) / ln(δ) = ??
```

With flat bearings the formulas use t′ and the cone height h₀′ = l₀ − t′, and K₄ (Formula 6) corrects for the bearing faces. With t′ = t, K₄ is 1. The deflection to flat is h₀ = l₀ − t either way, as in EN 16983's tables.

```calc
> Cone height, the deflection to flat
h_0 = l_0 - t = ?? mm
> Cone height with the reduced thickness
h_0′ = l_0 - t′ = ?? mm
> Formula 6
C_1 = (t′ / t)^2 / ((l_0 / (4 t) - t′ / t + 3 / 4) (5 l_0 / (8 t) - t′ / t + 3 / 8)) = ??
C_2 = C_1 / (t′ / t)^3 (5 / 32 (l_0 / t - 1)^2 + 1) = ??
K_4 = sqrt(-C_1 / 2 + sqrt(C_1^2 / 4 + C_2)) = ??
```

### Force, rate and work

```calc
function discForce(E_r, De, K1, K4, t, h, s)
    return E_r t^4 / (K1 De^2) K4^2 (s / t) (K4^2 (h / t - s / t) (h / t - s / (2 t)) + 1)
end
```

```calc
function discOM(E_r, De, K1, K4, t, s)
    return -E_r t^2 / (K1 De^2) K4 (s / t) 3 / π
end
```

```calc {description="Deflection of the disc from free"}
s = 0.65 mm_
```

```calc
> Modulus term of the formulas
E_r = 4 E / (1 - ν^2) = ?? MPa
> Force at s, Formulas 7, 8a and 8b
F = discForce(E_r, D_e, K_1, K_4, t′, h_0′, s) = ?? N
> Rate dF/ds at s, Formula 14
R = E_r t′^3 / (K_1 D_e^2) K_4^2 (K_4^2 ((h_0′ / t′)^2 - 3 (h_0′ / t′) (s / t′) + 1.5 (s / t′)^2) + 1) = ?? N/mm
> Work stored from free to s, Formula 16
W = E_r / 2 t′^5 / (K_1 D_e^2) K_4^2 (s / t′)^2 (K_4^2 (h_0′ / t′ - s / (2 t′))^2 + 1) = ?? N·mm
```

### Stresses at s

Tension is positive. OM is on the upper face over the centre of rotation, I the upper inner edge, II the lower inner edge, III the lower outer edge and IV the upper outer edge. Under static load σOM (and σI) decide whether the disc sets; under fatigue load the tensile stress at II or III, whichever is larger, decides the life.

```calc
> Lever term of the stresses
a_s = h_0′ / t′ - s / (2 t′) = ??
> Stress at OM, Formula 9
σ_OM = discOM(E_r, D_e, K_1, K_4, t′, s) = ?? MPa
> Common factor of Formulas 10 to 13
σ_s = -E_r t′^2 / (K_1 D_e^2) K_4 (s / t′) = ?? MPa
> Stress at I, Formula 10
σ_I = σ_s (K_4 K_2 a_s + K_3) = ?? MPa
> Stress at II, Formula 11
σ_II = σ_s (K_4 K_2 a_s - K_3) = ?? MPa
> Stress at III, Formula 12
σ_III = σ_s / δ (K_4 (K_2 - 2 K_3) a_s - K_3) = ?? MPa
> Stress at IV, Formula 13
σ_IV = σ_s / δ (K_4 (K_2 - 2 K_3) a_s + K_3) = ?? MPa
```

### Flat and test loads

EN 16983's tables give each size's force at the test length l_t = l₀ − 0.75 h₀, which is also where its load tolerance applies, and σOM when pressed flat. Clause 10.1 limits that flat σOM to 1600 MPa in compression for its steels under static or rarely changing load; a spring that exceeds it sets if it is pressed flat.

```calc {description="Limit on the magnitude of σOM when flat (EN 16983 Clause 10.1)"}
σ_(OM,lim) = 1600 MPa_
```

```calc
> Force when flat, s = h0
F_c = discForce(E_r, D_e, K_1, K_4, t′, h_0′, h_0) = ?? N
> Stress at OM when flat
σ_(OM,c) = discOM(E_r, D_e, K_1, K_4, t′, h_0) = ?? MPa
> Test length, EN 16983
l_t = l_0 - 0.75 h_0 = ?? mm
> Force at the test length
F_t = discForce(E_r, D_e, K_1, K_4, t′, h_0′, 0.75 h_0) = ?? N
```

### Checks

```calc
@test 0 'mm' ≤ s ≤ h_0
```

```calc
@test 0 'mm' < t′ ≤ t
```

```calc
@test abs(σ_(OM,c)) ≤ σ_(OM,lim)
```
