+++
cruncha-template = 1
id = "mechanical.stressConcentration.shaftShoulder"
version = "1.0.1"
title = "Stress concentration: stepped shaft with shoulder fillet"
summary = "Kt in tension and bending and Kts in torsion of a round shaft with a shoulder fillet, from the curve fits of Peterson's Charts 3.4, 3.10 and 3.12; Neuber notch sensitivity q from Sut for steels; fatigue factors Kf = 1 + q(Kt − 1); nominal, peak and fatigue stresses."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["stress concentration", "Kt", "Kf", "notch sensitivity", "Neuber", "Peterson", "shaft", "shoulder", "fillet", "fatigue"]
from = "lacewing:mechanical.stressConcentration"
license = "GPL-3.0-or-later"
inputs = ["D", "d", "r", "S_ut", "P", "M", "T"]
outputs = ["K_ta", "K_tb", "K_ts", "q", "q_s", "K_fa", "K_fb", "K_fs", "σ_a", "σ_b", "τ", "σ_(b,max)", "σ_(b,f)", "τ_max", "τ_f"]

[[references]]
work = "Peterson's Stress Concentration Factors (W. D. Pilkey, D. F. Pilkey)"
edition = "3rd"
locator = "Charts 3.4 (p. 157), 3.10 (p. 165) and 3.12 (p. 167): the curve-fit equations for a stepped round bar in tension, bending and torsion"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 6-10, Eqs. 6-32, 6-34, 6-35a and 6-35b (Figs. 6-20, 6-21); Figs. A-15-7 to A-15-9; Example 6-6"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 4.12, Fig. 4.35; Sec. 8.10, Eq. 8.2; Sample Problem 8.3"

[[tests]]
name = "Shigley Ex. 6-6: shoulder 38/32 mm, r = 3 mm, bending, Sut = 690 MPa"
source = "Shigley 10th ed., Example 6-6: Kt = 1.65 read from Fig. A-15-9, √a = 0.0622 √in, q = 0.85, Kf = 1.55. The Chart 3.10 fit gives Kt = 1.68, within the reading of the figure (Lacewing Lab stress_concentration_test)."
inputs = { D = "38 mm", d = "32 mm", r = "3 mm", S_ut = "690 MPa" }
expect = { K_tb = "1.65", q = "0.85", K_fb = "1.55" }
tolerance = "2.5%"
checks = "pass"

[[tests]]
name = "The same shoulder: the fits' own values"
source = "By hand: t = 3 mm, t/r = 1, 2t/D = 0.157 89; Chart 3.10 C1..C4 = 2.022, −2.468, 2.091, −0.648 give Kt = 1.6819; Chart 3.4 C = 1.984, −2.063, 1.931, −0.855 give 1.7030; Chart 3.12 C = 1.613, −1.853, 2.052, −0.804 give Kts = 1.3684. √a = 0.062 235 √in (bending), 0.047 254 √in (torsion) at 100.08 kpsi; √r = 0.343 68 √in; q = 0.846 68, qs = 0.879 12. σb = 32 × 150 N·m/(π 32³ mm³) = 46.627 MPa; Kf = 1 + 0.846 68 × 0.681 90 = 1.577 35."
inputs = { D = "38 mm", d = "32 mm", r = "3 mm", S_ut = "690 MPa", M = "150 N·m" }
expect = { K_tb = "1.68190", K_ta = "1.70304", K_ts = "1.36841", q = "0.846677", q_s = "0.879124", K_fb = "1.577345", "σ_b" = "46.6274 MPa" }
tolerance = "1e-4"
checks = "pass"

[[tests]]
name = "Juvinall SP 8.3: shoulder D/d = 1.2, r/d = 0.05, d = 30 mm, torsion 2 × 1000 N·m, Su = 1.2 GPa"
source = "Juvinall 5th ed., Sample Problem 8.3: Kts = 1.57 from Fig. 4.35c, Kfs = 1.54 with q = 0.95 read from his Fig. 8.24. The Chart 3.12 fit gives 1.56; Shigley's torsion curve (Eq. 6-35b) gives q = 0.92 (Lacewing Lab stress_concentration_test), so Kfs = 1.515. τ = 16 × 2000/(π 30³) = 377.26 MPa."
inputs = { D = "36 mm", d = "30 mm", r = "1.5 mm", S_ut = "1.2 GPa", T = "2000 N·m" }
expect = { K_ts = "1.57", q_s = "0.92", K_fs = "1.54", "τ" = "377.26 MPa" }
tolerance = "2%"
checks = "pass"

