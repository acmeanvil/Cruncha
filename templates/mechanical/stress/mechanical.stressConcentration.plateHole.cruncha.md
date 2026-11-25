+++
cruncha-template = 1
id = "mechanical.stressConcentration.plateHole"
version = "1.0.0"
title = "Stress concentration: plate with a central hole"
summary = "Kt of a finite-width plate with a central circular hole in tension (Howland's solution, Peterson Chart 4.1) and in out-of-plane bending (Chart 4.92), on the net section; Neuber notch sensitivity q from Sut for steels; fatigue factors Kf = 1 + q(Kt − 1); nominal, peak and fatigue stresses."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["stress concentration", "Kt", "Kf", "notch sensitivity", "Neuber", "Peterson", "Howland", "plate", "hole", "bar", "fatigue"]
from = "lacewing:mechanical.stressConcentration"
license = "GPL-3.0-or-later"
inputs = ["H", "d", "h", "S_ut", "P", "M"]
outputs = ["K_ta", "K_tb", "q", "K_fa", "K_fb", "σ_a", "σ_b", "σ_(a,max)", "σ_(b,max)", "σ_(a,f)", "σ_(b,f)"]

[[references]]
work = "Peterson's Stress Concentration Factors (W. D. Pilkey, D. F. Pilkey)"
edition = "3rd"
locator = "Chart 4.1 (p. 270): Howland's curve-fit equation, finite-width plate with a central hole in tension; Chart 4.92 (p. 383): the curve-fit equation for simple bending of a plate with a hole"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-13, Example 3-13; Fig. A-15-1; Sec. 6-10, Eqs. 6-32, 6-34, 6-35a"

