+++
cruncha-template = 1
id = "mechanical.pressureVessels.thinCone"
version = "1.0.0"
title = "Thin-walled cone under pressure (Roark)"
summary = "Membrane stresses at a section of a thin-walled conical shell under internal and external pressure, tangentially supported at its edge: meridional and hoop stress, von Mises stress, the factor of safety on yield, the growth in diameter, the change in the apex-to-section distance and the rotation of the meridian, after Roark Table 13.1 case 2a."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["pressure vessel", "cone", "conical shell", "reducer", "thin wall", "membrane stress", "hoop stress", "meridional stress", "von Mises", "Roark"]
from = "lacewing:mechanical.pressureVessels"
license = "GPL-3.0-or-later"
inputs = ["D_i", "t", "α", "p_i", "p_o", "E", "ν", "S_y", "n_d"]
outputs = ["q", "R", "R_t", "σ_1", "σ_2", "σ_vm", "n_y", "ΔD", "Δy", "ψ"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 13.1 and Table 13.1, case 2a (cone under uniform internal pressure, tangential edge support), p. 594; R2/t > 10 for the membrane formulas"

[[tests]]
name = "Lacewing Lab: 600 mm diameter at the section, 5 mm wall, 30° half-angle, 1 MPa"
source = "Lacewing Lab pressure_vessels_test (thin cone): σ2 = qR/(t cos α) = 1 × 302.5/(5 cos 30°) = 69.859383 MPa; by hand σ1 = 34.929691 MPa, σvm = (√3/2)σ2 = 60.5 MPa, n = 250/60.5 = 4.1322314, ΔD = 2qR²(1 − ν/2)/(Et cos α) = 0.17962594 mm, Δy = qR²(1 − 2ν − 3 tan²α)/(4Et sin α) = −0.027451875 mm, ψ = 3qR tan α/(2Et cos α) = 3.025e-4 rad (E 200 GPa, ν 0.3)"
inputs = { D_i = "600 mm", t = "5 mm", "α" = "30 deg", p_i = "1 MPa", p_o = "0 MPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { R = "302.5 mm", R_t = "69.859383", "σ_1" = "34.929691 MPa", "σ_2" = "69.859383 MPa", "σ_vm" = "60.5 MPa", n_y = "4.1322314", "ΔD" = "0.17962594 mm", "Δy" = "-0.027451875 mm", "ψ" = "3.025e-4 rad" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same cone against a design factor of 5"
source = "By hand: n = 4.1322314, under 5"
inputs = { D_i = "600 mm", t = "5 mm", "α" = "30 deg", p_i = "1 MPa", p_o = "0 MPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "5" }
expect = { n_y = "4.1322314" }
tolerance = "1e-6"
checks = "fail"
+++

## Thin-walled cone under pressure

The membrane stresses at one section of a thin-walled conical shell under internal pressure p_i and external pressure p_o, after Roark Table 13.1 case 2a. The section is where the inside diameter is D_i; R is the radius of the mid-surface's circumference there, and α is the half-angle between the cone's axis and its wall. The edge is supported tangentially: the meridional load is carried along the wall. The radial stress is neglected, and the formulas need R2/t > 10, with R2 = R/cos α the second principal radius.

Discontinuity stresses at the cone's junctions with cylinders (Roark Table 13.3), where a real reducer has its largest stresses, are not included; nor is buckling under net external pressure. It is not a design-code calculation.

### Geometry and loading

```calc {description="Inside diameter at the section whose stresses are wanted"}
D_i = 600 mm_
```

```calc {description="Wall thickness, less any corrosion allowance"}
t = 5 mm_
```

```calc {description="Half-angle between the cone's axis and its wall (0° to 90°)"}
α = 30 deg_
```

```calc {description="Internal (gauge) pressure"}
p_i = 1 MPa_
```

```calc {description="External (gauge) pressure"}
p_o = 0 MPa_
```

### Material

```calc {description="Young's modulus"}
E = 200 GPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc {description="Yield strength"}
S_y = 250 MPa_
```

```calc {description="Design factor the factor of safety on yield must reach"}
n_d = 1.5_
```

### Membrane stresses

```calc
> Net pressure
q = p_i - p_o = ?? MPa
> Mid-surface radius of the circumference at the section
R = D_i / 2 + t / 2 = ?? mm
> Slenderness R2/t, R2 = R/cos α; the membrane formulas need it over 10
R_t = R / (t cos(α)) = ??
> Meridional stress, Table 13.1 case 2a
σ_1 = q R / (2 t cos(α)) = ?? MPa
> Hoop stress, case 2a
σ_2 = q R / (t cos(α)) = ?? MPa
```

```calc
> Von Mises stress (the radial stress neglected)
σ_vm = sqrt(σ_1^2 - σ_1 σ_2 + σ_2^2) = ?? MPa
> Factor of safety on yield, distortion energy
n_y = S_y / σ_vm = ??
```

### Deformation

Δy is the change in the distance from the apex to the section, along the axis; ψ is the rotation of the meridian there.

```calc
> Change in mid-surface diameter, case 2a
ΔD = 2 q R^2 / (E t cos(α)) (1 - ν / 2) = ?? mm
> Change in the apex-to-section distance, case 2a
Δy = q R^2 / (4 E t sin(α)) (1 - 2 ν - 3 tan(α)^2) = ?? mm
> Rotation of the meridian, case 2a
ψ = 3 q R tan(α) / (2 E t cos(α)) = ?? rad
```

### Checks

```calc
@test R_t > 10
```

```calc
@test n_y ≥ n_d
```
