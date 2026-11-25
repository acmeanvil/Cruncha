+++
cruncha-template = 1
id = "fluids.buoyancy.immersed"
version = "1.0.0"
title = "Buoyancy of an immersed body"
summary = "Buoyant force on a body wholly immersed in a fluid (Archimedes, ρ_f g V), its weight, and its apparent weight in the fluid: the force needed to hold it up, or (when negative) down."
category = ["Fluids", "Hydrodynamics"]
tags = ["buoyancy", "Archimedes", "buoyant force", "apparent weight", "immersed", "submerged", "lift", "balloon", "airship", "Marks"]
from = "lacewing:fluids.buoyancy"
license = "GPL-3.0-or-later"
inputs = ["V", "ρ_b", "ρ_f"]
outputs = ["F_B", "W", "W_a"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "Sec. 3.3 (buoyancy: the net force (γ_f − γ_0)V on an immersed body, and the airship example)"

[[tests]]
name = "Marks Sec. 3.3: airship of 3 700 000 ft³ of hydrogen in air at 59 °F"
source = "Lacewing Lab fluids_test, after Marks 10th Sec. 3.3: air and hydrogen at 14.696 psia and 518.7 °R from p/RT (R = 53.34 and 766.8 ft·lbf/(lb·°R)) are 0.0764878 and 0.00532063 lb/ft³; the gross lift is 263 300 lbf"
inputs = { V = "3.7e6 ft^3", "ρ_b" = "0.00532063 lb/ft^3", "ρ_f" = "0.0764878 lb/ft^3" }
expect = { W_a = "-263300 lbf" }
tolerance = "0.05%"

[[tests]]
name = "Hand check: 10 L of steel in sea water"
source = "By hand: F_B = 1025 × 9.80665 × 0.01 = 100.51816 N, W = 7850 × 9.80665 × 0.01 = 769.82203 N, W_a = 669.30386 N"
inputs = { V = "0.01 m^3", "ρ_b" = "7850 kg/m^3", "ρ_f" = "1025 kg/m^3" }
expect = { F_B = "100.5181625 N", W = "769.822025 N", W_a = "669.3038625 N" }
tolerance = "1e-9"
+++

## Buoyancy of an immersed body

A body wholly immersed in a fluid is pushed up by the weight of the fluid it displaces (Archimedes; Marks Sec. 3.3). Its apparent weight, what a scale would read with the body hanging in the fluid, is its weight less that buoyancy: positive, it sinks and must be held up by that force; negative, it rises and must be held down. Standard gravity is used.

```calc {description="Volume of the body"}
V = 0.01 m³_
```

```calc {description="Mean density of the body (its mass over its volume; steel about 7850 kg/m³)"}
ρ_b = 7850 kg/m³_
```

```calc {description="Density of the fluid (sea water about 1025 kg/m³)"}
ρ_f = 1025 kg/m³_
```

```calc {description="Standard gravity"}
g_n = 9.80665 m/s²
```

```calc
> Buoyant force: the weight of fluid displaced
F_B = ρ_f g_n V = ?? N
> Weight of the body
W = ρ_b g_n V = ?? N
> Apparent weight in the fluid (negative: it floats up)
W_a = W - F_B = ?? N
```
