+++
cruncha-template = 1
id = "mechanical.lubricantViscosity.saeGrade"
version = "1.0.0"
title = "SAE oil viscosity at a temperature (Seireg-Dandage fit)"
summary = "The absolute viscosity of a straight SAE grade oil at a temperature by Seireg and Dandage's curve fit μ = μ0 exp[b/(T + 95)] (T in °F) as Shigley tabulates it, with the oil's density by Juvinall's relation and its kinematic viscosity."
category = ["Mechanical engineering", "Bearings"]
tags = ["viscosity", "absolute viscosity", "dynamic viscosity", "kinematic viscosity", "SAE grade", "lubricant", "oil", "Seireg", "Dandage", "reyn", "density", "journal bearing", "Shigley"]
from = "lacewing:mechanical.lubricantViscosity"
license = "GPL-3.0-or-later"
inputs = ["μ_0", "b", "SG", "T"]
outputs = ["T_F", "μ", "ρ", "ν"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 12-8, Table 12-1 (curve fits to the SAE grades' viscosity charts, after Seireg and Dandage); Figs. 12-12 and 12-13; Example 12-6"

[[references]]
work = "Seireg and Dandage, Empirical Design Procedure for the Thermodynamic Behavior of Journal Bearings, J. Lubrication Technology vol. 104"
edition = "1982"
locator = "pp. 135-148"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 13.5, Eq. 13.3 (kinematic viscosity), Eq. 13.6b (density of petroleum oils)"

[[tests]]
name = "Shigley Ex. 12-6: SAE 20 at 170 °F"
source = "Lacewing Lab lubricants_test, after Shigley 10th Example 12-6: μ = 0.0136 exp[1271.6/(170 + 95)] = 1.650 μreyn"
inputs = { "μ_0" = "0.0136e-6 lbf·s/in²", b = "1271.6", SG = "0.89", T = "170 °F" }
expect = { "μ" = "1.650e-6 lbf·s/in²" }
tolerance = "0.1%"

[[tests]]
name = "Shigley Ex. 12-6, second trial: SAE 20 at 168.5 °F"
source = "Lacewing Lab lubricants_test, after Shigley 10th Example 12-6: 1.693 μreyn at 168.5 °F (the fit gives 1.6957)"
inputs = { "μ_0" = "0.0136e-6 lbf·s/in²", b = "1271.6", SG = "0.89", T = "168.5 °F" }
expect = { "μ" = "1.693e-6 lbf·s/in²" }
tolerance = "0.2%"

[[tests]]
name = "Hand check: SAE 20 at 170 °F, density and kinematic viscosity"
source = "By hand: T = 76.6667 °C, μ = 1.6500492 μreyn = 11.376688 mPa·s, ρ = (0.89 − 0.00063 × (76.6667 − 15.6)) g/cm³ = 851.528 kg/m³, ν = 13.360322 mm²/s"
inputs = { "μ_0" = "0.0136e-6 lbf·s/in²", b = "1271.6", SG = "0.89", T = "170 °F" }
expect = { T_F = "170", "μ" = "11.3766885 mPa·s", "ρ" = "851.528 kg/m³", "ν" = "13.3603222 mm²/s" }
tolerance = "1e-6"
+++

## SAE oil viscosity at a temperature

Seireg and Dandage fitted the viscosity-temperature charts of the straight SAE grades (the charts Raimondi and Boyd's bearing analyses use) with μ = μ₀ exp[b/(T + 95)], T in degrees Fahrenheit. Shigley's Table 12-1 lists μ₀ and b for SAE 10 to 60; the defaults are SAE 20's. A real oil can sit anywhere in its grade's band, so the fit is a typical value; for a particular oil, fit its data sheet's two viscosities with the Walther (ASTM D341) or Roelands template. Multigrade oils are not covered.

The density is Juvinall's for petroleum oils, with the oil's own specific gravity at 15.6 °C (60 °F) in place of his typical 0.89.

```calc {description="Constant μ0 of the fit for the grade, Shigley Table 12-1"}
μ_0 = 0.0136e-6 lbf·s/in²_
```

```calc {description="Constant b of the fit for the grade, in °F, Shigley Table 12-1"}
b = 1271.6_
```

```calc {description="Specific gravity of the oil at 15.6 °C (60 °F)"}
SG = 0.89_
```

```calc {description="Oil temperature"}
T = 170 °F_
```

```calc
> The temperature as a number of °F, as the fit takes it
T_F = (T - 0 '°F') / (1 'Δ°F') = ??
> Absolute viscosity, Shigley Table 12-1
μ = μ_0 exp(b / (T_F + 95)) = ?? mPa·s
> Density, Juvinall Eq. 13.6b
ρ = (SG - 0.00063 (T - 15.6 '°C') / (1 'K')) × 1 'g/cm³' = ?? kg/m³
> Kinematic viscosity, Juvinall Eq. 13.3
ν = μ / ρ = ?? mm²/s
```
