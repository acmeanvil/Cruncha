+++
cruncha-template = 1
id = "mechanical.shaftDesign.factorOfSafety"
version = "1.0.0"
title = "Shaft design: fatigue and yield factors of safety at a section (DE-Goodman, DE-Gerber, DE-ASME elliptic, DE-Soderberg)"
summary = "One section of a rotating shaft, solid or hollow, under alternating and midrange bending, torque and axial load: the von Mises alternating and midrange stresses with the fatigue notch factors, the fatigue factor of safety by the four distortion-energy criteria of Shigley Sec. 7-4, and the first-cycle yield factor of safety on the von Mises maximum stress."
category = ["Mechanical engineering", "Shafts"]
tags = ["shaft", "shaft design", "fatigue", "DE-Goodman", "DE-Gerber", "DE-ASME elliptic", "DE-Soderberg", "von Mises", "factor of safety", "yield", "Langer", "hollow shaft", "Shigley"]
from = "lacewing:mechanical.shaftDesign"
license = "GPL-3.0-or-later"
inputs = ["d", "d_i", "M_a", "M_m", "T_a", "T_m", "F_a", "F_m", "K_f", "K_fs", "K_fa", "S_e", "S_ut", "S_y", "crit", "n_d"]
outputs = ["Z", "A", "σ_a′", "σ_m′", "σ_max′", "n_Goodman", "n_Gerber", "n_ASME", "n_Soderberg", "n_f", "n_y", "n_(y,quick)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-4, Eqs. 7-1 to 7-7, 7-9, 7-11, 7-13, 7-15, 7-16; Sec. 6-14, Eqs. 6-55, 6-56 (axial load); Examples 7-1 and 7-2"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.5 (shaft design for fatigue; the cross-check method this template does not use)"

[[tests]]
name = "Shigley Ex. 7-1: 1.100 in shoulder, Ma 1260 lbf·in, steady Tm 1100 lbf·in"
source = "Lacewing Lab shaft_design_test, after Shigley 10th Example 7-1: Kf 1.58, Kfs 1.37, Se 29.3 kpsi (99 % reliability), Sut 105, Sy 82 kpsi; σ′a 15 235 psi, σ′m 9988 psi, σ′max 18 220 psi; n 1.63 DE-Goodman, 1.86 DE-Gerber (the book prints 1.87), 1.87 DE-ASME elliptic (the book 1.88), 1.56 DE-Soderberg; ny 4.50, quick 3.25"
inputs = { d = "1.1 in", d_i = "0 in", M_a = "1260 lbf·in", M_m = "0 lbf·in", T_a = "0 lbf·in", T_m = "1100 lbf·in", F_a = "0 lbf", F_m = "0 lbf", K_f = "1.58", K_fs = "1.37", K_fa = "1", S_e = "29.3 kpsi", S_ut = "105 kpsi", S_y = "82 kpsi", crit = "1", n_d = "1.5" }
expect = { "σ_a′" = "15235 psi", "σ_m′" = "9988 psi", "σ_max′" = "18217 psi", n_Goodman = "1.626", n_Gerber = "1.863", n_ASME = "1.872", n_Soderberg = "1.558", n_y = "4.50", "n_(y,quick)" = "3.25" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 7-2: the keyway end of the countershaft, 1020 CD, fails at n = 1.5"
source = "Lacewing Lab shaft_design_test, after Shigley 10th Example 7-2: d 1.625 in, Ma 3750 lbf·in, Tm 3240 lbf·in, Kf 1.74, Kfs 2.42, Se 25.1, Sut 68, Sy 57 kpsi: nf = 1.17 by DE-Goodman, under the design factor 1.5"
inputs = { d = "1.625 in", d_i = "0 in", M_a = "3750 lbf·in", M_m = "0 lbf·in", T_a = "0 lbf·in", T_m = "3240 lbf·in", F_a = "0 lbf", F_m = "0 lbf", K_f = "1.74", K_fs = "2.42", K_fa = "1", S_e = "25.1 kpsi", S_ut = "68 kpsi", S_y = "57 kpsi", crit = "1", n_d = "1.5" }
expect = { n_Goodman = "1.17", n_f = "1.17" }
tolerance = "0.3%"
checks = "fail"

[[tests]]
name = "Hand check: hollow shaft with axial load, DE-Gerber chosen"
source = "By hand, Eqs. 7-1 to 7-16 with the axial term of Eq. 6-55: d 40 mm, bore 20 mm, Z = π(40⁴ − 20⁴)/(32·40) = 5890.49 mm³, A = 942.478 mm²; Ma 400 N·m, Tm 300 N·m, Fa 2 kN, Fm 1 kN; Kf 1.6, Kfs 1.3, Kf,ax 1.5; σ′a = √((108.65 + 3.1831/0.85)²) = 112.395 MPa, σ′m = √(1.5915² + 3·33.104²) = 57.360 MPa; Se 200, Sut 600, Sy 450 MPa: Goodman 1.52074, Gerber 1.73073, ASME 1.73536, Soderberg 1.45045; σ′max = √((1.5915 + 108.65 + 3.1831)² + 3·33.104²) = 127.094 MPa, ny 3.54070"
inputs = { d = "40 mm", d_i = "20 mm", M_a = "400 N·m", M_m = "0 N·m", T_a = "0 N·m", T_m = "300 N·m", F_a = "2 kN", F_m = "1 kN", K_f = "1.6", K_fs = "1.3", K_fa = "1.5", S_e = "200 MPa", S_ut = "600 MPa", S_y = "450 MPa", crit = "2", n_d = "1.5" }
expect = { Z = "5890.486 mm³", A = "942.4778 mm²", "σ_a′" = "112.3946 MPa", "σ_m′" = "57.36029 MPa", "σ_max′" = "127.0935 MPa", n_Goodman = "1.520743", n_Gerber = "1.730730", n_ASME = "1.735364", n_Soderberg = "1.450452", n_f = "1.730730", n_y = "3.540699", "n_(y,quick)" = "2.650881" }
tolerance = "1e-5"
checks = "pass"
+++

## Shaft design: factors of safety at a section

The fatigue and yield factors of safety at one section of a rotating shaft, after Shigley Sec. 7-4. Bending, torsion and axial stresses are found at the section with the fatigue stress-concentration factors, combined into von Mises alternating and midrange stresses, and placed on a fatigue failure line through the endurance limit. Four lines are given: DE-Goodman is simple and conservative and is Shigley's choice for a first design; DE-Gerber and DE-ASME elliptic follow test data more closely; DE-Soderberg is the most conservative. Goodman and Gerber do not guard against yield, so the first-cycle yield check on the von Mises maximum stress goes with them.

A steady bending moment on a rotating shaft is completely reversed in the material: enter it as the alternating moment M_a. A steady torque is a midrange torque T_m. Shigley's Eqs. 7-7 to 7-14 leave axial load out; it is added here as Shigley Sec. 6-14 does, the alternating axial stress divided by 0.85 (the load factor) so that S_e is the bending endurance limit. The loads are taken to rise and fall together.

The endurance limit S_e is the part's, at this diameter, with its Marin factors and k_c = 1: the endurance-limit template gives it, and the notch-factor or shaft-shoulder templates give K_f and K_fs. For a first pass before the geometry is known, Shigley Table 7-1 lists first-iteration estimates of K_t.

### Section

```calc {description="Shaft diameter at the section (the smaller diameter at a shoulder)"}
d = 1.1 in_
```

```calc {description="Bore of a hollow shaft (0 for a solid shaft)"}
d_i = 0 in_
```

```calc
> Section modulus in bending, Eq. 7-3 for a solid shaft
Z = π (d^4 - d_i^4) / (32 d) = ?? in³
> Area, for axial load
A = π (d^2 - d_i^2) / 4 = ?? in²
```

### Loads at the section

```calc {description="Alternating bending moment (a steady moment on a rotating shaft goes here)"}
M_a = 1260 lbf·in_
```

```calc {description="Midrange bending moment (a moment that rotates with the shaft)"}
M_m = 0 lbf·in_
```

```calc {description="Alternating torque"}
T_a = 0 lbf·in_
```

```calc {description="Midrange (steady) torque"}
T_m = 1100 lbf·in_
```

```calc {description="Alternating axial force"}
F_a = 0 lbf_
```

```calc {description="Midrange axial force (tension positive)"}
F_m = 0 lbf_
```

### Notch factors and strengths

```calc {description="Fatigue stress-concentration factor in bending (1 for no notch)"}
K_f = 1.58_
```

```calc {description="Fatigue stress-concentration factor in torsion (1 for no notch)"}
K_fs = 1.37_
```

```calc {description="Fatigue stress-concentration factor in axial load (1 for no notch)"}
K_fa = 1_
```

```calc {description="Endurance limit of the shaft at this diameter (Marin factors, k_c = 1)"}
S_e = 29.3 kpsi_
```

```calc {description="Tensile ultimate strength"}
S_ut = 105 kpsi_
```

```calc {description="Tensile yield strength"}
S_y = 82 kpsi_
```

### Von Mises stresses

```calc
> Alternating bending stress with K_f, Eq. 7-1
σ_(a,b) = K_f abs(M_a) / Z = ?? psi
> Midrange bending stress with K_f
σ_(m,b) = K_f M_m / Z = ?? psi
> Alternating torsional stress with K_fs, Eq. 7-2 (polar modulus 2Z)
τ_a = K_fs abs(T_a) / (2 Z) = ?? psi
> Midrange torsional stress with K_fs
τ_m = K_fs T_m / (2 Z) = ?? psi
> Alternating axial stress with K_fa
σ_(a,ax) = K_fa abs(F_a) / A = ?? psi
> Midrange axial stress with K_fa
σ_(m,ax) = K_fa F_m / A = ?? psi
```

```calc
> Von Mises alternating stress, Eq. 7-5 (Eq. 6-55 with axial load)
σ_a′ = hypot(σ_(a,b) + σ_(a,ax) / 0.85, sqrt(3) τ_a) = ?? psi
> Von Mises midrange stress, Eq. 7-6 (Eq. 6-56)
σ_m′ = hypot(σ_(m,b) + σ_(m,ax), sqrt(3) abs(τ_m)) = ?? psi
> Von Mises maximum stress for first-cycle yield, Eq. 7-15
σ_max′ = hypot(abs(σ_(m,b) + σ_(m,ax)) + σ_(a,b) + σ_(a,ax), sqrt(3) (τ_a + abs(τ_m))) = ?? psi
```

### Fatigue factors of safety

```calc {description="Fatigue factor of safety, DE-Goodman, Eq. 7-7"}
n_Goodman = 1 / (σ_a′ / S_e + σ_m′ / S_ut) = ??
```

```calc {description="Fatigue factor of safety, DE-Gerber, Eq. 7-9"}
n_Gerber = { S_e / σ_a′ if σ_m′ ≤ 0 'psi'; S_ut / σ_m′ if σ_a′ ≤ 0 'psi'; 0.5 (S_ut / σ_m′)^2 (σ_a′ / S_e) (-1 + sqrt(1 + (2 σ_m′ S_e / (S_ut σ_a′))^2)) otherwise } = ??
```

```calc {description="Fatigue factor of safety, DE-ASME elliptic, Eq. 7-11"}
n_ASME = 1 / sqrt((σ_a′ / S_e)^2 + (σ_m′ / S_y)^2) = ??
```

```calc {description="Fatigue factor of safety, DE-Soderberg, Eq. 7-13"}
n_Soderberg = 1 / (σ_a′ / S_e + σ_m′ / S_y) = ??
```

The criterion checked against the design factor: 1 DE-Goodman, 2 DE-Gerber, 3 DE-ASME elliptic, 4 DE-Soderberg.

```calc {description="Criterion for the check: 1 DE-Goodman, 2 DE-Gerber, 3 DE-ASME elliptic, 4 DE-Soderberg"}
crit = 1_
```

```calc {description="Design factor required against fatigue and first-cycle yield"}
n_d = 1.5_
```

```calc {description="Fatigue factor of safety by the chosen criterion"}
n_f = { n_Goodman if crit == 1; n_Gerber if crit == 2; n_ASME if crit == 3; n_Soderberg if crit == 4 } = ??
```

### First-cycle yield

```calc
> Yield factor of safety on the von Mises maximum, Eq. 7-16
n_y = S_y / σ_max′ = ??
> Quick conservative yield check, Sy/(σ′a + σ′m), Shigley p. 362
n_(y,quick) = S_y / (σ_a′ + σ_m′) = ??
```

### Checks

```calc
@test n_f ≥ n_d
```

```calc
@test n_y ≥ n_d
```
