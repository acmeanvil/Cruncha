+++
cruncha-template = 1
id = "structural.wind.componentsLowRise"
version = "1.0.0"
title = "Wind on components and cladding, h ≤ 60 ft, ASCE 7-10"
summary = "Design wind pressures, positive and negative, on one component or cladding element of a building with h ≤ 60 ft (Eq. 30.4-1): GCp interpolated by effective wind area, internal pressure, the 16 psf minimum and the end-zone width a."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "components and cladding", "C&C", "Chapter 30", "Eq. 30.4-1", "GCp", "GCpi", "effective wind area", "zone", "roof", "wall", "cladding", "fastener"]
from = "lacewing:structural.windComponents"
license = "GPL-3.0-or-later"
inputs = ["V", "exposure", "K_zt", "K_d", "h", "b_min", "GC_pi", "A", "A_1", "A_2", "GC_(pos,1)", "GC_(pos,2)", "GC_(neg,1)", "GC_(neg,2)"]
outputs = ["K_h", "q_h", "a", "GC_pos", "GC_neg", "p_pos", "p_neg", "p_(pos,d)", "p_(neg,d)"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 30.4 Eq. 30.4-1; Figs. 30.4-1 (walls) and 30.4-2A to 2C (roofs); Table 30.3-1 (Kh); Sec. 30.2.2 (16 psf minimum); Table 26.11-1 (GCpi); Fig. 30.5-1 (pnet30, used as a check)"

[[tests]]
name = "Roof zone 3, θ ≤ 7°, 20 ft², 115 mph, exposure B, h 30 ft"
source = "ASCE 7-10 Fig. 30.5-1 (verified in the PDF): pnet30 roof 0-7°, zone 3, 20 sf, 115 mph = 9.1 / −49.8 psf (exposure B, h 30 ft, enclosed, Kzt 1). Here GCp from Fig. 30.4-2A zone 3 (+0.3 → +0.2, −2.8 → −1.1 from 10 to 100 ft², as Lacewing Lab's asce7_wind table): 9.07 / −49.76 psf"
inputs = { V = "115 mph", exposure = '"B"', h = "30 ft", A = "20 ft^2", A_1 = "10 ft^2", A_2 = "100 ft^2", "GC_(pos,1)" = "0.3", "GC_(pos,2)" = "0.2", "GC_(neg,1)" = "-2.8", "GC_(neg,2)" = "-1.1" }
expect = { q_h = "20.16 psf", p_pos = "9.1 psf", p_neg = "-49.8 psf", "p_(pos,d)" = "16 psf" }
tolerance = "0.5%"

[[tests]]
name = "Roof zone 2, θ ≤ 7°, 50 ft²: log-linear interpolation"
source = "ASCE 7-10 Fig. 30.5-1: zone 2, 50 sf, 115 mph = 8.3 / −30.1 psf (verified in the PDF). By hand GCp− = −1.8 + 0.7 log(5)/log(10) = −1.3107, p = 20.161 × (−1.3107 − 0.18) = −30.05 psf"
inputs = { V = "115 mph", exposure = '"B"', h = "30 ft", A = "50 ft^2", A_1 = "10 ft^2", A_2 = "100 ft^2", "GC_(pos,1)" = "0.3", "GC_(pos,2)" = "0.2", "GC_(neg,1)" = "-1.8", "GC_(neg,2)" = "-1.1" }
expect = { GC_neg = "-1.3107", p_pos = "8.3 psf", p_neg = "-30.1 psf" }
tolerance = "0.4%"

[[tests]]
name = "Wall zone 5, 100 ft² (GCp not reduced)"
source = "ASCE 7-10 Fig. 30.5-1: wall zone 5, 100 sf, 115 mph = 20.2 / −24.7 psf (verified in the PDF), with Fig. 30.4-1's zone 5 lines unreduced (+1.0 → +0.7, −1.4 → −0.8 from 10 to 500 ft²)"
inputs = { V = "115 mph", exposure = '"B"', h = "30 ft", A = "100 ft^2", A_1 = "10 ft^2", A_2 = "500 ft^2", "GC_(pos,1)" = "1.0", "GC_(pos,2)" = "0.7", "GC_(neg,1)" = "-1.4", "GC_(neg,2)" = "-0.8" }
expect = { p_pos = "20.2 psf", p_neg = "-24.7 psf" }
tolerance = "0.3%"

[[tests]]
name = "Exposure C: Fig. 30.5-1 times λ = 1.40"
source = "ASCE 7-10 Fig. 30.5-1: roof zone 1, 10 sf, 115 mph 9.7 / −23.8 psf, λ = 1.40 for exposure C at 30 ft (Eq. 30.5-1): 13.6 / −33.3 psf (verified in the PDF). By hand Kh 0.98225, qh 28.267 psf: 13.57 / −33.35 psf"
inputs = { V = "115 mph", exposure = '"C"', h = "30 ft", A = "10 ft^2", A_1 = "10 ft^2", A_2 = "100 ft^2", "GC_(pos,1)" = "0.3", "GC_(pos,2)" = "0.2", "GC_(neg,1)" = "-1.0", "GC_(neg,2)" = "-0.9" }
expect = { p_pos = "13.6 psf", p_neg = "-33.3 psf" }
tolerance = "0.4%"

[[tests]]
name = "Exposure B at h = 15 ft: Kh held at its 30 ft value"
source = "ASCE 7-10 Table 30.3-1 note (z not below 30 ft in exposure B) and Fig. 30.5-1's λ = 1.00 for B at 15 ft (verified in the PDF): the same 9.7 / −23.8 psf as at 30 ft"
inputs = { V = "115 mph", exposure = '"B"', h = "15 ft", A = "10 ft^2", A_1 = "10 ft^2", A_2 = "100 ft^2", "GC_(pos,1)" = "0.3", "GC_(pos,2)" = "0.2", "GC_(neg,1)" = "-1.0", "GC_(neg,2)" = "-0.9" }
expect = { K_h = "0.70059", p_pos = "9.7 psf", p_neg = "-23.8 psf" }
tolerance = "0.3%"

[[tests]]
name = "Low wind speed: the 16 psf minimum governs both ways"
source = "By hand: 90 mph, exposure B, h 30 ft: qh = 12.348 psf; zone 1 at 10 ft²: 12.348 × 0.48 = 5.93 psf and 12.348 × (−1.18) = −14.57 psf, both inside 16 psf, so the design pressures are ±16 psf (Sec. 30.2.2). a = max(min(0.1 × 60, 0.4 × 30), max(0.04 × 60, 3)) = 6 ft"
inputs = { V = "90 mph", exposure = '"B"', h = "30 ft", b_min = "60 ft", A = "10 ft^2", A_1 = "10 ft^2", A_2 = "100 ft^2", "GC_(pos,1)" = "0.3", "GC_(pos,2)" = "0.2", "GC_(neg,1)" = "-1.0", "GC_(neg,2)" = "-0.9" }
expect = { p_pos = "5.9273 psf", p_neg = "-14.5710 psf", "p_(pos,d)" = "16 psf", "p_(neg,d)" = "-16 psf", a = "6 ft" }
tolerance = "1e-4"

[[tests]]
name = "Narrow building: a held at 3 ft"
source = "By hand: least dimension 25 ft, h 20 ft: min(2.5, 8) = 2.5 ft, below max(1.0, 3) = 3 ft, so a = 3 ft"
inputs = { h = "20 ft", b_min = "25 ft" }
expect = { a = "3 ft" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Building taller than 60 ft: outside Part 1"
source = "ASCE 7-10 Sec. 30.1.1: Part 1 applies to h ≤ 60 ft; at 70 ft the h check fails"
inputs = { h = "70 ft" }
expect = { a = "6 ft" }
tolerance = "1e-9"
checks = "fail"
+++

## Wind on components and cladding, h ≤ 60 ft (ASCE 7-10)

The design wind pressures on one component or cladding element (wall panel, girt, roof deck, purlin, fastener, glazing) of an enclosed or partially enclosed building with mean roof height h ≤ 60 ft, by Eq. 30.4-1 of Chapter 30 Part 1: p = q_h [(GC_p) − (GC_pi)]. Each component is designed for its largest positive and largest negative pressure, and neither may be less than 16 psf (Sec. 30.2.2).

The external coefficients GC_p are given **only as graphs** (Figures 30.4-1 for walls, 30.4-2A to 2C for roofs, by zone and roof slope), with no formula, so they are inputs: read the two ends of the zone's sloping line (GC_p at A_1 and at A_2) from the figure, and the template interpolates linearly in log A between them, holding each end value outside. GC_pi (Table 26.11-1) and K_d (Table 26.6-1) are inputs too.

Zones: walls 4 (interior) and 5 (corners, width a); roofs 1 (interior), 2 (edges, width a) and 3 (corners, a × a). Figure 30.4-1 note 5 reduces the wall values by 10 % where θ ≤ 10°: enter them already reduced. A parapet of 3 ft or more around a roof of θ ≤ 7° changes zone 3 (Fig. 30.4-2A note 5).

Limits: h ≤ 60 ft; for taller buildings use Part 3 (Fig. 30.6-1), not this template. ASCE 7-16 changed the roof coefficients and zones substantially; these are ASCE 7-10's.

### Site and building

```calc {description="Basic wind speed for the risk category, Fig. 26.5-1 (strength level)"}
V = 115 mph_
```

```calc {description="Exposure category B, C or D (in quotes)"}
exposure = "C"_
```

```calc {description="Topographic factor, Sec. 26.8 (1 on flat ground)"}
K_zt = 1_
```

```calc {description="Wind directionality factor, Table 26.6-1 (0.85 for components and cladding)"}
K_d = 0.85_
```

```calc {description="Mean roof height (eave height where θ ≤ 10°)"}
h = 30 ft_
```

```calc {description="Least horizontal dimension of the building"}
b_min = 60 ft_
```

```calc {description="Internal pressure coefficient (magnitude), Table 26.11-1: 0.18 enclosed, 0.55 partially enclosed"}
GC_pi = 0.18_
```

### Component and its zone

The effective wind area is the span times an effective width of at least one third of the span (Sec. 26.2); for a fastener, the area tributary to it.

```calc {description="Effective wind area of the component"}
A = 10 ft²_
```

The defaults are roof zone 3 of Figure 30.4-2A (θ ≤ 7°), whose line runs from 10 to 100 ft². Wall lines run from 10 to 500 ft².

```calc {description="Effective wind area at the start of the zone's sloping line"}
A_1 = 10 ft²_
```

```calc {description="Effective wind area at the end of the zone's sloping line"}
A_2 = 100 ft²_
```

```calc {description="Positive GCp at A1, Fig. 30.4-1 or 30.4-2"}
GC_(pos,1) = 0.3_
```

```calc {description="Positive GCp at A2"}
GC_(pos,2) = 0.2_
```

```calc {description="Negative GCp at A1, Fig. 30.4-1 or 30.4-2"}
GC_(neg,1) = -2.8_
```

```calc {description="Negative GCp at A2"}
GC_(neg,2) = -1.1_
```

```calc
@test h ≤ 60 'ft'
```

### Velocity pressure (Table 30.3-1)

As Table 27.3-1, except that in exposure B the height is not taken below 30 ft.

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
> Lowest height Kh is taken at
z_min = {30 'ft' if exposure == "B"; 15 'ft' otherwise} = ?? ft
```

```calc {description="Velocity pressure exposure coefficient at h, Table 30.3-1"}
K_h = 2.01 (min(max(h, z_min), z_g) / z_g)^(2 / α) = ??
```

```calc {description="Velocity pressure at h, Eq. 30.3-1"}
q_h = 0.00256 'psf/mph^2' K_h K_zt K_d V^2 = ?? psf
```

### End-zone width

```calc {description="End-zone width a: 10 % of the least dimension or 0.4h, the smaller, but not below 4 % of it or 3 ft"}
a = max(min(0.1 b_min, 0.4 h), max(0.04 b_min, 3 'ft')) = ?? ft
```

### Pressure coefficients at A

```calc {description="Positive GCp at the effective wind area"}
GC_pos = {GC_(pos,1) if A ≤ A_1; GC_(pos,2) if A ≥ A_2; GC_(pos,1) + (GC_(pos,2) - GC_(pos,1)) ln(A / A_1) / ln(A_2 / A_1) otherwise} = ??
```

```calc {description="Negative GCp at the effective wind area"}
GC_neg = {GC_(neg,1) if A ≤ A_1; GC_(neg,2) if A ≥ A_2; GC_(neg,1) + (GC_(neg,2) - GC_(neg,1)) ln(A / A_1) / ln(A_2 / A_1) otherwise} = ??
```

### Design pressures, Eq. 30.4-1

The positive pressure is worst with internal suction (−GC_pi), the negative with internal pressure (+GC_pi).

```calc
> Net positive pressure (toward the surface)
p_pos = q_h (GC_pos + GC_pi) = ?? psf
> Net negative pressure (away from the surface)
p_neg = q_h (GC_neg - GC_pi) = ?? psf
```

```calc
> Design positive pressure, not less than 16 psf (Sec. 30.2.2)
p_(pos,d) = max(p_pos, 16 'psf') = ?? psf
> Design negative pressure, not less than 16 psf
p_(neg,d) = min(p_neg, -16 'psf') = ?? psf
```
