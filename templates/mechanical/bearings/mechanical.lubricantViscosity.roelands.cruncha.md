+++
cruncha-template = 1
id = "mechanical.lubricantViscosity.roelands"
version = "1.0.0"
title = "Oil viscosity through two points, Roelands"
summary = "The absolute viscosity of an oil at any temperature from two measured absolute viscosities, by Roelands' viscosity-temperature relation log(log η + 1.200) = −S0 log(1 + t/135) + log G0 (η in cP, t in °C), with its constants S0 and G0."
category = ["Mechanical engineering", "Bearings"]
tags = ["viscosity", "absolute viscosity", "dynamic viscosity", "viscosity-temperature", "Roelands", "lubricant", "oil", "two-point", "data sheet", "NASA RP-1255", "Hamrock"]
from = "lacewing:mechanical.lubricantViscosity"
license = "GPL-3.0-or-later"
inputs = ["T_1", "η_1", "T_2", "η_2", "T"]
outputs = ["S_0", "G_0", "η"]

[[references]]
work = "Hamrock, Schmid and Jacobson, Fundamentals of Fluid Film Lubrication (NASA RP-1255)"
edition = "1991"
locator = "Sec. 4.5, Eq. 4-13 (Roelands' viscosity-temperature relation); Table 4-7 (Klaman's base oils)"

[[references]]
work = "Roelands, Correlational Aspects of the Viscosity-Temperature-Pressure Relationship of Lubricating Oils (thesis, Delft)"
edition = "1966"
locator = "the temperature relation at atmospheric pressure"

[[tests]]
name = "RP-1255 Table 4-7: Klaman's paraffinic base oil through 40 and 100 °C, at 20 °C"
source = "Lacewing Lab lubricants_test, after Hamrock (NASA RP-1255) Table 4-7: measured 25.8 cP at 40 °C and 4.32 cP at 100 °C give 68 cP at 20 °C within 5 % (the relation gives 66.0)"
inputs = { T_1 = "40 °C", "η_1" = "25.8 mPa·s", T_2 = "100 °C", "η_2" = "4.32 mPa·s", T = "20 °C" }
expect = { "η" = "68 mPa·s" }
tolerance = "5%"

[[tests]]
name = "RP-1255 Table 4-7: the same oil at 60 °C"
source = "Lacewing Lab lubricants_test, after Hamrock (NASA RP-1255) Table 4-7: 12.33 cP measured at 60 °C (within 3 %)"
inputs = { T_1 = "40 °C", "η_1" = "25.8 mPa·s", T_2 = "100 °C", "η_2" = "4.32 mPa·s", T = "60 °C" }
expect = { "η" = "12.33 mPa·s" }
tolerance = "3%"

[[tests]]
name = "Hand check: Klaman's paraffinic oil at 150 °C"
source = "By hand: F = log(log η + 1.2) is 0.4169099 at 25.8 cP and 0.2637505 at 4.32 cP; x = log(1 + t/135) is 0.1127043 and 0.2407341; S0 = 1.1962791, log G0 = 0.5517357; at 150 °C, η = 10^(10^(0.5517357 − 1.1962791 log(1 + 150/135)) − 1.2) = 1.8081561 cP (RP-1255 measured 1.83)"
inputs = { T_1 = "40 °C", "η_1" = "25.8 mPa·s", T_2 = "100 °C", "η_2" = "4.32 mPa·s", T = "150 °C" }
expect = { S_0 = "1.19627913", "η" = "1.80815612 mPa·s" }
tolerance = "1e-6"
+++

## Oil viscosity through two points, Roelands

Roelands' relation describes how the viscosity of a lubricating oil falls with temperature at atmospheric pressure: log(log η + 1.200) is linear in log(1 + t/135), with η in centipoise (mPa·s) and t in degrees Celsius. Two measured viscosities fix its two constants: the slope S₀ and the intercept log G₀. Hamrock checks it against Klaman's measurements of base oils: through 40 and 100 °C it gives the other temperatures to a few percent. For kinematic viscosities, the Walther (ASTM D341) template does the same job.

```calc {description="First temperature of measurement"}
T_1 = 40 °C_
```

```calc {description="Absolute viscosity at the first temperature"}
η_1 = 25.8 mPa·s_
```

```calc {description="Second temperature of measurement"}
T_2 = 100 °C_
```

```calc {description="Absolute viscosity at the second temperature"}
η_2 = 4.32 mPa·s_
```

```calc {description="Temperature at which the viscosity is wanted"}
T = 150 °C_
```

```calc
> Roelands' viscosity function at the two points
F_1 = log10(log10(η_1 / (1 'mPa·s')) + 1.2) = ??
F_2 = log10(log10(η_2 / (1 'mPa·s')) + 1.2) = ??
> Temperature function at the two points
x_1 = log10(1 + (T_1 - 0 '°C') / (135 'K')) = ??
x_2 = log10(1 + (T_2 - 0 '°C') / (135 'K')) = ??
> Slope S0
S_0 = (F_1 - F_2) / (x_2 - x_1) = ??
> Intercept log G0
g_0 = F_1 + S_0 x_1 = ??
> G0
G_0 = 10^g_0 = ??
> Absolute viscosity at T, RP-1255 Eq. 4-13
η = 10^(10^(g_0 - S_0 log10(1 + (T - 0 '°C') / (135 'K'))) - 1.2) × 1 'mPa·s' = ?? mPa·s
```
