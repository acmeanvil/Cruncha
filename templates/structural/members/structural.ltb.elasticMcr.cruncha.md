+++
cruncha-template = 1
id = "structural.ltb.elasticMcr"
version = "1.0.0"
title = "Elastic lateral-torsional buckling moment, general theory"
summary = "Elastic critical moment Mcr of an I-shaped, channel or rectangular beam segment between braces, with moment gradient Cb, load height above or below the shear centre, monosymmetry βx and end restraint against lateral bending and warping."
category = ["Structural engineering", "Members"]
tags = ["lateral-torsional buckling", "LTB", "Mcr", "elastic critical moment", "load height", "monosymmetry", "warping", "SSRC", "ADM 2015", "Cb", "beam", "stability"]
from = "lacewing:structural.lateralBuckling"
license = "GPL-3.0-or-later"
inputs = ["E", "G", "I_y", "J", "C_w", "L_b", "C_b", "C_1", "C_2", "g_0", "β_x", "K_y", "K_z"]
outputs = ["U", "M_cr"]

[[references]]
work = "Ziemian (ed.), Guide to Stability Design Criteria for Metal Structures (SSRC)"
edition = "6th"
locator = "Ch. 5: Eqs. 5.2, 5.3, 5.10 to 5.13, 5.18a; Fig. 5.9 (end restraint); Sec. 5.2.7 example (W24×55), pp. 208-222"

[[references]]
work = "Aluminum Design Manual 2015, Part I Specification for Aluminum Structures"
edition = "2015"
locator = "Sec. F.4.2.5, Eqs. F.4-9 to F.4-12 (the general form with load height and monosymmetry)"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 15.1, case 11 (narrow rectangle in pure bending)"

