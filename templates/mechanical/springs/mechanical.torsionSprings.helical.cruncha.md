+++
cruncha-template = 1
id = "mechanical.torsionSprings.helical"
version = "1.0.0"
title = "Helical torsion spring (Shigley)"
summary = "A round-wire helical torsion spring with straight ends: active turns including the ends, rate per turn with Shigley's friction allowance, inner-fibre bending stress with Ki, the yield moment, wind-up, the diameter change and clearance on a pin, and the Gerber fatigue factor of safety with Table 10-10's repeated-bending strength."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "torsion spring", "helical", "coil", "music wire", "wind-up", "pin clearance", "bending", "curvature factor", "fatigue", "Gerber", "Shigley"]
from = "lacewing:mechanical.torsionSprings"
license = "GPL-3.0-or-later"
inputs = ["d", "OD", "N_b", "l_1", "l_2", "E", "A", "m", "f_y", "f_r", "D_p", "M_max", "M_min"]
outputs = ["D", "C", "K_i", "N_a", "k′", "k", "S_ut", "S_y", "M_y", "σ", "n_y", "θ", "θ_c′", "D′", "Δ", "σ_a", "σ_m", "S_r", "S_e", "S_a", "n_f"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 10-12: Eqs. 10-43 (Ki), 10-44 (stress), 10-48 (active turns), 10-51 (rate), 10-53 to 10-55 (diameter and clearance), 10-57 (yield), 10-58 to 10-60 (Gerber fatigue), Table 10-10; Eq. 10-14 and Tables 10-4, 10-5 (wire); Example 10-8"

[[references]]
work = "Machinery's Handbook (Oberg, Jones, Horton, Ryffel)"
edition = "26th"
locator = "p. 325 (spring index and number of coils of torsion springs)"

[[tests]]
name = "Shigley Ex. 10-8(a, b): music wire 0.072 in, OD 19/32 in, 4¼ turns, at the yield moment over a 0.400 in pin"
source = "Lacewing Lab coil_springs_test, after Shigley 10th Example 10-8: Sut = 294.4 kpsi, Ki = 1.115, My = 7.55 lbf·in, Na = 4.657, k′ = 29.18 lbf·in/turn, θ′c = 0.236 turn, D′ = 0.494 in, Δ = 0.022 in (the book subtracts from D′ rounded to 0.494 in; unrounded Δ = 0.0223 in)"
inputs = { d = "0.072 in", OD = "0.59375 in", N_b = "4.25", l_1 = "1 in", l_2 = "1 in", E = "28.5 Mpsi", A = "201 kpsi", m = "0.145", f_y = "0.78", D_p = "0.400 in", M_max = "7.546 lbf·in", M_min = "1 lbf·in" }
expect = { S_ut = "294.4 kpsi", K_i = "1.115", M_y = "7.546 lbf·in", N_a = "4.657", "k′" = "29.18 lbf·in/turn", "θ_c′" = "0.236", "D′" = "0.494 in", "Δ" = "0.022 in" }
tolerance = "1.5%"

[[tests]]
name = "Shigley Ex. 10-8(c): the same spring cycling between 1 and 5 lbf·in, 10⁶ cycles, not peened"
source = "Lacewing Lab coil_springs_test, after Shigley 10th Example 10-8(c): σa = 60.86 kpsi, Sr = 0.50 Sut (Table 10-10), Se = 78.51 kpsi, nf = 1.13"
inputs = { d = "0.072 in", OD = "0.59375 in", N_b = "4.25", l_1 = "1 in", l_2 = "1 in", E = "28.5 Mpsi", A = "201 kpsi", m = "0.145", f_y = "0.78", f_r = "0.50", D_p = "0.400 in", M_max = "5 lbf·in", M_min = "1 lbf·in" }
expect = { "σ_a" = "60.86 kpsi", S_e = "78.51 kpsi", n_f = "1.13" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 10-8 at 5 lbf·in"
source = "By hand: D = 0.52175 in, C = 7.24653, Ki = (4C² − C − 1)/(4C(C − 1)) = 1.114544; Na = 4.25 + 2/(3π·0.52175) = 4.656721; k′ = 0.072⁴ × 28.5e6 / (10.8 × 0.52175 × 4.656721) = 29.18828 lbf·in/turn; σ = 1.114544 × 32 × 5 / (π 0.072³) = 152.0792 kpsi; θ′ = 5/29.18828 = 0.171302 turn = 61.6686°"
inputs = { d = "0.072 in", OD = "0.59375 in", N_b = "4.25", l_1 = "1 in", l_2 = "1 in", E = "28.5 Mpsi", A = "201 kpsi", m = "0.145", f_y = "0.78", f_r = "0.50", D_p = "0.400 in", M_max = "5 lbf·in", M_min = "1 lbf·in" }
expect = { D = "0.52175 in", C = "7.246528", K_i = "1.114544", N_a = "4.656721", "k′" = "29.18828 lbf·in/turn", "σ" = "152.0792 kpsi", "θ" = "61.6686 °" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Ex. 10-8 cycling between 1 and 7 lbf·in: fatigue fails"
source = "As Ex. 10-8(c) with a larger moment: σa = 91.29 kpsi against a Gerber strength of about 75 kpsi, nf under 1"
inputs = { d = "0.072 in", OD = "0.59375 in", N_b = "4.25", l_1 = "1 in", l_2 = "1 in", E = "28.5 Mpsi", A = "201 kpsi", m = "0.145", f_y = "0.78", f_r = "0.50", D_p = "0.400 in", M_max = "7 lbf·in", M_min = "1 lbf·in" }
expect = { "σ_a" = "91.29 kpsi" }
tolerance = "0.1%"
checks = "fail"
+++

## Helical torsion spring

A close-wound helical spring of round wire with two straight ends, loaded by a moment about the coil's axis in the sense that winds it tighter (Shigley Sec. 10-12). The wire is in bending, so the rate goes by Young's modulus, and the stress is largest at the inner fibre of the coil, raised by the curvature factor K_i. Wound the other way, residual coiling stresses work against the spring and the allowables here do not apply. For a straight bar in twist, use a torsion bar template.

### Wire and coil

Shigley's Table 10-4 gives the minimum tensile strength of the common spring wires as S_ut = A/d^m (Eq. 10-14), with A in kpsi for d in inches; Table 10-5 gives E for each wire by size. For a wire known only by its strength, enter that strength as A with m = 0.

```calc {description="Wire diameter"}
d = 0.072 in_
```

```calc {description="Outside diameter of the coil, unloaded"}
OD = 0.59375 in_
```

```calc {description="Turns in the coil body"}
N_b = 4.25_
```

```calc {description="Length of the first straight end, from the body to the load"}
l_1 = 1 in_
```

```calc {description="Length of the second straight end, from the body to the load"}
l_2 = 1 in_
```

```calc {description="Young's modulus of the wire (Shigley Table 10-5)"}
E = 28.5 Mpsi_
```

```calc {description="Strength constant A of the wire, in kpsi with d in inches (Shigley Table 10-4)"}
A = 201 kpsi_
```

```calc {description="Strength exponent m of the wire (Shigley Table 10-4)"}
m = 0.145_
```

```calc
> Mean coil diameter
D = OD - d = ?? in
> Spring index, Eq. 10-1
C = D / d = ??
> Inner-fibre curvature factor, Eq. 10-43
K_i = (4 C^2 - C - 1) / (4 C (C - 1)) = ??
```

### Rate

The straight ends bend too, adding (l₁ + l₂)/(3πD) turns (Eq. 10-48). Shigley's rate per turn uses 10.8 rather than the 10.2 of plain beam bending, to allow for friction between the coils and on the arbor (Eq. 10-51).

```calc
> Active turns, Eq. 10-48
N_a = N_b + (l_1 + l_2) / (3 π D) = ??
> Rate, moment per turn of wind-up, Eq. 10-51
k′ = d^4 E / (10.8 D N_a × 1 'turn') = ?? lbf·in/turn
> Rate, moment per radian
k = k′ = ?? lbf·in/rad
```

### Strength

The bending yield strength of torsion-spring wire is a fraction f_y of S_ut (Eq. 10-57): 0.78 for music wire and cold-drawn carbon steel, 0.87 for hardened and tempered carbon and low-alloy steels, 0.61 for austenitic stainless steel and nonferrous alloys.

```calc {description="Bending yield strength as a fraction of Sut, Eq. 10-57"}
f_y = 0.78_
```

```calc
> Minimum tensile strength of the wire, Eq. 10-14
S_ut = A / (d / 1 'in')^m = ?? kpsi
> Bending yield strength, Eq. 10-57
S_y = f_y S_ut = ?? kpsi
> Largest static moment, which just yields the inner fibre, Eq. 10-44
M_y = π d^3 S_y / (32 K_i) = ?? lbf·in
```

### Working moment

```calc {description="Largest moment in service"}
M_max = 5 lbf·in_
```

```calc
> Bending stress at the inner fibre, Eq. 10-44
σ = K_i 32 M_max / (π d^3) = ?? kpsi
> Static factor of safety against yield
n_y = S_y / σ = ??
> Wind-up at the moment
θ = M_max / k′ = ?? °
```

### Clearance on the pin

As the spring winds up, its body gains turns and its mean diameter shrinks (Eq. 10-53). The pin or arbor it works on should leave a diametral clearance; Shigley's Eq. 10-54 takes the body's wind-up alone, without the ends.

```calc {description="Diameter of the pin or arbor the spring works on"}
D_p = 0.400 in_
```

```calc
> Wind-up of the body alone, in turns, Eq. 10-54
θ_c′ = 10.8 M_max D N_b / (d^4 E) = ??
> Mean diameter wound up, Eq. 10-53
D′ = N_b D / (N_b + θ_c′) = ?? in
> Diametral clearance on the pin, Eq. 10-55
Δ = D′ - d - D_p = ?? in
```

### Fatigue

The moment cycles between M_min and M_max. Table 10-10 (after Associated Spring) gives the repeated-bending endurance strength S_r of torsion springs as a fraction f_r of S_ut, by wire, life (10⁵ or 10⁶ cycles) and shot peening; for music wire at 10⁶ cycles, not peened, it is 0.50. Shigley converts it to a fully reversed strength and applies the Gerber criterion along the load line r = M_a/M_m (Eqs. 10-58 to 10-60). A peak moment equal to the smallest (static load) leaves r = 0 and no fatigue result.

```calc {description="Repeated-bending endurance strength as a fraction of Sut (Shigley Table 10-10)"}
f_r = 0.50_
```

```calc {description="Smallest moment over a load cycle"}
M_min = 1 lbf·in_
```

```calc
> Moment amplitude
M_a = (M_max - M_min) / 2 = ?? lbf·in
> Mean moment
M_m = (M_max + M_min) / 2 = ?? lbf·in
> Stress amplitude, Eq. 10-44
σ_a = K_i 32 M_a / (π d^3) = ?? kpsi
> Mean stress
σ_m = K_i 32 M_m / (π d^3) = ?? kpsi
> Repeated-bending endurance strength, Table 10-10
S_r = f_r S_ut = ?? kpsi
> Fully reversed endurance strength, Gerber, Eq. 10-58
S_e = (S_r / 2) / (1 - (S_r / (2 S_ut))^2) = ?? kpsi
> Load line
r = M_a / M_m = ??
> Strength amplitude on the load line, Gerber, Eq. 10-59
S_a = r^2 S_ut^2 / (2 S_e) (-1 + sqrt(1 + (2 S_e / (r S_ut))^2)) = ?? kpsi
> Fatigue factor of safety, Eq. 10-60
n_f = S_a / σ_a = ??
```

### Checks

Machinery's Handbook recommends a spring index between 4 and 14 and at least three body turns; shorter springs tend to buckle.

```calc
@test 4 ≤ C ≤ 14
```

```calc
@test N_b ≥ 3
```

```calc
@test n_y ≥ 1
```

```calc
@test Δ > 0 'in'
```

```calc
@test n_f ≥ 1
```
