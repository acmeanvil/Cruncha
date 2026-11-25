+++
cruncha-template = 1
id = "fluids.depthPressure.pressure"
version = "1.0.0"
title = "Hydrostatic pressure at a depth"
summary = "Gauge and absolute pressure at a depth below the free surface of a liquid at rest, p = p0 + ρgh, the pressure per unit depth, and the gauge pressure as a column of water and of mercury."
category = ["Fluids", "Hydrodynamics"]
tags = ["hydrostatics", "fluid statics", "pressure", "depth", "head", "gauge pressure", "absolute pressure", "water column", "mercury column", "tank", "Marks"]
from = "lacewing:fluids.depthPressure"
license = "GPL-3.0-or-later"
inputs = ["ρ", "g", "p_0", "h"]
outputs = ["γ", "p_g", "p", "h_w", "h_Hg"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "Sec. 3.3, Fluid Statics (pressure in a liquid at rest); Table 3.3.1, Density of Liquids at Atmospheric Pressure"

[[tests]]
name = "Marks Sec. 3.3: benzene tank, 11 ft deep, 10 psi on the surface"
source = "Lacewing Lab fluids_test, after Marks 10th Sec. 3.3: benzene at 20 °C (1.705 slug/ft³ from Table 3.3.1) 11 ft deep under 10 psi gives 14.19 psi at the bottom"
inputs = { "ρ" = "1.705 slug/ft^3", h = "11 ft", p_0 = "10 psi" }
expect = { p = "14.19 psi" }
tolerance = "0.05%"

[[tests]]
name = "Hand check: 10 m of sea water under the atmosphere"
source = "By hand: ρg = 1025 × 9.80665 = 10 051.82 Pa/m, p_g = 100 518.16 Pa, p = 201 843.16 Pa; as fresh water at 4 °C 100 518.16/(999.97 × 9.80665) = 10.250308 m, as mercury at 0 °C (13 595.18 kg/m³) 0.7539438 m"
inputs = { "ρ" = "1025 kg/m^3", h = "10 m", p_0 = "101.325 kPa" }
expect = { "γ" = "10051.81625 Pa/m", p_g = "100518.1625 Pa", p = "201843.1625 Pa", h_w = "10.250308 m", h_Hg = "0.7539438 m" }
tolerance = "1e-6"
+++

## Pressure at a depth

The pressure in a liquid at rest of one density rises linearly with depth below its free surface: p = p₀ + ρgh (Marks Sec. 3.3). The gauge pressure ρgh is the rise above the surface pressure, which is what a gauge in the liquid reads; the absolute pressure adds the pressure on the surface, the atmosphere for an open tank.

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

```calc {description="Depth below the free surface"}
h = 10 m_
```

```calc
> Pressure per unit depth
γ = ρ g = ?? kPa/m
> Gauge pressure at the depth
p_g = ρ g h = ?? kPa
> Absolute pressure at the depth
p = p_0 + p_g = ?? kPa
```

### As a column of water or mercury

Manometers and older gauges read a pressure as the height of a column of liquid: fresh water at 4 °C (999.97 kg/m³) or mercury at 0 °C (Marks Table 3.3.1, 26.379 slug/ft³).

```calc {description="Density of fresh water at 4 °C"}
ρ_w = 999.97 kg/m³
```

```calc {description="Density of mercury at 0 °C, Marks Table 3.3.1"}
ρ_Hg = 13595.18 kg/m³
```

```calc
> Gauge pressure as a column of water
h_w = p_g / (ρ_w g) = ?? m
> Gauge pressure as a column of mercury
h_Hg = p_g / (ρ_Hg g) = ?? mm
```