[[tests]]
name = "SSRC Guide Sec. 5.2.7: W24×55, 48 ft, uniform moment: Mocr 644 in-kips"
source = "SSRC Guide 6th ed. Sec. 5.2.7, pp. 219-220: Mocr = 644 in.-kips (checked against the PDF). Iy 29.1 in⁴, J 1.18 in⁴, Cw 3870 in⁶ (AISC Manual Table 1-1), E 29 000, G 11 200 ksi; end moments, so C1 0, C2 1. Script: 644.662 kip-in"
inputs = { E = "29000 ksi", G = "11200 ksi", I_y = "29.1 in^4", J = "1.18 in^4", C_w = "3870 in^6", L_b = "48 ft", C_b = "1", C_1 = "0", C_2 = "1", g_0 = "0 in", "β_x" = "0 in", K_y = "1", K_z = "1" }
expect = { M_cr = "644 kip·in" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "W24×55, 48 ft, uniform load on the top flange"
source = "By the formula (script): Cb 1.136 for a uniform load on an unbraced simple span (AISC Manual Table 3-1, 1.14), C1 = C2 = 0.5, g0 = −d/2 = −11.8 in. (toward the shear centre from above); U = −5.9 in., Mcr = 583.159 kip-in, against 732.336 kip-in with the load at the shear centre"
inputs = { E = "29000 ksi", G = "11200 ksi", I_y = "29.1 in^4", J = "1.18 in^4", C_w = "3870 in^6", L_b = "48 ft", C_b = "1.136", C_1 = "0.5", C_2 = "0.5", g_0 = "-11.8 in", "β_x" = "0 in", K_y = "1", K_z = "1" }
expect = { U = "-5.9 in", M_cr = "583.1586 kip·in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "W24×55, 48 ft, ends fixed against lateral bending and warping"
source = "SSRC Guide Eq. 5.18a for a doubly symmetric beam, Ky = Kz = 0.5: π/(0.5L) √(EIy GJ) √(1 + E Cw/(GJ) π²/(0.5L)²) = 1633.435 kip-in (script)"
inputs = { E = "29000 ksi", G = "11200 ksi", I_y = "29.1 in^4", J = "1.18 in^4", C_w = "3870 in^6", L_b = "48 ft", C_b = "1", C_1 = "0", C_2 = "1", g_0 = "0 in", "β_x" = "0 in", K_y = "0.5", K_z = "0.5" }
expect = { M_cr = "1633.435 kip·in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Roark Table 15.1 case 11: narrow rectangle 200 × 20 mm, 4 m, pure bending"
source = "Roark 7th ed. Table 15.1 case 11: M′ = π b³ d √(EG(1 − 0.63 b/d))/(6 l) = π × 0.02³ × 0.2 × √(200e9 × 77.2e9 × 0.937)/24 = 25 191.39 N·m; the general form with Cw = 0 and J = d b³(1 − 0.63 b/d)/3 gives it exactly (Lacewing Lab lateral_torsional_buckling_test)"
inputs = { E = "200 GPa", G = "77.2 GPa", I_y = "133333.3333333 mm^4", J = "499733.3333333 mm^4", C_w = "0 mm^6", L_b = "4 m", C_b = "1", C_1 = "0", C_2 = "1", g_0 = "0 mm", "β_x" = "0 mm", K_y = "1", K_z = "1" }
expect = { M_cr = "25191.39 N·m" }
tolerance = "1e-6"
checks = "pass"
+++

## Elastic lateral-torsional buckling moment

The elastic critical moment of a beam segment of length L_b between points braced against lateral movement and twist, by the general theory (SSRC Guide Ch. 5; ADM 2015 Eq. F.4-9):

M_cr = C_b π² E I_y/(K_y L_b)² [U + √(U² + (C_w/I_y)(K_y/K_z)² + G J (K_y L_b)²/(π² E I_y))],  U = C_1 g_0 + C_2 β_x/2

- **C_b**: the moment gradient factor over the segment (AISC Eq. F1-1; see the C_b template).
- **Load height g_0**: distance from the shear centre to the point where a transverse load acts, **negative** when the load is above the shear centre and acts toward it (a gravity load on the top flange), which lowers M_cr; positive below. It acts only with transverse loads: use C_1 = C_2 = 0.5 for transverse loads and C_1 = 0, C_2 = 1 for end moments only (ADM F.4.2.5).
- **Monosymmetry β_x**: zero for doubly symmetric sections; for a singly symmetric I, β_x ≈ 0.9 h_o (2 I_yc/I_y − 1)[1 − (I_y/I_x)²], positive with the larger flange in compression.
- **End restraint**: K_y against lateral bending of the whole section, K_z against warping, 1.0 free and 0.5 fully fixed (SSRC Fig. 5.9).

M_cr is elastic. The design strength of a steel beam follows AISC 360 Chapter F (the F2 template), which uses its own form of M_cr with K = 1, and yields or buckles inelastically below this value. For a rectangle, C_w ≈ 0.

### Material and section

```calc {description="Modulus of elasticity"}
E = 29000 ksi_
```

```calc {description="Shear modulus"}
G = 11200 ksi_
```

```calc {description="Second moment of area about the minor axis"}
I_y = 29.1 in⁴_
```

```calc {description="Torsional constant"}
J = 1.18 in⁴_
```

```calc {description="Warping constant"}
C_w = 3870 in⁶_
```

### Segment and loading

```calc {description="Unbraced length between lateral and torsional braces"}
L_b = 48 ft_
```

```calc {description="Moment gradient factor over the segment"}
C_b = 1_
```

```calc {description="Load-height coefficient C_1: 0.5 transverse loads, 0 end moments"}
C_1 = 0_
```

```calc {description="Monosymmetry coefficient C_2: 0.5 transverse loads, 1 end moments"}
C_2 = 1_
```

```calc {description="Load height from the shear centre, negative above it (toward it)"}
g_0 = 0 in_
```

```calc {description="Monosymmetry parameter, zero for doubly symmetric sections"}
β_x = 0 in_
```

```calc {description="Effective-length factor for lateral bending"}
K_y = 1_
```

```calc {description="Effective-length factor for warping"}
K_z = 1_
```

### Critical moment

```calc
> Load-height and monosymmetry term
U = C_1 g_0 + C_2 β_x / 2 = ?? in
> Effective length for lateral bending
L_y = K_y L_b = ?? ft
```

```calc {description="Elastic critical moment, SSRC Eqs. 5.10 to 5.13, ADM Eq. F.4-9"}
M_cr = C_b π^2 E I_y / L_y^2 (U + sqrt(U^2 + C_w / I_y (K_y / K_z)^2 + G J L_y^2 / (π^2 E I_y))) = ?? kip·in
```
