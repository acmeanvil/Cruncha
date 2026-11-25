+++
cruncha-template = 1
id = "fluids.depthPressure.gate"
version = "1.0.0"
title = "Hydrostatic force on a submerged gate"
summary = "The resultant force of a liquid at rest on a plane rectangular or circular gate, F = ρg·h_c·A, the depth of its centre of pressure h_F = h_c + I_G sin²θ/(h_c A), its offset below the centroid along the gate, and the pressure at the gate's bottom edge."
category = ["Fluids", "Hydrodynamics"]
tags = ["hydrostatics", "fluid statics", "gate", "hatch", "submerged surface", "plane surface", "centre of pressure", "center of pressure", "tank wall", "dam", "Marks"]
from = "lacewing:fluids.depthPressure"
license = "GPL-3.0-or-later"
inputs = ["ρ", "g", "shape", "b", "L", "d", "H", "θ"]
outputs = ["A", "I_G", "h_c", "F", "h_F", "e", "p_bot"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "Sec. 3.3, Fluid Statics: liquid forces on plane surfaces and the centre of pressure, Fig. 3.3.7 and its gate example; Table 3.3.1"

[[tests]]
name = "Marks Sec. 3.3: 3 × 5 ft vertical gate, bottom edge 12 ft under water"
source = "Lacewing Lab fluids_test, after Marks 10th Sec. 3.3: water 1.937 slug/ft³, h_c = 9.5 ft, F = 8 880 lbf by the book's own arithmetic (Marks prints 8 800), centre of pressure 9.719 ft deep"
inputs = { "ρ" = "1.937 slug/ft^3", shape = "1", b = "3 ft", L = "5 ft", H = "12 ft", "θ" = "90°" }
expect = { h_c = "9.5 ft", F = "8880 lbf", h_F = "9.719 ft" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "The same gate inclined 30° from horizontal"
source = "Lacewing Lab fluids_test: h_c = 12 − 2.5 sin 30° = 10.75 ft, F = 10 048 lbf, h_F = 10.80 ft. By hand: h_F = 10.75 + 31.25 × 0.25/(10.75 × 15) = 10.798450 ft, offset 31.25 × 0.5/(10.75 × 15) = 0.0968992 ft"
inputs = { "ρ" = "1.937 slug/ft^3", shape = "1", b = "3 ft", L = "5 ft", H = "12 ft", "θ" = "30°" }
expect = { h_c = "10.75 ft", F = "10049.28 lbf", h_F = "10.798450 ft", e = "0.0968992 ft" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Hand check: 1 m circular hatch in a vertical wall, bottom 3 m under fresh water"
source = "By hand: A = π/4 = 0.7853982 m², h_c = 2.5 m, F = 1000 × 9.80665 × 2.5 × 0.7853982 = 19 255.312 N, h_F = 2.5 + (π/64)/(2.5 × π/4) = 2.525 m, p at the bottom 29.41995 kPa"
inputs = { "ρ" = "1000 kg/m^3", shape = "2", d = "1 m", H = "3 m", "θ" = "90°" }
expect = { A = "0.78539816 m^2", h_c = "2.5 m", F = "19255.312 N", h_F = "2.525 m", e = "0.025 m", p_bot = "29.41995 kPa" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "A gate that breaks the surface"
source = "By hand: a 5 ft vertical gate with its bottom edge only 4 ft deep stands 1 ft out of the water, which the formulas don't cover"
inputs = { "ρ" = "1.937 slug/ft^3", shape = "1", b = "3 ft", L = "5 ft", H = "4 ft", "θ" = "90°" }
expect = { h_c = "1.5 ft" }
tolerance = "1e-9"
checks = "fail"
+++

## Force on a submerged gate

The resultant of a liquid's pressure on a plane gate wholly below the free surface, from the liquid alone (Marks Sec. 3.3, Fig. 3.3.7). It acts normal to the gate with the magnitude of the pressure at the centroid times the area. Because the pressure grows with depth, it acts below the centroid, at the centre of pressure. The surface pressure is left out: for an open tank the atmosphere acts on both faces of the gate.

The liquid is at rest and of one density. Densities by temperature are in Marks Table 3.3.1.

### Liquid

```calc {description="Density of the liquid (e.g. Marks Table 3.3.1)"}
ρ = 998.3 kg/m³_
```

```calc {description="Acceleration of gravity (standard 9.80665 m/s²)"}
g = 9.80665 m/s²_
```

### Gate

Enter the shape as a number: 1 rectangle (width b along the horizontal, height L measured up its slope), 2 circle (diameter d). The inclination θ is the gate's plane from horizontal: 90° for a vertical wall. The bottom edge is H below the free surface.

```calc {description="Gate shape: 1 rectangle, 2 circle"}
shape = 1_
```

```calc {description="Width of a rectangular gate, along the horizontal"}
b = 3 ft_
```

```calc {description="Height of a rectangular gate, measured along its slope"}
L = 5 ft_
```

```calc {description="Diameter of a circular gate"}
d = 1 m_
```

```calc {description="Depth of the gate's bottom edge below the free surface"}
H = 12 ft_
```

```calc {description="Inclination of the gate's plane from horizontal (90° vertical)"}
θ = 90°_
```

```calc
> Length of the gate along its slope
ℓ = {L if shape == 1; d if shape == 2} = ?? ft
> Area
A = {b L if shape == 1; π d^2 / 4 if shape == 2} = ?? ft²
> Second moment of the area about its centroidal axis parallel to the surface
I_G = {b L^3 / 12 if shape == 1; π d^4 / 64 if shape == 2} = ?? ft⁴
> Depth of the centroid
h_c = H - ℓ sin(θ) / 2 = ?? ft
```

### Force and centre of pressure

```calc
> Resultant force of the liquid, normal to the gate
F = ρ g h_c A = ?? lbf
> Depth of the centre of pressure, Fig. 3.3.7
h_F = h_c + I_G sin(θ)^2 / (h_c A) = ?? ft
> Distance of the centre of pressure below the centroid, along the gate
e = I_G sin(θ) / (h_c A) = ?? ft
> Gauge pressure at the bottom edge, the highest on the gate
p_bot = ρ g H = ?? psi
```

### Check

The gate must lie wholly below the free surface.

```calc
@test H ≥ ℓ sin(θ)
```
