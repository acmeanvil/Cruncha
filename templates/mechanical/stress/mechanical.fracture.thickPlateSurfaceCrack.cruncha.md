+++
cruncha-template = 1
id = "mechanical.fracture.thickPlateSurfaceCrack"
version = "1.0.1"
title = "Fracture: surface crack in a thick plate (largest flaw, allowable stress)"
summary = "A semi-elliptical surface crack of depth a in a thick plate, Juvinall's Eq. 6.4 with its plasticity correction: the stress intensity and safety factor of a flaw, the largest flaw the stress allows, and the stress at which the flaw runs."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["fracture", "LEFM", "surface crack", "thumbnail crack", "thick plate", "fracture toughness", "KIc", "critical crack", "flaw size", "allowable stress"]
from = "lacewing:mechanical.kIcSolver"
license = "GPL-3.0-or-later"
inputs = ["a", "t", "σ", "K_Ic", "S_y", "n_d"]
outputs = ["Y", "K_I", "n", "a_cr", "σ_c"]

[[references]]
work = "Juvinall and Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 6.4, Eq. 6.4 (p. 255); Sample Problem 6.2 (p. 256); Table 6.1"

[[tests]]
name = "Juvinall SP 6.2: Ti-6Al-4V plate at 0.73 Sy, largest flaw 0.20 in"
source = "Juvinall 5th Sample Problem 6.2 (p. 256), Lacewing Lab fracture_test and fracture_solve_test: Table 6.1's KIc 65 ksi·√in, Sy 120 ksi, σ = 87.6 ksi; a_cr = (0.39 − 0.053 × 0.73²)(65/87.6)² = 0.1992 in. A 0.1 in flaw: KI = 46.06 ksi·√in"
inputs = { a = "0.1 in", t = "1 in", "σ" = "87.6 ksi", K_Ic = "65 ksi·√in", S_y = "120 ksi", n_d = "1.2" }
expect = { a_cr = "0.1992 in", K_I = "46.06 ksi·√in", n = "1.4113", Y = "0.93803" }
tolerance = "0.02%"
checks = "pass"

[[tests]]
name = "Thick SI plate: 5 mm flaw at 600 MPa, Sy 1000 MPa, KIc 80 MPa·√m"
source = "By hand, Eq. 6.4: q = 0.39 − 0.053 × 0.6² = 0.37092; KI = 600 √0.005/√q = 69.66 MPa·√m; a_cr = q (80/600)² = 6.594 mm; σc = √(0.39 × 80²/(0.005 + 0.053 × 80²/1000²)) = 683.7 MPa"
inputs = { a = "5 mm", t = "50 mm", "σ" = "600 MPa", K_Ic = "80 MPa·√m", S_y = "1000 MPa", n_d = "1.1" }
expect = { K_I = "69.662 MPa·√m", a_cr = "6.5941 mm", "σ_c" = "683.73 MPa" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Juvinall SP 6.2 plate with a 0.25 in flaw: it runs"
source = "Juvinall 5th Sample Problem 6.2 (p. 256): the flaw is deeper than the 0.20 in critical depth; by hand KI = 87.6 √0.25/√0.36176 = 72.82 ksi·√in > 65"
inputs = { a = "0.25 in", t = "1 in", "σ" = "87.6 ksi", K_Ic = "65 ksi·√in", S_y = "120 ksi", n_d = "1" }
expect = { K_I = "72.823 ksi·√in", "σ_c" = "78.772 ksi" }
tolerance = "0.01%"
checks = "fail"
+++

## Fracture: surface crack in a thick plate

A semi-elliptical surface (thumbnail) crack of depth a, about twice as long at the surface as it is deep (a/2c ≈ 0.25), in a plate under tension. Juvinall's Eq. 6.4 gives its stress intensity, including a correction for crack-tip plasticity that depends on σ/S_y:

K_I = σ√a / √(0.39 − 0.053 (σ/S_y)²)

The equation holds for plates more than six times as wide as they are thick (2w/t > 6) and more than three times as wide as the crack is long (w/c > 3, with 2w the width and 2c the crack's length), for cracks less than half the thickness deep (a/t < 0.5), and for σ/S_y below 0.8. The first two are for the engineer to confirm; the last two are checks below. Because the correction depends on σ, the stress at which a flaw runs is Eq. 6.4 solved for σ, not a simple ratio of K's.

For other cracks, or a geometry factor read from a chart, use the critical crack template.

### Crack, plate and material

```calc {description="Depth of the surface crack (the flaw found, or the largest inspection can miss)"}
a = 0.1 in_
```

```calc {description="Plate thickness"}
t = 1 in_
```

```calc {description="Nominal tensile stress remote from the crack"}
σ = 87.6 ksi_
```

KIc is the plane-strain fracture toughness from a test to ASTM E399, or a typical value from Juvinall Table 6.1 or Shigley Table 5-1 (certify the actual material).

```calc {description="Plane-strain fracture toughness"}
K_Ic = 65 ksi·√in_
```

```calc {description="Yield strength"}
S_y = 120 ksi_
```

```calc {description="Design factor against fast fracture"}
n_d = 1.2_
```

### Stress intensity of the flaw

```calc
> Plasticity-corrected shape term of Eq. 6.4
q = 0.39 - 0.053 (σ / S_y)^2 = ??
> The equivalent geometry factor, K = Yσ√(πa)
Y = 1 / sqrt(π q) = ??
> Stress intensity, Eq. 6.4
K_I = σ sqrt(a / q) = ?? ksi·√in
> Safety factor against fast fracture
n = K_Ic / K_I = ??
```

### The largest flaw and the allowable stress

```calc
> Largest flaw the applied stress allows: Eq. 6.4 with K = KIc, as Sample Problem 6.2
a_cr = q (K_Ic / σ)^2 = ?? in
> Stress at which this flaw runs: Eq. 6.4 with K = KIc, solved for σ
σ_c = sqrt(0.39 K_Ic^2 / (a + 0.053 K_Ic^2 / S_y^2)) = ?? ksi
```

### Checks

```calc {description="Eq. 6.4 holds for σ/Sy below 0.8"}
@test σ ≤ 0.8 S_y
```

```calc {description="Eq. 6.4 holds for cracks less than half the thickness deep"}
@test a ≤ 0.5 t
```

```calc {description="Design factor against fast fracture"}
@test n ≥ n_d
```