[[tests]]
name = "Peterson Chart 3.4 at D/d = 2, r/d = 0.1: outside the torsion fit"
source = "Lacewing Lab stress_concentration_test: Kt = 1.97 read off Chart 3.4. t/r = 5 is beyond Chart 3.12's fit (t/r ≤ 4), so the range check fails."
inputs = { D = "2 in", d = "1 in", r = "0.1 in" }
expect = { K_ta = "1.97" }
tolerance = "1%"
checks = "fail"
+++

## Stress concentration: stepped shaft with shoulder fillet

A round shaft steps from diameter D down to d through a fillet of radius r. The theoretical factors are defined on the nominal stress in the smaller diameter d: K_ta = σ_max/σ_a in tension, K_tb = σ_max/σ_b in bending, K_ts = τ_max/τ in torsion.

Each factor comes from the curve-fit equation printed on its Peterson chart: Kt = C1 + C2(2t/D) + C3(2t/D)² + C4(2t/D)³, with t = (D − d)/2 the shoulder height and each Ci = a + b√(t/r) + c·t/r over the stated range of t/r. The fits follow the charts within about 3 %. Shigley's Figs. A-15-7 to A-15-9 and Juvinall's Fig. 4.35 draw the same curves.

Notch sensitivity q = 1/(1 + √(a/r)) uses Neuber's constant √a as a function of Sut, the fits Shigley gives for steels (Eq. 6-35a for bending and axial loads, 6-35b for torsion). Shigley's curves stop at r = 0.16 in (4 mm), and larger radii take the value there. For other materials, or when in doubt, replace q (and q_s) with 1, so Kf = Kt. Peterson's form q = 1/(1 + a/r), and Juvinall's Fig. 8.24, can read a little higher.

Static loads on ductile materials usually need no Kt: local yielding relieves the peak. Brittle materials, impact and fatigue do.

### Shaft and material

```calc {description="Larger diameter"}
D = 38 mm_
```

```calc {description="Smaller diameter, at the fillet"}
d = 32 mm_
```

```calc {description="Fillet radius"}
r = 3 mm_
```

```calc {description="Ultimate tensile strength (steel, for q)"}
S_ut = 690 MPa_
```

### Loads

```calc {description="Axial force"}
P = 0 N_
```

```calc {description="Bending moment"}
M = 150 N·m_
```

```calc {description="Torque"}
T = 0 N·m_
```

### Geometry ratios

```calc
> Shoulder height
t = (D - d) / 2 = ?? mm
> Shoulder height over fillet radius
β = t / r = ??
> Relative shoulder height, 2t/D
x = 2 t / D = ??
```

### Theoretical factors

```calc
function ktPeterson(c, β, x)
    s = sqrt(β)
    C_1 = c[1, 1] + c[1, 2] s + c[1, 3] β
    C_2 = c[2, 1] + c[2, 2] s + c[2, 3] β
    C_3 = c[3, 1] + c[3, 2] s + c[3, 3] β
    C_4 = c[4, 1] + c[4, 2] s + c[4, 3] β
    return max(1, C_1 + C_2 x + C_3 x^2 + C_4 x^3)
end
```

```calc {description="Chart 3.4 fit, tension, 0.1 ≤ t/r ≤ 2 (rows C1..C4, columns a, b, c)"}
c_(a,1) = [0.926, 1.157, -0.099; 0.012, -3.036, 0.961; -0.302, 3.977, -1.744; 0.365, -2.098, 0.878] = @
```

```calc {description="Chart 3.4 fit, tension, 2 < t/r ≤ 20"}
c_(a,2) = [1.200, 0.860, -0.022; -1.805, -0.346, -0.038; 2.198, -0.486, 0.165; -0.593, -0.028, -0.106] = @
```

