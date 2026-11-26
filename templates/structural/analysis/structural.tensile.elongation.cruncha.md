+++
cruncha-template = 1
id = "structural.tensile.elongation"
version = "1.0.0"
title = "Tension member: elongation, axial stiffness, stress and breaking load"
summary = "Elastic and thermal elongation, axial stiffness EA/L, stress, strain, mass per length and the breaking load with its factor of safety, for a rod, bar, cable or any section in tension."
category = ["Structural engineering", "Analysis"]
tags = ["tension member", "tie rod", "cable", "rod", "elongation", "stretch", "axial stiffness", "thermal expansion", "breaking load", "turnbuckle"]
from = "lacewing:structural.tensileElongation"
license = "GPL-3.0-or-later"
inputs = ["A", "E", "ρ", "σ_u", "L", "T", "α", "ΔT"]
outputs = ["σ", "ε", "δ_el", "δ_th", "δ", "k", "μ", "F_u", "FS"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 4-2, Eqs. 4-3 (δ = Fl/(AE)) and 4-4 (k = AE/l); the free thermal strain αΔT is not from this source"

[[tests]]
name = "Lacewing Lab: 100 kN on 1000 mm², 10 m, ΔT 20 K"
source = "Lacewing Lab tensile_test (elongation and stiffness): σ = 100 MPa, elastic 5 mm, thermal 12e-6 × 20 × 10 m = 2.4 mm, k = 2e7 N/m; FS and μ by hand: Fu = 400 MPa × 1000 mm² = 400 kN, FS = 4, μ = 7.85 kg/m"
inputs = { A = "1000 mm^2", E = "200 GPa", "ρ" = "7850 kg/m^3", "σ_u" = "400 MPa", L = "10 m", T = "100 kN", "α" = "12e-6 1/K", "ΔT" = "20 K" }
expect = { "σ" = "100 MPa", "ε" = "0.0005", "δ_el" = "5 mm", "δ_th" = "2.4 mm", "δ" = "7.4 mm", k = "2e7 N/m", "μ" = "7.85 kg/m", F_u = "400 kN", FS = "4" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "1 in. A36 tie rod, 20 ft, 15 kips, cooled 40 °F"
source = "By hand: A = π/4 = 0.78540 in², σ = 19.099 ksi, δel = 15 000 × 240/(29e6 × 0.7854) = 0.158057 in, δth = 6.5e-6 × (−40) × 240 = −0.0624 in, k = 29e6 × 0.7854/240 = 94 902 lbf/in, Fu = 58 ksi × 0.7854 = 45.553 kips, FS = 3.0369"
inputs = { A = "0.78539816 in^2", E = "29000 ksi", "ρ" = "490 lb/ft^3", "σ_u" = "58 ksi", L = "20 ft", T = "15 kips", "α" = "6.5e-6 1/Δ°F", "ΔT" = "-40 Δ°F" }
expect = { "σ" = "19.098593 ksi", "δ_el" = "0.15805732 in", "δ_th" = "-0.0624 in", "δ" = "0.09565732 in", k = "94902.278 lbf/in", F_u = "45.553093 kips", FS = "3.0368729" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Over the breaking load: the check fails"
source = "As the first case at 450 kN: FS = 0.889"
inputs = { A = "1000 mm^2", E = "200 GPa", "ρ" = "7850 kg/m^3", "σ_u" = "400 MPa", L = "10 m", T = "450 kN", "α" = "12e-6 1/K", "ΔT" = "20 K" }
expect = { FS = "0.88888889" }
tolerance = "1e-7"
checks = "fail"
+++

## Tension member: elongation and stiffness

The axial behaviour of a straight member in tension: a tie rod, a flat bar, a structural strand or rope, or any section with a known load-carrying area. Linear elastic: Hooke's law for the stretch, free expansion for a temperature change. Use it to size a tie, to find how far to take up a turnbuckle, or for the spring rate of a stay in a frame model.

For a solid round rod A = πd²/4; for a bar, w·t. For a structural strand or rope, use the published metallic area and the rope's own modulus on that area (well below that of its wire), and compare the tension with the manufacturer's minimum breaking strength, which governs over σ_u·A. Constructional stretch and prestretching of cables are not included.

### Member

```calc {description="Load-carrying (metallic) area of the section"}
A = 1000 mm²_
```

```calc {description="Modulus of elasticity (for a cable, the rope's modulus on its metallic area)"}
E = 200 GPa_
```

```calc {description="Density, for the mass per length"}
ρ = 7850 kg/m³_
```

```calc {description="Ultimate tensile strength"}
σ_u = 400 MPa_
```

```calc {description="Free length between end connections"}
L = 10 m_
```

### Load and temperature

The temperature change is from when the length was set; warming is positive and lengthens the member. Typical expansion coefficients: steel about 12 × 10⁻⁶ /K, austenitic stainless about 17 × 10⁻⁶ /K, aluminium about 23 × 10⁻⁶ /K.

```calc {description="Axial tension"}
T = 100 kN_
```

```calc {description="Mean coefficient of linear thermal expansion"}
α = 12e-6 1/K_
```

```calc {description="Temperature change since the length was set (warming positive)"}
ΔT = 20 K_
```

### Stress and stretch

```calc
> Axial stress on the metallic area
σ = T / A = ?? MPa
> Elastic strain
ε = σ / E = ??
> Elastic elongation
δ_el = T L / (E A) = ?? mm
> Free thermal elongation
δ_th = α ΔT L = ?? mm
> Total change of length
δ = δ_el + δ_th = ?? mm
```

Between fixed ends the thermal part cannot happen: the tension changes by E A α ΔT instead (falling when warmed).

### Stiffness, mass and strength

```calc
> Axial stiffness, the spring rate of the member
k = E A / L = ?? kN/mm
> Mass per length
μ = ρ A = ?? kg/m
> Breaking load on the metallic area
F_u = σ_u A = ?? kN
> Factor of safety against breaking
FS = F_u / T = ??
```

### Checks

```calc
@test FS > 1
```
