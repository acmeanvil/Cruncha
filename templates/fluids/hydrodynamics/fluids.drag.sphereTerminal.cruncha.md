+++
cruncha-template = 1
id = "fluids.drag.sphereTerminal"
version = "1.0.0"
title = "Terminal velocity of a sphere"
summary = "The steady speed at which drag balances a sphere's weight less its buoyancy, v_t = √(4 g d |ρ_b − ρ|/(3 C_D ρ)), with the Reynolds number to read C_D again from Marks Fig. 11.4.23; and Stokes' law, v = |ρ_b − ρ| g d²/(18 μ), for Re below about 1."
category = ["Fluids", "Hydrodynamics"]
tags = ["terminal velocity", "settling velocity", "sphere", "particle", "sedimentation", "Stokes' law", "drag coefficient", "falling", "rising", "bubble", "Reynolds number", "Marks"]
from = "lacewing:fluids.drag"
license = "GPL-3.0-or-later"
inputs = ["d", "ρ_b", "ρ", "μ", "g", "C_D"]
outputs = ["v_t", "Re_t", "v_St", "Re_St"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "Sec. 3.3 (resistance of immersed bodies; Stokes' law); Sec. 11.4, Fig. 11.4.23 (C_D of a sphere against Re)"

[[tests]]
name = "Lacewing Lab: 1 mm steel ball falling in water"
source = "Lacewing Lab fluids_test: its iteration on Marks Fig. 11.4.23 (as Lacewing reads it) settles at Re 361.45, C_D 0.68440; with that C_D, v_t = √(4 × 9.80665 × 0.001 × 6852/(3 × 0.6844 × 998)) = 0.36218 m/s"
inputs = { d = "1 mm", "ρ_b" = "7850 kg/m^3", "ρ" = "998 kg/m^3", "μ" = "1 mPa·s", C_D = "0.6843965" }
expect = { v_t = "0.36217562 m/s", Re_t = "361.45127" }
tolerance = "1e-6"

[[tests]]
name = "Stokes' law: 10 µm quartz grain settling in water"
source = "Lacewing Lab fluids_test (Stokes' law for a tiny sphere). By hand: v = 1650 × 9.80665 × (1e-5)²/(18 × 1e-3) = 8.9894292e-5 m/s, Re = 1000 × 8.9894e-5 × 1e-5/1e-3 = 8.989e-4"
inputs = { d = "0.01 mm", "ρ_b" = "2650 kg/m^3", "ρ" = "1000 kg/m^3", "μ" = "1 mPa·s" }
expect = { v_St = "8.9894292e-5 m/s", Re_St = "8.9894292e-4" }
tolerance = "1e-7"
+++

## Terminal velocity of a sphere

A sphere heavier than the fluid falls, and one lighter rises, until the drag C_D ρv²/2 on its frontal area πd²/4 balances its weight less its buoyancy, |ρ_b − ρ| g πd³/6 (Marks Sec. 3.3). That speed is the terminal velocity.

C_D depends on the Reynolds number at the terminal velocity, so the balance is solved by iteration: guess C_D (0.4 to 0.5 for 10³ < Re < 2 × 10⁵), read Re_t, look up C_D again in Marks Fig. 11.4.23 at Re_t, and repeat until it stops changing. Read off a small figure, C_D is good to about ±10 %.

For Re below about 1 the drag is Stokes', C_D = 24/Re, and the terminal velocity is v_St directly; it overestimates the speed above that.

### Sphere and fluid

```calc {description="Diameter of the sphere"}
d = 1 mm_
```

```calc {description="Density of the sphere (quartz sand about 2650 kg/m³, steel 7850 kg/m³)"}
ρ_b = 2650 kg/m³_
```

```calc {description="Density of the fluid"}
ρ = 998.2 kg/m³_
```

```calc {description="Dynamic viscosity of the fluid"}
μ = 0.98 mPa·s_
```

```calc {description="Acceleration of gravity"}
g = 9.80665 m/s²_
```

### Terminal velocity

```calc {description="Drag coefficient of a sphere at Re_t, Marks Fig. 11.4.23 (iterate)"}
C_D = 0.7_
```

```calc
> Terminal velocity: drag equals weight less buoyancy
v_t = sqrt(4 g d abs(ρ_b - ρ) / (3 C_D ρ)) = ?? m/s
> Reynolds number at that speed: read C_D here again
Re_t = ρ v_t d / μ = ??
```

### Stokes' law

```calc
> Terminal velocity by Stokes' law (Re below about 1)
v_St = abs(ρ_b - ρ) g d^2 / (18 μ) = ?? m/s
> Reynolds number at the Stokes velocity
Re_St = ρ v_St d / μ = ??
```