```calc {description="Chart 3.10 fit, bending, 0.1 ≤ t/r ≤ 2"}
c_(b,1) = [0.947, 1.206, -0.131; 0.022, -3.405, 0.915; 0.869, 1.777, -0.555; -0.810, 0.422, -0.260] = @
```

```calc {description="Chart 3.10 fit, bending, 2 < t/r ≤ 20"}
c_(b,2) = [1.232, 0.832, -0.008; -3.813, 0.968, -0.260; 7.423, -4.868, 0.869; -3.839, 3.070, -0.600] = @
```

```calc {description="Chart 3.12 fit, torsion, 0.25 ≤ t/r ≤ 4"}
c_s = [0.905, 0.783, -0.075; -0.437, -1.969, 0.553; 1.557, 1.073, -0.578; -1.061, 0.171, 0.086] = @
```

```calc
> Tension, on 4P/(πd²), Chart 3.4
K_ta = ktPeterson({c_(a,1) if β ≤ 2; c_(a,2) otherwise}, β, x) = ??
> Bending, on 32M/(πd³), Chart 3.10
K_tb = ktPeterson({c_(b,1) if β ≤ 2; c_(b,2) otherwise}, β, x) = ??
> Torsion, on 16T/(πd³), Chart 3.12
K_ts = ktPeterson(c_s, β, x) = ??
```

### Range of the fits

```calc
@test d < D
```

```calc {description="Tension and bending fits: t/r from 0.1 to 20"}
@test 0.1 ≤ β ≤ 20
```

```calc {description="Torsion fit: t/r from 0.25 to 4"}
@test 0.25 ≤ β ≤ 4
```

```calc {description="Tension and bending charts drawn to D/d = 3"}
@test D / d ≤ 3
```

```calc {description="Torsion chart drawn to D/d = 2.5"}
@test D / d ≤ 2.5
```

```calc {description="Charts drawn to r/d = 0.3"}
@test r / d ≤ 0.3
```

### Notch sensitivity and fatigue factors

```calc
> Sut in kpsi, for Shigley's fits
s_u = S_ut / 1 'ksi' = ??
> Notch radius for q: Figs. 6-20 and 6-21 stop at 0.16 in
r_q = min(r, 0.16 'in') = ?? mm
> Neuber constant, bending and axial, Eq. 6-35a (squared)
a_N = (0.246 - 3.08e-3 s_u + 1.51e-5 s_u^2 - 2.67e-8 s_u^3)^2 × 1 'in' = ?? in
> Neuber constant, torsion, Eq. 6-35b (squared)
a_Ns = (0.190 - 2.51e-3 s_u + 1.35e-5 s_u^2 - 2.67e-8 s_u^3)^2 × 1 'in' = ?? in
> Notch sensitivity, bending and axial, Eq. 6-34
q = 1 / (1 + sqrt(a_N / r_q)) = ??
> Notch sensitivity, torsion
q_s = 1 / (1 + sqrt(a_Ns / r_q)) = ??
```

```calc
> Fatigue factor, tension, Eq. 6-32
K_fa = 1 + q (K_ta - 1) = ??
> Fatigue factor, bending
K_fb = 1 + q (K_tb - 1) = ??
> Fatigue factor, torsion
K_fs = 1 + q_s (K_ts - 1) = ??
```

### Stresses

```calc
> Nominal axial stress
σ_a = 4 P / (π d^2) = ?? MPa
> Nominal bending stress
σ_b = 32 M / (π d^3) = ?? MPa
> Nominal shear stress
τ = 16 T / (π d^3) = ?? MPa
```

```calc
> Peak axial stress
σ_(a,max) = K_ta σ_a = ?? MPa
> Peak bending stress
σ_(b,max) = K_tb σ_b = ?? MPa
> Peak shear stress
τ_max = K_ts τ = ?? MPa
```

```calc
> Fatigue stress, axial, Kf·σ
σ_(a,f) = K_fa σ_a = ?? MPa
> Fatigue stress, bending
σ_(b,f) = K_fb σ_b = ?? MPa
> Fatigue stress, torsion
τ_f = K_fs τ = ?? MPa
```
