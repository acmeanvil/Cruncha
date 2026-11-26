+++
cruncha-template = 1
id = "structural.wind.velocityPressure"
version = "1.0.0"
title = "Wind velocity pressure qz, ASCE 7-10"
summary = "Velocity pressure at a height above ground from the basic wind speed, the exposure coefficient Kz (Table 27.3-1 formula), the topographic factor Kzt and the directionality factor Kd."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "velocity pressure", "qz", "qh", "Kz", "Kd", "Kzt", "exposure", "Eq. 27.3-1"]
from = "lacewing:structural.windVelocity"
license = "GPL-3.0-or-later"
inputs = ["V", "z", "exposure", "K_zt", "K_d"]
outputs = ["α", "z_g", "K_z", "q_z"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Eq. 27.3-1; Table 27.3-1 note 1 (Kz formula); Table 26.9-1 (α, zg); Table 26.6-1 (Kd); Sec. 26.8 (Kzt); Fig. 26.5-1 (V)"

[[tests]]
name = "115 mph, exposure C, 30 ft, building MWFRS"
source = "Lacewing Lab asce7_wind_test (28.2 psf with the printed Kz 0.98). By hand with the formula: Kz = 2.01 (30/900)^(2/9.5) = 0.98225; qz = 0.00256 × 0.98225 × 1 × 0.85 × 115² = 28.267 psf"
inputs = { V = "115 mph", z = "30 ft", exposure = '"C"', K_zt = "1", K_d = "0.85" }
expect = { K_z = "0.98225", q_z = "28.267 psf" }
tolerance = "0.01%"

[[tests]]
name = "Kz against the printed table: D at 60 ft"
source = "ASCE 7-10 Table 28.3-1 (same formula as Table 27.3-1 for these heights): D 60 ft 1.31 (verified in the PDF); the formula gives 1.3111"
inputs = { z = "60 ft", exposure = '"D"' }
expect = { K_z = "1.31" }
tolerance = "0.4%"

[[tests]]
name = "Exposure D below 15 ft: Kz held at its 15 ft value"
source = "ASCE 7-10 Table 28.3-1, D 0-15 ft: 1.03 (verified in the PDF); by hand 2.01 (15/700)^(2/11.5) = 1.0302"
inputs = { z = "8 ft", exposure = '"D"' }
expect = { K_z = "1.03" }
tolerance = "0.1%"

[[tests]]
name = "Exposure B at 100 ft"
source = "ASCE 7-10 Table 27.3-1, B 100 ft: 0.99 (as quoted in Lacewing Lab asce7_wind_test; the table is an image in this PDF, not re-read); by hand 2.01 (100/1200)^(2/7) = 0.98823"
inputs = { z = "100 ft", exposure = '"B"' }
expect = { K_z = "0.98823" }
tolerance = "1e-4"

[[tests]]
name = "SI: 50 m/s at 12 m, exposure C, square chimney (Kd 0.90)"
source = "By hand: Kz = 2.01 (39.37/900)^(2/9.5) = 1.04012; qz = 0.613 × 1.04012 × 0.90 × 50² = 1434.6 Pa with the SI coefficient (0.00256 psf/mph² is 0.61327 Pa/(m/s)², 0.05 % more: 1435.4 Pa)"
inputs = { V = "50 m/s", z = "12 m", exposure = '"C"', K_d = "0.90" }
expect = { K_z = "1.04012", q_z = "1435 Pa" }
tolerance = "0.1%"
+++

## Wind velocity pressure (ASCE 7-10)

The velocity pressure q_z at a height z above ground, Eq. 27.3-1. It is where every ASCE 7 wind pressure and force starts: the design pressures multiply it by a gust-effect factor and a pressure or force coefficient (the other wind templates).

The exposure coefficient K_z is computed by the power-law formula of Table 27.3-1 note 1, held at its 15 ft value below 15 ft and at the gradient height above it. The printed table rounds: it lists exposure C at 60 ft as 1.13 where the formula gives 1.137. Components and cladding take K_z at no less than 30 ft in exposure B (Table 30.3-1); that template does it.

V is already at strength level (ASCE 7-10 maps): LRFD combinations take 1.0W, ASD 0.6W. ASCE 7-16 and later editions changed the maps and moved ground elevation into a factor K_e: use the edition your jurisdiction adopts.

### Site and structure

```calc {description="Basic wind speed, 3-s gust at 33 ft in exposure C, Fig. 26.5-1 for the risk category"}
V = 115 mph_
```

```calc {description="Height above ground where q is wanted"}
z = 30 ft_
```

Exposure (Sec. 26.7): B urban, suburban or wooded; C open terrain with scattered obstructions; D flat, unobstructed areas and water surfaces.

```calc {description="Exposure category B, C or D (in quotes)"}
exposure = "C"_
```

The topographic factor is 1 on flat ground or where the conditions of Sec. 26.8.1 aren't all met; the topographic factor template computes it for a hill, ridge or escarpment.

```calc {description="Topographic factor, Sec. 26.8 (1 on flat ground)"}
K_zt = 1_
```

Directionality factor K_d (Table 26.6-1): 0.85 for buildings (MWFRS and components), arched roofs, solid signs and walls, open signs and lattice frames, and triangular, square or rectangular trussed towers; 0.90 for square chimneys and tanks; 0.95 for hexagonal and round ones and for trussed towers of other sections.

```calc {description="Wind directionality factor, Table 26.6-1"}
K_d = 0.85_
```

### Exposure constants (Table 26.9-1)

```calc {description="Terrain exposure constants α and zg, ASCE 7-10 Table 26.9-1"}
exposures = ``#exposure	α	z_g
		ft
B	7.0	1200
C	9.5	900
D	11.5	700`` = !
```

```calc {description="3-s gust power-law exponent"}
α = exposures[exposure, "α"] = ??
```

```calc {description="Nominal gradient height"}
z_g = exposures[exposure, "z_g"] = ?? ft
```

### Velocity pressure

```calc {description="Velocity pressure exposure coefficient, Table 27.3-1 note 1"}
K_z = 2.01 (min(max(z, 15 'ft'), z_g) / z_g)^(2 / α) = ??
```

The coefficient 0.00256 is ½ρ of standard air in psf per mph² (0.613 Pa per (m/s)² in SI); it works in either system here.

```calc {description="Velocity pressure at z, Eq. 27.3-1"}
q_z = 0.00256 'psf/mph^2' K_z K_zt K_d V^2 = ?? psf
```
