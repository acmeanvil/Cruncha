+++
cruncha-template = 1
id = "mechanical.staticFailure.planeStress"
version = "1.0.0"
title = "Static yield of a ductile material, plane stress"
summary = "Principal stresses, maximum shear, von Mises and octahedral stresses of a plane stress state, and the factor of safety against yield by maximum shear stress (Tresca), distortion energy (von Mises) and ductile Coulomb-Mohr, with the theory Shigley's Fig. 5-21 recommends checked against a design factor."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["static failure", "yield", "von Mises", "distortion energy", "Tresca", "maximum shear stress", "Coulomb-Mohr", "principal stress", "Mohr's circle", "plane stress", "factor of safety", "ductile"]
from = "lacewing:mechanical.staticFailure"
license = "GPL-3.0-or-later"
inputs = ["σ_x", "σ_y", "τ_xy", "S_yt", "S_yc", "n_d"]
outputs = ["σ_A", "σ_B", "σ_1", "σ_2", "σ_3", "τ_max", "σ′", "τ_oct", "n_MSS", "n_DE", "n_DCM", "n"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Eqs. 3-13, 3-16 (principal and maximum shear stresses); Eqs. 5-3, 5-13, 5-16, 5-19 (MSS, distortion energy, octahedral shear), 5-26 (ductile Coulomb-Mohr); Fig. 5-21 (choice of theory); Examples 5-1 and 5-2"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 6.7 to 6.10; Sample Problem 6.3"

[[tests]]
name = "Shigley Ex. 5-1(a): σx = σy = 70 ksi, Sy = 100 ksi"
source = "Lacewing Lab stress_test, after Shigley 10th Example 5-1(a): n = 1.43 by both distortion energy and maximum shear stress"
inputs = { "σ_x" = "70 ksi", "σ_y" = "70 ksi", "τ_xy" = "0 ksi", S_yt = "100 ksi", S_yc = "100 ksi", n_d = "1.2" }
expect = { "σ′" = "70 ksi", n_DE = "1.43", n_MSS = "1.43" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 5-1(b): 60, 40, −15 ksi"
source = "Lacewing Lab stress_test, after Shigley 10th Example 5-1(b): σA = 68.0, σB = 32.0 ksi; n = 1.70 by distortion energy, 1.47 by maximum shear stress"
inputs = { "σ_x" = "60 ksi", "σ_y" = "40 ksi", "τ_xy" = "-15 ksi", S_yt = "100 ksi", S_yc = "100 ksi", n_d = "1.5" }
expect = { "σ_A" = "68.0 ksi", "σ_B" = "32.0 ksi", n_DE = "1.70", n_MSS = "1.47", n = "1.70" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 5-1(c): 0, 40, 45 ksi"
source = "Lacewing Lab stress_test, after Shigley 10th Example 5-1(c): σA = 69.2, σB = −29.2 ksi; n = 1.14 by distortion energy, 1.02 by maximum shear stress"
inputs = { "σ_x" = "0 ksi", "σ_y" = "40 ksi", "τ_xy" = "45 ksi", S_yt = "100 ksi", S_yc = "100 ksi", n_d = "1.1" }
expect = { "σ_1" = "69.2 ksi", "σ_3" = "-29.2 ksi", n_DE = "1.14", n_MSS = "1.02" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 5-1(d): −40, −60, 15 ksi (all compressive)"
source = "Lacewing Lab stress_test, after Shigley 10th Example 5-1(d): n = 1.70 by distortion energy, 1.47 by maximum shear stress"
inputs = { "σ_x" = "-40 ksi", "σ_y" = "-60 ksi", "τ_xy" = "15 ksi", S_yt = "100 ksi", S_yc = "100 ksi", n_d = "1.5" }
expect = { "σ_1" = "0 ksi", "σ_3" = "-68.03 ksi", n_DE = "1.70", n_MSS = "1.47" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 5-2: pure torsion of cast 195-T6 aluminum, ductile Coulomb-Mohr"
source = "Lacewing Lab stress_test, after Shigley 10th Example 5-2: Syt 160, Syc 170 MPa, τ 75 MPa gives n = 1.10, the recommended theory for unequal yield strengths"
inputs = { "σ_x" = "0 MPa", "σ_y" = "0 MPa", "τ_xy" = "75 MPa", S_yt = "160 MPa", S_yc = "170 MPa", n_d = "1" }
expect = { n_DCM = "1.10", n = "1.10" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 6.3: σ1 = 35, σ2 = −25 ksi, Sy = 100 ksi"
source = "Lacewing Lab stress_test, after Juvinall & Marshek 5th Sample Problem 6.3: σ′ = 52.2 ksi, n = 1.92 by distortion energy, 1.67 by maximum shear stress"
inputs = { "σ_x" = "35 ksi", "σ_y" = "-25 ksi", "τ_xy" = "0 ksi", S_yt = "100 ksi", S_yc = "100 ksi", n_d = "1.5" }
expect = { "σ′" = "52.2 ksi", n_DE = "1.92", n_MSS = "1.67", "τ_oct" = "24.6 ksi" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 5-1(b) exactly"
source = "By hand: σA,B = 50 ± √(10² + 15²) = 68.0278, 31.9722 ksi; σ′ = √(σA² − σAσB + σB²) = √3475 = 58.9491 ksi; τmax = 34.0139 ksi; n_DE = 100/58.9491 = 1.69638; n_MSS = 100/68.0278 = 1.46999"
inputs = { "σ_x" = "60 ksi", "σ_y" = "40 ksi", "τ_xy" = "-15 ksi", S_yt = "100 ksi", S_yc = "100 ksi", n_d = "1.5" }
expect = { "σ_A" = "68.0277564 ksi", "σ_B" = "31.9722436 ksi", "σ′" = "58.9491306 ksi", "τ_max" = "34.0138782 ksi", n_DE = "1.6963779", n_MSS = "1.4699882" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 5-1(b) against a design factor of 2: not enough margin"
source = "Shigley 10th Example 5-1(b): n = 1.70 by distortion energy, under 2"
inputs = { "σ_x" = "60 ksi", "σ_y" = "40 ksi", "τ_xy" = "-15 ksi", S_yt = "100 ksi", S_yc = "100 ksi", n_d = "2" }
expect = { n = "1.6963779" }
tolerance = "1e-6"
checks = "fail"
+++

## Static yield of a ductile material, plane stress

Factor of safety against yielding at one point of a part under static load, for a ductile material, by the theories of Shigley ch. 5. The stress state is plane: σx, σy and τxy act in the plane, and the out-of-plane stress is zero. Tension is positive. For a full three-dimensional state, use the triaxial template; for a brittle material (strain at fracture under 5 %), the brittle fracture template; for fluctuating loads, a fatigue check.

The stresses are those at the point. Static loads on ductile materials usually need no stress concentration factor (Shigley Sec. 3-13).

### Stress at the point

```calc {description="Normal stress on the x face (tension positive)"}
σ_x = 60 ksi_
```

```calc {description="Normal stress on the y face (tension positive)"}
σ_y = 40 ksi_
```

```calc {description="Shear stress on the x face, y direction"}
τ_xy = -15 ksi_
```

### Material

Enter S_yc equal to S_yt unless the material's compressive yield strength differs (some cast alloys). Where they differ by more than 5 %, ductile Coulomb-Mohr is the recommended theory.

```calc {description="Yield strength in tension"}
S_yt = 100 ksi_
```

```calc {description="Yield strength in compression"}
S_yc = 100 ksi_
```

```calc {description="Design factor the factor of safety must reach"}
n_d = 1.5_
```

### Principal stresses

The in-plane principal stresses are the centre of Mohr's circle plus and minus its radius. With the zero out-of-plane stress they are ordered σ₁ ≥ σ₂ ≥ σ₃, so σ₁ is never negative and σ₃ never positive.

```calc
> Centre of the in-plane Mohr circle
σ_c = (σ_x + σ_y) / 2 = ?? ksi
> Its radius, the largest in-plane shear stress
R_c = hypot((σ_x - σ_y) / 2, τ_xy) = ?? ksi
> In-plane principal stresses, Eq. 3-13
σ_A = σ_c + R_c = ?? ksi
σ_B = σ_c - R_c = ?? ksi
> Largest principal stress, with the zero out-of-plane stress
σ_1 = max(σ_A, 0 'ksi') = ?? ksi
> Smallest principal stress
σ_3 = min(σ_B, 0 'ksi') = ?? ksi
> Middle principal stress
σ_2 = σ_A + σ_B - σ_1 - σ_3 = ?? ksi
> Maximum shear stress, the largest Mohr circle's radius, Eq. 3-16
τ_max = (σ_1 - σ_3) / 2 = ?? ksi
```

### Von Mises stress

```calc
> Von Mises stress, plane stress, Eq. 5-13
σ′ = sqrt(σ_A^2 - σ_A σ_B + σ_B^2) = ?? ksi
> Octahedral shear stress, Eq. 5-16
τ_oct = sqrt(2) / 3 σ′ = ?? ksi
```

### Factors of safety

```calc {description="Maximum shear stress (Tresca), Eq. 5-3: yield when τmax reaches Sy/2"}
n_MSS = S_yt / (σ_1 - σ_3) = ??
```

```calc {description="Distortion energy (von Mises), Eq. 5-19"}
n_DE = S_yt / σ′ = ??
```

```calc {description="Ductile Coulomb-Mohr, Eq. 5-26"}
n_DCM = 1 / (σ_1 / S_yt - σ_3 / S_yc) = ??
```

Fig. 5-21 recommends distortion energy for a ductile material with equal yield strengths (maximum shear stress is the conservative alternative) and ductile Coulomb-Mohr where they differ.

```calc {description="Factor of safety by the theory Fig. 5-21 recommends"}
n = { n_DE if abs(S_yc - S_yt) ≤ 0.05 S_yt; n_DCM otherwise } = ??
```

### Check

```calc
@test n ≥ n_d
```
