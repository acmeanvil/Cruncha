+++
cruncha-template = 1
id = "mechanical.shaftDesign.diameter"
version = "1.0.0"
title = "Shaft design: diameter for a design factor (DE-Goodman, DE-Gerber, DE-ASME elliptic, DE-Soderberg)"
summary = "The smallest solid shaft diameter at a section that reaches a design factor in fatigue by Shigley's four distortion-energy criteria (Eqs. 7-8, 7-10, 7-12, 7-14), with the endurance limit and notch factors held fixed, and the diameter for first-cycle yield on the von Mises maximum stress."
category = ["Mechanical engineering", "Shafts"]
tags = ["shaft", "shaft design", "shaft diameter", "fatigue", "DE-Goodman", "DE-Gerber", "DE-ASME elliptic", "DE-Soderberg", "von Mises", "design factor", "Shigley"]
from = "lacewing:mechanical.shaftDesign"
license = "GPL-3.0-or-later"
inputs = ["M_a", "M_m", "T_a", "T_m", "K_f", "K_fs", "S_e", "S_ut", "S_y", "n_d"]
outputs = ["A_DE", "B_DE", "d_Goodman", "d_Gerber", "d_ASME", "d_Soderberg", "d_y"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-4, Eqs. 7-8, 7-10, 7-12, 7-14 (diameters), 7-15 (von Mises maximum); Table 7-1 (first-iteration Kt); Examples 7-1 and 7-2"

[[tests]]
name = "Shigley Ex. 7-2: first pass at the shoulder, n = 1.5"
source = "Lacewing Lab shaft_design_test, after Shigley 10th Example 7-2: Ma 3651 lbf·in, Tm 3240 lbf·in, Kf 1.7, Kfs 1.5 (Table 7-1 estimates with q = 1), Se 27.0, Sut 68 kpsi: d = 1.65 in by Eq. 7-8. The other criteria and the yield diameter by hand: A = 2·1.7·3651 = 12 413.4, B = √3·1.5·3240 = 8417.77 lbf·in; Gerber 1.55372, ASME 1.54515, Soderberg (Sy 57) 1.66797, yield 1.26205 in"
inputs = { M_a = "3651 lbf·in", M_m = "0 lbf·in", T_a = "0 lbf·in", T_m = "3240 lbf·in", K_f = "1.7", K_fs = "1.5", S_e = "27.0 kpsi", S_ut = "68 kpsi", S_y = "57 kpsi", n_d = "1.5" }
expect = { d_Goodman = "1.65 in", A_DE = "12413.4 lbf·in", B_DE = "8417.767 lbf·in", d_Gerber = "1.553715 in", d_ASME = "1.545149 in", d_Soderberg = "1.667966 in", d_y = "1.262051 in" }
tolerance = "0.3%"

[[tests]]
name = "Shigley Ex. 7-1 in reverse, DE-Goodman: n = 1.6258 at 1.100 in gives back 1.100 in"
source = "By hand: at d = 1.100 in Ex. 7-1's loads (Ma 1260, Tm 1100 lbf·in, Kf 1.58, Kfs 1.37, Se 29.3, Sut 105 kpsi) give n = 1.6257636 by DE-Goodman (the factor-of-safety template); Eq. 7-8 at that n returns 1.100 in"
inputs = { M_a = "1260 lbf·in", M_m = "0 lbf·in", T_a = "0 lbf·in", T_m = "1100 lbf·in", K_f = "1.58", K_fs = "1.37", S_e = "29.3 kpsi", S_ut = "105 kpsi", S_y = "82 kpsi", n_d = "1.6257636220" }
expect = { d_Goodman = "1.1 in" }
tolerance = "1e-8"

[[tests]]
name = "Shigley Ex. 7-1 in reverse, DE-Gerber: n = 1.8628 at 1.100 in gives back 1.100 in"
source = "By hand: Ex. 7-1's loads give n = 1.8627903 by DE-Gerber at 1.100 in; Eq. 7-10 at that n returns 1.100 in"
inputs = { M_a = "1260 lbf·in", M_m = "0 lbf·in", T_a = "0 lbf·in", T_m = "1100 lbf·in", K_f = "1.58", K_fs = "1.37", S_e = "29.3 kpsi", S_ut = "105 kpsi", S_y = "82 kpsi", n_d = "1.8627903216" }
expect = { d_Gerber = "1.1 in" }
tolerance = "1e-8"

+++

## Shaft design: diameter for a design factor

The smallest solid shaft diameter at one section that reaches the design factor n_d in fatigue, after Shigley Sec. 7-4. These are Shigley's closed forms: they hold the endurance limit S_e and the fatigue notch factors fixed, although both depend on the diameter (the size factor k_b and the notch sensitivity). So, as Shigley does in Ex. 7-2, start from estimates (S_e from the material with k_b guessed, K_t from Table 7-1 with q = 1), round the diameter to a standard size, then check the section with the factor-of-safety template at the real S_e, K_f and K_fs.

A steady bending moment on a rotating shaft is completely reversed in the material: enter it as the alternating moment M_a. Axial loads are left out, as in Shigley's Eqs. 7-8 to 7-14; check a section with axial load on the factor-of-safety template.

### Loads at the section

```calc {description="Alternating bending moment (a steady moment on a rotating shaft goes here)"}
M_a = 3651 lbf·in_
```

```calc {description="Midrange bending moment"}
M_m = 0 lbf·in_
```

```calc {description="Alternating torque"}
T_a = 0 lbf·in_
```

```calc {description="Midrange (steady) torque"}
T_m = 3240 lbf·in_
```

### Notch factors and strengths

```calc {description="Fatigue stress-concentration factor in bending (Table 7-1 estimate on a first pass)"}
K_f = 1.7_
```

```calc {description="Fatigue stress-concentration factor in torsion (Table 7-1 estimate on a first pass)"}
K_fs = 1.5_
```

```calc {description="Endurance limit of the shaft (Marin factors, k_b estimated)"}
S_e = 27.0 kpsi_
```

```calc {description="Tensile ultimate strength"}
S_ut = 68 kpsi_
```

```calc {description="Tensile yield strength"}
S_y = 57 kpsi_
```

```calc {description="Design factor"}
n_d = 1.5_
```

### Diameters

The alternating and midrange loads combine as A = √(4(K_f M_a)² + 3(K_fs T_a)²) and B = √(4(K_f M_m)² + 3(K_fs T_m)²), the moments of distortion energy behind Eqs. 7-7 to 7-14.

```calc
> Alternating distortion-energy moment
A_DE = sqrt(4 (K_f M_a)^2 + 3 (K_fs T_a)^2) = ?? lbf·in
> Midrange distortion-energy moment
B_DE = sqrt(4 (K_f M_m)^2 + 3 (K_fs T_m)^2) = ?? lbf·in
```

```calc {description="Diameter by DE-Goodman, Eq. 7-8"}
d_Goodman = (16 n_d / π (A_DE / S_e + B_DE / S_ut))^(1/3) = ?? in
```

```calc {description="Diameter by DE-Gerber, Eq. 7-10 (no alternating load: the static limit on S_ut)"}
d_Gerber = { (16 n_d B_DE / (π S_ut))^(1/3) if A_DE ≤ 0 'lbf·in'; (8 n_d A_DE / (π S_e) (1 + sqrt(1 + (2 B_DE S_e / (A_DE S_ut))^2)))^(1/3) otherwise } = ?? in
```

```calc {description="Diameter by DE-ASME elliptic, Eq. 7-12"}
d_ASME = (16 n_d / π sqrt((A_DE / S_e)^2 + (B_DE / S_y)^2))^(1/3) = ?? in
```

```calc {description="Diameter by DE-Soderberg, Eq. 7-14"}
d_Soderberg = (16 n_d / π (A_DE / S_e + B_DE / S_y))^(1/3) = ?? in
```

First-cycle yield: the von Mises maximum stress of Eq. 7-15 is 16/(πd³)·√(4(K_f(M_m + M_a))² + 3(K_fs(T_m + T_a))²), so the diameter at which S_y/σ′max reaches n_d follows directly (derived from Eqs. 7-15 and 7-16).

```calc {description="Diameter for first-cycle yield at the design factor, from Eqs. 7-15, 7-16"}
d_y = (16 n_d / (π S_y) sqrt(4 (K_f (abs(M_m) + abs(M_a)))^2 + 3 (K_fs (abs(T_m) + abs(T_a)))^2))^(1/3) = ?? in
```

The diameter needed is the larger of the chosen fatigue diameter and d_y; DE-Goodman is Shigley's choice for a first design.
