+++
cruncha-template = 1
id = "mechanical.fracture.leakBeforeBreak"
version = "1.0.1"
title = "Pressure vessel: leak before break and proof test"
summary = "A thin-walled pressure vessel against fast fracture, general yield and leak before break (a crack grown through the wall still doesn't run), with the thinnest wall and largest pressure for leak before break, and the proof-test pressure that guarantees a fatigue life by the Paris law."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["pressure vessel", "leak before break", "LBB", "fracture", "fracture toughness", "proof test", "hydrostatic test", "Paris law", "crack growth", "thin wall", "LEFM"]
from = "lacewing:mechanical.fractureMechanics"
license = "GPL-3.0-or-later"
inputs = ["p", "r", "t", "c_v", "K_Ic", "S_y", "Y", "S", "N_p", "r_o", "ΔK_o", "m"]
outputs = ["σ", "a_c", "σ_LBB", "p_LBB", "t_LBB", "a_y", "p_y", "r_p", "a_0", "σ_proof", "p_proof"]

[[references]]
work = "M. F. Ashby and D. R. H. Jones, Engineering Materials 1"
edition = "2nd (1996)"
locator = "Ch. 16 (pp. 155-165): Case Studies 1 and 2, Fig. 16.5, leak before break (pp. 160-161); Eq. 14.2 (p. 141); Appendix 1, Example 34 (p. 285)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-14 (p. 128), Eqs. 3-53 and 3-54 (thin-walled vessels); Sec. 5-12, Eq. 5-37"

