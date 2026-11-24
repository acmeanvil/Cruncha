+++
cruncha-template = 1
id = "chemistry.combustion.stoichiometricAir"
version = "1.0.0"
title = "Combustion: stoichiometric and actual air for a fuel CₓHᵧO_z"
summary = "Oxygen and air for complete combustion of a fuel CₓHᵧO_z, the air–fuel ratio by mass at a given excess air, the products per mole of fuel and the CO₂ in the dry flue gas."
category = ["Chemistry", "Combustion"]
tags = ["combustion", "stoichiometry", "air-fuel ratio", "AFR", "excess air", "equivalence ratio", "flue gas", "CO2", "methane", "propane"]
license = "GPL-3.0-or-later"
inputs = ["x", "y", "z", "e"]
outputs = ["a_s", "AFR_s", "AFR", "ϕ", "n_CO2", "n_H2O", "n_O2", "n_N2", "X_(CO2,dry)"]

[[references]]
work = "Turns, S.R., An Introduction to Combustion: Concepts and Applications"
edition = "3rd"
locator = "Sec. 2.3, Stoichiometry (stoichiometric air for CₓHᵧ, air–fuel ratio, equivalence ratio)"

[[references]]
work = "IUPAC Commission on Isotopic Abundances and Atomic Weights, Standard atomic weights (conventional values)"
edition = "2021"
url = "https://iupac.qmul.ac.uk/AtWt/"

[[references]]
work = "ASHRAE Handbook — Fundamentals"
edition = "2017"
locator = "Ch. 1, Table 1 (composition of dry air: O₂ mole fraction 0.20946, molar mass 28.966)"

[[tests]]
name = "Methane, stoichiometric"
source = "By hand: CH₄ + 2(O₂ + 3.774 N₂); A/F = 2/0.20946 × 28.965/16.043 = 17.24 (commonly quoted as 17.2); dry CO₂ 11.7 %"
inputs = { x = "1", y = "4", z = "0", e = "0" }
expect = { a_s = "2", AFR_s = "17.2392", "X_(CO2,dry)" = "0.116981" }
tolerance = "1e-5"

[[tests]]
name = "Propane, stoichiometric"
source = "By hand: C₃H₈ needs 5 mol O₂; A/F 15.68, commonly quoted as 15.7"
inputs = { x = "3", y = "8", z = "0", e = "0" }
expect = { a_s = "5", AFR_s = "15.6795" }
tolerance = "1e-5"

[[tests]]
name = "Octane, stoichiometric"
source = "By hand: C₈H₁₈ needs 12.5 mol O₂; A/F 15.13, commonly quoted as 15.1"
inputs = { x = "8", y = "18", z = "0", e = "0" }
expect = { a_s = "12.5", AFR_s = "15.1319" }
tolerance = "1e-5"

[[tests]]
name = "Methanol: the fuel's own oxygen"
source = "By hand: CH₃OH needs 1 + 4/4 − 1/2 = 1.5 mol O₂; A/F 6.47"
inputs = { x = "1", y = "4", z = "1", e = "0" }
expect = { a_s = "1.5", AFR_s = "6.47357" }
tolerance = "1e-5"

[[tests]]
name = "Methane with 20 % excess air"
source = "By hand: A/F = 1.2 × 17.239 = 20.687, ϕ = 1/1.2; dry CO₂ 1/(1 + 0.4 + 9.077) = 9.56 %"
inputs = { x = "1", y = "4", z = "0", e = "0.2" }
expect = { AFR = "20.6870", "ϕ" = "0.833333", n_O2 = "0.4", "X_(CO2,dry)" = "0.0956203" }
tolerance = "1e-5"
+++

## Combustion: stoichiometric and actual air

Complete combustion of a fuel CₓHᵧO_z in dry air: all carbon to CO₂, all hydrogen to H₂O, with no CO, dissociation or NOₓ. Sulfur, nitrogen and ash in the fuel are left out. Air is taken as O₂ and N₂ only, with argon and the other minor gases counted as nitrogen.

### Fuel and air

Give the fuel as atoms per molecule: methane CH₄ is x = 1, y = 4, z = 0; methanol CH₃OH is x = 1, y = 4, z = 1. A mixture can be given per mole of mixture, with fractional x, y and z.

```calc {description="Carbon atoms per molecule of fuel"}
x = 1_
```

```calc {description="Hydrogen atoms per molecule of fuel"}
y = 4_
```

```calc {description="Oxygen atoms per molecule of fuel"}
z = 0_
```

```calc {description="Excess air, a fraction of the stoichiometric air (0.2 for 20 %)"}
e = 0_
```

```calc
@test e ≥ 0
```

```calc {description="Mole fraction of O₂ in dry air"}
X_O2 = 0.20946
```

```calc {description="Molar mass of dry air"}
M_air = 28.965 g/mol
```

### Stoichiometric air

```calc
> Moles of O₂ per mole of fuel for complete combustion
a_s = x + y / 4 - z / 2 = ??
> Molar mass of the fuel, from the atomic weights of C, H and O
M_f = (12.011 x + 1.008 y + 15.999 z) 'g/mol' = ?? g/mol
> Stoichiometric air–fuel ratio by mass
AFR_s = a_s / X_O2 M_air / M_f = ??
```

### Actual air and products

```calc
> Actual air–fuel ratio by mass
AFR = (1 + e) AFR_s = ??
> Equivalence ratio (below 1: lean)
ϕ = AFR_s / AFR = ??
```

```calc
> CO₂ per mole of fuel
n_CO2 = x = ??
> H₂O per mole of fuel
n_H2O = y / 2 = ??
> Excess O₂ per mole of fuel
n_O2 = e a_s = ??
> N₂ per mole of fuel
n_N2 = (1 + e) a_s (1 - X_O2) / X_O2 = ??
> CO₂ mole fraction in the dry flue gas
X_(CO2,dry) = n_CO2 / (n_CO2 + n_O2 + n_N2) = ??
```
