+++
cruncha-template = 1
id = "mechanical.lubricantViscosity.saybolt"
version = "1.0.0"
title = "Oil viscosity from Saybolt seconds"
summary = "Kinematic viscosity from a Saybolt Universal viscometer reading, ν = 0.22t − 180/t cSt, the oil's density at the test temperature by Juvinall's relation, and its absolute viscosity."
category = ["Mechanical engineering", "Bearings"]
tags = ["viscosity", "Saybolt", "SUS", "Saybolt Universal seconds", "kinematic viscosity", "absolute viscosity", "density", "lubricant", "oil", "conversion", "Juvinall", "Shigley"]
from = "lacewing:mechanical.lubricantViscosity"
license = "GPL-3.0-or-later"
inputs = ["t", "T", "SG"]
outputs = ["ν", "ρ", "μ"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 13.5: Eqs. 13.3 (μ = ρν), 13.4 and 13.5 (Saybolt seconds to kinematic viscosity), 13.6b (density of petroleum oils); Sample Problem 13.1"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 12-2, Eq. 12-3"

[[tests]]
name = "Juvinall Sample Problem 13.1: 58 s at 100 °C"
source = "Lacewing Lab lubricants_test, after Juvinall & Marshek 5th Sample Problem 13.1: ρ 0.837 g/cm³, μ 8.08 mPa·s = 1.17 μreyn"
inputs = { t = "58 s", T = "100 °C", SG = "0.89" }
expect = { "ρ" = "0.837 g/cm³", "μ" = "8.08 mPa·s" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: Sample Problem 13.1 exactly"
source = "By hand: ν = 0.22 × 58 − 180/58 = 9.6565517 mm²/s, ρ = 0.89 − 0.00063 × 84.4 = 0.836828 g/cm³, μ = 8.0808729 mPa·s = 1.1720315e-6 lbf·s/in²"
inputs = { t = "58 s", T = "100 °C", SG = "0.89" }
expect = { "ν" = "9.65655172 mm²/s", "ρ" = "836.828 kg/m³", "μ" = "1.17203152e-6 lbf·s/in²" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Below the formula's range: 30 s"
source = "By hand: ν = 0.22 × 30 − 180/30 = 0.6 mm²/s, but the conversion holds only from about 32 s (it turns negative below 28.6 s)"
inputs = { t = "30 s", T = "40 °C", SG = "0.89" }
expect = { "ν" = "0.6 mm²/s" }
tolerance = "1e-9"
checks = "fail"
+++

## Oil viscosity from Saybolt seconds

A Saybolt Universal viscometer times 60 mL of oil running out through a standard orifice at a set temperature; the seconds (SUS) are an old but still common way to state viscosity. The empirical conversion to kinematic viscosity is ν = 0.22t − 180/t in mm²/s (cSt), from about 32 s up. The absolute viscosity a bearing analysis needs is the kinematic viscosity times the density at the same temperature, here Juvinall's relation for petroleum oils with the oil's own specific gravity at 15.6 °C (60 °F).

```calc {description="Saybolt Universal seconds"}
t = 58 s_
```

```calc {description="Temperature of the test"}
T = 100 °C_
```

```calc {description="Specific gravity of the oil at 15.6 °C (60 °F)"}
SG = 0.89_
```

```calc
> Kinematic viscosity, Juvinall Eq. 13.4
ν = (0.22 t / (1 's') - 180 (1 's') / t) × 1 'mm²/s' = ?? mm²/s
> Density at the test temperature, Juvinall Eq. 13.6b
ρ = (SG - 0.00063 (T - 15.6 '°C') / (1 'K')) × 1 'g/cm³' = ?? kg/m³
> Absolute viscosity, Juvinall Eq. 13.3
μ = ρ ν = ?? mPa·s
```

```calc
@test t ≥ 32 's'
```
