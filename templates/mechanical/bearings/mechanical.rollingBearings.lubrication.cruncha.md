+++
cruncha-template = 1
id = "mechanical.rollingBearings.lubrication"
version = "1.0.0"
title = "Rolling bearing speed, lubrication and friction (ISO 281)"
summary = "The running conditions of a rolling bearing: the mean diameter and the speed factor n·dm, ISO 281's reference viscosity ν1 for the speed and size, the viscosity ratio κ = ν/ν1, the speed against the catalogue's reference and limiting speeds, and the friction torque and power loss by a constant friction coefficient."
category = ["Mechanical engineering", "Bearings"]
tags = ["rolling bearing", "lubrication", "viscosity ratio", "kappa", "reference viscosity", "ISO 281", "n·dm", "speed factor", "limiting speed", "friction torque", "power loss", "SKF", "JTEKT"]
from = "lacewing:mechanical.rollingBearings"
license = "GPL-3.0-or-later"
inputs = ["d", "D", "n", "ν", "n_ref", "n_lim", "P", "μ"]
outputs = ["d_m", "n_d", "ν_1", "κ", "r_ref", "r_lim", "M", "P_loss"]

[[references]]
work = "ISO 281 Rolling bearings — Dynamic load ratings and rating life"
edition = "ISO 281:2007"
locator = "Reference viscosity ν1 and viscosity ratio κ (ν1 as the NSK technical note on ISO 281:2007 gives it: 45 000·n^−0.83·dm^−0.5 below 1000 r/min, 4500·n^−0.5·dm^−0.5 from 1000)"

[[references]]
work = "SKF Rolling bearings (PUB BU/P1 17000/1 EN)"
edition = "2018"
locator = "Viscosity ratio κ (pp. 101-103), speed factor n·dm (p. 102), friction power loss (p. 132), reference and limiting speeds (p. 135)"

[[references]]
work = "JTEKT (Koyo) Basic Bearing Knowledge"
edition = "Catalogue technical section"
locator = "Friction torque M = μ·P·d/2, Eq. 8-2; reference friction coefficients μ, Table 8-1"

[[tests]]
name = "6210 deep groove ball bearing, 5 kN, 1500 rev/min, oil at 20 mm²/s"
source = "Lacewing Lab rolling_bearings_test ('the whole check'): d = 50 mm, D = 90 mm, dm = 70 mm, n·dm = 105 000 mm/min, ν1 = 4500/√(1500 × 70) = 13.887301 mm²/s, κ = 1.4401645; catalogue speeds 15 000 and 10 000 r/min: n/nlim = 0.15; μ = 0.00125 (middle of JTEKT's 0.0010-0.0015 for deep groove bearings): M = 0.00125 × 5000 × 0.025 = 0.15625 N·m and SKF's 1.05e-4 × 156.25 × 1500 = 24.6 W (M·ω exactly 24.543693 W)"
inputs = { d = "50 mm", D = "90 mm", n = "1500 rpm", "ν" = "20 mm²/s", n_ref = "15000 rpm", n_lim = "10000 rpm", P = "5 kN", "μ" = "0.00125" }
expect = { d_m = "70 mm", n_d = "105000 mm/min", "ν_1" = "13.887301 mm²/s", "κ" = "1.4401645", r_lim = "0.15", r_ref = "0.1", M = "0.15625 N·m", P_loss = "24.543693 W" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: slow running, 300 rev/min, oil at 10 mm²/s"
source = "By hand: d = 50 mm, D = 90 mm, n = 300 r/min: ν1 = 45 000 × 300^−0.83 × 70^−0.5 = 47.277320 mm²/s, κ = 0.21151791, under 1"
inputs = { d = "50 mm", D = "90 mm", n = "300 rpm", "ν" = "10 mm²/s", n_ref = "15000 rpm", n_lim = "10000 rpm", P = "5 kN", "μ" = "0.00125" }
expect = { "ν_1" = "47.277320 mm²/s", "κ" = "0.21151791" }
tolerance = "1e-6"
checks = "fail"
+++

## Rolling bearing speed, lubrication and friction

The running checks that go with a rolling bearing's life calculation. They describe the oil film and the heat, not fatigue; ISO 281's life factor a_ISO is read from its diagrams at the κ found here.

### Bearing and speed

```calc {description="Bearing bore"}
d = 50 mm_
```

```calc {description="Bearing outside diameter"}
D = 90 mm_
```

```calc {description="Speed"}
n = 1500 rpm_
```

Cruncha's rpm is an angular speed (2π radians a revolution), so the revolutions are counted as n/(2π).

```calc
> Revolutions per unit time
n_r = n / (2 π) = ?? 1/min
> Mean (pitch) diameter
d_m = (d + D) / 2 = ?? mm
> Speed factor n·dm
n_d = n_r d_m = ?? mm/min
```

Below about 10 000 mm/min the speed is too low to build a full film without a high viscosity; above about 500 000 (400 000 for d_m over 200 mm) the operating temperature needs watching (SKF p. 102).

### Lubrication

ν is the kinematic viscosity of the oil, or of a grease's base oil, at the operating temperature, from its ISO VG grade and a viscosity-temperature chart. ISO 281's reference viscosity ν_1 is the viscosity that gives adequate lubrication at the speed and size. κ from 1 to 4 is the usual design range; below 1 use oils with EP and anti-wear additives, and below 0.1 the life model does not apply. Above 4 adds friction but no life (SKF pp. 101-102).

```calc {description="Kinematic viscosity of the oil at the operating temperature"}
ν = 20 mm²/s_
```

```calc
> Reference viscosity, ISO 281 (n in r/min, dm in mm)
ν_1 = {45000 (n_r · 1 'min')^(-0.83) (d_m / 1 'mm')^(-0.5) if n_r < 1000 '1/min'; 4500 (n_r · 1 'min')^(-0.5) (d_m / 1 'mm')^(-0.5) otherwise} × 1 'mm²/s' = ?? mm²/s
> Viscosity ratio
κ = ν / ν_1 = ??
```

### Speed limits

The catalogue's reference speed is a thermal limit under the standard conditions of ISO 15312; its limiting speed is the mechanical limit of the cage and rolling elements, which the speed should not pass.

```calc {description="Reference (thermal) speed, from the catalogue"}
n_ref = 15000 rpm_
```

```calc {description="Limiting (mechanical) speed, from the catalogue"}
n_lim = 10000 rpm_
```

```calc
> Speed over the reference speed
r_ref = n / n_ref = ??
> Speed over the limiting speed
r_lim = n / n_lim = ??
```

### Friction

With good lubrication and a load of about 0.1C, the friction torque is roughly μ P d/2 (JTEKT Eq. 8-2), μ a constant for the bearing type from JTEKT Table 8-1 or the maker (for deep groove ball bearings 0.0010 to 0.0015). Seals, churning of the oil and preload add to it.

```calc {description="Equivalent dynamic load on the bearing"}
P = 5 kN_
```

```calc {description="Friction coefficient of the bearing type, JTEKT Table 8-1"}
μ = 0.00125_
```

```calc
> Friction torque, JTEKT Eq. 8-2
M = μ P d / 2 = ?? N·m
> Power lost to friction
P_loss = M n = ?? W
```

### Checks

```calc
@test 1 ≤ κ ≤ 4
```

```calc
@test n ≤ n_lim
```
