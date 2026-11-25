+++
cruncha-template = 1
id = "mechanical.fracture.parisLife"
version = "1.0.1"
title = "Fatigue crack growth: Paris-law life"
summary = "Cycles for a crack to grow from an initial size to a final one, and to the critical size, under a constant-amplitude stress cycle, by the Paris equation dc/dN = C·ΔK^m with a constant geometry factor (closed-form integral)."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["fatigue", "crack growth", "Paris law", "Paris equation", "fracture mechanics", "LEFM", "damage tolerance", "inspection interval", "cycles", "critical crack"]
from = "lacewing:mechanical.fractureMechanics"
license = "GPL-3.0-or-later"
inputs = ["Y", "σ_max", "σ_min", "c_1", "c_2", "K_Ic", "r_o", "ΔK_o", "m", "N_req"]
outputs = ["Δσ", "ΔK_1", "ΔK_2", "c_cr", "N", "N_cr"]

[[references]]
work = "Juvinall and Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 8.16 (pp. 351-356), Eqs. 8.4 and 8.6; the reference-point form of the Paris constant (p. 353)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 6-6 (pp. 286-290), Eqs. 6-4 to 6-6, Table 6-1; Example 6-1"

[[references]]
work = "M. F. Ashby and D. R. H. Jones, Engineering Materials 1"
edition = "2nd (1996)"
locator = "Ch. 16, Case Study 3 (pp. 163-165); Appendix 1, Example 31 (p. 284)"

