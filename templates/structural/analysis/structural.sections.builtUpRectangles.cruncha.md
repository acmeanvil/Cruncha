+++
cruncha-template = 1
id = "structural.sections.builtUpRectangles"
version = "1.0.0"
title = "Built-up section from rectangles: area, centroid, I, S, Z, principal axes and J"
summary = "Section properties of any section made of non-overlapping rectangular plates aligned with the axes (a built-up girder, a tee, an angle, a box, plates added to a member): area, centroid, second moments and product of inertia, principal axes, elastic and plastic moduli, radii of gyration and an approximate torsion constant."
category = ["Structural engineering", "Analysis"]
tags = ["section properties", "built-up section", "composite area", "parallel axis", "plastic modulus", "principal axes", "product of inertia", "angle", "tee", "plate girder", "torsion constant"]
from = "lacewing:structural.sectionBuilder"
license = "GPL-3.0-or-later"
inputs = ["b", "h", "x", "y"]
outputs = ["A", "x_c", "y_c", "I_x", "I_y", "I_xy", "I_u", "I_v", "θ_p", "S_xt", "S_xb", "S_yl", "S_yr", "r_x", "r_y", "r_v", "y_p", "Z_x", "x_p", "Z_y", "J"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table A.1 (properties of plane areas: the rectangle, parallel axes and principal axes); Table 10.1, case 4 (solid rectangular section, K = ab³[16/3 − 3.36(b/a)(1 − b⁴/12a⁴)]), p. 401"

[[references]]
work = "AISC Design Guide 9, Torsional Analysis of Structural Steel Members (Seaburg, Carter)"
edition = "2003"
locator = "Sec. 3.2.1, Eq. 3.4 (J ≈ Σbt³/3 for open sections)"

[[tests]]
name = "300 mm I-beam from three plates: the same as the I-section template"
source = "By hand (as the I-section template's first case): A 4680 mm², Ix 74.076e6 mm⁴, Iy 5.63004e6 mm⁴, Zx 552 600 mm³, Zy 115 020 mm³; J by Roark's rectangle formula for each plate, Σ l s³ (1/3 − 0.21 (s/l)(1 − s⁴/12l⁴)) = 115 688 mm⁴ (Σbt³/3 gives 120 880)"
inputs = { b = "[150; 6; 150] mm", h = "[10; 280; 10] mm", x = "[75; 75; 75] mm", y = "[5; 150; 295] mm" }
expect = { A = "4680 mm^2", x_c = "75 mm", y_c = "150 mm", I_x = "74076000 mm^4", I_y = "5630040 mm^4", I_xy = "0 mm^4", S_xt = "493840 mm^3", y_p = "150 mm", Z_x = "552600 mm^3", Z_y = "115020 mm^3", r_x = "125.81019 mm", J = "115687.85 mm^4" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Equal-leg angle 100 × 100 × 10, sharp corners: principal axes at 45°"
source = "By hand: legs 10 × 100 (vertical) and 90 × 10 (horizontal): A = 1900 mm², centroid 28.684 mm from each back, Ix = Iy = 1.80004e6 mm⁴, Ixy = −1.06579e6 mm⁴, Iu = 2.86583e6, Iv = 0.734254e6 mm⁴ at 45°, rv = 19.658 mm; PNA 9.5 mm from the back, Zx = 45 475 mm³. Rolled L100×100×10 tables (with fillets) are close: about 19.2 cm², 177 cm⁴ and a centroid at about 28 mm"
inputs = { b = "[10; 90] mm", h = "[100; 10] mm", x = "[5; 55] mm", y = "[50; 5] mm" }
expect = { A = "1900 mm^2", x_c = "28.684211 mm", y_c = "28.684211 mm", I_x = "1800043.9 mm^4", I_xy = "-1065789.5 mm^4", I_u = "2865833.3 mm^4", I_v = "734254.39 mm^4", "θ_p" = "45°", S_xt = "25240.467 mm^3", S_xb = "62753.823 mm^3", r_v = "19.658323 mm", y_p = "9.5 mm", Z_x = "45475 mm^3" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Tee 200 × 20 flange on a 180 × 12 stem: PNA in the flange"
source = "By hand: A = 6160 mm², ȳ = 154.935 mm from the stem tip, Ix = 19.9913e6 mm⁴, Sx to the stem tip 129 030 mm³; the flange holds 4000 of 6160 mm², so the PNA is in it at 200 − 3080/200 = 184.6 mm, Zx = 230 168 mm³"
inputs = { b = "[200; 12] mm", h = "[20; 180] mm", x = "[0; 0] mm", y = "[190; 90] mm" }
expect = { A = "6160 mm^2", y_c = "154.93506 mm", I_x = "19991307 mm^4", I_y = "13359253 mm^4", S_xb = "129030.23 mm^3", S_xt = "443611.14 mm^3", y_p = "184.6 mm", Z_x = "230168 mm^3", Z_y = "206480 mm^3" }
tolerance = "1e-6"
checks = "pass"
+++

## Built-up section from rectangles

The properties of a cross-section made of rectangular plates whose sides are parallel to the x (horizontal) and y (vertical) axes: a welded girder, a tee or angle with sharp corners, a box of four plates, or plates added to a member. Each plate is given by its width b (along x), its height h (along y) and the position x, y of its centre from any datum; list them in the same order in the four columns. The plates must not overlap (overlapping area would be counted twice) and holes can't be subtracted. Fillets, welds and rounded corners are left out.

The second moments follow from each plate's own centroidal values and the parallel-axis theorem (Roark Table A.1). The plastic neutral axes, which divide the area in half, are found by bisection. The torsion constant J is the sum of each plate's own constant (Roark Table 10.1, case 4), which suits open sections of thin plates (DG9 Eq. 3.4); it does not apply to closed sections such as a box, whose J is far larger.

Lacewing Lab's section builder also has channels, zees, tubes and cold-formed shapes with fillets and bends, the warping constant and the shear centre; that interactive builder is not in this template. The I-section template gives Cw and the shear centre of an I with unequal flanges.

### Plates

```calc {description="Width of each plate, along x"}
b = [150; 6; 150] mm_
```

```calc {description="Height of each plate, along y"}
h = [10; 280; 10] mm_
```

```calc {description="x of each plate's centre from the datum"}
x = [75; 75; 75] mm_
```

```calc {description="y of each plate's centre from the datum"}
y = [5; 150; 295] mm_
```

### Area and centroid

```calc
> Area of each plate
a_i = b ∘ h = ?? mm²
> Total area
A = sum(a_i) = ?? mm²
> Centroid, x from the datum
x_c = sum(a_i ∘ x) / A = ?? mm
> Centroid, y from the datum
y_c = sum(a_i ∘ y) / A = ?? mm
```

### Second moments about the centroid

```calc
> About the horizontal centroidal axis
I_x = sum(b ∘ h^3) / 12 + sum(a_i ∘ (y - y_c)^2) = ?? mm⁴
> About the vertical centroidal axis
I_y = sum(h ∘ b^3) / 12 + sum(a_i ∘ (x - x_c)^2) = ?? mm⁴
> Product of inertia (zero if either axis is an axis of symmetry)
I_xy = sum(a_i ∘ (x - x_c) ∘ (y - y_c)) = ?? mm⁴
```

### Principal axes

An unsymmetric section (an angle, a zee) bends most easily about its minor principal axis, v; an unbraced angle buckles about it. θ_p is the angle of the major axis u, counterclockwise from x.

```calc
> Major principal second moment
I_u = (I_x + I_y) / 2 + sqrt(((I_x - I_y) / 2)^2 + I_xy^2) = ?? mm⁴
> Minor principal second moment
I_v = (I_x + I_y) / 2 - sqrt(((I_x - I_y) / 2)^2 + I_xy^2) = ?? mm⁴
> Angle of the major principal axis from x
θ_p = atan2(I_x - I_y, -2 I_xy) / 2 = ?? °
```

### Elastic moduli and radii of gyration

```calc
> Top fibre, from the datum
y_top = max(y + h / 2) = ?? mm
> Bottom fibre
y_bot = min(y - h / 2) = ?? mm
> Right fibre
x_rt = max(x + b / 2) = ?? mm
> Left fibre
x_lf = min(x - b / 2) = ?? mm
```

```calc
> Elastic section modulus about x, to the top fibre
S_xt = I_x / (y_top - y_c) = ?? mm³
> Elastic section modulus about x, to the bottom fibre
S_xb = I_x / (y_c - y_bot) = ?? mm³
> Elastic section modulus about y, to the left fibre
S_yl = I_y / (x_c - x_lf) = ?? mm³
> Elastic section modulus about y, to the right fibre
S_yr = I_y / (x_rt - x_c) = ?? mm³
> Radius of gyration about x
r_x = sqrt(I_x / A) = ?? mm
> Radius of gyration about y
r_y = sqrt(I_y / A) = ?? mm
> Least radius of gyration, about the minor principal axis
r_v = sqrt(I_v / A) = ?? mm
```

### Plastic moduli

```calc
function areaBelow(w, c, s, p)
    total = 0 w[1] s[1]
    for i in 1:length(w)
        t = p - (c[i] - s[i] / 2)
        t = (t + abs(t)) / 2
        t = (t + s[i] - abs(t - s[i])) / 2
        total = total + w[i] t
    end
    return total
end
```

```calc
function plasticAxis(w, c, s, A)
    lo = min(c - s / 2)
    hi = max(c + s / 2)
    for k in 1:60
        m = (lo + hi) / 2
        if areaBelow(w, c, s, m) < A / 2
            lo = m
        else
            hi = m
        end
    end
    return (lo + hi) / 2
end
```

```calc
function plasticModulus(w, c, s, p)
    z = 0 w[1] s[1] s[1]
    for i in 1:length(w)
        u = c[i] + s[i] / 2 - p
        l = c[i] - s[i] / 2 - p
        z = z + w[i] / 2 (u abs(u) - l abs(l))
    end
    return z
end
```

```calc
> Plastic neutral axis for bending about x, y from the datum
y_p = plasticAxis(b, y, h, A) = ?? mm
> Plastic section modulus about x
Z_x = plasticModulus(b, y, h, y_p) = ?? mm³
> Plastic neutral axis for bending about y, x from the datum
x_p = plasticAxis(h, x, b, A) = ?? mm
> Plastic section modulus about y
Z_y = plasticModulus(h, x, b, x_p) = ?? mm³
```

### Torsion constant (open sections)

```calc
> Longer side of each plate
l_i = (b + h + abs(b - h)) / 2 = ?? mm
> Shorter side of each plate
s_i = (b + h - abs(b - h)) / 2 = ?? mm
> St Venant torsion constant, sum of the plates (Roark Table 10.1 case 4)
J = sum(l_i ∘ s_i^3 ∘ (1 / 3 - 0.21 s_i / l_i ∘ (1 - s_i^4 / (12 l_i^4)))) = ?? mm⁴
```

### Checks

```calc
@test min(b) > 0 'mm' and min(h) > 0 'mm'
```
