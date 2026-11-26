+++
cruncha-template = 1
id = "structural.plateBuckling.general"
version = "1.0.0"
title = "Plate buckling: compression or in-plane bending, any edges (K given)"
summary = "Elastic critical stress of a flat rectangular plate under uniform compression or in-plane bending for a buckling coefficient K read for its edge supports and aspect ratio, the critical edge load, and whether it yields first."
category = ["Structural engineering", "Members"]
tags = ["plate buckling", "elastic buckling", "Roark", "Table 15.2", "compression", "bending", "clamped", "free edge", "buckling coefficient", "plate", "stability"]
from = "lacewing:structural.plateBuckling"
license = "GPL-3.0-or-later"
inputs = ["a", "b", "t", "K", "E", "ν", "F_y"]
outputs = ["σ_cr", "N_cr", "ρ_y"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 15.2; Table 15.2, cases 1a to 1f (uniform compression) and 3a (pure bending in the plane)"

[[tests]]
name = "Case 1b, all edges clamped, square plate: K 7.7"
source = "Roark 7th ed. Table 15.2 case 1b, K 7.7 at a/b 1 (Roark 7th ed. p. 730, checked against the PDF). By hand: σ′ = 7.7 × 200 000 / 0.91 × (6/250)² = 974.77 MPa"
inputs = { a = "250 mm", b = "250 mm", t = "6 mm", K = "7.7", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { "σ_cr" = "974.769 MPa", N_cr = "5848.62 N/mm" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Case 3a, in-plane bending, a/b 2/3: K 19.7"
source = "Roark 7th ed. Table 15.2 case 3a, K 19.7 at a/b 0.667 (Roark 7th ed. p. 731, checked against the PDF). By hand: σ′ = 19.7 × 219 780.2 × (6/250)² = 2493.89 MPa at the extreme fibre"
inputs = { a = "166.7 mm", b = "250 mm", t = "6 mm", K = "19.7", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { "σ_cr" = "2493.890 MPa" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Case 1d, one side free, long thin outstand: buckles before yield"
source = "Roark 7th ed. Table 15.2 case 1d, K 0.425 at a/b 4 (Roark 7th ed. p. 731, checked against the PDF). By hand: σ′ = 0.425 × 219 780.2 × (3/100)² = 84.066 MPa; ρ = 84.066/250 = 0.33626"
inputs = { a = "400 mm", b = "100 mm", t = "3 mm", K = "0.425", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { "σ_cr" = "84.066 MPa", "ρ_y" = "0.336264" }
tolerance = "1e-5"
checks = "fail"
+++

## Plate buckling: compression or in-plane bending

The elastic buckling stress of a flat rectangular plate of uniform thickness loaded in its plane, σ′ = K E/(1 − ν²) (t/b)², in the form of Roark's Table 15.2. a is the length in the direction of the stress and b the width of the loaded edge. K depends on the loading, on how each edge is supported, and on a/b: read it from Table 15.2 for the case (1a to 1f, uniform compression with simply supported, clamped or free sides; 3a, pure bending in the plane with all edges simply supported). For case 1a, the compression template with all edges simply supported computes K exactly; shear has its own template.

In bending, σ′ is the extreme-fibre stress on the loaded edge at buckling. The result is elastic: a stocky plate yields first and σ′ does not apply. Post-buckling strength, imperfections and plasticity are not included.

### Plate

```calc {description="Plate length in the direction of the stress"}
a = 500 mm_
```

```calc {description="Plate width: the loaded edge"}
b = 250 mm_
```

```calc {description="Plate thickness"}
t = 6 mm_
```

```calc {description="Buckling coefficient for the case and a/b, Roark Table 15.2"}
K = 3.29_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc {description="Yield stress, for comparison"}
F_y = 250 MPa_
```

### Critical stress

```calc {description="Aspect ratio, to read K"}
α = a / b = ??
```

```calc {description="Elastic critical stress on the loaded edge"}
σ_cr = K E / (1 - ν^2) (t / b)^2 = ?? MPa
```

```calc {description="Critical load per unit width of the loaded edge (uniform compression)"}
N_cr = σ_cr t = ?? N/mm
```

```calc {description="Critical stress over yield: above 1, the plate yields first"}
ρ_y = σ_cr / F_y = ??
```

### Check

The test passes when the plate yields before it buckles elastically.

```calc
@test σ_cr ≥ F_y
```
