+++
cruncha-template = 1
id = "mechanical.fatigue.fluctuating"
version = "1.0.0"
title = "Fatigue: fluctuating stress, factors of safety (Goodman, Gerber, ASME-elliptic, Soderberg) and Langer yield"
summary = "Combined fluctuating bending, axial and torsional stresses at a notch reduced to von Mises alternating and mean stresses, the fatigue factor of safety by the Gerber, modified Goodman, ASME-elliptic, Soderberg and Smith-Dolan criteria, and the first-cycle yield factor by Langer."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["fatigue", "fluctuating stress", "mean stress", "Goodman", "Gerber", "ASME-elliptic", "Soderberg", "Smith-Dolan", "Langer", "von Mises", "factor of safety", "Shigley", "shaft"]
from = "lacewing:mechanical.fatigue"
license = "GPL-3.0-or-later"
inputs = ["S_e", "S_ut", "S_y", "K_fb", "K_fa", "K_fs", "σ_(a,b)", "σ_(m,b)", "σ_(a,ax)", "σ_(m,ax)", "τ_a", "τ_m", "n_d"]
outputs = ["σ_a′", "σ_m′", "σ_max′", "n_Gerber", "n_Goodman", "n_ASME", "n_Soderberg", "n_SmithDolan", "n_y"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 6-12 to 6-14: Eqs. 6-45 to 6-49 and Table 6-7 (criteria, Langer), Eq. 6-51 (Smith-Dolan), Eqs. 6-55 and 6-56 (combined loading); Examples 6-7 and 6-10"

[[tests]]
name = "Shigley Ex. 6-10: 1050 CD bar, axial 0 to 16 kip, Kf = 1.85"
source = "Lacewing Lab stress_test: Gerber nf = 3.66, Langer ny = 5.01, Sut 100, Sy 84 kpsi, σa = σm = 4.53 kpsi nominal. Shigley puts kc = 0.85 into Se = 33.9 kpsi; this template divides σa by 0.85 instead, so Se is entered as 33.9/0.85 = 39.88 kpsi (the same ratios). Goodman and ASME-elliptic as Lacewing's test: 1/(8.38/33.9 + 8.38/100) = 3.02, 1/√((8.38/33.9)² + (8.38/84)²) = 3.75; Soderberg by hand 1/(8.38/33.9 + 8.38/84) = 2.882"
inputs = { S_e = "39.882 ksi", S_ut = "100 ksi", S_y = "84 ksi", K_fb = "1", K_fa = "1.85", K_fs = "1", "σ_(a,b)" = "0 ksi", "σ_(m,b)" = "0 ksi", "σ_(a,ax)" = "4.53 ksi", "σ_(m,ax)" = "4.53 ksi", "τ_a" = "0 ksi", "τ_m" = "0 ksi", n_d = "1.5" }
expect = { "σ_a′" = "9.859 ksi", "σ_m′" = "8.381 ksi", n_Gerber = "3.66", n_y = "5.01", n_Goodman = "3.02", n_ASME = "3.75", n_Soderberg = "2.882" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 6-7: 260 MPa reversed bending at Kf = 1.55, Se = 280 MPa: finite life"
source = "Lacewing Lab stress_test: σ′a = 1.55 × 260 = 403 MPa (Sut 690, Sy 600 MPa as the test takes it); with no mean stress every criterion gives nf = Se/σ′a = 280/403 = 0.695, below 1"
inputs = { S_e = "280 MPa", S_ut = "690 MPa", S_y = "600 MPa", K_fb = "1.55", K_fa = "1", K_fs = "1", "σ_(a,b)" = "260 MPa", "σ_(m,b)" = "0 MPa", "σ_(a,ax)" = "0 MPa", "σ_(m,ax)" = "0 MPa", "τ_a" = "0 MPa", "τ_m" = "0 MPa", n_d = "1" }
expect = { "σ_a′" = "403 MPa", n_Gerber = "0.6948", n_Goodman = "0.6948", n_SmithDolan = "0.6948", n_y = "1.4888" }
tolerance = "0.01%"
checks = "fail"

[[tests]]
name = "Rotating shaft at a fillet: reversed bending with steady torsion"
source = "By hand, Eqs. 6-55, 6-56 and Table 6-7: σ′a = 1.55 × 70 = 108.5 MPa; σ′m = √3 × 1.31 × 60 = 136.1 MPa; Gerber nf = ½(690/136.1)²(108.5/236)[−1 + √(1 + (2 × 136.1 × 236/(690 × 108.5))²)] = 1.877; Goodman 1/(108.5/236 + 136.1/690) = 1.522; ny = 580/√(108.5² + 3(1.31 × 60)²) = 3.33"
inputs = { S_e = "236 MPa", S_ut = "690 MPa", S_y = "580 MPa", K_fb = "1.55", K_fa = "1", K_fs = "1.31", "σ_(a,b)" = "70 MPa", "σ_(m,b)" = "0 MPa", "σ_(a,ax)" = "0 MPa", "σ_(m,ax)" = "0 MPa", "τ_a" = "0 MPa", "τ_m" = "60 MPa", n_d = "1.5" }
expect = { "σ_a′" = "108.5 MPa", "σ_m′" = "136.14 MPa", n_Gerber = "1.877", n_Goodman = "1.522", n_y = "3.33" }
tolerance = "0.5%"
checks = "pass"
+++

## Fluctuating stress: fatigue and first-cycle yield

The factor of safety of a ductile part under fluctuating stress, for infinite life, by Shigley's method for combined loading (Sec. 6-14). Each stress component is split into its alternating part (σmax − σmin)/2 and its mean (midrange) part (σmax + σmin)/2. The fatigue stress-concentration factors multiply both parts, and the components are combined into von Mises alternating and mean stresses, which are then checked against the endurance limit and the ultimate and yield strengths. The loads are assumed to rise and fall together (proportional loading).

The endurance limit S_e is the part's, with its Marin factors and k_c = 1 (see the endurance-limit template): the alternating axial stress is divided by 0.85 here instead (Eq. 6-55), and torsion counts through the von Mises stress.

### Strengths and notch factors

```calc {description="Endurance limit of the part (Marin factors, k_c = 1)"}
S_e = 236 MPa_
```

```calc {description="Tensile ultimate strength"}
S_ut = 690 MPa_
```

```calc {description="Tensile yield strength"}
S_y = 580 MPa_
```

```calc {description="Fatigue stress-concentration factor in bending (1 for no notch)"}
K_fb = 1.55_
```

```calc {description="Fatigue stress-concentration factor in axial load (1 for no notch)"}
K_fa = 1_
```

```calc {description="Fatigue stress-concentration factor in torsion (1 for no notch)"}
K_fs = 1.31_
```

### Nominal stresses at the critical point

Nominal stresses, before the notch factors. A rotating shaft under a steady bending moment sees fully reversed bending (all alternating); a steady torque is all mean.

```calc {description="Bending stress, alternating"}
σ_(a,b) = 70 MPa_
```

```calc {description="Bending stress, mean (midrange)"}
σ_(m,b) = 0 MPa_
```

```calc {description="Axial stress, alternating"}
σ_(a,ax) = 0 MPa_
```

```calc {description="Axial stress, mean (midrange)"}
σ_(m,ax) = 0 MPa_
```

```calc {description="Torsional shear stress, alternating"}
τ_a = 0 MPa_
```

```calc {description="Torsional shear stress, mean (midrange)"}
τ_m = 60 MPa_
```

```calc {description="Design factor required against fatigue and first-cycle yield"}
n_d = 1.5_
```

### Von Mises alternating and mean stresses

```calc {description="Von Mises alternating stress, Eq. 6-55: √(σ² + 3τ²)"}
σ_a′ = hypot(K_fb σ_(a,b) + K_fa σ_(a,ax) / 0.85, sqrt(3) K_fs τ_a) = ?? MPa
```

```calc {description="Von Mises mean stress, Eq. 6-56: √(σ² + 3τ²)"}
σ_m′ = hypot(K_fb σ_(m,b) + K_fa σ_(m,ax), sqrt(3) K_fs τ_m) = ?? MPa
```

The largest stress of the cycle combines the largest normal and shear stresses; a compressive mean adds to it as a tensile one does.

```calc {description="Largest von Mises stress of the cycle, for first-cycle yield"}
σ_max′ = hypot(abs(K_fb σ_(m,b) + K_fa σ_(m,ax)) + K_fb abs(σ_(a,b)) + K_fa abs(σ_(a,ax)), sqrt(3) K_fs (abs(τ_a) + abs(τ_m))) = ?? MPa
```

### Fatigue factors of safety

Gerber's parabola fits test data for ductile materials best and is the check here. Modified Goodman and Soderberg are more conservative; ASME-elliptic is close to Gerber but uses the yield strength. Smith-Dolan is for brittle materials. The von Mises mean stress is never negative, so a compressive mean is treated as a tensile one.

```calc {description="Fatigue factor of safety, Gerber, Table 6-7"}
n_Gerber = { S_e / σ_a′ if σ_m′ ≤ 0 'MPa'; S_ut / σ_m′ if σ_a′ ≤ 0 'MPa'; 0.5 (S_ut / σ_m′)^2 (σ_a′ / S_e) (-1 + sqrt(1 + (2 σ_m′ S_e / (S_ut σ_a′))^2)) otherwise } = ??
```

```calc {description="Fatigue factor of safety, modified Goodman, Eq. 6-46"}
n_Goodman = 1 / (σ_a′ / S_e + σ_m′ / S_ut) = ??
```

```calc {description="Fatigue factor of safety, ASME-elliptic, Eq. 6-48"}
n_ASME = 1 / sqrt((σ_a′ / S_e)^2 + (σ_m′ / S_y)^2) = ??
```

```calc {description="Fatigue factor of safety, Soderberg, Eq. 6-45"}
n_Soderberg = 1 / (σ_a′ / S_e + σ_m′ / S_y) = ??
```

```calc
> Smith-Dolan, Eq. 6-51 solved for n: the terms of the quadratic
c_SD = σ_a′ / S_e × σ_m′ / S_ut = ??
> Smith-Dolan, linear term
b_SD = σ_a′ / S_e + σ_m′ / S_ut = ??
> Fatigue factor of safety, Smith-Dolan (brittle materials)
n_SmithDolan = { S_e / σ_a′ if σ_m′ ≤ 0 'MPa'; S_ut / σ_m′ if σ_a′ ≤ 0 'MPa'; (-b_SD + sqrt(b_SD^2 + 4 c_SD)) / (2 c_SD) otherwise } = ??
```

### First-cycle yield

```calc {description="Yield factor of safety on the first cycle, Langer, Eq. 6-49"}
n_y = S_y / σ_max′ = ??
```

### Checks

If n_Gerber is below 1 the part has a finite life: find it with the fatigue-life template from σ′a and σ′m.

```calc
@test n_Gerber ≥ n_d
```

```calc
@test n_y ≥ n_d
```
