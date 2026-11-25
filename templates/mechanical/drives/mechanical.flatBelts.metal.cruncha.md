+++
cruncha-template = 1
id = "mechanical.flatBelts.metal"
version = "1.0.0"
title = "Flat metal belt drive (Shigley)"
summary = "A thin flat metal friction belt on two pulleys, by Shigley's metal-belt steps: endurance strength for a life in belt passes (Eq. 17-15 for 301/302 stainless, a third of the yield strength for the other alloys), bending stress over the small pulley, the allowable tension per width, the tension difference for the torque, the narrowest belt that does not slip, and the tight-side, slack-side and initial tensions and the friction developed for the width chosen."
category = ["Mechanical engineering", "Belts and chains"]
tags = ["belt", "metal belt", "flat belt", "stainless steel belt", "belt drive", "bending stress", "belt passes", "endurance", "initial tension", "friction", "Shigley"]
from = "lacewing:mechanical.flatBelts"
license = "GPL-3.0-or-later"
inputs = ["T", "d", "D", "C", "t", "b", "f", "E", "ν", "stainless", "N_p", "S_y", "d_min"]
outputs = ["φ", "r_t", "S_f", "σ_b", "a", "ΔF", "b_min", "F_(1,a)", "F_2", "F_i", "f′"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 17-2, flat metal belts (pp. 887-890): Eq. 17-1 (wrap), 17-14 (bending stress), 17-15 (stainless endurance, fitted to Table 17-6), the nine design steps of p. 889, Table 17-7 (smallest pulley), Table 17-8 (alloys); Example 17-3"

[[tests]]
name = "Shigley Ex. 17-3: 0.003 in 301 stainless belt on two 4 in pulleys, 30 lbf·in, 10⁶ passes"
source = "Lacewing Lab flat_belts_test, after Shigley 10th Example 17-3: f = 0.35, exp(fφ) = 3.00, Sf = 51 210 psi, a = 85.1 lbf/in, ΔF = 15 lbf, bmin = 0.264 in; with the 0.75 in belt chosen (F1)a = 63.8, F2 = 48.8, Fi = 56.3 lbf and f′ = 0.0853; Table 17-7's smallest pulley for 0.003 in is 1.8 in"
inputs = { T = "30 lbf·in", d = "4 in", D = "4 in", C = "20 in", t = "0.003 in", b = "0.75 in", f = "0.35", E = "28 Mpsi", "ν" = "0.285", stainless = "1", N_p = "1e6", d_min = "1.8 in" }
expect = { r_t = "3.00", S_f = "51210 psi", a = "85.1 lbf/in", "ΔF" = "15 lbf", b_min = "0.264 in", "F_(1,a)" = "63.8 lbf", F_2 = "48.8 lbf", F_i = "56.3 lbf", "f′" = "0.0853" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 17-3 exactly"
source = "By hand: φ = π, exp(0.35π) = 3.0028368, Sf = 14.17e6 × (10⁶)^−0.407 = 51 211.778 psi, σb = 28e6 × 0.003/((1 − 0.285²) × 4) = 22 856.521 psi, a = (51 211.778 − 22 856.521) × 0.003 = 85.06577 lbf/in, bmin = 15/85.06577 × 3.0028368/2.0028368 = 0.26437634 in, (F1)a = 63.799327, F2 = 48.799327, Fi = 56.299327 lbf, f′ = ln(63.799327/48.799327)/π = 0.085315363"
inputs = { T = "30 lbf·in", d = "4 in", D = "4 in", C = "20 in", t = "0.003 in", b = "0.75 in", f = "0.35", E = "28 Mpsi", "ν" = "0.285", stainless = "1", N_p = "1e6", d_min = "1.8 in" }
expect = { "φ" = "3.14159265", r_t = "3.0028368", S_f = "51211.778 psi", "σ_b" = "22856.521 psi", a = "85.06577 lbf/in", b_min = "0.26437634 in", "F_(1,a)" = "63.799327 lbf", F_2 = "48.799327 lbf", F_i = "56.299327 lbf", "f′" = "0.085315363" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: beryllium copper belt, Sf = Sy/3, on 4 and 6 in pulleys"
source = "By hand, Table 17-8 values for beryllium copper (Sy = 170 kpsi, E = 17 Mpsi, ν = 0.22): φ = π − 2 asin(2/40) = 3.0415509, exp(0.35φ) = 2.8995131, Sf = 56 666.667 psi, σb = 17e6 × 0.003/((1 − 0.0484) × 4) = 13 398.487 psi, a = 129.80454 lbf/in, bmin = 0.17639414 in; at b = 0.5 in (F1)a = 64.90227, F2 = 49.90227, Fi = 57.40227 lbf, f′ = 0.086408583"
inputs = { T = "30 lbf·in", d = "4 in", D = "6 in", C = "20 in", t = "0.003 in", b = "0.5 in", f = "0.35", E = "17 Mpsi", "ν" = "0.22", stainless = "0", S_y = "170 kpsi", d_min = "1.8 in" }
expect = { "φ" = "3.0415509", r_t = "2.8995131", S_f = "56666.667 psi", "σ_b" = "13398.487 psi", a = "129.80454 lbf/in", b_min = "0.17639414 in", "F_(1,a)" = "64.90227 lbf", F_2 = "49.90227 lbf", F_i = "57.40227 lbf", "f′" = "0.086408583" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 17-3 with a 0.25 in belt: under bmin, it slips"
source = "By hand: (F1)a = 85.06577 × 0.25 = 21.266443 lbf, F2 = 6.266443 lbf, f′ = ln(21.266443/6.266443)/π = 0.38893, over f = 0.35"
inputs = { T = "30 lbf·in", d = "4 in", D = "4 in", C = "20 in", t = "0.003 in", b = "0.25 in", f = "0.35", E = "28 Mpsi", "ν" = "0.285", stainless = "1", N_p = "1e6", d_min = "1.8 in" }
expect = { "F_(1,a)" = "21.266443 lbf", F_2 = "6.266443 lbf" }
tolerance = "1e-6"
checks = "fail"
+++

## Flat metal belt drive

A thin flat metal belt (stainless, beryllium copper, carbon steel, titanium or Inconel) driving by friction, after Shigley's steps for metal belts (pp. 887-890). A metal belt is light, so the centrifugal tension is left out. Its allowable tension is set by fatigue: the endurance strength for the life wanted, less the bending stress the belt takes each time it wraps the small pulley, times the thickness gives the allowable tension per width. The torque fixes the difference of the tensions; the narrowest belt that carries it with friction just fully developed is b_min, and a wider belt runs with less friction than f.

### Drive

```calc {description="Torque transmitted at the driving (small) pulley"}
T = 30 lbf·in_
```

```calc {description="Driving (small) pulley diameter"}
d = 4 in_
```

```calc {description="Driven pulley diameter"}
D = 4 in_
```

```calc {description="Centre distance"}
C = 20 in_
```

```calc {description="Smallest pulley for the belt thickness, Shigley Table 17-7"}
d_min = 1.8 in_
```

### Belt

The alloys' yield strength, modulus and Poisson's ratio are in Shigley Table 17-8. For 301 or 302 stainless the endurance strength for a life of N_p belt passes is Shigley's fit to Table 17-6 (Eq. 17-15), S_f = 14.17 × 10⁶ N_p^−0.407 psi, fitted to lives from about 85 000 to over 10⁶ passes; for the other alloys Shigley takes a third of the yield strength (p. 889, step 2).

```calc {description="Belt thickness"}
t = 0.003 in_
```

```calc {description="Belt width chosen"}
b = 0.75 in_
```

```calc {description="Coefficient of friction of the belt on the pulley"}
f = 0.35_
```

```calc {description="Young's modulus of the belt alloy, Shigley Table 17-8"}
E = 28 Mpsi_
```

```calc {description="Poisson's ratio of the belt alloy, Shigley Table 17-8"}
ν = 0.285_
```

```calc {description="Alloy: 1 for 301 or 302 stainless (Eq. 17-15), 0 for another alloy (Sy/3)"}
stainless = 1_
```

```calc {description="Life wanted in belt passes (stainless only)"}
N_p = 1e6_
```

```calc {description="Yield strength of the belt alloy, Shigley Table 17-8 (other alloys only)"}
S_y = 175 kpsi_
```

### Allowable tension

```calc
> Wrap on the small pulley, Eq. 17-1
φ = π - 2 asin((D - d) / (2 C)) = ??
> Tension ratio at full friction, exp(fφ)
r_t = exp(f φ) = ??
> Endurance strength, Eq. 17-15 or p. 889 step 2
S_f = {14.17e6 'psi' N_p^(-0.407) if stainless == 1; S_y / 3 if stainless == 0} = ?? psi
> Bending stress over the small pulley, Eq. 17-14
σ_b = E t / ((1 - ν^2) d) = ?? psi
> Allowable tension per width, p. 889 step 3
a = (S_f - σ_b) t = ?? lbf/in
```

### Tensions and width

```calc
> Tension difference, p. 889 step 4
ΔF = 2 T / d = ?? lbf
> Narrowest belt with friction just fully developed, p. 889 step 7
b_min = ΔF / a r_t / (r_t - 1) = ?? in
> Tight-side tension at the allowable, p. 889 step 8
F_(1,a) = a b = ?? lbf
> Slack-side tension
F_2 = F_(1,a) - ΔF = ?? lbf
> Initial tension
F_i = (F_(1,a) + F_2) / 2 = ?? lbf
> Friction developed, p. 890 step 9
f′ = ln(F_(1,a) / F_2) / φ = ??
```

### Checks

The bending stress must leave some of the endurance strength for tension, the belt must be at least b_min wide (the friction developed under f), and the small pulley at least Table 17-7's smallest.

```calc
@test σ_b < S_f
```

```calc
@test f′ < f
```

```calc
@test d ≥ d_min
```
