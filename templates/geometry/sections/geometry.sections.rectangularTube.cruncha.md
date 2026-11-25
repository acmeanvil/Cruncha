+++
cruncha-template = 1
id = "geometry.sections.rectangularTube"
version = "1.0.1"
title = "Section properties: rectangle and rectangular tube"
summary = "Area, second moments, elastic and plastic section moduli and radii of gyration of a solid rectangle or a rectangular tube with square corners."
category = ["Geometry", "Section properties"]
tags = ["section properties", "rectangle", "hollow rectangle", "tube", "box", "moment of inertia", "section modulus", "plastic modulus"]
from = "lacewing:geometry.secondMoment"
license = "GPL-3.0-or-later"
inputs = ["b", "h", "t"]
outputs = ["A", "I_x", "I_y", "S_x", "S_y", "Z_x", "Z_y", "r_x", "r_y"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas, Sadegh)"
edition = "8th"
locator = "Table A.1, cases 1 and 3 (rectangle; hollow rectangle)"

[[tests]]
name = "Solid rectangle 100 × 50 mm"
source = "Lacewing Lab section_shapes_test sample; by hand: A = bh, I_x = bh³/12, S_x = bh²/6, Z_x = bh²/4"
inputs = { b = "100 mm", h = "50 mm", t = "25 mm" }
expect = { A = "5000 mm^2", I_x = "1041666.667 mm^4", I_y = "4166666.667 mm^4", S_x = "41666.6667 mm^3", Z_x = "62500 mm^3", Z_y = "125000 mm^3" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Tube 100 × 150 × 6 mm"
source = "Lacewing Lab section_shapes_test sample; expected values integrated over the four walls separately"
inputs = { b = "100 mm", h = "150 mm", t = "6 mm" }
expect = { A = "2856 mm^2", I_x = "8852472 mm^4", I_y = "4663072 mm^4", S_x = "118032.96 mm^3", S_y = "93261.44 mm^3", Z_x = "143532 mm^3", Z_y = "107832 mm^3", r_x = "55.674097 mm", r_y = "40.407033 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "A wall thicker than half the width is refused"
source = "100 × 50 mm with t = 30 mm: the walls would overlap"
inputs = { b = "100 mm", h = "50 mm", t = "30 mm" }
expect = { A = "5000 mm^2" }
tolerance = "1e-8"
checks = "fail"
+++

## Section properties: rectangle and rectangular tube

A rectangle b wide and h deep, hollowed by a wall of thickness t on all four sides, with square corners inside and out. The x axis is horizontal through the centroid, so I_x and S_x are for bending about the width. For a solid rectangle, give t as half the smaller side.

Rolled and formed tubes have rounded corners, which take a few per cent off these values; use the manufacturer's or the AISC Manual's properties for them.

### Dimensions

```calc {description="Overall width, parallel to the x axis"}
b = 100 mm_
```

```calc {description="Overall depth, parallel to the y axis"}
h = 150 mm_
```

```calc {description="Wall thickness (half the smaller side for a solid rectangle)"}
t = 6 mm_
```

```calc
@test 2 t ≤ min(b, h)
```

### The hole

```calc
> Inside width, zero for a solid section
b_i = max(b - 2 t, 0 'mm') = ?? mm
> Inside depth, zero for a solid section
h_i = max(h - 2 t, 0 'mm') = ?? mm
```

### Properties

Each property is the outer rectangle's less the hole's.

```calc {description="Area"}
A = b h - b_i h_i = ?? mm²
```

```calc
> Second moment of area about x
I_x = (b h^3 - b_i h_i^3) / 12 = ?? mm⁴
> Elastic section modulus about x
S_x = 2 I_x / h = ?? mm³
> Plastic section modulus about x
Z_x = (b h^2 - b_i h_i^2) / 4 = ?? mm³
> Radius of gyration about x
r_x = sqrt(I_x / A) = ?? mm
```

```calc
> Second moment of area about y
I_y = (h b^3 - h_i b_i^3) / 12 = ?? mm⁴
> Elastic section modulus about y
S_y = 2 I_y / b = ?? mm³
> Plastic section modulus about y
Z_y = (h b^2 - h_i b_i^2) / 4 = ?? mm³
> Radius of gyration about y
r_y = sqrt(I_y / A) = ?? mm
```
