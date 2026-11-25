+++
cruncha-template = 1
id = "fluids.drag.body"
version = "1.0.0"
title = "Drag force on an immersed body"
summary = "Drag on a body in a steady stream, F_D = C_D ρV²A/2 on its frontal area, with the dynamic pressure and the Reynolds number; the drag coefficient is read from Marks Table 11.4.4 (plates, disks, cups, cones, cylinders end-on) or Fig. 11.4.23 (sphere, cylinder) at that Reynolds number."
category = ["Fluids", "Hydrodynamics"]
tags = ["drag", "drag force", "drag coefficient", "wind load", "dynamic pressure", "Reynolds number", "flat plate", "disk", "sphere", "cone", "bluff body", "Marks"]
from = "lacewing:fluids.drag"
license = "GPL-3.0-or-later"
inputs = ["ρ", "μ", "V", "A", "ℓ", "C_D"]
outputs = ["q", "Re", "F_D"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "Sec. 3.3 (resistance of immersed bodies, F_D = C_D ρV²A/2); Sec. 11.4, Table 11.4.4 (drag coefficients of shapes) and Fig. 11.4.23 (C_D of a sphere and a cylinder against Re)"

[[tests]]
name = "Lacewing Lab's opening case: a circular disk of 0.1 m² face-on to air at 20 m/s"
source = "Lacewing Lab fluid drag page, C_D = 1.11 for a disk (Marks Table 11.4.4), air at 20 °C 1.204 kg/m³ and 0.0181 mPa·s. By hand: q = 1.204 × 20²/2 = 240.8 Pa, F_D = 1.11 × 240.8 × 0.1 = 26.7288 N, Re = 1.204 × 20 × √0.1/1.81e-5 = 420 705"
inputs = { "ρ" = "1.204 kg/m^3", "μ" = "0.0181 mPa·s", V = "20 m/s", A = "0.1 m^2", "ℓ" = "0.316227766 m", C_D = "1.11" }
expect = { q = "240.8 Pa", F_D = "26.7288 N", Re = "420705.23" }
tolerance = "1e-6"

[[tests]]
name = "Hand check: a 50 mm sphere in water at 2 m/s"
source = "By hand: Re = 998.2 × 2 × 0.05/0.98e-3 = 101 857, near which Marks Fig. 11.4.23 gives C_D ≈ 0.47 (Lacewing Lab reads 0.47 at 10⁵); A = π 0.05²/4 = 0.0019634954 m², q = 1996.4 Pa, F_D = 0.47 × 1996.4 × 0.0019634954 = 1.8423634 N"
inputs = { "ρ" = "998.2 kg/m^3", "μ" = "0.98 mPa·s", V = "2 m/s", A = "0.0019634954 m^2", "ℓ" = "0.05 m", C_D = "0.47" }
expect = { Re = "101857.14", q = "1996.4 Pa", F_D = "1.8423634 N" }
tolerance = "1e-6"
+++

## Drag on an immersed body

The force along the flow on a body held in a steady stream (or moving through still fluid) is the dynamic pressure ρV²/2 times the frontal area A times a drag coefficient C_D (Marks Sec. 3.3). Use it for wind on signs, poles and stacks, or the force on a body towed through water.

C_D depends on the shape and the Reynolds number. It is chart and table data, so it is an input here:
- **Shapes** (flat plates of several aspect ratios, disks, pairs of disks, cylinders end-on, hemispherical cups, cones): Marks Table 11.4.4, on the frontal area. These are pressure drag and hold above Re ≈ 10³; below that the table understates the drag. For these, take ℓ = √A.
- **A sphere or a cylinder across the flow**: Marks Fig. 11.4.23 at the Reynolds number on its diameter (ℓ = d; A = πd²/4 for a sphere, d·l for a cylinder). Near Re ≈ 3 × 10⁵ the boundary layer turns turbulent and C_D drops sharply (the drag crisis). For a cylinder, the template fluids.drag.cylinder also gives the vortex-shedding frequency.

Read off a small figure, C_D is good to about ±10 %: treat the force as an engineering estimate.

### Fluid and flow

```calc {description="Density of the fluid (air at 20 °C, 1 atm: 1.204 kg/m³; water about 998 kg/m³)"}
ρ = 1.204 kg/m³_
```

```calc {description="Dynamic viscosity of the fluid (air at 20 °C about 0.0181 mPa·s, water about 1 mPa·s (1 cP))"}
μ = 0.0181 mPa·s_
```

```calc {description="Velocity of the body relative to the undisturbed fluid"}
V = 20 m/s_
```

### Body

```calc {description="Frontal area: the body projected on a plane normal to the flow"}
A = 0.1 m²_
```

```calc {description="Length the Reynolds number is based on: diameter of a sphere or cylinder, √A for a Table 11.4.4 shape"}
ℓ = 0.316 m_
```

```calc {description="Drag coefficient on the frontal area, Marks Table 11.4.4 or Fig. 11.4.23 at Re (circular disk 1.11)"}
C_D = 1.11_
```

### Drag

```calc
> Dynamic pressure, the stagnation pressure rise
q = ρ V^2 / 2 = ?? Pa
> Reynolds number
Re = ρ V ℓ / μ = ??
> Drag force
F_D = C_D q A = ?? N
```