[[tests]]
name = "Shigley Ex. 6-1: β = 1.07, from 0.004 in to fracture at 115.2 kpsi"
source = "Shigley 10th Ex. 6-1 (p. 289), Lacewing Lab fracture_test: C = 3.8(10⁻¹¹) in/cycle per (kpsi·√in)³, m = 3; the final crack 0.112 in is where KI reaches KIc = 73 kpsi·√in (0.1116 unrounded), after 64.7 thousand cycles"
inputs = { Y = "1.07", "σ_max" = "115.2 ksi", "σ_min" = "0 ksi", c_1 = "0.004 in", c_2 = "0.1 in", K_Ic = "73 ksi·√in", r_o = "3.8e-11 in", "ΔK_o" = "1 ksi·√in", m = "3", N_req = "50000" }
expect = { N_cr = "64692", c_cr = "0.11164 in", "ΔK_1" = "13.818 ksi·√in" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Ashby and Jones Case Study 3: the Stretham engine beam, 2 to 3 cm"
source = "Ashby and Jones, Engineering Materials 1, 2nd ed., Case Study 3 (pp. 163-165), Lacewing Lab fracture_ashby_test: ΔK 0.40 MPa·√m at 1.6 MPa, A = 4.3 × 10⁻⁸ m/cycle per (MPa·√m)⁴; about 5.9 million cycles (5.99 by the closed form)"
inputs = { Y = "1", "σ_max" = "1.6 MPa", "σ_min" = "0 MPa", c_1 = "20 mm", c_2 = "30 mm", K_Ic = "18 MPa·√m", r_o = "4.3e-8 m", "ΔK_o" = "1 MPa·√m", m = "4", N_req = "1000000" }
expect = { "ΔK_1" = "0.40106 MPa·√m", N = "5.9924e6" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Ashby and Jones Appendix 1 Ex. 31: 0.2 mm crack grown to fracture"
source = "Ashby and Jones, Engineering Materials 1, 2nd ed., Appendix 1, Example 31 (p. 284), Lacewing Lab fracture_ashby_test: Kc 54 MPa·√m, Δσ 180 MPa from zero, A = 4 × 10⁻¹³, m = 4; 2.4 million cycles to the critical size (54/180)²/π = 28.6 mm"
inputs = { Y = "1", "σ_max" = "180 MPa", "σ_min" = "0 MPa", c_1 = "0.1 mm", c_2 = "1 mm", K_Ic = "54 MPa·√m", r_o = "4e-13 m", "ΔK_o" = "1 MPa·√m", m = "4", N_req = "2000000" }
expect = { c_cr = "28.648 mm", N_cr = "2.4045e6" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "m = 2: the logarithmic integral"
source = "By hand: N = c₁/C (1/ΔK₁)² ln(c₂/c₁) with ΔK₁ = 100√(π·0.001) = 5.605 MPa·√m, C = 1e-9 m/cycle: N = 0.001/1e-9/31.416 × ln 10 = 73 294"
inputs = { Y = "1", "σ_max" = "100 MPa", "σ_min" = "0 MPa", c_1 = "1 mm", c_2 = "10 mm", K_Ic = "50 MPa·√m", r_o = "1e-9 m", "ΔK_o" = "1 MPa·√m", m = "2", N_req = "50000" }
expect = { N = "73293.56", N_cr = "139315.7" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 6-1 against a required 100 000 cycles: too short"
source = "Shigley 10th Ex. 6-1 (p. 289): 64.7 thousand cycles to fracture"
inputs = { Y = "1.07", "σ_max" = "115.2 ksi", "σ_min" = "0 ksi", c_1 = "0.004 in", c_2 = "0.1 in", K_Ic = "73 ksi·√in", r_o = "3.8e-11 in", "ΔK_o" = "1 ksi·√in", m = "3", N_req = "100000" }
expect = { N_cr = "64692" }
tolerance = "0.01%"
checks = "fail"
+++

## Fatigue crack growth: Paris-law life

A crack grows a little with each stress cycle. In the middle of the range of stress intensity (Stage II), the growth rate follows the Paris equation, dc/dN = C·ΔK^m, with ΔK = Y·Δσ·√(πc) (Juvinall Eq. 8.4, Shigley Eq. 6-5). Integrated from an initial crack c₁ to a final one (Juvinall Eq. 8.6, Shigley Eq. 6-6), it gives the cycles of life. Final fracture comes when K at the peak stress reaches the toughness.

Assumptions:

- Y is constant while the crack grows. That is the case for a crack short against the part; where Y rises with c/w (Juvinall Figs. 6.5), this overestimates the life. Use Y at the final crack for a conservative answer, or integrate numerically.
- Constant-amplitude cycling, Stage II growth only. Growth near the threshold, and the speed-up just before fracture (Juvinall Eq. 8.5), are not included.
- Both books take the range Δσ = σ_max − σ_min; enter σ_min = 0 to leave out a compressive part of the cycle, which doesn't open the crack.

### Crack and stress cycle

Y comes from a chart for the configuration: 1 for a centre crack short against the plate's width, about 1.12 for a short edge crack; Juvinall Figs. 6.5a-h, Shigley Figs. 5-25 to 5-30, or Rooke and Cartwright's *Compendium of Stress Intensity Factors* otherwise.

```calc {description="Geometry factor Y (Shigley's β), taken as constant"}
Y = 1_
```

```calc {description="Maximum stress of the cycle"}
σ_max = 180 MPa_
```

```calc {description="Minimum stress of the cycle (0 to leave out compression)"}
σ_min = 0 MPa_
```

```calc {description="Initial crack: the crack found, or the largest inspection can miss"}
c_1 = 0.1 mm_
```

```calc {description="Final crack: the size to grow to, such as the next inspection's detectable size"}
c_2 = 1 mm_
```

```calc {description="Fracture toughness"}
K_Ic = 54 MPa·√m_
```

### Paris constants

The constant is given by a point on the material's growth line: the rate r_o (length per cycle) at the range ΔK_o, so dc/dN = r_o (ΔK/ΔK_o)^m, as Juvinall gives it. For a table's C, such as Shigley Table 6-1's, in m/cycle with ΔK in MPa·√m (or in/cycle with ksi·√in), enter ΔK_o = 1 MPa·√m and r_o = C in m (or 1 ksi·√in and C in in). The point form keeps the units consistent: C's own unit depends on m.

```calc {description="Growth rate per cycle at the reference range ΔK_o (length per cycle)"}
r_o = 4e-13 m_
```

```calc {description="Reference stress-intensity range"}
ΔK_o = 1 MPa·√m_
```

```calc {description="Paris exponent (Juvinall's n, Shigley's m), about 2 to 4 for metals"}
m = 4_
```

```calc {description="Cycles the part must survive"}
N_req = 2000000_
```

### Stress intensity

```calc
> Stress range
Δσ = σ_max - σ_min = ?? MPa
> Stress-intensity range at the initial crack
ΔK_1 = Y Δσ sqrt(π c_1) = ?? MPa·√m
> Stress-intensity range at the final crack
ΔK_2 = Y Δσ sqrt(π c_2) = ?? MPa·√m
> Critical crack: K at the peak stress reaches KIc
c_cr = (K_Ic / (Y σ_max))^2 / π = ?? mm
```

### Life

With Y constant, ∫ dc/(r_o (ΔK/ΔK_o)^m) from c₁ to c₂ is c₁/r_o · (ΔK_o/ΔK₁)^m · ((c₂/c₁)^(1 − m/2) − 1)/(1 − m/2), or c₁/r_o · (ΔK_o/ΔK₁)² · ln(c₂/c₁) when m = 2.

```calc
> Cycles to grow from c₁ to c₂
N = { c_1 / r_o (ΔK_o / ΔK_1)^2 ln(c_2 / c_1) if m == 2; c_1 / r_o (ΔK_o / ΔK_1)^m ((c_2 / c_1)^(1 - m / 2) - 1) / (1 - m / 2) otherwise } = ??
> Cycles to grow from c₁ to the critical crack: the life to fracture
N_cr = { c_1 / r_o (ΔK_o / ΔK_1)^2 ln(c_cr / c_1) if m == 2; c_1 / r_o (ΔK_o / ΔK_1)^m ((c_cr / c_1)^(1 - m / 2) - 1) / (1 - m / 2) otherwise } = ??
```

### Checks

```calc {description="The final crack is below the critical size"}
@test c_2 < c_cr
```

```calc {description="The life to fracture covers the cycles required"}
@test N_cr ≥ N_req
```
