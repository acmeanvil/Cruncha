+++
cruncha-template = 1
id = "mechanical.staticFailure.triaxial"
version = "1.0.0"
title = "Static yield of a ductile material, 3D stress"
summary = "Principal stresses of a full three-dimensional stress state (the roots of the stress cubic, in closed form), maximum shear, von Mises and octahedral stresses, and the factor of safety against yield by maximum shear stress, distortion energy and ductile Coulomb-Mohr, with the theory Shigley's Fig. 5-21 recommends checked against a design factor."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["static failure", "yield", "von Mises", "distortion energy", "Tresca", "maximum shear stress", "Coulomb-Mohr", "principal stress", "stress invariants", "triaxial", "3D stress", "factor of safety", "ductile"]
from = "lacewing:mechanical.staticFailure"
license = "GPL-3.0-or-later"
inputs = ["σ_x", "σ_y", "σ_z", "τ_xy", "τ_yz", "τ_zx", "S_yt", "S_yc", "n_d"]
outputs = ["I_1", "I_2", "I_3", "σ_1", "σ_2", "σ_3", "τ_max", "σ′", "τ_oct", "n_MSS", "n_DE", "n_DCM", "n"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Eqs. 3-15, 3-16 (principal stresses as the roots of the cubic, maximum shear); Eqs. 5-3, 5-14, 5-16, 5-19 (MSS, distortion energy from the components, octahedral shear), 5-26 (ductile Coulomb-Mohr); Fig. 5-21; Examples 5-1 and 5-2"

[[tests]]
name = "A general 3D state: 80, −30, 20 MPa; τ 40, −15, 25 MPa"
source = "Lacewing Lab stress_test's 3D state; principal stresses checked independently with numpy.linalg.eigvalsh: 97.923206, 22.240559, −50.163765 MPa; σ′ = 128.257553 MPa; I3 = −109 250 MPa³"
inputs = { "σ_x" = "80 MPa", "σ_y" = "-30 MPa", "σ_z" = "20 MPa", "τ_xy" = "40 MPa", "τ_yz" = "-15 MPa", "τ_zx" = "25 MPa", S_yt = "250 MPa", S_yc = "250 MPa", n_d = "1.5" }
expect = { I_1 = "70 MPa", I_2 = "-3850 MPa^2", I_3 = "-109250 MPa^3", "σ_1" = "97.9232058 MPa", "σ_2" = "22.2405590 MPa", "σ_3" = "-50.1637648 MPa", "σ′" = "128.2575534 MPa", "τ_oct" = "60.4611905 MPa", n_DE = "1.9492029", n_MSS = "1.6881971", n = "1.9492029" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Shigley Ex. 5-1(b) as a 3D state: 60, 40, −15 ksi"
source = "Shigley 10th Example 5-1(b) (Lacewing Lab stress_test): n = 1.70 by distortion energy, 1.47 by maximum shear stress; the out-of-plane principal stress is zero"
inputs = { "σ_x" = "60 ksi", "σ_y" = "40 ksi", "σ_z" = "0 ksi", "τ_xy" = "-15 ksi", "τ_yz" = "0 ksi", "τ_zx" = "0 ksi", S_yt = "100 ksi", S_yc = "100 ksi", n_d = "1.5" }
expect = { "σ_1" = "68.03 ksi", "σ_2" = "31.97 ksi", "σ_3" = "0 ksi", n_DE = "1.70", n_MSS = "1.47" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 5-2: torsion of cast 195-T6 aluminum, ductile Coulomb-Mohr"
source = "Shigley 10th Example 5-2 (Lacewing Lab stress_test): Syt 160, Syc 170 MPa, τ 75 MPa, n = 1.10"
inputs = { "σ_x" = "0 MPa", "σ_y" = "0 MPa", "σ_z" = "0 MPa", "τ_xy" = "75 MPa", "τ_yz" = "0 MPa", "τ_zx" = "0 MPa", S_yt = "160 MPa", S_yc = "170 MPa", n_d = "1" }
expect = { "σ_1" = "75 MPa", "σ_3" = "-75 MPa", n_DCM = "1.10", n = "1.10" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shear in the yz plane with unequal yield strengths"
source = "Checked independently with numpy.linalg.eigvalsh: σ = 45.413813, −15.413813, −50 MPa; σ′ = 83.666003 MPa; n_DCM = 1/(45.413813/160 + 50/170) = 1.730242"
inputs = { "σ_x" = "-50 MPa", "σ_y" = "20 MPa", "σ_z" = "10 MPa", "τ_xy" = "0 MPa", "τ_yz" = "30 MPa", "τ_zx" = "0 MPa", S_yt = "160 MPa", S_yc = "170 MPa", n_d = "1.5" }
expect = { "σ_1" = "45.4138127 MPa", "σ_2" = "-15.4138127 MPa", "σ_3" = "-50 MPa", "σ′" = "83.6660027 MPa", n_DE = "1.9123658", n_DCM = "1.7302416", n = "1.7302416" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The general 3D state against a design factor of 2"
source = "As the first case: n = 1.949 by distortion energy, under 2"
inputs = { "σ_x" = "80 MPa", "σ_y" = "-30 MPa", "σ_z" = "20 MPa", "τ_xy" = "40 MPa", "τ_yz" = "-15 MPa", "τ_zx" = "25 MPa", S_yt = "250 MPa", S_yc = "250 MPa", n_d = "2" }
expect = { n = "1.9492029" }
tolerance = "1e-6"
checks = "fail"
+++

## Static yield of a ductile material, 3D stress

Factor of safety against yielding at one point of a part under static load, for a ductile material, when all six stress components may act. Tension is positive. For plane stress the plane-stress template is shorter and gives the same results; for a brittle material, use the brittle fracture template.

A purely hydrostatic state (σx = σy = σz, no shear) does not distort the material and cannot yield it by these theories; the template does not evaluate it (the deviatoric invariant is zero).

### Stress at the point

```calc {description="Normal stress on the x face (tension positive)"}
σ_x = 80 MPa_
```

```calc {description="Normal stress on the y face (tension positive)"}
σ_y = -30 MPa_
```

```calc {description="Normal stress on the z face (tension positive)"}
σ_z = 20 MPa_
```

```calc {description="Shear stress on the x face, y direction"}
τ_xy = 40 MPa_
```

```calc {description="Shear stress on the y face, z direction"}
τ_yz = -15 MPa_
```

```calc {description="Shear stress on the z face, x direction"}
τ_zx = 25 MPa_
```

### Material

Enter S_yc equal to S_yt unless the compressive yield strength differs. Where they differ by more than 5 %, ductile Coulomb-Mohr is the recommended theory.

```calc {description="Yield strength in tension"}
S_yt = 250 MPa_
```

```calc {description="Yield strength in compression"}
S_yc = 250 MPa_
```

```calc {description="Design factor the factor of safety must reach"}
n_d = 1.5_
```

### Principal stresses

The principal stresses are the roots of the cubic σ³ − I₁σ² + I₂σ − I₃ = 0 (Eq. 3-15), whose coefficients are the stress invariants. The cubic's three real roots are found in closed form from the mean stress and the deviatoric invariants J₂ and J₃ (the trigonometric solution), which orders them σ₁ ≥ σ₂ ≥ σ₃. The roots agree with an eigenvalue solver to about a billionth of the largest stress; a principal stress that is exactly zero may print as a tiny number.

```calc
> First invariant
I_1 = σ_x + σ_y + σ_z = ?? MPa
> Second invariant
I_2 = σ_x σ_y + σ_y σ_z + σ_z σ_x - τ_xy^2 - τ_yz^2 - τ_zx^2 = ?? MPa^2
> Third invariant, the determinant of the stress tensor
I_3 = σ_x σ_y σ_z + 2 τ_xy τ_yz τ_zx - σ_x τ_yz^2 - σ_y τ_zx^2 - σ_z τ_xy^2 = ?? MPa^3
```

```calc
> Mean (hydrostatic) stress
σ_m = I_1 / 3 = ?? MPa
> Second deviatoric invariant
J_2 = I_1^2 / 3 - I_2 = ?? MPa^2
> Third deviatoric invariant
J_3 = 2 I_1^3 / 27 - I_1 I_2 / 3 + I_3 = ?? MPa^3
> Lode angle, 0 to π/3
θ_L = acos(min(max(J_3 / 2 (3 / J_2)^(3/2), -1), 1)) / 3 = ??
> Radius of the deviatoric roots
r_d = 2 sqrt(J_2 / 3) = ?? MPa
```

```calc
> Largest principal stress
σ_1 = σ_m + r_d cos(θ_L) = ?? MPa
> Middle principal stress
σ_2 = σ_m + r_d cos(θ_L - 2 π / 3) = ?? MPa
> Smallest principal stress
σ_3 = σ_m + r_d cos(θ_L + 2 π / 3) = ?? MPa
> Maximum shear stress, Eq. 3-16
τ_max = (σ_1 - σ_3) / 2 = ?? MPa
```

### Von Mises stress

```calc
> Von Mises stress from the components, Eq. 5-14
σ′ = sqrt(((σ_x - σ_y)^2 + (σ_y - σ_z)^2 + (σ_z - σ_x)^2 + 6 (τ_xy^2 + τ_yz^2 + τ_zx^2)) / 2) = ?? MPa
> Octahedral shear stress, Eq. 5-16
τ_oct = sqrt(2) / 3 σ′ = ?? MPa
```

### Factors of safety

```calc {description="Maximum shear stress (Tresca), Eq. 5-3"}
n_MSS = S_yt / (σ_1 - σ_3) = ??
```

```calc {description="Distortion energy (von Mises), Eq. 5-19"}
n_DE = S_yt / σ′ = ??
```

Ductile Coulomb-Mohr (Eq. 5-26) uses σ₁ and σ₃; a positive σ₃ or a negative σ₁ counts as zero.

```calc {description="Ductile Coulomb-Mohr, Eq. 5-26"}
n_DCM = 1 / (max(σ_1, 0 'MPa') / S_yt - min(σ_3, 0 'MPa') / S_yc) = ??
```

Fig. 5-21 recommends distortion energy for a ductile material with equal yield strengths (maximum shear stress is the conservative alternative) and ductile Coulomb-Mohr where they differ.

```calc {description="Factor of safety by the theory Fig. 5-21 recommends"}
n = { n_DE if abs(S_yc - S_yt) ≤ 0.05 S_yt; n_DCM otherwise } = ??
```

### Check

```calc
@test n ≥ n_d
```
