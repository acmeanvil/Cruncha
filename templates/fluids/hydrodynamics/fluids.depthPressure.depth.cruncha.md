+++
cruncha-template = 1
id = "fluids.depthPressure.depth"
version = "1.0.0"
title = "Depth for a hydrostatic pressure"
summary = "The depth below the free surface of a liquid at rest at which a gauge pressure is reached, h = p_g/(ρg), and the absolute pressure there."
category = ["Fluids", "Hydrodynamics"]
tags = ["hydrostatics", "fluid statics", "pressure", "depth", "head", "gauge pressure", "submergence", "Marks"]
from = "lacewing:fluids.depthPressure"
license = "GPL-3.0-or-later"
inputs = ["ρ", "g", "p_0", "p_g"]
outputs = ["h", "p"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "Sec. 3.3, Fluid Statics (pressure in a liquid at rest); Table 3.3.1, Density of Liquids at Atmospheric Pressure"

[[tests]]
name = "One standard atmosphere of fresh water at 20 °C"
source = "Lacewing Lab fluids_test (about 10.3 m of water to an atmosphere). By hand: h = 101 325/(998.2 × 9.80665) = 10.350906 m"
inputs = { "ρ" = "998.2 kg/m^3", p_g = "101.325 kPa" }
expect = { h = "10.350906 m", p = "202.65 kPa" }
tolerance = "1e-6"

[[tests]]
name = "Hand check: 100 kPa gauge in fresh water, Marks Table 3.3.1 at 20 °C"
source = "By hand: ρ = 1.937 slug/ft³ = 998.28877 kg/m³, h = 100 000/(998.28877 × 9.80665) = 10.214642 m"
inputs = { "ρ" = "1.937 slug/ft^3", p_g = "100 kPa" }
expect = { h = "10.214642 m" }
tolerance = "1e-6"
+++

## Depth for a pressure

The inverse of p = p₀ + ρgh for a liquid at rest of one density (Marks Sec. 3.3): the depth below the free surface at which the pressure has risen by a gauge pressure p_g above the surface pressure. The depth does not depend on the surface pressure; the absolute pressure there does.

Densities of common liquids by temperature are in Marks Table 3.3.1: fresh water is about 998 kg/m³ at 20 °C, sea water about 1025 kg/m³.

```calc {description="Density of the liquid (e.g. Marks Table 3.3.1)"}
ρ = 998.3 kg/m³_
```

```calc {description="Acceleration of gravity (standard 9.80665 m/s²; local or another planet's)"}
g = 9.80665 m/s²_
```

```calc {description="Absolute pressure on the free surface (101.325 kPa for a tank open to the atmosphere)"}
p_0 = 101.325 kPa_
```

```calc {description="Gauge pressure, above the surface pressure"}
p_g = 100 kPa_
```

```calc
> Depth at which the gauge pressure is reached
h = p_g / (ρ g) = ?? m
> Absolute pressure there
p = p_0 + p_g = ?? kPa
```
