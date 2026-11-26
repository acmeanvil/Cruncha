+++
cruncha-template = 1
id = "structural.plateBuckling.compressionSS"
version = "1.0.0"
title = "Plate buckling: uniform compression, all edges simply supported"
summary = "Elastic critical stress of a flat rectangular plate compressed uniformly on two opposite edges, all four edges simply supported, by Bryan's exact solution, and whether it yields before it buckles."
category = ["Structural engineering", "Members"]
tags = ["plate buckling", "elastic buckling", "Bryan", "Roark", "Table 15.2", "compression", "simply supported", "critical stress", "plate", "stability"]
from = "lacewing:structural.plateBuckling"
license = "GPL-3.0-or-later"
inputs = ["a", "b", "t", "E", "ν", "F_y"]
outputs = ["K", "σ_cr", "N_cr", "ρ_y"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 15.2; Table 15.2, case 1a"

[[references]]
work = "Timoshenko and Gere, Theory of Elastic Stability"
edition = "2nd"
locator = "Sec. 9.2 (buckling of simply supported rectangular plates uniformly compressed in one direction)"

[[tests]]
name = "Lacewing's opening case: 500 × 250 × 6 mm steel plate"
source = "Lacewing Lab help (structural.plateBuckling): a/b = 2, K 3.29, σ′ about 417 MPa, above Fy 250 MPa. By hand: K = (π²/12)(1 + 1)² = 3.2899; σ′ = 3.2899 × 200 000 / 0.91 × (6/250)² = 416.48 MPa"
inputs = { a = "500 mm", b = "250 mm", t = "6 mm", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { K = "3.289868", "σ_cr" = "416.476 MPa", "ρ_y" = "1.66590" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "10 mm plate, 500 × 250 mm"
source = "Lacewing Lab connections_test: K 3.29, σ′ = 3.29 × 200e9 / 0.91 × (0.01/0.25)² ≈ 1157 MPa; by hand 3.289868 × 219 780.2 MPa × 0.0016 = 1156.88 MPa"
inputs = { a = "500 mm", b = "250 mm", t = "10 mm", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { "σ_cr" = "1156.877 MPa", N_cr = "11568.77 N/mm" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Short plate a/b = 0.2: one half-wave, K 22.2"
source = "Roark 7th ed. Table 15.2 case 1a gives K 22.2 at a/b 0.2 (Roark 7th ed. p. 730, checked against the PDF); Bryan with m = 1: (π²/12)(5 + 0.2)² = 22.240"
inputs = { a = "50 mm", b = "250 mm", t = "6 mm", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { K = "22.2" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "a/b = 1.4 between half-wave counts: K 3.68"
source = "Roark 7th ed. Table 15.2 case 1a, K 3.68 at a/b 1.4 (Roark 7th ed. p. 730, checked against the PDF); Bryan with m = 1: (π²/12)(1/1.4 + 1.4)² = 3.6766"
inputs = { a = "350 mm", b = "250 mm", t = "6 mm", E = "200 GPa", "ν" = "0.3", F_y = "250 MPa" }
expect = { K = "3.68" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Thin aluminium plate buckles elastically before yield"
source = "By hand: a/b = 3, K = (π²/12) × 4 = 3.2899; σ′ = 3.2899 × 70 000 / (1 − 0.33²) × (3/300)² = 3.2899 × 78 551.2 × 1e-4 = 25.843 MPa; ρ = 25.843 / 240 = 0.1077"
inputs = { a = "900 mm", b = "300 mm", t = "3 mm", E = "70 GPa", "ν" = "0.33", F_y = "240 MPa" }
expect = { "σ_cr" = "25.843 MPa", "ρ_y" = "0.10768" }
tolerance = "1e-4"
checks = "fail"
+++

## Plate buckling: uniform compression, all edges simply supported

The elastic buckling stress of a flat rectangular plate of uniform thickness, compressed uniformly in its plane across the edges of width b, with all four edges simply supported (Roark Table 15.2, case 1a). The plate buckles into m half-waves along its length a; the buckling coefficient K is Bryan's exact solution, the least over whole numbers m, which is what the table tabulates.

The result is elastic. If σ′ is above the yield stress the plate yields (or buckles inelastically) first and σ′ does not apply. Post-buckling strength, initial out-of-flatness, residual stresses and plasticity are not included; for a plate element of a steel or aluminium member use the specification's width-to-thickness limits and effective widths instead. Other edge conditions, bending and shear are in the general plate buckling template.

### Plate

```calc {description="Plate length in the direction of the compressive stress"}
a = 500 mm_
```

```calc {description="Plate width: the loaded edge"}
b = 250 mm_
```

```calc {description="Plate thickness"}
t = 6 mm_
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

### Buckling coefficient

The term (m b/a + a/(m b))² is least for the whole number of half-waves m next to a/b, so only the two neighbours need to be compared.

```calc
> Aspect ratio
α = a / b = ??
> Half-waves on either side of a/b
m_1 = max(floor(α), 1) = ??
m_2 = m_1 + 1 = ??
> Buckling coefficient, Bryan (Roark Table 15.2 case 1a)
K = π^2 / 12 min((m_1 / α + α / m_1)^2, (m_2 / α + α / m_2)^2) = ??
```

### Critical stress

```calc {description="Elastic critical compressive stress"}
σ_cr = K E / (1 - ν^2) (t / b)^2 = ?? MPa
```

```calc {description="Critical load per unit width of the loaded edge"}
N_cr = σ_cr t = ?? N/mm
```

```calc {description="Critical stress over yield: above 1, the plate yields first"}
ρ_y = σ_cr / F_y = ??
```

### Check

The test passes when the plate yields before it buckles elastically, so that the elastic buckling stress is not the limit.

```calc
@test σ_cr ≥ F_y
```
