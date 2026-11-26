+++
cruncha-template = 1
id = "structural.wind.velocityPressure"
version = "1.1.0"
title = "Wind velocity pressure qz and qh, ASCE 7-10"
summary = "Velocity pressures qz at a height z and qh at the mean roof height h, from the basic wind speed, the exposure coefficient (Table 27.3-1 formula), the topographic factor Kzt and the directionality factor Kd. The other wind templates take qz or qh from here."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "velocity pressure", "qz", "qh", "Kz", "Kd", "Kzt", "exposure", "Eq. 27.3-1"]
from = "lacewing:structural.windVelocity"
license = "GPL-3.0-or-later"
inputs = ["V", "z", "h", "α", "z_g", "K_zt", "K_d"]
outputs = ["K_z", "q_z", "K_h", "q_h"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Eq. 27.3-1; Table 27.3-1 note 1 (Kz formula); Table 26.9-1 (α, zg); Table 26.6-1 (Kd); Sec. 26.8 (Kzt); Fig. 26.5-1 (V)"

[[tests]]
name = "115 mph, exposure C, 30 ft, building MWFRS"
source = "Lacewing Lab asce7_wind_test (28.2 psf with the printed Kz 0.98). By hand with the formula: Kz = 2.01 (30/900)^(2/9.5) = 0.98225; qz = 0.00256 × 0.98225 × 1 × 0.85 × 115² = 28.267 psf"
inputs = { V = "115 mph", z = "30 ft", h = "30 ft", "α" = "9.5", z_g = "900 ft", K_zt = "1", K_d = "0.85" }
expect = { K_z = "0.98225", q_z = "28.267 psf", K_h = "0.98225", q_h = "28.267 psf" }
tolerance = "0.01%"

[[tests]]
name = "Kz against the printed table: D at 60 ft"
source = "ASCE 7-10 Table 28.3-1 (same formula as Table 27.3-1 for these heights): D 60 ft 1.31 (verified in the PDF); the formula gives 1.3111"
inputs = { z = "60 ft", "α" = "11.5", z_g = "700 ft" }
expect = { K_z = "1.31" }
tolerance = "0.4%"

[[tests]]
name = "Exposure D below 15 ft: Kz held at its 15 ft value"
source = "ASCE 7-10 Table 28.3-1, D 0-15 ft: 1.03 (verified in the PDF); by hand 2.01 (15/700)^(2/11.5) = 1.0302"
inputs = { z = "8 ft", "α" = "11.5", z_g = "700 ft" }
expect = { K_z = "1.03" }
tolerance = "0.1%"

[[tests]]
name = "Exposure B at 100 ft"
source = "ASCE 7-10 Table 27.3-1, B 100 ft: 0.99 (as quoted in Lacewing Lab asce7_wind_test; the table is an image in this PDF, not re-read); by hand 2.01 (100/1200)^(2/7) = 0.98823"
inputs = { z = "100 ft", "α" = "7.0", z_g = "1200 ft" }
expect = { K_z = "0.98823" }
tolerance = "1e-4"

[[tests]]
name = "SI: 50 m/s at 12 m, exposure C, square chimney (Kd 0.90)"
source = "By hand: Kz = 2.01 (39.37/900)^(2/9.5) = 1.04012; qz = 0.613 × 1.04012 × 0.90 × 50² = 1434.6 Pa with the SI coefficient (0.00256 psf/mph² is 0.61327 Pa/(m/s)², 0.05 % more: 1435.4 Pa)"
inputs = { V = "50 m/s", z = "12 m", "α" = "9.5", z_g = "900 ft", K_d = "0.90" }
expect = { K_z = "1.04012", q_z = "1435 Pa" }
tolerance = "0.1%"

[[tests]]
name = "60 ft building in exposure B at 130 mph: qh at h and qz at 40 ft"
source = "By hand: Kh = 2.01 (60/1200)^(2/7) = 0.85403, qh = 0.00256 × 0.85403 × 0.85 × 130² = 31.406 psf; Kz at 40 ft = 0.76061, qz = 27.971 psf (the inputs of the MWFRS template's third worked example)"
inputs = { V = "130 mph", z = "40 ft", h = "60 ft", "α" = "7.0", z_g = "1200 ft" }
expect = { K_h = "0.85403", K_z = "0.76061", q_h = "31.4064 psf", q_z = "27.9709 psf" }
tolerance = "1e-4"

[[tests]]
name = "Wind Design Manual: tornado shelter, 250 mph, exposure C, h 12 ft, Kd 1.0"
source = "Wind Design Manual Based on the 2018 IBC and ASCE/SEI 7-16, Design Example 5, Table 5-3 (verified in the PDF; Ke = 1, Kd = 1.0 by ICC 500): Kz 0.85, q 136.0 psf. The formula's Kz 0.84888 gives 135.82 psf, 0.13 % less"
inputs = { V = "250 mph", z = "12 ft", h = "12 ft", "α" = "9.5", z_g = "900 ft", K_d = "1.0" }
expect = { K_h = "0.85", q_h = "136.0 psf" }
tolerance = "0.2%"
+++

## Wind velocity pressure (ASCE 7-10)

The velocity pressures q_z at a height z above ground and q_h at the mean roof height h, Eq. 27.3-1. They are where every ASCE 7 wind pressure and force starts: the other wind templates take q_z or q_h as an input and multiply it by a gust-effect factor and a pressure or force coefficient. Insert this one first and bind their q_z and q_h to it; where a template needs q at another height (a sign's top, a chimney's centroid, a tower segment's mid-height), set z to that height or insert this template again with its names renamed.

The exposure coefficients K_z and K_h are computed by the power-law formula of Table 27.3-1 note 1, held at their 15 ft value below 15 ft and at the gradient height above it. The printed table rounds: it lists exposure C at 60 ft as 1.13 where the formula gives 1.137. Components and cladding take q_h at no less than 30 ft in exposure B (Table 30.3-1): for them, set h to at least 30 ft.

V is already at strength level (ASCE 7-10 maps): LRFD combinations take 1.0W, ASD 0.6W. ASCE 7-16 and later editions changed the maps and moved ground elevation into a factor K_e: use the edition your jurisdiction adopts.

### Site and structure

```calc {description="Basic wind speed, 3-s gust at 33 ft in exposure C, Fig. 26.5-1 for the risk category"}
V = 115 mph_
```

```calc {description="Height above ground where qz is wanted"}
z = 30 ft_
```

```calc {description="Mean roof height, where qh is wanted (the top of a sign or structure where a template asks for qh there)"}
h = 30 ft_
```

The exposure category (Sec. 26.7: B urban, suburban or wooded; C open terrain with scattered obstructions; D flat, unobstructed areas and water surfaces) sets the two terrain constants of Table 26.9-1.

```calc {description="3-s gust power-law exponent α, ASCE 7-10 Table 26.9-1 for the exposure (9.5 is for exposure C)"}
α = 9.5_
```

```calc {description="Nominal gradient height zg, ASCE 7-10 Table 26.9-1 for the exposure (900 ft is for exposure C)"}
z_g = 900 ft_
```

The topographic factor is 1 on flat ground or where the conditions of Sec. 26.8.1 aren't all met; the topographic factor template computes it for a hill, ridge or escarpment.

```calc {description="Topographic factor, Sec. 26.8 (1 on flat ground; from structural.wind.topographicFactor)"}
K_zt = 1_
```

```calc {description="Wind directionality factor for the structure type, ASCE 7-10 Table 26.6-1 (0.85 is for buildings)"}
K_d = 0.85_
```

### Velocity pressures

```calc {description="Velocity pressure exposure coefficient at z, Table 27.3-1 note 1"}
K_z = 2.01 (min(max(z, 15 'ft'), z_g) / z_g)^(2 / α) = ??
```

```calc {description="Velocity pressure exposure coefficient at h, Table 27.3-1 note 1"}
K_h = 2.01 (min(max(h, 15 'ft'), z_g) / z_g)^(2 / α) = ??
```

The coefficient 0.00256 is ½ρ of standard air in psf per mph² (0.613 Pa per (m/s)² in SI); it works in either system here.

```calc {description="Velocity pressure at z, Eq. 27.3-1"}
q_z = 0.00256 'psf/mph^2' K_z K_zt K_d V^2 = ?? psf
```

```calc {description="Velocity pressure at h, Eq. 27.3-1"}
q_h = 0.00256 'psf/mph^2' K_h K_zt K_d V^2 = ?? psf
```
