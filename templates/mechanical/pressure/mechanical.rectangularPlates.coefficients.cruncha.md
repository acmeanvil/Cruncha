+++
cruncha-template = 1
id = "mechanical.rectangularPlates.coefficients"
version = "1.0.0"
title = "Rectangular plate under pressure, tabulated coefficients (Den Hartog)"
summary = "Largest deflection, bending moment per unit length and bending stress of a flat rectangular plate with simply supported, clamped or mixed edges under uniform or hydrostatic pressure, from the deflection and moment coefficients of Den Hartog's catalogue (§19, Cases 19, 20 and 22 to 25) entered for the plate's side ratio, with the factor of safety on yield and the small-deflection limit checked."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["plate", "flat plate", "rectangular plate", "clamped plate", "fixed edges", "simply supported", "hydrostatic", "tank wall", "floor slab", "plate coefficients", "Den Hartog", "Roark"]
from = "lacewing:mechanical.rectangularPlates"
license = "GPL-3.0-or-later"
inputs = ["a", "b", "t", "p_0", "α_w", "β_M", "E", "S_y", "n_d"]
outputs = ["ratio", "w", "M_1", "σ", "n_y"]

[[references]]
work = "Den Hartog, Advanced Strength of Materials"
edition = "1952"
locator = "§19 Catalogue of Results, Cases 19 to 25 (deflection w = coefficient·p₀a⁴/(Et³), moment M₁ = coefficient·p₀a², ν = 0.3); Eq. 68 (σ = 6M/t²); §20 (limit of small deflections)"

[[tests]]
name = "Lacewing Lab worked example: Case 24, all edges clamped, 300 × 600 × 6 mm steel at 1 bar"
source = "Lacewing Lab rectangular_plates help and test: Den Hartog Case 24 at b/a = 2 gives 0.0277 and 0.0829, so w = 0.0277 × 10⁵ × 0.3⁴/(200e9 × 0.006³) = 0.5194 mm, M1 = 0.0829 × 10⁵ × 0.3² = 746.1 N·m/m and σ = 6M1/t² = 124.35 MPa"
inputs = { a = "300 mm", b = "600 mm", t = "6 mm", p_0 = "100 kPa", "α_w" = "0.0277", "β_M" = "0.0829", E = "200 GPa", S_y = "250 MPa", n_d = "1.5" }
expect = { ratio = "2", w = "0.5194 mm", M_1 = "746.1 N·m/m", "σ" = "124.35 MPa", n_y = "2.0105" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Lacewing Lab: Case 24, 500 × 1000 × 10 mm at 1 bar"
source = "Lacewing Lab rectangular_plates_test: w = 0.0277 × 10⁵ × 0.5⁴/(2e11 × 1e-6) = 0.865625 mm, M1 = 0.0829 × 10⁵ × 0.25 = 2072.5 N·m/m, σ = 6 × 2072.5/1e-4 = 124.35 MPa"
inputs = { a = "500 mm", b = "1000 mm", t = "10 mm", p_0 = "100 kPa", "α_w" = "0.0277", "β_M" = "0.0829", E = "200 GPa", S_y = "250 MPa", n_d = "1.5" }
expect = { w = "0.865625 mm", M_1 = "2072.5 N·m/m", "σ" = "124.35 MPa" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "The worked example at 3 bar: over the design factor"
source = "By hand: σ = 3 × 124.35 = 373.05 MPa, n = 0.67015, and w = 1.558125 mm is still under t/2"
inputs = { a = "300 mm", b = "600 mm", t = "6 mm", p_0 = "300 kPa", "α_w" = "0.0277", "β_M" = "0.0829", E = "200 GPa", S_y = "250 MPa", n_d = "1.5" }
expect = { "σ" = "373.05 MPa", w = "1.558125 mm" }
tolerance = "1e-6"
checks = "fail"
+++

## Rectangular plate under pressure, tabulated coefficients

The largest deflection, bending moment and bending stress of a flat rectangular plate a × b, thickness t, under pressure, from the coefficients of Den Hartog's catalogue of plate results (§19). Read the two coefficients for your edges, load and b/a from his tables, interpolating between the tabulated ratios (linearly in a/b, so b/a = ∞ is the long-strip limit), and enter them below. The coefficients take ν = 0.3, so they suit metals.

- Case 19: all edges simply supported, uniform pressure; coefficients α, β; a the shorter side (b/a ≥ 1).
- Case 20: simply supported, pressure rising across a from 0 to p₀ (a tank wall); γ, δ; b/a from 0.25.
- Case 22: the a edges clamped, the b edges simply supported, uniform; ζ, η; b/a from 0.25.
- Case 23: one a edge clamped, three simply supported, uniform; κ, λ; b/a from 0.25.
- Case 24: all edges clamped, uniform; μ, ν; a the shorter side (b/a ≥ 1).
- Case 25: a field of a floor slab on corner columns, uniform; ξ, ρ; a the shorter side (b/a ≥ 1).

For Case 19 the simply supported template sums Navier's series instead of reading a table. Under a central point load (Case 21) the moment at the load is unbounded, and this template does not apply.

Clamped edges assume the neighbouring wall or beam is much stiffer than the plate; simply supported ones that it does not restrain rotation. The plate must be thin and its deflection well under t: as w approaches t/2 it carries load as a membrane too, and these results overstate the deflection and stress (§20). Clamped plates have their largest moment at the middle of the long clamped edge; simply supported ones at the centre.

### Plate

```calc {description="Span a, the side Den Hartog's coefficients are based on (across which a hydrostatic pressure varies)"}
a = 300 mm_
```

```calc {description="The other side, b"}
b = 600 mm_
```

```calc {description="Plate thickness"}
t = 6 mm_
```

```calc {description="Pressure: uniform, or the largest (at the deep edge) for the hydrostatic case"}
p_0 = 100 kPa_
```

```calc {description="Side ratio b/a, to read the coefficients at"}
ratio = b / a = ??
```

### Coefficients

```calc {description="Deflection coefficient for the case at b/a, Den Hartog §19 Cases 19-25 (α, γ, ζ, κ, μ or ξ); 0.0277 for Case 24 at b/a = 2"}
α_w = 0.0277_
```

```calc {description="Moment coefficient for the case at b/a, Den Hartog §19 Cases 19-25 (β, δ, η, λ, ν or ρ); 0.0829 for Case 24 at b/a = 2"}
β_M = 0.0829_
```

### Material

```calc {description="Young's modulus"}
E = 200 GPa_
```

```calc {description="Yield strength"}
S_y = 250 MPa_
```

```calc {description="Design factor the factor of safety on yield must reach"}
n_d = 1.5_
```

### Deflection and stress

```calc
> Largest deflection, Den Hartog §19
w = α_w p_0 a^4 / (E t^3) = ?? mm
> Largest bending moment per unit length, §19
M_1 = β_M p_0 a^2 = ?? N·m/m
> Largest bending stress, Eq. 68
σ = 6 M_1 / t^2 = ?? MPa
> Factor of safety on yield
n_y = S_y / σ = ??
```

### Checks

```calc
@test w ≤ t / 2
```

```calc
@test n_y ≥ n_d
```
