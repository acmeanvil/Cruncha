+++
cruncha-template = 1
id = "mechanical.staticFailure.brittle"
version = "1.0.0"
title = "Static fracture of a brittle material, plane stress"
summary = "Principal stresses of a plane stress state and the factor of safety against fracture of a brittle material by maximum normal stress (Rankine), brittle Coulomb-Mohr and modified Mohr, the theory Shigley's Fig. 5-21 recommends, checked against a design factor."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["static failure", "fracture", "brittle", "cast iron", "modified Mohr", "Coulomb-Mohr", "maximum normal stress", "Rankine", "principal stress", "plane stress", "factor of safety"]
from = "lacewing:mechanical.staticFailure"
license = "GPL-3.0-or-later"
inputs = ["σ_x", "σ_y", "τ_xy", "S_ut", "S_uc", "n_d"]
outputs = ["σ_A", "σ_B", "σ_1", "σ_3", "n_MNS", "n_BCM", "n_MM"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Eq. 3-13 (principal stresses); Eqs. 5-31 (brittle Coulomb-Mohr), 5-32 (modified Mohr); Fig. 5-21 (choice of theory); Example 5-5"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 6.6 and Fig. 6.7 (maximum normal stress), Fig. 6.12 (modified Mohr), Sec. 6.10 (selection); Sample Problem 6.3"

[[tests]]
name = "Shigley Ex. 5-5: grade 30 cast iron wrench at F = 100 lbf"
source = "Lacewing Lab stress_test, after Shigley 10th Example 5-5: σx = 142.6F, τxy = 76.4F psi; Sut 31, Suc 109 kpsi; fracture at F = 167 lbf by brittle Coulomb-Mohr and 176 lbf by modified Mohr, so n = 1.67 and 1.76 at 100 lbf"
inputs = { "σ_x" = "14.26 ksi", "σ_y" = "0 ksi", "τ_xy" = "7.64 ksi", S_ut = "31 ksi", S_uc = "109 ksi", n_d = "1.5" }
expect = { n_BCM = "1.67", n_MM = "1.76" }
tolerance = "0.6%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 6.3: σ1 = 35, σ2 = −25 ksi, S = 100 ksi, maximum normal stress"
source = "Lacewing Lab stress_test, after Juvinall & Marshek 5th Sample Problem 6.3: the maximum normal stress theory gives n = 2.86"
inputs = { "σ_x" = "35 ksi", "σ_y" = "-25 ksi", "τ_xy" = "0 ksi", S_ut = "100 ksi", S_uc = "100 ksi", n_d = "1.5" }
expect = { n_MNS = "2.86" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 5-5 at 100 lbf to six figures"
source = "By hand: σA,B = 7.13 ± √(7.13² + 7.64²) = 17.58019, −3.32019 ksi; |σB/σA| ≤ 1 so n_MM = 31/17.58019 = 1.763348; n_BCM = 1/(17.58019/31 + 3.32019/109) = 1.673462; n_MNS = 31/17.58019 = 1.763348"
inputs = { "σ_x" = "14.26 ksi", "σ_y" = "0 ksi", "τ_xy" = "7.64 ksi", S_ut = "31 ksi", S_uc = "109 ksi", n_d = "1.5" }
expect = { "σ_A" = "17.58019 ksi", "σ_B" = "-3.32019 ksi", n_BCM = "1.673462", n_MM = "1.763348", n_MNS = "1.763348" }
tolerance = "2e-6"
checks = "pass"

[[tests]]
name = "Hand check: compression and shear, the modified Mohr's fourth-quadrant line"
source = "By hand: σx = −20, τxy = 10 ksi: σA,B = −10 ± 14.14214 = 4.142136, −24.14214 ksi; |σB/σA| > 1 so n_MM = 1/(78 × 4.142136/(109 × 31) + 24.14214/109) = 3.153544; n_BCM = 1/(4.142136/31 + 24.14214/109) = 2.816070; n_MNS = 109/24.14214 = 4.514927"
inputs = { "σ_x" = "-20 ksi", "σ_y" = "0 ksi", "τ_xy" = "10 ksi", S_ut = "31 ksi", S_uc = "109 ksi", n_d = "2" }
expect = { "σ_1" = "4.142136 ksi", "σ_3" = "-24.14214 ksi", n_MM = "3.153544", n_BCM = "2.816070", n_MNS = "4.514927" }
tolerance = "2e-6"
checks = "pass"

[[tests]]
name = "The Ex. 5-5 wrench at 180 lbf breaks"
source = "Shigley 10th Example 5-5: modified Mohr predicts fracture at 176 lbf; by hand n_MM = 1.763348 × 100/180 = 0.979638"
inputs = { "σ_x" = "25.668 ksi", "σ_y" = "0 ksi", "τ_xy" = "13.752 ksi", S_ut = "31 ksi", S_uc = "109 ksi", n_d = "1" }
expect = { n_MM = "0.979638" }
tolerance = "2e-6"
checks = "fail"
+++

## Static fracture of a brittle material, plane stress

Factor of safety against fracture at one point of a brittle part under static load. Shigley's Fig. 5-21 treats a material as brittle when its true strain at fracture is under 5 % (cast irons, for example); for a ductile one, use the plane-stress yield template. The stress state is plane (σx, σy and τxy, the out-of-plane stress zero), tension positive.

Brittle materials are sensitive to stress raisers, so the stresses should include the stress concentration factor where there is one. Their compressive ultimate strength is often three to four times the tensile one: for cast irons, enter it.

### Stress at the point

```calc {description="Normal stress on the x face (tension positive)"}
σ_x = 14.26 ksi_
```

```calc {description="Normal stress on the y face (tension positive)"}
σ_y = 0 ksi_
```

```calc {description="Shear stress on the x face, y direction"}
τ_xy = 7.64 ksi_
```

### Material

```calc {description="Ultimate strength in tension"}
S_ut = 31 ksi_
```

```calc {description="Ultimate strength in compression (a positive number)"}
S_uc = 109 ksi_
```

```calc {description="Design factor the factor of safety must reach"}
n_d = 1.5_
```

### Principal stresses

With the zero out-of-plane stress, σ₁ is the larger of σA and zero, σ₃ the smaller of σB and zero.

```calc
> Centre of the in-plane Mohr circle
σ_c = (σ_x + σ_y) / 2 = ?? ksi
> Its radius
R_c = hypot((σ_x - σ_y) / 2, τ_xy) = ?? ksi
> In-plane principal stresses, Eq. 3-13
σ_A = σ_c + R_c = ?? ksi
σ_B = σ_c - R_c = ?? ksi
> Largest principal stress
σ_1 = max(σ_A, 0 'ksi') = ?? ksi
> Smallest principal stress
σ_3 = min(σ_B, 0 'ksi') = ?? ksi
```

### Factors of safety

```calc {description="Maximum normal stress (Rankine): fracture when σ₁ reaches Sut or σ₃ reaches −Suc (Juvinall Sec. 6.6)"}
n_MNS = 1 / max(σ_1 / S_ut, -σ_3 / S_uc) = ??
```

```calc {description="Brittle Coulomb-Mohr, Eq. 5-31 (conservative)"}
n_BCM = 1 / (σ_1 / S_ut - σ_3 / S_uc) = ??
```

The modified Mohr theory follows the maximum normal stress line until the compressive principal stress exceeds the tensile one in size, then a straight line to the compressive strength (Eq. 5-32, with σA = σ₁ and σB = σ₃). It is closest to the test data for brittle materials, and is the theory Fig. 5-21 recommends; brittle Coulomb-Mohr is the conservative alternative.

```calc {description="Modified Mohr, Eq. 5-32: the recommended theory"}
n_MM = { S_ut / σ_1 if -σ_3 ≤ σ_1; 1 / ((S_uc - S_ut) σ_1 / (S_uc S_ut) - σ_3 / S_uc) otherwise } = ??
```

### Check

```calc
@test n_MM ≥ n_d
```
