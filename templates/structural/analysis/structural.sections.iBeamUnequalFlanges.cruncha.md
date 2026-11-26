+++
cruncha-template = 1
id = "structural.sections.iBeamUnequalFlanges"
version = "1.0.0"
title = "I-section with unequal flanges: section properties, J, Cw and shear centre"
summary = "Area, centroid, second moments, elastic and plastic moduli, radii of gyration, the torsion and warping constants J and Cw, the shear centre and the polar radius of gyration of a built-up or rolled I-section whose flanges may differ (sharp corners, thin-walled J and Cw)."
category = ["Structural engineering", "Analysis"]
tags = ["section properties", "I-beam", "plate girder", "monosymmetric", "singly symmetric", "crane girder", "plastic modulus", "warping constant", "torsion constant", "shear centre", "AISC 360"]
from = "lacewing:structural.sectionBuilder"
license = "GPL-3.0-or-later"
inputs = ["d", "b_ft", "t_ft", "b_fb", "t_fb", "t_w"]
outputs = ["A", "y_c", "I_x", "S_xt", "S_xb", "y_p", "Z_x", "I_y", "S_y", "Z_y", "r_x", "r_y", "J", "C_w", "y_o", "r_o"]

[[references]]
work = "AISC Design Guide 9, Torsional Analysis of Structural Steel Members (Seaburg, Carter)"
edition = "2003"
locator = "Sec. 3.2.1, Eq. 3.4 (J ≈ Σbt³/3 for open sections); Sec. 3.2.2, Eq. 3.5 (Cw = Iy h²/4 for doubly symmetric I-shapes, h = d − tf, fillets neglected), the limit of the unequal-flange form used here"

[[references]]
work = "ANSI/AISC 360-16 Specification for Structural Steel Buildings"
edition = "2016"
locator = "Eq. E4-9 (polar radius of gyration about the shear centre, r̄o² = xo² + yo² + (Ix + Iy)/Ag)"

[[references]]
work = "AISC Shapes Database"
edition = "v16.0 (2023)"
locator = "W14X22: d, bf, tf, tw and the tabulated Iy and Cw, for the comparison case"

