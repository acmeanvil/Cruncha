+++
cruncha-template = 1
id = "geometry.sections.iSection"
version = "1.0.1"
title = "Section properties: doubly symmetric I-section"
summary = "Area, second moments, section moduli, radii of gyration, torsional and warping constants and r_ts of a welded I-section with equal flanges."
category = ["Geometry", "Section properties"]
tags = ["section properties", "I-section", "plate girder", "built-up", "moment of inertia", "section modulus", "plastic modulus", "torsion constant", "warping constant", "rts"]
from = "lacewing:geometry.secondMoment"
license = "GPL-3.0-or-later"
inputs = ["d", "b_f", "t_f", "t_w"]
outputs = ["A", "I_x", "I_y", "S_x", "S_y", "Z_x", "Z_y", "r_x", "r_y", "J", "C_w", "r_ts", "h_o"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas, Sadegh)"
edition = "8th"
locator = "Table A.1, case 6 (I-section)"

[[references]]
work = "AISC Design Guide 9, Torsional Analysis of Structural Steel Members (Seaburg, Carter)"
edition = "1997"
locator = "Sec. 3.1 and Appendix C (J and Cw of thin-walled open sections)"

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. F2.2, Eq. F2-7 (rts) and its User Note (Cw = Iy ho²/4)"

[[tests]]
name = "Plates 203 deep, 133 × 7.8 flanges, 5.8 web (mm)"
source = "Lacewing Lab section_shapes_test sample; A, I and Z integrated over the three plates separately"
inputs = { d = "203 mm", b_f = "133 mm", t_f = "7.8 mm", t_w = "5.8 mm" }
expect = { A = "3161.72 mm^2", I_x = "22955506.3023 mm^4", I_y = "3061475.0991 mm^4", S_x = "226162.6237 mm^3", S_y = "46037.2195 mm^3", Z_x = "253422.682 mm^3", Z_y = "70563.134 mm^3", r_x = "85.208267 mm", r_y = "31.117426 mm" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Thin-walled constants of the same plates"
source = "By hand: J = (2 × 133 × 7.8³ + 195.2 × 5.8³)/3 = 54 772.2 mm⁴, Cw = Iy × 195.2²/4 = 2.916 29 × 10¹⁰ mm⁶"
inputs = { d = "203 mm", b_f = "133 mm", t_f = "7.8 mm", t_w = "5.8 mm" }
expect = { h_o = "195.2 mm", J = "54772.2315 mm^4", C_w = "29162877040 mm^6" }
tolerance = "1e-6"
+++

## Section properties: doubly symmetric I-section

An I-section built from three plates: two equal flanges b_f × t_f and a web t_w thick, d deep overall, welded with no fillets counted. The x axis is the strong axis.

Rolled shapes have root fillets, which add to J and a little to the other properties; use the AISC Manual's values for them. J and C_w here are the thin-walled approximations.

### Plates

```calc {description="Overall depth"}
d = 203 mm_
```

```calc {description="Flange width"}
b_f = 133 mm_
```

```calc {description="Flange thickness"}
t_f = 7.8 mm_
```

```calc {description="Web thickness"}
t_w = 5.8 mm_
```

```calc
@test 2 t_f < d
```

```calc
@test t_w ≤ b_f
```

### Area and bending

```calc {description="Clear depth of the web between flanges"}
h_w = d - 2 t_f = ?? mm
```

```calc {description="Area"}
A = 2 b_f t_f + h_w t_w = ?? mm²
```

```calc
> Second moment of area about x
I_x = (b_f d^3 - (b_f - t_w) h_w^3) / 12 = ?? mm⁴
> Elastic section modulus about x
S_x = 2 I_x / d = ?? mm³
> Plastic section modulus about x
Z_x = b_f t_f (d - t_f) + t_w h_w^2 / 4 = ?? mm³
> Radius of gyration about x
r_x = sqrt(I_x / A) = ?? mm
```

```calc
> Second moment of area about y
I_y = (2 t_f b_f^3 + h_w t_w^3) / 12 = ?? mm⁴
> Elastic section modulus about y
S_y = 2 I_y / b_f = ?? mm³
> Plastic section modulus about y
Z_y = t_f b_f^2 / 2 + h_w t_w^2 / 4 = ?? mm³
> Radius of gyration about y
r_y = sqrt(I_y / A) = ?? mm
```

### Torsion

```calc
> Distance between flange centroids
h_o = d - t_f = ?? mm
> Torsional constant, thin-walled
J = (2 b_f t_f^3 + h_o t_w^3) / 3 = ?? mm⁴
> Warping constant, doubly symmetric I
C_w = I_y h_o^2 / 4 = ?? mm⁶
> Effective radius of gyration for lateral-torsional buckling, Eq. F2-7
r_ts = sqrt(sqrt(I_y C_w) / S_x) = ?? mm
```
