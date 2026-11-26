+++
cruncha-template = 1
id = "structural.wind.structureForce"
version = "1.1.0"
title = "Wind on chimneys, tanks, open signs and lattice frames, ASCE 7-10"
summary = "Design wind force F = qz G Cf Af on a chimney, tank, rooftop structure, open sign or lattice framework (Eq. 29.5-1), with qz at the centroid of the projected area and the 16 psf minimum."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "other structures", "chimney", "stack", "tank", "silo", "open sign", "lattice", "Chapter 29", "Eq. 29.5-1", "Cf", "force coefficient"]
from = "lacewing:structural.windOther"
license = "GPL-3.0-or-later"
inputs = ["q_z", "G", "h", "z_b", "D", "C_f", "A_f"]
outputs = ["z_c", "r_hD", "r_Dq", "F_struct", "F_(struct,d)"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 29.5 Eq. 29.5-1; Figs. 29.5-1 (chimneys, tanks, rooftop equipment) and 29.5-2 (open signs, lattice frameworks); Sec. 29.8 (minimum); Eq. 29.3-1 (qz)"

[[tests]]
name = "SI: 16 m square stack on a 4 m base, 50 m/s, exposure C"
source = "Lacewing Lab structural_screens_test (other structures, chimney): qz at the centroid, 12 m; h/D from the stack's own height, 16. Cf by Fig. 29.5-1, square normal to a face, 1.4 at h/D 7 and 2.0 at 25 (verified in the PDF), linear: 1.7. By hand: qz from velocityPressure at z = 12 m (Kz 1.04010, Kd 0.90) = 0.61327 × 1.04010 × 0.90 × 50² = 1435.4 Pa, F = 1435.4 × 0.85 × 1.7 × 16 = 33 186 N"
inputs = { q_z = "1435.3604 Pa", h = "20 m", z_b = "4 m", D = "1 m", C_f = "1.7", A_f = "16 m^2" }
expect = { z_c = "12 m", r_hD = "16", F_struct = "33185.5 N" }
tolerance = "1e-4"

[[tests]]
name = "Open sign of flat members, ε = 0.2: qz at the middle of the face"
source = "Cf = 1.8 from Fig. 29.5-2 (flat-sided members, ε 0.1 to 0.29; verified in the PDF; Lacewing Lab asce7_wind_test). By hand: face 8 ft high with its top at 20 ft, centroid 16 ft; qz from velocityPressure at z = 16 ft (Kz = 2.01 (16/900)^(2/9.5) = 0.86050, Kd 0.85) = 0.00256 × 0.86050 × 0.85 × 115² = 24.763 psf; Af = 0.2 × 8 × 20 = 32 ft²; F = 24.763 × 0.85 × 1.8 × 32 = 1212.4 lb"
inputs = { q_z = "24.76303 psf", h = "20 ft", z_b = "12 ft", D = "0.1 ft", C_f = "1.8", A_f = "32 ft^2" }
expect = { z_c = "16 ft", F_struct = "1212.40 lbf", "F_(struct,d)" = "1212.40 lbf" }
tolerance = "1e-4"

[[tests]]
name = "Round chimney on the ground: D√qz above 2.5"
source = "By hand: 40 ft chimney, 4 ft diameter, round (Kd 0.95), 115 mph, exposure C: z_c = 20 ft, qz from velocityPressure at 20 ft (Kz 0.90189) = 29.008 psf, D√qz = 4 × √29.008 = 21.54 (> 2.5: the surface rows of Fig. 29.5-1 apply); h/D 10; Cf 0.62 (moderately smooth, between 0.6 and 0.7); F = 29.008 × 0.85 × 0.62 × 160 = 2445.9 lb"
inputs = { q_z = "29.00752 psf", h = "40 ft", z_b = "0 ft", D = "4 ft", C_f = "0.62", A_f = "160 ft^2" }
expect = { r_hD = "10", r_Dq = "21.543", F_struct = "2445.91 lbf" }
tolerance = "1e-4"

[[tests]]
name = "Small force: the 16 psf minimum governs"
source = "By hand: as the open sign at 60 mph: qz = 24.763 × (60/115)² = 6.7407 psf; F = 6.7407 × 0.85 × 1.8 × 32 = 330.0 lb < 16 × 32 = 512 lb (Sec. 29.8)"
inputs = { q_z = "6.740787 psf", h = "20 ft", z_b = "12 ft", D = "0.1 ft", C_f = "1.8", A_f = "32 ft^2" }
expect = { F_struct = "330.03 lbf", "F_(struct,d)" = "512 lbf" }
tolerance = "1e-4"
+++

## Wind on chimneys, tanks, open signs and lattice frames (ASCE 7-10)

The design wind force on other structures, F = q_z G C_f A_f (Eq. 29.5-1): chimneys, tanks and similar structures, rooftop structures and equipment on buildings taller than 60 ft, open signs and lattice frameworks. q_z is taken at the height of the centroid of A_f, here the middle of a uniform outline between z_b and h. For a tapering or tall structure, work in segments. The force acts parallel to the wind, at strength level.

The force coefficients C_f are given only as **tables** (no formula), so C_f is an input:
- **Figure 29.5-1** (chimneys, tanks, rooftop equipment): C_f by cross-section, surface roughness and h/D, with linear interpolation in h/D between 1, 7 and 25 (note 2). Round sections with D√q_z ≤ 2.5 (D in ft, q_z in psf; 5.3 in SI) take one row whatever the surface. The template computes h/D and D√q_z for you to read it.
- **Figure 29.5-2** (open signs and lattice frameworks): C_f by solidity ε (below 0.1, 0.1 to 0.29, 0.3 to 0.7) and by flat-sided or rounded members (rounded ones by D√q_z, D the member diameter). A_f is then the solid area projected normal to the wind (note 3). A sign with openings of less than 30 % of its gross area is a solid sign: use that template.

Trussed towers have their own template (Figure 29.5-3 gives C_f by formula). Rooftop equipment on buildings with h ≤ 60 ft uses Sec. 29.5.1 (GC_r), not this template.

### Gust

```calc {description="Gust-effect factor, Sec. 26.9 (0.85 for a rigid structure; from structural.wind.gustRigid)"}
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

The velocity pressure q_z is an input: compute it with the velocity pressure template (structural.wind.velocityPressure) with z at the centroid z_c and K_d for the structure type, and bind it to its q_z.

```calc {description="Height of the centroid of Af (middle of a uniform outline), where qz is taken"}
z_c = (z_b + h) / 2 = ?? ft
```

```calc {description="Velocity pressure at the centroid z_c, Eq. 29.3-1, from structural.wind.velocityPressure"}
q_z = 29.01 psf_
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

```calc {description="Design wind force on the structure, Eq. 29.5-1"}
F_struct = q_z G C_f A_f = ?? lbf
```

```calc {description="Design force with the 16 psf minimum on Af, Sec. 29.8"}
F_(struct,d) = max(F_struct, 16 'psf' A_f) = ?? lbf
```