[[tests]]
name = "Doubly symmetric 300 mm I-beam with 150 × 10 mm flanges (Lacewing Lab's opening section)"
source = "By hand, sharp corners: A = 2 × 1500 + 280 × 6 = 4680 mm²; Ix = 6 × 280³/12 + 2(150 × 10³/12 + 1500 × 145²) = 74.076e6 mm⁴; Zx = 2 × 1500 × 145 + 6 × 140² = 552 600 mm³; Iy = 2 × 10 × 150³/12 + 280 × 6³/12 = 5.63004e6 mm⁴; J = (2 × 150 × 10³ + 290 × 6³)/3 = 120 880 mm⁴; Cw = Iy,flange·ho²/2 = 2.8125e6 × 290²/2 = 1.18266e11 mm⁶ (= Iy ho²/4 for equal flanges)"
inputs = { d = "300 mm", b_ft = "150 mm", t_ft = "10 mm", b_fb = "150 mm", t_fb = "10 mm", t_w = "6 mm" }
expect = { A = "4680 mm^2", y_c = "150 mm", I_x = "74076000 mm^4", S_xt = "493840 mm^3", Z_x = "552600 mm^3", I_y = "5630040 mm^4", Z_y = "115020 mm^3", J = "120880 mm^4", C_w = "1.1826562e11 mm^6", y_o = "0 mm", r_o = "130.50366 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Plate girder with a small top flange: PNA in the web, shear centre near the big flange"
source = "By hand: d 600, top 200 × 15, bottom 400 × 20, web 10 mm: A = 16 650 mm², ȳ = 214.21 mm, Ix = 957.59e6 mm⁴; PNA at 20 + (8325 − 8000)/10 = 52.5 mm, Zx = 3.38306e6 mm³; ho = 582.5 mm, I1 = 10e6, I2 = 106.667e6 mm⁴, Cw = ho²·I1·I2/(I1 + I2) = 3.10223e12 mm⁶, shear centre 154.28 mm below the centroid"
inputs = { d = "600 mm", b_ft = "200 mm", t_ft = "15 mm", b_fb = "400 mm", t_fb = "20 mm", t_w = "10 mm" }
expect = { A = "16650 mm^2", y_c = "214.21171 mm", I_x = "9.575909e8 mm^4", S_xt = "2482166.9 mm^3", S_xb = "4470301.3 mm^3", y_p = "52.5 mm", Z_x = "3383062.5 mm^3", I_y = "1.1671375e8 mm^4", J = "1485833.3 mm^4", C_w = "3.1022286e12 mm^6", y_o = "-154.28314 mm", r_o = "297.19705 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Heavy top flange: the PNA in the flange"
source = "By hand: d 100, top 200 × 40, bottom 20 × 5, web 8 mm: the top flange holds 8000 of 8540 mm², so the PNA is in it at 100 − 8540/400 = 78.65 mm; Zx = 108 285.5 mm³"
inputs = { d = "100 mm", b_ft = "200 mm", t_ft = "40 mm", b_fb = "20 mm", t_fb = "5 mm", t_w = "8 mm" }
expect = { A = "8540 mm^2", y_c = "76.645199 mm", y_p = "78.65 mm", Z_x = "108285.5 mm^3", Z_y = "401380 mm^3" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "W14X22 from its tabulated dimensions, against the AISC Shapes Database"
source = "AISC Shapes Database v16.0, W14X22: d 13.7, bf 5.00, tf 0.335, tw 0.230 in; tabulated Iy 7.00 in⁴, Cw 314 in⁶ (values as Lacewing Lab's table holds them). Sharp corners give Iy 6.992 and Cw 311.7 (within 1 %); A 6.347 against 6.49 in², Ix 192.0 against 199 in⁴ and J 0.180 against 0.208 in⁴ are lower because the fillets are left out"
inputs = { d = "13.7 in", b_ft = "5 in", t_ft = "0.335 in", b_fb = "5 in", t_fb = "0.335 in", t_w = "0.23 in" }
expect = { I_y = "7.00 in^4", C_w = "314 in^6" }
tolerance = "1%"
checks = "pass"
+++

## I-section with unequal flanges

The geometric properties of an I-section built from three rectangles: a top flange, a bottom flange and a web, on one vertical centre line. The flanges may differ, as in a crane girder, a plate girder or a beam with a cover plate, so the section may be symmetric about its vertical axis only. Corners are sharp: root fillets and welds are left out, so A, Ix and J come out a little below a rolled shape's tabulated values. J and Cw are the thin-walled values on the plate centre lines (J = Σbt³/3).

Lacewing Lab's section builder also draws channels, tees, angles, zees, tubes and cold-formed shapes with fillets and bends, integrated exactly over the outline; that interactive builder is not in this template. The built-up section template handles any set of rectangles.

Axes: x horizontal through the centroid, y vertical; heights are measured from the underside of the bottom flange.

### Dimensions

```calc {description="Overall depth, outside of flange to outside of flange"}
d = 300 mm_
```

```calc {description="Top flange width"}
b_ft = 150 mm_
```

```calc {description="Top flange thickness"}
t_ft = 10 mm_
```

```calc {description="Bottom flange width"}
b_fb = 150 mm_
```

```calc {description="Bottom flange thickness"}
t_fb = 10 mm_
```

```calc {description="Web thickness"}
t_w = 6 mm_
```

### Area and centroid

```calc
> Clear height of the web between the flanges
h_w = d - t_ft - t_fb = ?? mm
> Area of the top flange
A_t = b_ft t_ft = ?? mm²
> Area of the bottom flange
A_b = b_fb t_fb = ?? mm²
> Area of the web
A_w = h_w t_w = ?? mm²
> Total area
A = A_t + A_b + A_w = ?? mm²
> Centroid, above the underside
y_c = (A_b t_fb / 2 + A_w (t_fb + h_w / 2) + A_t (d - t_ft / 2)) / A = ?? mm
```

### Bending about x

```calc
> Second moment of area about the horizontal centroidal axis
I_x = b_fb t_fb^3 / 12 + A_b (y_c - t_fb / 2)^2 + t_w h_w^3 / 12 + A_w (t_fb + h_w / 2 - y_c)^2 + b_ft t_ft^3 / 12 + A_t (d - t_ft / 2 - y_c)^2 = ?? mm⁴
> Elastic section modulus to the top fibre
S_xt = I_x / (d - y_c) = ?? mm³
> Elastic section modulus to the bottom fibre
S_xb = I_x / y_c = ?? mm³
> Radius of gyration about x
r_x = sqrt(I_x / A) = ?? mm
```

The plastic neutral axis divides the area in half. It lies in the bottom flange if that flange holds half the area, in the top flange if that one does, and in the web otherwise. The plastic modulus is the first moment of each part's area about it.

```calc
> Plastic neutral axis, above the underside
y_p = { A / (2 b_fb) if A_b ≥ A / 2; d - A / (2 b_ft) if A_t ≥ A / 2; t_fb + (A / 2 - A_b) / t_w otherwise } = ?? mm
```

```calc
function firstMoment(b, y1, y2, p)
    return b / 2 ((y2 - p) abs(y2 - p) - (y1 - p) abs(y1 - p))
end
```

```calc
> Plastic section modulus about x
Z_x = firstMoment(b_fb, 0 'mm', t_fb, y_p) + firstMoment(t_w, t_fb, d - t_ft, y_p) + firstMoment(b_ft, d - t_ft, d, y_p) = ?? mm³
```

### Bending about y

```calc
> Second moment of area about the vertical axis
I_y = (t_fb b_fb^3 + h_w t_w^3 + t_ft b_ft^3) / 12 = ?? mm⁴
> Elastic section modulus to the tip of the wider flange
S_y = I_y / (max(b_ft, b_fb) / 2) = ?? mm³
> Plastic section modulus about y
Z_y = (t_fb b_fb^2 + h_w t_w^2 + t_ft b_ft^2) / 4 = ?? mm³
> Radius of gyration about y
r_y = sqrt(I_y / A) = ?? mm
```

### Torsion, warping and shear centre

Each flange bends about the web like a beam on its own, and the shear centre lies on the web between the flange centres, nearer the stiffer flange, so that the flanges' shares of a lateral load balance. On the centre line the warping constant is h_o² I₁I₂/(I₁ + I₂), which is I_y h_o²/4 for equal flanges.

```calc
> Distance between flange centres
h_o = d - t_ft / 2 - t_fb / 2 = ?? mm
> Second moment of the top flange about the web
I_1 = t_ft b_ft^3 / 12 = ?? mm⁴
> Second moment of the bottom flange about the web
I_2 = t_fb b_fb^3 / 12 = ?? mm⁴
> St Venant torsion constant, thin-walled (DG9 Eq. 3.4)
J = (b_ft t_ft^3 + b_fb t_fb^3 + h_o t_w^3) / 3 = ?? mm⁴
> Warping constant, thin-walled
C_w = h_o^2 I_1 I_2 / (I_1 + I_2) = ?? mm⁶
> Shear centre above the centroid (negative: below)
y_o = d - t_ft / 2 - h_o I_2 / (I_1 + I_2) - y_c = ?? mm
> Polar radius of gyration about the shear centre, AISC Eq. E4-9 (x_o = 0)
r_o = sqrt(y_o^2 + (I_x + I_y) / A) = ?? mm
```

### Checks

```calc
@test t_ft + t_fb < d and t_w ≤ min(b_ft, b_fb)
```
