+++
cruncha-template = 1
id = "structural.wind.trussedTower"
version = "1.0.0"
title = "Wind on a trussed tower segment, ASCE 7-10"
summary = "Force coefficient of a square or triangular trussed tower by the formulas of Figure 29.5-3 (rounded members, wind on the diagonal) and the design wind force on one segment, Eq. 29.5-1."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "trussed tower", "lattice tower", "tower", "mast", "solidity", "Chapter 29", "Eq. 29.5-1", "Fig. 29.5-3", "Cf", "force coefficient"]
from = "lacewing:structural.windOther"
license = "GPL-3.0-or-later"
inputs = ["V", "exposure", "K_zt", "K_d", "G", "h", "z_b", "section", "members", "direction", "ε", "A_f"]
outputs = ["z", "K_z", "q_z", "C_(f,0)", "C_f", "F", "F_d"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 29.5 Eq. 29.5-1; Fig. 29.5-3 (Cf formulas, notes 1 to 6); Sec. 29.8 (minimum); Eq. 29.3-1 and Table 29.3-1 (qz, Kz); Table 26.6-1 (Kd)"

[[tests]]
name = "Square tower, flat members, ε = 0.2, wind normal to a face"
source = "ASCE 7-10 Fig. 29.5-3 (verified in the PDF): Cf = 4.0ε² − 5.9ε + 4.0 = 0.16 − 1.18 + 4.0 = 2.98, as Lacewing Lab asce7_wind_test"
inputs = { section = '"square"', members = '"flat"', direction = '"normal"', "ε" = "0.2" }
expect = { C_f = "2.98" }
tolerance = "1e-9"

[[tests]]
name = "Square tower on the diagonal: × (1 + 0.75ε)"
source = "Fig. 29.5-3 note 4 (verified in the PDF): 2.98 × 1.15 = 3.427, as Lacewing Lab asce7_wind_test"
inputs = { section = '"square"', members = '"flat"', direction = '"diagonal"', "ε" = "0.2" }
expect = { C_f = "3.427" }
tolerance = "1e-9"

[[tests]]
name = "Diagonal factor capped at 1.2"
source = "Fig. 29.5-3 note 4: at ε = 0.4, 1 + 0.3 = 1.3 is taken as 1.2; Cf = (0.64 − 2.36 + 4.0) × 1.2 = 2.28 × 1.2 = 2.736"
inputs = { section = '"square"', members = '"flat"', direction = '"diagonal"', "ε" = "0.4" }
expect = { "C_(f,0)" = "2.28", C_f = "2.736" }
tolerance = "1e-9"

[[tests]]
name = "Triangular tower of rounded members, ε = 0.3"
source = "Fig. 29.5-3 and note 3 (verified in the PDF): 3.4 × 0.09 − 4.7 × 0.3 + 3.4 = 2.296; × (0.51 × 0.09 + 0.57) = 0.6159: Cf = 1.41411"
inputs = { section = '"triangle"', members = '"rounded"', direction = '"normal"', "ε" = "0.3" }
expect = { C_f = "1.4141064" }
tolerance = "1e-7"

[[tests]]
name = "Segment force: 100 ft square tower, 115 mph, exposure C"
source = "By hand: qz at the segment's mid-height, 50 ft: Kz = 2.01 (50/900)^(2/9.5) = 1.09378, qz = 0.00256 × 1.09378 × 0.85 × 115² = 31.476 psf; Cf 2.98 (ε 0.2, flat, normal); Af 120 ft²: F = 31.476 × 0.85 × 2.98 × 120 = 9567.5 lb"
inputs = { V = "115 mph", exposure = '"C"', h = "100 ft", z_b = "0 ft", section = '"square"', members = '"flat"', direction = '"normal"', "ε" = "0.2", A_f = "120 ft^2" }
expect = { z = "50 ft", q_z = "31.4762 psf", F = "9567.5 lbf" }
tolerance = "1e-4"
checks = "pass"
+++

## Wind on a trussed tower segment (ASCE 7-10)

The design wind force on one segment of an open trussed (lattice) tower of square or triangular cross-section, F = q_z G C_f A_f (Eq. 29.5-1), with q_z at the segment's mid-height. Figure 29.5-3 gives C_f **by formula** in the solidity ratio ε of one face, so it is computed here, with the figure's two adjustments:
- **Rounded members** (note 3): C_f may be multiplied by 0.51ε² + 0.57, not more than 1.0.
- **Square towers with the wind on a diagonal** (note 4): C_f is multiplied by 1 + 0.75ε, not more than 1.2.

A_f is the solid area of one tower face, projected on the plane of that face, for the segment (note 1). Work up a tapering tower in segments. Wind forces on appurtenances (ladders, conduits, lights, antennas) and ice (Chapter 10) are additional (notes 5 and 6), and the wind must be applied in the directions that give the largest member forces (note 4).

K_d is 0.85 for triangular, square and rectangular trussed towers (Table 26.6-1). Towers that are flexible (below 1 Hz) need G_f; guyed masts and telecommunications towers are often designed to ANSI/TIA-222 instead.

### Site

```calc {description="Basic wind speed for the risk category, Fig. 26.5-1 (strength level)"}
V = 115 mph_
```

```calc {description="Exposure category B, C or D (in quotes)"}
exposure = "C"_
```

```calc {description="Topographic factor, Sec. 26.8 (1 on flat ground)"}
K_zt = 1_
```

```calc {description="Wind directionality factor, Table 26.6-1 (0.85 for square or triangular towers)"}
K_d = 0.85_
```

```calc {description="Gust-effect factor, Sec. 26.9 (0.85 for a rigid structure)"}
G = 0.85_
```

### Tower segment

```calc {description="Height of the top of the segment above ground"}
h = 100 ft_
```

```calc {description="Height of the base of the segment above ground (0 for the whole tower)"}
z_b = 0 ft_
```

```calc {description="Cross-section, square or triangle (in quotes)"}
section = "square"_
```

```calc {description="Members, flat (angles and similar) or rounded (in quotes)"}
members = "flat"_
```

```calc {description="Wind direction on a square tower, normal to a face or diagonal (in quotes)"}
direction = "normal"_
```

```calc {description="Solidity ratio of one face: solid area over gross area of the face, for the segment"}
ε = 0.2_
```

```calc {description="Solid area of one tower face for the segment, projected on the plane of the face"}
A_f = 120 ft²_
```

```calc
@test 0 < ε ≤ 1
```

### Force coefficient (Figure 29.5-3)

```calc {description="Force coefficient for flat-sided members, wind normal to a face"}
C_(f,0) = {4.0 ε^2 - 5.9 ε + 4.0 if section == "square"; 3.4 ε^2 - 4.7 ε + 3.4 otherwise} = ??
```

```calc
> Factor for rounded members, note 3
k_r = {min(0.51 ε^2 + 0.57, 1.0) if members == "rounded"; 1 otherwise} = ??
> Factor for wind on the diagonal of a square tower, note 4
k_d = {min(1 + 0.75 ε, 1.2) if section == "square" and direction == "diagonal"; 1 otherwise} = ??
```

```calc {description="Force coefficient of the tower segment"}
C_f = C_(f,0) k_r k_d = ??
```

### Force

```calc {description="Terrain exposure constants α and zg, ASCE 7-10 Table 26.9-1"}
exposures = ``#exposure	α	z_g
		ft
B	7.0	1200
C	9.5	900
D	11.5	700`` = !
```

```calc
> 3-s gust power-law exponent
α = exposures[exposure, "α"] = ??
> Nominal gradient height
z_g = exposures[exposure, "z_g"] = ?? ft
```

```calc {description="Mid-height of the segment, where qz is taken"}
z = (z_b + h) / 2 = ?? ft
```

```calc {description="Velocity pressure exposure coefficient at z, Table 29.3-1"}
K_z = 2.01 (min(max(z, 15 'ft'), z_g) / z_g)^(2 / α) = ??
```

```calc {description="Velocity pressure at z, Eq. 29.3-1"}
q_z = 0.00256 'psf/mph^2' K_z K_zt K_d V^2 = ?? psf
```

```calc {description="Design wind force on the segment, Eq. 29.5-1"}
F = q_z G C_f A_f = ?? lbf
```

```calc {description="Design force with the 16 psf minimum on Af, Sec. 29.8"}
F_d = max(F, 16 'psf' A_f) = ?? lbf
```
