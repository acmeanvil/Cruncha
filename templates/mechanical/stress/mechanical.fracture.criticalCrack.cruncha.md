+++
cruncha-template = 1
id = "mechanical.fracture.criticalCrack"
version = "1.0.1"
title = "Fracture: stress intensity, critical crack and failure stress"
summary = "Linear-elastic fracture mechanics for a crack with a known geometry factor: K = Yσ√(πc) against the fracture toughness, the safety factor, the stress at which the crack runs, the largest crack the stress allows, the toughness needed for a design factor, and the plane-strain thickness."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["fracture", "LEFM", "stress intensity", "fracture toughness", "KIc", "critical crack", "flaw size", "failure stress", "plane strain", "crack"]
from = "lacewing:mechanical.kIcSolver"
license = "GPL-3.0-or-later"
inputs = ["Y", "c", "σ", "K_Ic", "S_y", "B", "n_d"]
outputs = ["K_I", "n", "σ_c", "c_cr", "K_(Ic,req)", "B_min"]

[[references]]
work = "Juvinall and Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 6.3-6.4 (pp. 250-263), Eqs. 6.1-6.2, Figs. 6.5a-h; Sample Problem 6.1"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 5-12 (pp. 255-262), Eqs. 5-37, 5-38; Examples 5-6, 5-7 and 6-1"

[[references]]
work = "W. T. Matthews, Plane Strain Fracture Toughness (KIc) Data Handbook for Metals, AMMRC"
edition = "1973"
locator = "Introduction: the ASTM E399 thickness 2.5(KIc/Sy)²"

[[tests]]
name = "Shigley Ex. 5-6: deck plate, 65 mm centre crack at 50 MPa"
source = "Shigley 10th Ex. 5-6 (p. 261), Lacewing Lab fracture_test: KI = 16.0 MPa·√m, n = 1.77; failure stress 88.4 MPa as the book rounds KI (88.57 unrounded)"
inputs = { Y = "1", c = "32.5 mm", "σ" = "50 MPa", K_Ic = "28.3 MPa·√m", n_d = "1.5" }
expect = { K_I = "15.976 MPa·√m", n = "1.771", "σ_c" = "88.57 MPa", "K_(Ic,req)" = "23.96 MPa·√m" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 5-7: Ti-6Al-4V, 2.7 mm edge crack, β = 1.1, KIc 115 MPa·√m"
source = "Shigley 10th Ex. 5-7 (p. 261), Lacewing Lab fracture_test: the crack runs at 1135 MPa"
inputs = { Y = "1.1", c = "2.7 mm", "σ" = "500 MPa", K_Ic = "115 MPa·√m", S_y = "910 MPa", n_d = "1.5" }
expect = { "σ_c" = "1135 MPa" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 5-7, the stronger alloy at KIc 55 MPa·√m: the same crack at 600 MPa runs"
source = "Shigley 10th Ex. 5-7 (p. 261), Lacewing Lab fracture_test: 542.9 MPa, so 600 MPa fails"
inputs = { Y = "1.1", c = "2.7 mm", "σ" = "600 MPa", K_Ic = "55 MPa·√m", S_y = "1035 MPa", n_d = "1" }
expect = { "σ_c" = "542.9 MPa", n = "0.9048" }
tolerance = "0.1%"
checks = "fail"

[[tests]]
name = "Juvinall SP 6.1: 7075-T651 sheet, 1 in. centre crack, plane-stress KIc 60 ksi·√in"
source = "Juvinall 5th Sample Problem 6.1 (p. 254), Lacewing Lab fracture_test: σ = 60/(√π·√0.5) = 47.87 ksi (the book's 47.14 rounds √π to 1.8)"
inputs = { Y = "1", c = "0.5 in", "σ" = "20 ksi", K_Ic = "60 ksi·√in", S_y = "70 ksi", B = "0.06 in", n_d = "2" }
expect = { "σ_c" = "47.87 ksi", B_min = "1.837 in" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 6-1: the final crack at 115.2 kpsi, KIc 73 ksi·√in, β = 1.07"
source = "Shigley 10th Ex. 6-1 (p. 289), Lacewing Lab fracture_solve_test: af = 0.112 in"
inputs = { Y = "1.07", c = "0.004 in", "σ" = "115.2 ksi", K_Ic = "73 ksi·√in", S_y = "160 ksi", n_d = "2" }
expect = { c_cr = "0.112 in" }
tolerance = "0.5%"
checks = "pass"
+++

## Fracture: stress intensity, critical crack and failure stress

A crack in a linear-elastic part runs when its stress intensity K reaches the material's fracture toughness. K = Yσ√(πc), where σ is the nominal (gross-section) stress remote from the crack, c the crack size as the geometry factor's chart measures it, and Y the geometry (configuration) factor. Juvinall writes Y; Shigley writes β. This template solves the relation each way: the stress intensity and its safety factor, the stress at which a known crack runs, the largest crack a known stress allows, and the toughness a crack needs.

The geometry factor Y depends on the crack and the part:

- **1** for a centre crack short against the plate's width (c the half-length);
- **about 1.12** for a short edge crack (c the depth);
- otherwise read Y from a chart for the configuration, at the chart's ratio (such as c/w): Juvinall Figs. 6.5a-h, Shigley Figs. 5-25 to 5-30, or Rooke and Cartwright's *Compendium of Stress Intensity Factors*.

Y here is constant, so the critical crack is found as if Y did not change while the crack grows. Where Y rises with c/w, read Y again at c_cr and repeat. For a surface crack in a thick plate, use the thick-plate template (Juvinall Eq. 6.4).

### Crack, stress and material

```calc {description="Geometry factor Y = KI/(σ√(πc)), from a chart for the crack's configuration"}
Y = 1_
```

```calc {description="Crack size as the chart measures it (half-length of a centre crack, depth of an edge crack)"}
c = 32.5 mm_
```

```calc {description="Nominal stress remote from the crack"}
σ = 50 MPa_
```

KIc is the plane-strain fracture toughness from a test to ASTM E399, or a typical value from Juvinall Table 6.1 or Shigley Table 5-1 (certify the actual material). A thin sheet is in plane stress and tougher than KIc, so KIc is conservative for it.

```calc {description="Plane-strain fracture toughness"}
K_Ic = 28.3 MPa·√m_
```

```calc {description="Yield strength"}
S_y = 250 MPa_
```

```calc {description="Thickness of the part at the crack"}
B = 25 mm_
```

```calc {description="Design factor against fast fracture"}
n_d = 1.5_
```

### Stress intensity and safety factor

```calc
> Stress intensity, Juvinall Eq. 6.1, Shigley Eq. 5-37
K_I = Y σ sqrt(π c) = ?? MPa·√m
> Safety factor against fast fracture, Shigley Eq. 5-38
n = K_Ic / K_I = ??
```

### The crack's limits

```calc
> Nominal stress at which this crack runs, Juvinall Eq. 6.2
σ_c = K_Ic / (Y sqrt(π c)) = ?? MPa
> Largest crack the applied stress allows (Y constant)
c_cr = (K_Ic / (Y σ))^2 / π = ?? mm
> Toughness this crack needs for the design factor
K_(Ic,req) = n_d K_I = ?? MPa·√m
```

If σ_c is above S_y, the part yields before the crack runs, and yielding governs.

### Plane strain

```calc {description="Thickness for plane strain, ASTM E399: thinner parts are tougher than KIc"}
B_min = 2.5 (K_Ic / S_y)^2 = ?? mm
```

### Check

```calc
@test n ≥ n_d
```
