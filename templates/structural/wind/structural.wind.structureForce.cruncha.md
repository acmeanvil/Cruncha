+++
cruncha-template = 1
id = "structural.wind.structureForce"
version = "1.0.0"
title = "Wind on chimneys, tanks, open signs and lattice frames, ASCE 7-10"
summary = "Design wind force F = qz G Cf Af on a chimney, tank, rooftop structure, open sign or lattice framework (Eq. 29.5-1), with qz at the centroid of the projected area and the 16 psf minimum."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "other structures", "chimney", "stack", "tank", "silo", "open sign", "lattice", "Chapter 29", "Eq. 29.5-1", "Cf", "force coefficient"]
from = "lacewing:structural.windOther"
license = "GPL-3.0-or-later"
inputs = ["V", "exposure", "K_zt", "K_d", "G", "h", "z_b", "D", "C_f", "A_f"]
outputs = ["z", "K_z", "q_z", "r_hD", "r_Dq", "F", "F_d"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 29.5 Eq. 29.5-1; Figs. 29.5-1 (chimneys, tanks, rooftop equipment) and 29.5-2 (open signs, lattice frameworks); Sec. 29.8 (minimum); Eq. 29.3-1 and Table 29.3-1 (qz, Kz); Table 26.6-1 (Kd)"

[[tests]]
name = "SI: 16 m square stack on a 4 m base, 50 m/s, exposure C"
source = "Lacewing Lab structural_screens_test (other structures, chimney): qz at the centroid, 12 m; h/D from the stack's own height, 16. Cf by Fig. 29.5-1, square normal to a face, 1.4 at h/D 7 and 2.0 at 25 (verified in the PDF), linear: 1.7. By hand: Kz 1.04010, qz = 0.61327 × 1.04010 × 0.90 × 50² = 1435.4 Pa, F = 1435.4 × 0.85 × 1.7 × 16 = 33 186 N"
inputs = { V = "50 m/s", exposure = '"C"', K_d = "0.90", h = "20 m", z_b = "4 m", D = "1 m", C_f = "1.7", A_f = "16 m^2" }
expect = { z = "12 m", r_hD = "16", q_z = "1435.36 Pa", F = "33185.5 N" }
tolerance = "1e-4"

[[tests]]
name = "Open sign of flat members, ε = 0.2: qz at the middle of the face"
source = "Cf = 1.8 from Fig. 29.5-2 (flat-sided members, ε 0.1 to 0.29; verified in the PDF; Lacewing Lab asce7_wind_test). By hand: face 8 ft high with its top at 20 ft, centroid 16 ft; Kz = 2.01 (16/900)^(2/9.5) = 0.86050; qz = 0.00256 × 0.86050 × 0.85 × 115² = 24.763 psf; Af = 0.2 × 8 × 20 = 32 ft²; F = 24.763 × 0.85 × 1.8 × 32 = 1212.4 lb"
inputs = { V = "115 mph", exposure = '"C"', K_d = "0.85", h = "20 ft", z_b = "12 ft", D = "0.1 ft", C_f = "1.8", A_f = "32 ft^2" }
expect = { z = "16 ft", q_z = "24.763 psf", F = "1212.40 lbf", F_d = "1212.40 lbf" }
tolerance = "1e-4"

[[tests]]
name = "Round chimney on the ground: D√qz above 2.5"
source = "By hand: 40 ft chimney, 4 ft diameter, round (Kd 0.95), 115 mph, exposure C: z = 20 ft, Kz 0.90189, qz = 29.008 psf, D√qz = 4 × √29.008 = 21.54 (> 2.5: the surface rows of Fig. 29.5-1 apply); h/D 10; Cf 0.62 (moderately smooth, between 0.6 and 0.7); F = 29.008 × 0.85 × 0.62 × 160 = 2445.9 lb"
inputs = { V = "115 mph", exposure = '"C"', K_d = "0.95", h = "40 ft", z_b = "0 ft", D = "4 ft", C_f = "0.62", A_f = "160 ft^2" }
expect = { r_hD = "10", r_Dq = "21.543", F = "2445.91 lbf" }
tolerance = "1e-4"

[[tests]]
name = "Small force: the 16 psf minimum governs"
source = "By hand: as the open sign at 60 mph: qz = 24.763 × (60/115)² = 6.7407 psf; F = 6.7407 × 0.85 × 1.8 × 32 = 330.0 lb < 16 × 32 = 512 lb (Sec. 29.8)"
inputs = { V = "60 mph", exposure = '"C"', K_d = "0.85", h = "20 ft", z_b = "12 ft", D = "0.1 ft", C_f = "1.8", A_f = "32 ft^2" }
expect = { F = "330.03 lbf", F_d = "512 lbf" }
tolerance = "1e-4"
+++

## Wind on chimneys, tanks, open signs and lattice frames (ASCE 7-10)

The design wind force on other structures, F = q_z G C_f A_f (Eq. 29.5-1): chimneys, tanks and similar structures, rooftop structures and equipment on buildings taller than 60 ft, open signs and lattice frameworks. q_z is taken at the height of the centroid of A_f, here the middle of a uniform outline between z_b and h. For a tapering or tall structure, work in segments. The force acts parallel to the wind, at strength level.

The force coefficients C_f are given only as **tables** (no formula), so C_f is an input:
- **Figure 29.5-1** (chimneys, tanks, rooftop equipment): C_f by cross-section, surface roughness and h/D, with linear interpolation in h/D between 1, 7 and 25 (note 2). Round sections with D√q_z ≤ 2.5 (D in ft, q_z in psf; 5.3 in SI) take one row whatever the surface. The template computes h/D and D√q_z for you to read it.
- **Figure 29.5-2** (open signs and lattice frameworks): C_f by solidity ε (below 0.1, 0.1 to 0.29, 0.3 to 0.7) and by flat-sided or rounded members (rounded ones by D√q_z, D the member diameter). A_f is then the solid area projected normal to the wind (note 3). A sign with openings of less than 30 % of its gross area is a solid sign: use that template.

Trussed towers have their own template (Figure 29.5-3 gives C_f by formula). Rooftop equipment on buildings with h ≤ 60 ft uses Sec. 29.5.1 (GC_r), not this template.

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

K_d (Table 26.6-1): 0.90 for square chimneys and tanks, 0.95 for hexagonal and round ones, 0.85 for open signs and lattice frameworks.

```calc {description="Wind directionality factor, Table 26.6-1"}
K_d = 0.95_
```

```calc {description="Gust-effect factor, Sec. 26.9 (0.85 for a rigid structure)"}
G = 0.85_
```

### Structure

```calc {description="Height of the top of the structure (or of the sign face) above ground"}
h = 40 ft_
```

```calc {description="Height of its base (or the bottom of the sign face) above ground: 0 on the ground"}
z_b = 0 ft_
```

```calc {description="Diameter or least horizontal dimension (for an open sign, a typical member's diameter)"}
D = 4 ft_
```

```calc {description="Projected area normal to the wind (the solid area, for an open sign or lattice)"}
A_f = 160 ft²_
```

### Velocity pressure at the centroid

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

```calc {description="Height of the centroid of Af (middle of a uniform outline)"}
z = (z_b + h) / 2 = ?? ft
```

```calc {description="Velocity pressure exposure coefficient at z, Table 29.3-1"}
K_z = 2.01 (min(max(z, 15 'ft'), z_g) / z_g)^(2 / α) = ??
```

```calc {description="Velocity pressure at the centroid, Eq. 29.3-1"}
q_z = 0.00256 'psf/mph^2' K_z K_zt K_d V^2 = ?? psf
```

### Force coefficient

```calc
> Height over least dimension of the structure itself, for Fig. 29.5-1
r_hD = (h - z_b) / D = ??
> D√qz in ft and psf, against 2.5 in Figs. 29.5-1 and 29.5-2
r_Dq = (D / 1 'ft') sqrt(q_z / 1 'psf') = ??
```

```calc {description="Force coefficient, Fig. 29.5-1 or 29.5-2"}
C_f = 0.62_
```

### Force

```calc {description="Design wind force, Eq. 29.5-1"}
F = q_z G C_f A_f = ?? lbf
```

```calc {description="Design force with the 16 psf minimum on Af, Sec. 29.8"}
F_d = max(F, 16 'psf' A_f) = ?? lbf
```
