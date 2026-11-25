+++
cruncha-template = 1
id = "mechanical.discSprings.stack"
version = "1.0.0"
title = "Disc (Belleville) spring stack, DIN 2092 formulas"
summary = "A stack of identical disc springs, i sets in series of n nested in parallel, between two working points: free and flat lengths, the forces with and without DIN 2092's friction, the rate and work, and the lower and upper tensile stresses at points II and III that a fatigue check takes."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "disc spring", "Belleville", "stack", "series", "parallel", "nested", "friction", "DIN 2092", "EN 16983", "EN 16984", "Almen-László", "Schnorr", "preload", "fatigue"]
from = "lacewing:mechanical.discSprings"
license = "GPL-3.0-or-later"
inputs = ["D_e", "D_i", "t", "l_0", "t′", "E", "ν", "i", "n", "s_1", "s_2", "w_M", "w_R"]
outputs = ["K_1", "K_2", "K_3", "K_4", "h_0", "L_0", "s_max", "L_c", "F_c", "L_1", "L_2", "F_1", "F_2", "f_R", "F_(1,load)", "F_(1,unload)", "F_(2,load)", "F_(2,unload)", "R_2", "W_2", "W_12", "σ_(II,1)", "σ_(III,1)", "σ_(II,2)", "σ_(III,2)", "σ_u", "σ_o"]

[[references]]
work = "Schnorr, Handbook for Disc Springs (Adolf Schnorr GmbH)"
locator = "Sec. 1 (DIN 2092 formulas): Formulas 1 to 16 (single spring), 17 to 25 (stacks in series, in parallel and both), 26 (friction); friction factors and percentages, p. 63; Examples 1 and 2"

[[references]]
work = "DIN 2092, Disc springs: Calculation (superseded by DIN EN 16984)"
edition = "2006"
locator = "The Almen-László formulas and the friction formula, as Schnorr gives them"

[[references]]
work = "DIN EN 16983, Disc springs: Quality specifications, dimensions"
edition = "2017"
locator = "Tables 4 to 6 (standard sizes), Clause 5 (E and ν), Clause 10.2.1 (preload of fatigue-loaded springs)"

[[tests]]
name = "Schnorr Ex. 1: 45 × 22.4 × 1.75, l0 3.05 mm, single, between 0.34 and 0.64 mm"
source = "Lacewing Lab disc_spring_test, after Schnorr's Handbook Example 1: σu = 450 MPa and σo = 804 MPa at point III (read off its Fig. 12; the formulas give 452 and 803.5)"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", i = "1", n = "1", s_1 = "0.34 mm", s_2 = "0.64 mm" }
expect = { "σ_u" = "450 MPa", "σ_o" = "804 MPa", "σ_(III,2)" = "804 MPa" }
tolerance = "0.7%"
checks = "pass"