[[tests]]
name = "Ashby and Jones Ex. 34: 7.5 m vessel, 40 mm wall at 5.1 MN/m², 3000 cycles: proof test 8.97 MN/m²"
source = "Ashby and Jones, Engineering Materials 1, 2nd ed., Appendix 1, Example 34 (p. 285), Lacewing Lab fracture_ashby_test: Kc 200 MPa·√m, A = 2.44 × 10⁻¹⁴ m/cycle per (MPa·√m)⁴; proof pressure 8.97 MN/m². Sy 1000 MPa is Fig. 16.6's pressure-vessel steel (the example gives none). By hand: σ = 5.1 × 3750/40 = 478.1 MPa; σ_LBB = 200/√(π·0.04) = 564.2 MPa; a* = (200/1000)²/π = 12.73 mm; plastic zone 200²/(2π·1000²) = 6.37 mm"
inputs = { p = "5.1 MPa", r = "3.75 m", t = "40 mm", c_v = "1", K_Ic = "200 MPa·√m", S_y = "1000 MPa", Y = "1", S = "1", N_p = "3000", r_o = "2.44e-14 m", "ΔK_o" = "1 MPa·√m", m = "4" }
expect = { "σ" = "478.125 MPa", "σ_LBB" = "564.19 MPa", a_y = "12.732 mm", r_p = "6.3662 mm", p_proof = "8.97 MPa", t_LBB = "28.727 mm" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "The same vessel with a safety factor of 2 on stress: no leak before break"
source = "Ashby and Jones, Engineering Materials 1, 2nd ed., Fig. 16.5 (the safety-factor lines), Lacewing Lab fracture_ashby_test: S = 2 halves the leak-before-break pressure (3.01 MN/m²) and quadruples the thinnest wall (114.9 mm)"
inputs = { p = "5.1 MPa", r = "3.75 m", t = "40 mm", c_v = "1", K_Ic = "200 MPa·√m", S_y = "1000 MPa", Y = "1", S = "2", N_p = "3000", r_o = "2.44e-14 m", "ΔK_o" = "1 MPa·√m", m = "4" }
expect = { p_LBB = "3.0090 MPa", t_LBB = "114.909 mm" }
tolerance = "0.01%"
checks = "fail"

[[tests]]
name = "Ashby and Jones Case Study 1: ammonia tank, longitudinal stress 140 MPa"
source = "Ashby and Jones, Engineering Materials 1, 2nd ed., Case Study 1 (p. 157), Lacewing Lab fracture_ashby_test: p = 1.83 MPa, r = 1067 mm, t = 7 mm, pr/2t = 140 MPa; Kc 39 MPa·√m, Sy 940 MPa, Y = 1.92 for the crack in the weld's heat-affected zone. By hand: a_c = (39/(1.92 × 139.47))²/π = 6.75 mm; σ_LBB = 39/(1.92 √(π·0.007)) = 137.0 MPa, below the wall stress"
inputs = { p = "1.83 MPa", r = "1067 mm", t = "7 mm", c_v = "2", K_Ic = "39 MPa·√m", S_y = "940 MPa", Y = "1.92", S = "1", N_p = "3000", r_o = "2.44e-14 m", "ΔK_o" = "1 MPa·√m", m = "4" }
expect = { "σ" = "139.47 MPa", a_c = "6.7515 mm", "σ_LBB" = "136.97 MPa" }
tolerance = "0.01%"
checks = "fail"
+++

## Pressure vessel: leak before break and proof test

A thin-walled vessel under internal pressure fails safely if a crack grows through the wall and the vessel leaks, and is found, before the crack is long enough to run. Ashby and Jones (Engineering Materials 1, Ch. 16) take the safe-side reading: a crack grown through the wall is 2a = 2t long, so the vessel leaks before it breaks when K for a = t is still below the toughness, Yσ√(πt) ≤ KIc. Neither Juvinall nor Shigley states this criterion; they give the stress intensity and the thin-wall stresses used.

The same chapter checks general yield (the wall stress against S_y), and shows the smallest crack that runs before the wall yields: cracks shorter than that let the vessel yield first, which strain gauges can see.

A proof (hydrostatic) test above the working pressure breaks any vessel whose crack is already longer than a₀. If the largest crack that survives the test, a₀, takes N_p pressure cycles to grow to the critical size at the working pressure, a vessel that passes the test is good for N_p cycles (Ashby and Jones, Case Study 3 and Appendix 1 Example 34). The Paris law is integrated with Y constant and each cycle from zero to p.

Assumptions: thin wall (t below about r/10); the membrane stress is σ = p r/(c_v t) with the mean radius; linear-elastic fracture; Y constant.

### Vessel and material

```calc {description="Working pressure (gauge)"}
p = 5.1 MPa_
```

```calc {description="Mean radius of the wall"}
r = 3.75 m_
```

```calc {description="Wall thickness"}
t = 40 mm_
```

Which membrane stress the crack sees: c_v = 1 for the hoop stress of a cylinder (a crack along the axis); c_v = 2 for a cylinder's longitudinal stress (a circumferential crack) or a sphere.

```calc {description="Membrane stress divisor: 1 hoop (cylinder), 2 longitudinal or sphere"}
c_v = 1_
```

```calc {description="Fracture toughness"}
K_Ic = 200 MPa·√m_
```

```calc {description="Yield strength"}
S_y = 1000 MPa_
```

```calc {description="Geometry factor Y of the crack (1 for a crack short against the wall)"}
Y = 1_
```

```calc {description="Safety factor on stress"}
S = 1_
```

### Fast fracture and yield at the working pressure

```calc
> Membrane stress in the wall (Shigley Eqs. 3-53, 3-54)
σ = p r / (c_v t) = ?? MPa
> Crack that runs at the working stress, Yσ√(πa) = KIc
a_c = (K_Ic / (Y σ))^2 / π = ?? mm
> Smallest crack that runs before the wall yields (Ashby and Jones Fig. 16.5)
a_y = (K_Ic / (Y S_y))^2 / π = ?? mm
> Pressure for general yield, over S
p_y = S_y c_v t / (r S) = ?? MPa
> Plastic zone at the crack tip when it runs (Ashby and Jones Eq. 14.2)
r_p = K_Ic^2 / (2 π S_y^2) = ?? mm
```

If a_y is longer than the wall, a through crack still doesn't run at yield: the vessel yields before it breaks and leaks before it breaks even at yield.

### Leak before break

```calc
> Largest stress at which a crack through the wall (a = t) does not run
σ_LBB = K_Ic / (Y sqrt(π t)) = ?? MPa
> Largest pressure for leak before break, over S
p_LBB = σ_LBB c_v t / (r S) = ?? MPa
> Thinnest wall that leaks before it breaks at this pressure
t_LBB = π (S Y p r / (c_v K_Ic))^2 = ?? mm
```

### Proof test for a fatigue life

The Paris constant is given as a point on the growth line, dc/dN = r_o (ΔK/ΔK_o)^m. For a table's C in m/cycle with ΔK in MPa·√m (or in/cycle with ksi·√in), enter ΔK_o = 1 MPa·√m and r_o = C in m (or 1 ksi·√in and C in in). The rate is given at a reference range, rather than as C itself, because C's unit depends on m.

```calc {description="Pressure cycles, from zero to p, the vessel must survive"}
N_p = 3000_
```

```calc {description="Growth rate per cycle at the reference range ΔK_o (length per cycle)"}
r_o = 2.44e-14 m_
```

```calc {description="Reference stress-intensity range"}
ΔK_o = 1 MPa·√m_
```

```calc {description="Paris exponent, about 2 to 4 for metals"}
m = 4_
```

With Y constant and K reaching KIc at a_c, the crack a₀ that grows to a_c in N_p cycles satisfies (a₀/a_c)^(1 − m/2) = 1 − N_p (1 − m/2) r_o (KIc/ΔK_o)^m / a_c, or a₀ = a_c exp(−N_p r_o (KIc/ΔK_o)²/a_c) when m = 2. When m is below 2 and the life is long, no crack survives: the right side is negative and a₀ has no value.

```calc
> Largest crack that takes N_p cycles to grow to the critical size
a_0 = { a_c exp(-N_p r_o (K_Ic / ΔK_o)^2 / a_c) if m == 2; a_c (1 - N_p (1 - m / 2) r_o (K_Ic / ΔK_o)^m / a_c)^(1 / (1 - m / 2)) otherwise } = ?? mm
> Proof-test stress that breaks any vessel with a longer crack
σ_proof = K_Ic / (Y sqrt(π a_0)) = ?? MPa
> Proof-test pressure
p_proof = σ_proof c_v t / r = ?? MPa
```

### Checks

```calc {description="Leak before break: a crack through the wall does not run"}
@test S σ ≤ σ_LBB
```

```calc {description="No general yield at the working pressure"}
@test S σ ≤ S_y
```

```calc {description="The proof test does not yield the vessel"}
@test σ_proof ≤ S_y
```
