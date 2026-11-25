+++
cruncha-template = 1
id = "fluids.drag.cylinder"
version = "1.0.0"
title = "Cylinder in cross-flow: drag and vortex shedding"
summary = "Drag on a cylinder across a steady stream (a pole, stack, pipe or cable), F_D = C_D ρV²(d·l)/2 with C_D from Marks Fig. 11.4.23 at the Reynolds number, and the frequency at which it sheds vortices, f = S·V/d."
category = ["Fluids", "Hydrodynamics"]
tags = ["drag", "cylinder", "cross-flow", "vortex shedding", "Strouhal number", "Kármán vortex street", "wind load", "stack", "pole", "thermowell", "pipe in a current", "Reynolds number", "Marks"]
from = "lacewing:fluids.drag"
license = "GPL-3.0-or-later"
inputs = ["ρ", "μ", "V", "d", "l", "C_D", "S"]
outputs = ["Re", "q", "A", "F_D", "f_s"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "Sec. 3.3 (resistance of immersed bodies; vortex shedding, f = S·V/D with S ≈ 0.2); Sec. 11.4, Fig. 11.4.23 (C_D of a cylinder against Re)"

[[references]]
work = "ASME PTC 19.3 TW, Thermowells"
edition = "2016"
locator = "Strouhal number of a thermowell (about 0.22)"

[[tests]]
name = "Lacewing Lab: 0.5 m cylinder in a 10 m/s stream sheds at 4 Hz"
source = "Lacewing Lab fluids_test: f = 0.2 × 10/0.5 = 4 Hz"
inputs = { V = "10 m/s", d = "0.5 m", S = "0.2" }
expect = { f_s = "4 Hz" }
tolerance = "1e-9"

[[tests]]
name = "Hand check: 50 mm rod, 1 m long, in air at 3 m/s"
source = "By hand: Re = 1.204 × 3 × 0.05/1.81e-5 = 9 977.9, where Marks Fig. 11.4.23 gives C_D ≈ 1.2 (Lacewing Lab reads 1.2 at 10⁴); q = 5.418 Pa, A = 0.05 m², F_D = 1.2 × 5.418 × 0.05 = 0.32508 N, f = 0.2 × 3/0.05 = 12 Hz"
inputs = { "ρ" = "1.204 kg/m^3", "μ" = "0.0181 mPa·s", V = "3 m/s", d = "50 mm", l = "1 m", C_D = "1.2", S = "0.2" }
expect = { Re = "9977.9006", q = "5.418 Pa", A = "0.05 m^2", F_D = "0.32508 N", f_s = "12 Hz" }
tolerance = "1e-6"
+++

## Cylinder in cross-flow

A long circular cylinder with its axis square to a steady stream: a pole or stack in the wind, a pipe or cable in a current. The drag is C_D ρV²/2 on the projected area d·l (Marks Sec. 3.3).

C_D is that of an infinite cylinder, read from Marks Fig. 11.4.23 at the Reynolds number on the diameter: about 1.0 to 1.2 from Re 10³ to 2 × 10⁵, then a sharp drop (the drag crisis) to about 0.3 near 5 × 10⁵ as the boundary layer turns turbulent. A short cylinder with free ends has somewhat less drag. Read off a small figure, C_D is good to about ±10 %.

Vortices leave the two sides alternately at f = S·V/d, pushing the cylinder back and forth across the flow. Keep that frequency well away from the structure's natural frequencies (stacks, transmission lines, periscopes, pipes in a current). Marks takes the Strouhal number S ≈ 0.2; ASME PTC 19.3 takes 0.22 for thermowells.

### Fluid and flow

```calc {description="Density of the fluid (air at 20 °C, 1 atm: 1.204 kg/m³; water about 998 kg/m³)"}
ρ = 1.204 kg/m³_
```

```calc {description="Dynamic viscosity of the fluid (air at 20 °C about 0.0181 mPa·s, water about 1 mPa·s (1 cP))"}
μ = 0.0181 mPa·s_
```

```calc {description="Velocity of the stream"}
V = 5 m/s_
```

### Cylinder

```calc {description="Diameter of the cylinder"}
d = 0.3 m_
```

```calc {description="Length of the cylinder across the flow"}
l = 10 m_
```

```calc
> Reynolds number on the diameter
Re = ρ V d / μ = ??
> Dynamic pressure
q = ρ V^2 / 2 = ?? Pa
> Projected area
A = d l = ?? m²
```

```calc {description="Drag coefficient of a cylinder at Re, Marks Fig. 11.4.23"}
C_D = 1.2_
```

```calc {description="Drag force"}
F_D = C_D q A = ?? N
```

### Vortex shedding

```calc {description="Strouhal number (0.2, Marks; 0.22 for thermowells, ASME PTC 19.3)"}
S = 0.2_
```

```calc {description="Vortex-shedding frequency"}
f_s = S V / d = ?? Hz
```