[[tests]]
name = "Shigley Ex. 3-13: 40-mm bar, 8-mm hole, 10 kN"
source = "Shigley 10th ed., Example 3-13: Kt = 2.5 read from Fig. A-15-1, σ0 = 156 MPa on the net section; the Chart 4.1 fit gives 2.519 (Lacewing Lab stress_concentration_test, bar 2 mm thick)."
inputs = { H = "40 mm", d = "8 mm", h = "2 mm", P = "10 kN" }
expect = { K_ta = "2.5", "σ_a" = "156 MPa" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 3-13: the same bar with a 4-mm hole"
source = "Shigley 10th ed., Example 3-13: Kt = 2.7 read from Fig. A-15-1; the Chart 4.1 fit gives 2.732."
inputs = { H = "40 mm", d = "4 mm", h = "2 mm", P = "10 kN" }
expect = { K_ta = "2.7" }
tolerance = "1.5%"
checks = "pass"

[[tests]]
name = "A small hole in a wide plate tends to Kt = 3"
source = "By hand: d/H = 0.001, 1 − d/H = 0.999: Kt = 2 + 0.284 × 0.999 − 0.600 × 0.999² + 1.32 × 0.999³ = 3.000 96 (Kirsch's 3 for an infinite plate)."
inputs = { H = "1000 mm", d = "1 mm", h = "0.5 mm" }
expect = { K_ta = "3.000959" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Plate 100 × 10 mm, 10-mm hole, Sut = 500 MPa, 20 kN and 50 N·m"
source = "By hand: d/H = 0.1, h/d = 1: Chart 4.1 Kt = 2.731 88; Chart 4.92 C1 = 2.193 51, C2 = −2.373 68, C3 = 2.693 76 give Kt = 1.983 08. r = 5 mm caps at 0.16 in; at 72.52 kpsi √a = 0.091 870 √in, q = 0.813 22; Kfa = 2.408 41, Kfb = 1.799 46. σa = 20 kN/(90 × 10 mm²) = 22.222 MPa, σb = 6 × 50 N·m/(90 × 10² mm³) = 33.333 MPa."
inputs = { H = "100 mm", d = "10 mm", h = "10 mm", S_ut = "500 MPa", P = "20 kN", M = "50 N·m" }
expect = { K_ta = "2.73188", K_tb = "1.98308", q = "0.813223", K_fa = "2.408405", K_fb = "1.799463", "σ_a" = "22.2222 MPa", "σ_b" = "33.3333 MPa" }
tolerance = "1e-5"
checks = "pass"
+++

## Stress concentration: plate with a central hole

A flat plate or bar of width H and thickness h has a central circular hole of diameter d. The factors are defined on the nominal stress on the **net section** (H − d)·h, as Peterson's charts and Shigley's Fig. A-15-1 draw them:

- **Tension** P along the plate: Kt from Howland's solution, the curve-fit equation printed on Peterson's Chart 4.1, Kt = 2 + 0.284(1 − d/H) − 0.600(1 − d/H)² + 1.32(1 − d/H)³, drawn to d/H = 0.5.
- **Bending** of the plate out of its plane by a moment M about an axis across its width (simple bending): the curve-fit equation printed on Chart 4.92, Kt = C1 + C2(d/H) + C3(d/H)², each Ci a quadratic in h/d. M is the whole moment across the width. The chart draws the finite-thickness curves to d/H = 0.2 and d/h from 0.5.

The hole's radius d/2 is the notch radius for q = 1/(1 + √(a/r)), with Neuber's constant √a from Sut by Shigley's fit for steels (Eq. 6-35a). Shigley's curves stop at r = 0.16 in (4 mm), and larger radii take the value there. For other materials, or when in doubt, replace q with 1, so Kf = Kt.

Static loads on ductile materials usually need no Kt: local yielding relieves the peak. Brittle materials, impact and fatigue do.

### Plate and material

```calc {description="Plate width"}
H = 40 mm_
```

```calc {description="Hole diameter"}
d = 8 mm_
```

```calc {description="Plate thickness"}
h = 2 mm_
```

```calc {description="Ultimate tensile strength (steel, for q)"}
S_ut = 690 MPa_
```

### Loads

```calc {description="Axial force along the plate"}
P = 10 kN_
```

```calc {description="Bending moment across the width (out of plane)"}
M = 0 N·m_
```

### Theoretical factors

```calc
> Hole diameter over plate width
x = d / H = ??
> Thickness over hole diameter
β = h / d = ??
```

```calc {description="Tension, on P/((H − d)h), Peterson Chart 4.1 (Howland)"}
K_ta = 2 + 0.284 (1 - x) - 0.600 (1 - x)^2 + 1.32 (1 - x)^3 = ??
```

```calc
> Chart 4.92 coefficient C1
C_1 = 1.82 + 0.3901 β - 0.01659 β^2 = ??
> Chart 4.92 coefficient C2
C_2 = -1.9164 - 0.4376 β - 0.01968 β^2 = ??
> Chart 4.92 coefficient C3
C_3 = 2.0828 + 0.643 β - 0.03204 β^2 = ??
> Simple bending, on 6M/((H − d)h²), Chart 4.92
K_tb = C_1 + C_2 x + C_3 x^2 = ??
```

### Range of the fits

```calc {description="Tension: Howland's curve drawn to d/H = 0.5"}
@test x ≤ 0.5
```

```calc {description="Bending: finite-thickness curves drawn to d/H = 0.2"}
@test x ≤ 0.2
```

```calc {description="Bending: curves from d/h = 0.5"}
@test β ≤ 2
```

### Notch sensitivity and fatigue factors

```calc
> Sut in kpsi, for Shigley's fit
s_u = S_ut / 1 'ksi' = ??
> Notch radius for q: the hole's radius, Fig. 6-20 stops at 0.16 in
r_q = min(d / 2, 0.16 'in') = ?? mm
> Neuber constant, Eq. 6-35a (squared)
a_N = (0.246 - 3.08e-3 s_u + 1.51e-5 s_u^2 - 2.67e-8 s_u^3)^2 × 1 'in' = ?? in
> Notch sensitivity, Eq. 6-34
q = 1 / (1 + sqrt(a_N / r_q)) = ??
> Fatigue factor, tension, Eq. 6-32
K_fa = 1 + q (K_ta - 1) = ??
> Fatigue factor, bending
K_fb = 1 + q (K_tb - 1) = ??
```

### Stresses

```calc
> Nominal stress, tension, net section
σ_a = P / ((H - d) h) = ?? MPa
> Nominal stress, bending, net section
σ_b = 6 M / ((H - d) h^2) = ?? MPa
```

```calc
> Peak stress, tension
σ_(a,max) = K_ta σ_a = ?? MPa
> Peak stress, bending
σ_(b,max) = K_tb σ_b = ?? MPa
> Fatigue stress, tension, Kf·σ
σ_(a,f) = K_fa σ_a = ?? MPa
> Fatigue stress, bending
σ_(b,f) = K_fb σ_b = ?? MPa
```