[[tests]]
name = "Schnorr Ex. 2: two 60 × 30.5 × 1.5, l0 3.5 mm in series, 2100 N at 2.1 mm"
source = "Lacewing Lab disc_spring_test, after Schnorr's Handbook Example 2: L0 = 7.0 mm; two springs in series carry about 2100 N at a stack deflection of 2.1 mm (2103.9 N by the formulas)"
inputs = { D_e = "60 mm", D_i = "30.5 mm", t = "1.5 mm", l_0 = "3.5 mm", "t′" = "1.5 mm", i = "2", n = "1", s_1 = "0.6 mm", s_2 = "2.1 mm" }
expect = { L_0 = "7.0 mm", s_max = "4.0 mm", F_2 = "2100 N" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "DIN 2092 friction: three Ex. 1 springs nested in parallel, series A upper factors"
source = "Lacewing Lab disc_spring_test, after Schnorr's Handbook p. 63: with wM = 0.03 and wR = 0.05, three in parallel load 12.36 % above the frictionless force (Formula 26: 1/(1 − 0.11)); 3 × 3712.442 N at s = 1 mm gives 12 513.85 N"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", i = "1", n = "3", s_1 = "0.3 mm", s_2 = "1.0 mm", w_M = "0.03", w_R = "0.05" }
expect = { f_R = "0.11", F_2 = "11137.33 N", "F_(2,load)" = "12513.85 N" }
tolerance = "1e-5"

[[tests]]
name = "DIN 2092 friction: two in parallel unloading, series B lower factors"
source = "Lacewing Lab disc_spring_test, after Schnorr's Handbook p. 63: with wM = 0.003 and wR = 0.02, two in parallel unload 2.25 % below the frictionless force (1/1.023); 2 × 3712.442 N at s = 1 mm gives 7257.95 N"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", i = "1", n = "2", s_1 = "0.3 mm", s_2 = "1.0 mm", w_M = "0.003", w_R = "0.02" }
expect = { "F_(2,unload)" = "7257.951 N" }
tolerance = "1e-6"

[[tests]]
name = "Hand check: four sets in series of two Ex. 1 springs, between 1.2 and 2.0 mm"
source = "By hand, Formulas 17 to 25: L0 = 4(3.05 + 1.75) = 19.2 mm, smax = 5.2 mm, Lc = 14 mm; each spring at 0.3 and 0.5 mm: F1 = 2 × 1420.620 = 2841.241 N, F2 = 2 × 2193.460 = 4386.920 N, R2 = 2 × 3571.926 / 4 = 1785.963 N/mm, W2 = 8 × 585.251 = 4682.005 N·mm, W12 = 2907.452 N·mm; σIII = 401.772 and 644.993 MPa"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", i = "4", n = "2", s_1 = "1.2 mm", s_2 = "2.0 mm", w_M = "0", w_R = "0" }
expect = { L_0 = "19.2 mm", s_max = "5.2 mm", L_c = "14 mm", L_2 = "17.2 mm", F_1 = "2841.2406 N", F_2 = "4386.9205 N", R_2 = "1785.9628 N/mm", W_2 = "4682.0048 N·mm", W_12 = "2907.4519 N·mm", "σ_u" = "401.77237 MPa", "σ_o" = "644.99348 MPa", "σ_(II,2)" = "364.22520 MPa", "F_(2,load)" = "4386.9205 N" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 1 single, worked to 1.2 mm: beyond 0.75 h0"
source = "Schnorr Example 1 spring with s2 = 1.2 mm = 0.92 h0, past the 0.75 h0 to which the formulas' force is reliable"
inputs = { D_e = "45 mm", D_i = "22.4 mm", t = "1.75 mm", l_0 = "3.05 mm", "t′" = "1.75 mm", i = "1", n = "1", s_1 = "0.34 mm", s_2 = "1.2 mm" }
expect = { s_max = "1.3 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Disc (Belleville) spring stack

A stack of identical disc springs: i sets stacked face to face alternately (in series, each adding its deflection), each set n springs nested the same way round (in parallel, multiplying the force). The single-spring formulas are DIN 2092's (Almen and László), as Schnorr's Handbook for Disc Springs gives them; the stack formulas assume every spring deflects equally, which long stacks don't quite do, so keep i small or guide the stack well. The stack works between two deflections s₁ (preload) and s₂ (working), both measured on the whole stack from free. For one spring's stresses at all five points, use the single disc spring template.

### Spring

Standard sizes are in DIN EN 16983 Tables 4 to 6: look the size up there and enter its De, Di, t, l0 and, for a group 3 spring with flat bearings, t′. Otherwise enter t′ = t. EN 16983 Clause 5 designs steel disc springs with E = 206 000 MPa and ν = 0.3.

```calc {description="Outside diameter (EN 16983 Tables 4 to 6 for standard sizes)"}
D_e = 45 mm_
```

```calc {description="Inside diameter"}
D_i = 22.4 mm_
```

```calc {description="Material thickness"}
t = 1.75 mm_
```

```calc {description="Free height of one disc"}
l_0 = 3.05 mm_
```

```calc {description="Reduced thickness of a group 3 spring with flat bearings (t′ = t without)"}
t′ = 1.75 mm_
```

```calc {description="Young's modulus"}
E = 206000 MPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc
> Diameter ratio, Formula 2
δ = D_e / D_i = ??
> Formulas 3 to 5
K_1 = (1 / π) ((δ - 1) / δ)^2 / ((δ + 1) / (δ - 1) - 2 / ln(δ)) = ??
K_2 = (6 / π) ((δ - 1) / ln(δ) - 1) / ln(δ) = ??
K_3 = (3 / π) (δ - 1) / ln(δ) = ??
> Formula 6, flat bearings (1 when t′ = t)
C_1 = (t′ / t)^2 / ((l_0 / (4 t) - t′ / t + 3 / 4) (5 l_0 / (8 t) - t′ / t + 3 / 8)) = ??
C_2 = C_1 / (t′ / t)^3 (5 / 32 (l_0 / t - 1)^2 + 1) = ??
K_4 = sqrt(-C_1 / 2 + sqrt(C_1^2 / 4 + C_2)) = ??
> Deflection of one disc to flat
h_0 = l_0 - t = ?? mm
> Cone height with the reduced thickness
h_0′ = l_0 - t′ = ?? mm
> Modulus term of the formulas
E_r = 4 E / (1 - ν^2) = ?? MPa
```

The functions give one disc's force (Formulas 7, 8a, 8b), rate (Formula 14) and work (Formula 16) at its own deflection s, and the stresses at the lower inner edge II (Formula 11) and lower outer edge III (Formula 12), tension positive.

```calc
function discForce(E_r, De, K1, K4, t, h, s)
    return E_r t^4 / (K1 De^2) K4^2 (s / t) (K4^2 (h / t - s / t) (h / t - s / (2 t)) + 1)
end
```

```calc
function discRate(E_r, De, K1, K4, t, h, s)
    return E_r t^3 / (K1 De^2) K4^2 (K4^2 ((h / t)^2 - 3 (h / t) (s / t) + 1.5 (s / t)^2) + 1)
end
```

```calc
function discWork(E_r, De, K1, K4, t, h, s)
    return E_r / 2 t^5 / (K1 De^2) K4^2 (s / t)^2 (K4^2 (h / t - s / (2 t))^2 + 1)
end
```

```calc
function discII(E_r, De, K1, K2, K3, K4, t, h, s)
    return -E_r t^2 / (K1 De^2) K4 (s / t) (K4 K2 (h / t - s / (2 t)) - K3)
end
```

```calc
function discIII(E_r, De, d, K1, K2, K3, K4, t, h, s)
    return -E_r t^2 / (K1 De^2) K4 (s / t) / d (K4 (K2 - 2 K3) (h / t - s / (2 t)) - K3)
end
```

### Stack

```calc {description="Sets in series (stacked face to face alternately)"}
i = 1_
```

```calc {description="Springs nested in parallel in each set"}
n = 1_
```

```calc
> Free length of the stack, Formulas 19, 22, 25
L_0 = i (l_0 + (n - 1) t′) = ?? mm
> Largest stack deflection, Formulas 18, 21, 24
s_max = i h_0 = ?? mm
> Stack length with every spring flat
L_c = L_0 - s_max = ?? mm
> Stack force when flat
F_c = n discForce(E_r, D_e, K_1, K_4, t′, h_0′, h_0) = ?? N
```

### Working points

EN 16983 Clause 10.2.1 asks a fatigue-loaded spring to be preloaded to 0.15 h₀ to 0.20 h₀, so that cracks don't start at point I; the ratio s₁/s_max shows it.

```calc {description="Stack deflection at the preload point (s₁ < s₂)"}
s_1 = 0.34 mm_
```

```calc {description="Stack deflection at the working point"}
s_2 = 0.64 mm_
```

```calc
> Stack lengths at the two points
L_1 = L_0 - s_1 = ?? mm
L_2 = L_0 - s_2 = ?? mm
> Preload as a share of the travel
s_1 / s_max = ??
> Stack force at s1 without friction, Formulas 17, 20, 23
F_1 = n discForce(E_r, D_e, K_1, K_4, t′, h_0′, s_1 / i) = ?? N
> Stack force at s2 without friction
F_2 = n discForce(E_r, D_e, K_1, K_4, t′, h_0′, s_2 / i) = ?? N
> Stack rate at s2
R_2 = n discRate(E_r, D_e, K_1, K_4, t′, h_0′, s_2 / i) / i = ?? N/mm
> Work stored from free to s2
W_2 = i n discWork(E_r, D_e, K_1, K_4, t′, h_0′, s_2 / i) = ?? N·mm
> Work between the working points
W_12 = W_2 - i n discWork(E_r, D_e, K_1, K_4, t′, h_0′, s_1 / i) = ?? N·mm
```

### Friction

Friction between nested springs (surface factor w_M) and at the loaded edges (edge factor w_R) raises the force on loading and lowers it on unloading (DIN 2092's Formula 26). DIN 2092 gives a range of each factor for each of EN 16983's series A, B and C (Schnorr p. 63); in practice a single spring loses 2 % to 3 % per face pair. Enter 0 for both to leave friction out.

```calc {description="Surface friction factor between nested springs (DIN 2092, by series)"}
w_M = 0.01_
```

```calc {description="Edge friction factor at the bearing edges (DIN 2092, by series)"}
w_R = 0.03_
```

```calc
> Friction term, Formula 26
f_R = w_M (n - 1) + w_R = ??
> Forces with friction at s1, loading and unloading
F_(1,load) = F_1 / (1 - f_R) = ?? N
F_(1,unload) = F_1 / (1 + f_R) = ?? N
> Forces with friction at s2, loading and unloading
F_(2,load) = F_2 / (1 - f_R) = ?? N
F_(2,unload) = F_2 / (1 + f_R) = ?? N
```

### Stresses for fatigue

Fatigue cracks start at the lower edges, II or III, where the stress is tensile. The larger of the two at each point is the lower stress σ_u and upper stress σ_o that EN 16983's fatigue diagrams (Figures 5 to 7, by group) take; read the life there. Without friction, the stresses don't depend on n.

```calc
> Stresses at II and III at s1, Formulas 11 and 12
σ_(II,1) = discII(E_r, D_e, K_1, K_2, K_3, K_4, t′, h_0′, s_1 / i) = ?? MPa
σ_(III,1) = discIII(E_r, D_e, δ, K_1, K_2, K_3, K_4, t′, h_0′, s_1 / i) = ?? MPa
> Stresses at II and III at s2
σ_(II,2) = discII(E_r, D_e, K_1, K_2, K_3, K_4, t′, h_0′, s_2 / i) = ?? MPa
σ_(III,2) = discIII(E_r, D_e, δ, K_1, K_2, K_3, K_4, t′, h_0′, s_2 / i) = ?? MPa
> Lower and upper tensile stress of the cycle
σ_u = max(σ_(II,1), σ_(III,1)) = ?? MPa
σ_o = max(σ_(II,2), σ_(III,2)) = ?? MPa
```

### Checks

The working points must lie on the stack's travel. The force formulas hold to about 0.75 h₀ per spring and overstate the force nearer flat, so the working point is kept within it.

```calc
@test 0 'mm' ≤ s_1 < s_2 ≤ s_max
```

```calc
@test s_2 ≤ 0.75 s_max
```

```calc
@test i ≥ 1 and n ≥ 1
```
