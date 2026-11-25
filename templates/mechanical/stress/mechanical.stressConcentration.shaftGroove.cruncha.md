+++
cruncha-template = 1
id = "mechanical.stressConcentration.shaftGroove"
version = "1.0.1"
title = "Stress concentration: round shaft with U groove"
summary = "Kt in tension and bending and Kts in torsion of a round shaft with a circumferential U groove, from the curve fits of Peterson's Charts 2.41 and 2.47 and Roark's Table 17.1 case 15a; Neuber notch sensitivity q from Sut for steels; fatigue factors Kf = 1 + q(Kt − 1); nominal, peak and fatigue stresses."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["stress concentration", "Kt", "Kf", "notch sensitivity", "Neuber", "Peterson", "Roark", "shaft", "groove", "retaining ring", "fatigue"]
from = "lacewing:mechanical.stressConcentration"
license = "GPL-3.0-or-later"
inputs = ["D", "d", "r", "S_ut", "P", "M", "T"]
outputs = ["K_ta", "K_tb", "K_ts", "q", "q_s", "K_fa", "K_fb", "K_fs", "σ_a", "σ_b", "τ", "σ_(b,max)", "σ_(b,f)", "τ_max", "τ_f"]

[[references]]
work = "Peterson's Stress Concentration Factors (W. D. Pilkey, D. F. Pilkey)"
edition = "3rd"
locator = "Charts 2.41 (p. 122) and 2.47 (p. 128): the curve-fit equations for a U-grooved round bar in bending and torsion; Sec. 2.5.2 (p. 68) and Chart 2.19 (tension, drawn without a fit)"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 17.1, case 15a (p. 790): U groove in a round shaft, axial tension"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 6-10, Eqs. 6-32, 6-34, 6-35a and 6-35b; Figs. A-15-13 and A-15-14"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 4.12, Fig. 4.36; Sec. 8.10, Eq. 8.2"

[[tests]]
name = "Peterson Sec. 2.5.2: grooved bar in tension, D/d = 1.505, r/d = 0.209"
source = "Peterson 3rd ed., Sec. 2.5.2 (p. 68): Ktn = 1.92 read from Chart 2.19; the Roark case 15a fit gives 1.938 (Lacewing Lab stress_concentration_test)."
inputs = { D = "1.505 in", d = "1 in", r = "0.209 in" }
expect = { K_ta = "1.92" }
tolerance = "1.5%"
checks = "pass"

