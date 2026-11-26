+++
cruncha-template = 1
id = "structural.wind.gustRigid"
version = "1.0.0"
title = "Gust-effect factor G of a rigid structure, ASCE 7-10"
summary = "Gust-effect factor of a rigid building or other structure by Eq. 26.9-6, from the turbulence intensity, the integral length scale and the background response at the equivalent height 0.6h."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "gust-effect factor", "gust factor", "G", "rigid", "turbulence intensity", "Iz", "Lz", "background response", "Eq. 26.9-6"]
from = "lacewing:structural.windBuilding"
license = "GPL-3.0-or-later"
inputs = ["exposure", "h", "B"]
outputs = ["z̄", "I_z", "L_z", "Q", "G"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 26.9.1 (G = 0.85 permitted), Sec. 26.9.4 Eqs. 26.9-6 to 26.9-9; Table 26.9-1 (c, ℓ, ε̄, zmin)"

[[references]]
work = "ASCE/SEI 7-10 Commentary"
edition = "2010"
locator = "Sec. C26.9, example: Tables C26.9-1 and C26.9-2 (600 ft building, exposure B)"

[[references]]
work = "Wind Design Manual Based on the 2018 IBC and ASCE/SEI 7-16 (ICC)"
locator = "Design Example 4, gust factor of a rigid building (five-story variant)"

[[tests]]
name = "Wind Design Manual example: 60 ft building, B = 200 ft, exposure B"
source = "Wind Design Manual Based on the 2018 IBC and ASCE/SEI 7-16 (ICC), Design Example 4, five-story variant (verified in the PDF; ASCE 7-16 Eqs. 26.11-6 to 26.11-9 are 7-10's Eqs. 26.9-6 to 26.9-9): z̄ 36 ft, Iz̄ 0.2957, Lz̄ 329 ft, Q 0.8050. The manual's G = 0.85 uses 0.7 in place of the standard's 1.7 (it gives 0.8505 with 0.7); by hand with 1.7, G = 0.81127"
inputs = { exposure = '"B"', h = "60 ft", B = "200 ft" }
expect = { "z̄" = "36 ft", I_z = "0.2957", L_z = "329 ft", Q = "0.8050", G = "0.81127" }
tolerance = "0.2%"

[[tests]]
name = "Commentary example: 600 × 100 ft building, exposure B"
source = "ASCE 7-10 Commentary Table C26.9-2 (verified in the PDF): z̄ 360 ft, Iz̄ 0.201, Lz̄ 709.71 ft, Q² 0.616 (Q = 0.785). That building is flexible (n1 = 0.2 Hz, Gf = 1.062); G here is the rigid formula on the same Iz̄ and Q, by hand 0.925 (1 + 5.78 × 0.20145 × 0.78457)/(1 + 5.78 × 0.20145) = 0.81780"
inputs = { exposure = '"B"', h = "600 ft", B = "100 ft" }
expect = { "z̄" = "360 ft", I_z = "0.201", L_z = "709.71 ft", Q = "0.785" }
tolerance = "0.3%"

[[tests]]
name = "Commentary example: G by the rigid formula"
source = "By hand from Eq. 26.9-6 with the example's values: G = 0.81780"
inputs = { exposure = '"B"', h = "600 ft", B = "100 ft" }
expect = { G = "0.81780" }
tolerance = "1e-4"

[[tests]]
name = "Low building, exposure C: z̄ is 0.6h above zmin"
source = "Lacewing Lab asce7_wind_test (h 10 m, B 30 m, exposure C: G between 0.80 and 0.90). By hand: z̄ = 19.685 ft, Iz̄ = 0.2 (33/19.685)^(1/6) = 0.21798, Lz̄ = 500 (19.685/33)^(1/5) = 450.91 ft, Q = 0.88063, G = 0.86344"
inputs = { exposure = '"C"', h = "10 m", B = "30 m" }
expect = { I_z = "0.21798", L_z = "450.91 ft", Q = "0.88063", G = "0.86344" }
tolerance = "1e-4"

[[tests]]
name = "Short building in exposure B: z̄ held at zmin = 30 ft"
source = "By hand: 0.6 × 20 = 12 ft < 30 ft, so z̄ = 30 ft; Iz̄ = 0.3 (33/30)^(1/6) = 0.30480"
inputs = { exposure = '"B"', h = "20 ft", B = "80 ft" }
expect = { "z̄" = "30 ft", I_z = "0.30480" }
tolerance = "1e-4"
+++

## Gust-effect factor G of a rigid structure (ASCE 7-10)

The gust-effect factor of a **rigid** building or other structure (fundamental frequency of at least 1 Hz; low-rise buildings may be taken as rigid, Sec. 26.9.2), by Eq. 26.9-6. Sec. 26.9.1 permits 0.85 instead; the computed value is often a little below it, and may be used in its place.

Flexible structures (below 1 Hz) need G_f of Eq. 26.9-10 with the resonant response, which this template doesn't compute. Where ASCE 7 gives combined coefficients such as GC_p or GC_pi (components and cladding, low-rise envelope procedure), G is not determined separately (Sec. 26.9.7).

### Structure

```calc {description="Exposure category B, C or D (in quotes)"}
exposure = "C"_
```

```calc {description="Mean roof height (height of the structure)"}
h = 30 ft_
```

```calc {description="Horizontal dimension normal to the wind"}
B = 100 ft_
```

### Exposure constants (Table 26.9-1)

```calc {description="Turbulence constants c, ℓ, ε̄ and zmin, ASCE 7-10 Table 26.9-1"}
gust = ``#exposure	c	ℓ	ε̄	z_min
		ft		ft
B	0.30	320	0.333333333333333	30
C	0.20	500	0.2	15
D	0.15	650	0.125	7`` = !
```

```calc
> Turbulence intensity factor
c = gust[exposure, "c"] = ??
> Integral length scale factor
ℓ = gust[exposure, "ℓ"] = ?? ft
> Integral length scale power-law exponent
ε̄ = gust[exposure, "ε̄"] = ??
> Minimum equivalent height
z_min = gust[exposure, "z_min"] = ?? ft
```

### Gust-effect factor

I_z and L_z below are the standard's I and L at z̄ (the turbulence intensity and length scale at the equivalent height).

```calc {description="Equivalent height of the structure, 0.6h but not below zmin"}
z̄ = max(0.6 h, z_min) = ?? ft
```

```calc {description="Intensity of turbulence at z̄, Eq. 26.9-7"}
I_z = c (33 'ft' / z̄)^(1/6) = ??
```

```calc {description="Integral length scale of turbulence at z̄, Eq. 26.9-9"}
L_z = ℓ (z̄ / 33 'ft')^ε̄ = ?? ft
```

```calc {description="Background response factor, Eq. 26.9-8"}
Q = sqrt(1 / (1 + 0.63 ((B + h) / L_z)^0.63)) = ??
```

```calc {description="Peak factors for background response and wind response (Sec. 26.9.4)"}
g_Q = 3.4
```

```calc {description="Peak factor for wind response"}
g_v = 3.4
```

```calc {description="Gust-effect factor of a rigid structure, Eq. 26.9-6"}
G = 0.925 (1 + 1.7 g_Q I_z Q) / (1 + 1.7 g_v I_z) = ??
```
