+++
cruncha-template = 1
id = "dynamics.naturalFrequencies.rectangularPlate"
version = "1.0.0"
title = "Natural frequencies of a simply supported rectangular plate"
summary = "The bending natural frequencies of a thin, uniform rectangular plate simply supported on all four edges, ωij = π²((i/a)² + (j/b)²) √(D/(ρt)), for the first three half-waves each way, with the fundamental. Checks a forcing frequency's separation from all nine modes."
category = ["Mechanical engineering", "Vibrations"]
tags = ["natural frequency", "plate", "rectangular plate", "panel", "simply supported", "bending vibration", "flexural rigidity", "mode", "Navier", "Blevins", "Leissa"]
from = "lacewing:dynamics.naturalFrequencies"
license = "GPL-3.0-or-later"
inputs = ["E", "ν", "ρ", "t", "a", "b", "ω_f", "s_min"]
outputs = ["D", "μ_1", "ω", "ω_11", "f_11", "s"]

[[references]]
work = "Blevins, Formulas for Natural Frequency and Mode Shape"
edition = "1979 (Krieger reprint 2001)"
locator = "Ch. 11, Table 11-4 (rectangular plates; all edges simply supported, the closed form)"

[[references]]
work = "Leissa, Vibration of Plates (NASA SP-160)"
edition = "1969"
locator = "Ch. 4, Sec. 4.1 (rectangular plates simply supported on all edges)"

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "Appendix II, Eq. 6a (plate constant D)"

[[tests]]
name = "Hand check: square plate, λ² = 2π²"
source = "By hand (Blevins Table 11-4: a square plate simply supported all round has λ11² = 2π² = 19.739209): 500 mm square, 2 mm steel, D = 146.52015 N·m, √(D/(ρt)) = √(146.52015/15.7) = 3.0549130 m²/s, ω11 = 19.739209 × 3.0549130/0.5² = 241.20626 rad/s"
inputs = { E = "200 GPa", "ν" = "0.3", "ρ" = "7850 kg/m³", t = "2 mm", a = "500 mm", b = "500 mm", "ω_f" = "100 rad/s" }
expect = { D = "146.52015 N·m", "ω_11" = "241.20626 rad/s", f_11 = "38.389169 Hz" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Hand check: 500 × 300 mm panel, forced near the (2, 1) mode"
source = "By hand: ω11 = π² (1/0.5² + 1/0.3²) × 3.0549130 = 455.61183 rad/s, ω21 = π² (4/0.25 + 1/0.09) × 3.0549130 = 817.42122 rad/s; forcing at 800 rad/s is 2.1 % below (2, 1)"
inputs = { E = "200 GPa", "ν" = "0.3", "ρ" = "7850 kg/m³", t = "2 mm", a = "500 mm", b = "300 mm", "ω_f" = "800 rad/s", s_min = "0.15" }
expect = { "ω_11" = "455.61183 rad/s", s = "0.02131242" }
tolerance = "1e-6"
checks = "fail"
+++

## Natural frequencies of a simply supported rectangular plate

A thin, flat rectangular plate of uniform thickness and material, simply supported (hinged, free to rotate) on all four edges, vibrating in bending by classical (Kirchhoff) plate theory. This case has an exact closed form: the mode with i half-waves along a and j along b has ω_ij = π²((i/a)² + (j/b)²) √(D/(ρt)) (Blevins Table 11-4, Leissa Sec. 4.1). The thickness must be small against both sides, deflections small, and no in-plane load or fluid around it. Other edge conditions have no closed form; their coefficients are in Blevins Table 11-4 and Leissa.

### Plate

```calc {description="Young's modulus"}
E = 200 GPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc {description="Density"}
ρ = 7850 kg/m³_
```

```calc {description="Thickness"}
t = 2 mm_
```

```calc {description="Length of one side"}
a = 500 mm_
```

```calc {description="Length of the other side"}
b = 300 mm_
```

```calc
> Plate constant (flexural rigidity), Den Hartog Eq. 6a
D = E t^3 / (12 (1 - ν^2)) = ?? N·m
> Mass per area
μ_1 = ρ t = ?? kg/m²
```

Rows of ω are i = 1, 2, 3 half-waves along a; columns j = 1, 2, 3 along b.

```calc {description="Natural frequencies ω_ij, i = 1 to 3 down, j = 1 to 3 across"}
ω = π^2 (([1; 2; 3] / a)^2 [1, 1, 1] + [1; 1; 1] ([1, 2, 3] / b)^2) sqrt(D / μ_1) = ?? rad/s
```

```calc
> Fundamental, one half-wave each way
ω_11 = π^2 ((1 / a)^2 + (1 / b)^2) sqrt(D / μ_1) = ?? rad/s
f_11 = ω_11 / (2 π) = ?? Hz
```

### Separation from the forcing frequency

```calc {description="Forcing (circular) frequency"}
ω_f = 100 rad/s_
```

```calc {description="Smallest acceptable separation, as a fraction of the natural frequency"}
s_min = 0.2_
```

```calc {description="Separation of the forcing frequency from the nearest of the nine modes"}
s = min(abs(ω_f / ω - 1)) = ??
```

```calc
@test s ≥ s_min
```
