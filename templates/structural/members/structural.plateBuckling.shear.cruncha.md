+++
cruncha-template = 1
id = "structural.plateBuckling.shear"
version = "1.0.0"
title = "Plate buckling in shear (K given)"
summary = "Elastic critical shear stress of a flat rectangular plate with all edges simply supported or clamped, for a buckling coefficient K read at its aspect ratio, compared with the shear yield stress by von Mises."
category = ["Structural engineering", "Members"]
tags = ["plate buckling", "shear buckling", "elastic buckling", "Roark", "Table 15.2", "shear", "web", "panel", "plate", "stability"]
from = "lacewing:structural.plateBuckling"
license = "GPL-3.0-or-later"
inputs = ["a", "b", "t", "K", "E", "ν", "F_y"]
outputs = ["b_s", "τ_cr", "τ_y", "ρ_y"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 15.2; Table 15.2, cases 4a (all edges simply supported) and 4b (all edges clamped)"

[[tests]]
name = "Case 4a, 500 × 250 × 6 mm steel panel: K 5.43"
source = "Roark 7th ed. Table 15.2 case 4a, K 5.43 at a/b 2 (Roark 7th ed. p. 732, checked against the PDF). By hand: τ′ = 5.43 × 200 000 / 0.91 × (6/250)² = 687.40 MPa; τy = 250/√3 = 144.34 MPa; ρ = 4.7625"
inputs = { a = "500 mm", b = "250 mm", t = "6 mm", K = "5.43", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { b_s = "250 mm", "τ_cr" = "687.402 MPa", "ρ_y" = "4.76246" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "The same panel entered the other way round"
source = "Shear uses the shorter side whichever is entered as b (Lacewing Lab connections_test: the ratio of long to short); same numbers as above"
inputs = { a = "250 mm", b = "500 mm", t = "6 mm", K = "5.43", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { b_s = "250 mm", "τ_cr" = "687.402 MPa" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Thin square web panel, 4a: K 7.75, buckles before shear yield"
source = "Roark 7th ed. Table 15.2 case 4a, K 7.75 at a/b 1 (Roark 7th ed. p. 732, checked against the PDF). By hand: τ′ = 7.75 × 219 780.2 × (2/300)² = 75.702 MPa; ρ = 75.702/144.338 = 0.52448"
inputs = { a = "300 mm", b = "300 mm", t = "2 mm", K = "7.75", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { "τ_cr" = "75.7021 MPa", "ρ_y" = "0.524480" }
tolerance = "1e-5"
checks = "fail"
+++

## Plate buckling in shear

The elastic buckling stress of a flat rectangular plate under uniform shear on all four edges, τ′ = K E/(1 − ν²) (t/b)², with b the **shorter** side (Roark Table 15.2, cases 4a and 4b). Read K for the edge condition (all simply supported, 4a; all clamped, 4b) at the ratio of the longer to the shorter side.

The result is elastic and compared with the shear yield stress F_y/√3 (von Mises). Above 1, the panel yields in shear first. Tension-field action, which lets a stiffened web carry more than τ′, is not included: for webs of steel members use AISC 360 Chapter G.

### Panel

```calc {description="Panel side a"}
a = 500 mm_
```

```calc {description="Panel side b"}
b = 250 mm_
```

```calc {description="Plate thickness"}
t = 6 mm_
```

```calc {description="Shear buckling coefficient at the long/short ratio, Roark Table 15.2"}
K = 5.43_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc {description="Yield stress"}
F_y = 250 MPa_
```

### Critical stress

```calc
> Shorter side
b_s = min(a, b) = ?? mm
> Longer over shorter side, to read K
α = max(a, b) / b_s = ??
```

```calc {description="Elastic critical shear stress"}
τ_cr = K E / (1 - ν^2) (t / b_s)^2 = ?? MPa
```

```calc {description="Shear yield stress, von Mises"}
τ_y = F_y / sqrt(3) = ?? MPa
```

```calc {description="Critical shear over shear yield: above 1, the panel yields first"}
ρ_y = τ_cr / τ_y = ??
```

### Check

The test passes when the panel yields in shear before it buckles elastically.

```calc
@test τ_cr ≥ τ_y
```