[[tests]]
name = "Peterson Chart 2.41 at D/d = 1.3, r/d = 0.1, bending"
source = "Lacewing Lab stress_concentration_test: Kt = 2.03 read off Chart 2.41 (the fit gives 2.025)."
inputs = { D = "1.3 in", d = "1 in", r = "0.1 in" }
expect = { K_tb = "2.03" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "Peterson Chart 2.41 at D/d = 1.05, r/d = 0.1, bending"
source = "Lacewing Lab stress_concentration_test: Kt = 1.78 read off Chart 2.41 (the fit gives 1.759)."
inputs = { D = "1.05 in", d = "1 in", r = "0.1 in" }
expect = { K_tb = "1.78" }
tolerance = "1.5%"
checks = "pass"

[[tests]]
name = "Groove 50/45 mm, r = 2.5 mm, Sut = 600 MPa, 200 N·m bending and 300 N·m torque"
source = "By hand: t = 2.5 mm, t/r = 1, 2t/D = 0.1; Roark 15a C1..C4 = 3.040, −5.422, 6.273, −2.891 give Kt = 2.5576; Chart 2.41 C = 3.032, −7.431, 10.390, −5.009 give 2.3878; Chart 2.47 C = 2.000, −3.555, 4.898, −2.365 give Kts = 1.6911. At 87.02 kpsi √a = 0.074 726 √in, 0.056 212 √in (torsion); √r = 0.313 75 √in; q = 0.807 63, qs = 0.848 05; Kfb = 2.1208, Kfs = 1.5861. σb = 32 × 200/(π 45³) = 22.356 MPa, τ = 16 × 300/(π 45³) = 16.767 MPa."
inputs = { D = "50 mm", d = "45 mm", r = "2.5 mm", S_ut = "600 MPa", M = "200 N·m", T = "300 N·m" }
expect = { K_ta = "2.55764", K_tb = "2.38779", K_ts = "1.69111", q = "0.807632", q_s = "0.848051", K_fb = "2.12083", K_fs = "1.58610", "σ_b" = "22.3559 MPa", "τ" = "16.7669 MPa" }
tolerance = "1e-4"
checks = "pass"
+++

## Stress concentration: round shaft with U groove

A round shaft of diameter D has a circumferential groove with a semicircular-ended root of radius r, leaving a root diameter d (a retaining-ring or relief groove). The factors are defined on the nominal stress at the root diameter d: K_ta = σ_max/σ_a in tension, K_tb = σ_max/σ_b in bending, K_ts = τ_max/τ in torsion.

Each factor is a curve-fit equation, Kt = C1 + C2(2t/D) + C3(2t/D)² + C4(2t/D)³, with t = (D − d)/2 the groove depth and each Ci = a + b√(t/r) + c·t/r over the stated range of t/r. Bending and torsion use the fits printed on Peterson's Charts 2.41 and 2.47. Peterson draws the tension case (Chart 2.19) without a fit, so Roark's case 15a is used; it reproduces the Ktn = 1.92 that Peterson quotes from Chart 2.19. The bending fit runs up to about 9 % under Peterson's curve at D/d = 2 (unconservative), and agrees within about 3 % at small D/d.

Notch sensitivity q = 1/(1 + √(a/r)) uses Neuber's constant √a as a function of Sut, the fits Shigley gives for steels (Eq. 6-35a for bending and axial loads, 6-35b for torsion). Shigley's curves stop at r = 0.16 in (4 mm), and larger radii take the value there. For other materials, or when in doubt, replace q (and q_s) with 1, so Kf = Kt.

A sharp-cornered retaining-ring groove (flat bottom) is not this detail: its Kt is higher and Peterson charts it separately.

### Shaft and material

```calc {description="Shaft diameter"}
D = 50 mm_
```

```calc {description="Root diameter at the groove"}
d = 45 mm_
```

```calc {description="Groove root radius"}
r = 2.5 mm_
```

```calc {description="Ultimate tensile strength (steel, for q)"}
S_ut = 600 MPa_
```

### Loads

```calc {description="Axial force"}
P = 0 N_
```

```calc {description="Bending moment"}
M = 200 N·m_
```

```calc {description="Torque"}
T = 300 N·m_
```

### Geometry ratios

```calc
> Groove depth
t = (D - d) / 2 = ?? mm
> Groove depth over root radius
β = t / r = ??
> Relative groove depth, 2t/D
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

```calc {description="Roark Table 17.1 case 15a, tension, 0.25 ≤ t/r ≤ 2 (rows C1..C4, columns a, b, c)"}
c_(a,1) = [0.455, 3.354, -0.769; 3.129, -15.955, 7.404; -6.909, 29.286, -16.104; 4.325, -16.685, 9.469] = @
```

```calc {description="Roark Table 17.1 case 15a, tension, 2 < t/r ≤ 50"}
c_(a,2) = [0.935, 1.922, 0.004; 0.537, -3.708, 0.040; -2.538, 3.438, -0.012; 2.066, -1.652, -0.031] = @
```

```calc {description="Chart 2.41 fit, bending, 0.25 ≤ t/r ≤ 2"}
c_(b,1) = [0.594, 2.958, -0.520; 0.422, -10.545, 2.692; 0.501, 14.375, -4.486; -0.613, -6.573, 2.177] = @
```

```calc {description="Chart 2.41 fit, bending, 2 < t/r ≤ 50"}
c_(b,2) = [0.965, 1.926, 0.0; -2.773, -4.414, -0.017; 4.785, 4.681, 0.096; -1.995, -2.241, -0.074] = @
```

```calc {description="Chart 2.47 fit, torsion, 0.25 ≤ t/r ≤ 2"}
c_(s,1) = [0.966, 1.056, -0.022; -0.192, -4.037, 0.674; 0.808, 5.321, -1.231; -0.567, -2.364, 0.566] = @
```

```calc {description="Chart 2.47 fit, torsion, 2 < t/r ≤ 50"}
c_(s,2) = [1.089, 0.924, 0.018; -1.504, -2.141, -0.047; 2.486, 2.289, 0.091; -1.056, -1.104, -0.059] = @
```

```calc
> Tension, on 4P/(πd²), Roark case 15a
K_ta = ktPeterson({c_(a,1) if β ≤ 2; c_(a,2) otherwise}, β, x) = ??
> Bending, on 32M/(πd³), Chart 2.41
K_tb = ktPeterson({c_(b,1) if β ≤ 2; c_(b,2) otherwise}, β, x) = ??
> Torsion, on 16T/(πd³), Chart 2.47
K_ts = ktPeterson({c_(s,1) if β ≤ 2; c_(s,2) otherwise}, β, x) = ??
```

### Range of the fits

```calc
@test d < D
```

```calc {description="Fits: t/r from 0.25 to 50"}
@test 0.25 ≤ β ≤ 50
```

```calc {description="Charts drawn to D/d = 2"}
@test D / d ≤ 2
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
