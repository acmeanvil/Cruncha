+++
cruncha-template = 1
id = "structural.column.slenderness"
version = "1.0.0"
title = "Column effective length and slenderness"
summary = "Effective length K L and slenderness K L/r of a column, compared with the tangent slenderness between Euler's and Johnson's curves, AISC 360's elastic-buckling boundary 4.71√(E/Fy), and the preferred limit of 200."
category = ["Structural engineering", "Members"]
tags = ["column", "effective length", "K factor", "slenderness", "KL/r", "tangent point", "Euler", "Johnson", "AISC 360", "E2", "end conditions"]
from = "lacewing:structural.columnSlenderness"
license = "GPL-3.0-or-later"
inputs = ["K", "L", "r", "E", "F_y"]
outputs = ["L_c", "λ", "λ_1", "λ_e"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings, with Commentary"
edition = "2022"
locator = "Sec. E2 and its User Note (L_c/r ≤ 200); Sec. E3 (4.71√(E/Fy)); Commentary Appendix 7, Table C-A-7.1 (K for ideal end conditions)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 4-13, Eq. 4-45 (tangent slenderness); Table 4-3"

[[tests]]
name = "W8×31, 4 m, both ends pinned"
source = "Lacewing Lab's opening case (help, structural.columnSlenderness). By hand: ry 2.02 in (AISC Manual Table 1-1) = 51.308 mm; KL/r = 4000/51.308 = 77.961; (KL/r)₁ = π √(2 × 200 000/345) = 106.972; 4.71 √(200 000/345) = 113.403"
inputs = { K = "1.0", L = "4 m", r = "2.02 in", E = "200 GPa", F_y = "345 MPa" }
expect = { L_c = "4 m", "λ" = "77.9606", "λ_1" = "106.972", "λ_e" = "113.403" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Tangent slenderness at Sy 75 kpsi, E 30 Mpsi: 88.9"
source = "Shigley 10th ed. Table 4-3 / Ex. 4-19, (l/k)₁ = 88.9 (Shigley 10th ed. p. 202, checked against the PDF); π √(2 × 30e6/75e3) = 88.858"
inputs = { K = "1", L = "15 in", r = "0.180422 in", E = "30000000 psi", F_y = "75000 psi" }
expect = { "λ_1" = "88.9", "λ" = "83.14" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Flagpole bar too slender: K L/r over 200"
source = "By hand: recommended K 2.1 (Table C-A-7.1, fixed-free); 25 mm round bar, r = 6.25 mm; K L/r = 2.1 × 1000/6.25 = 336"
inputs = { K = "2.1", L = "1 m", r = "6.25 mm", E = "200 GPa", F_y = "250 MPa" }
expect = { L_c = "2.1 m", "λ" = "336" }
tolerance = "1e-9"
checks = "fail"
+++

## Column effective length and slenderness

The effective length L_c = K L is the length of the pin-ended column that buckles at the same load. K comes from the end conditions or, for a column in a frame, from the alignment charts (see the alignment-chart template).

For the ideal end conditions, Commentary Table C-A-7.1 gives a theoretical K and a recommended design value that allows for ends never being fully fixed:

- both ends fixed: 0.5 (0.65 recommended);
- one end fixed, one pinned: 0.7 (0.8);
- both ends pinned: 1.0 (1.0);
- one end fixed, the other fixed in rotation but free to translate: 1.0 (1.2);
- one end fixed, one free (flagpole): 2.0 (2.1);
- one end pinned, the other fixed in rotation but free to translate: 2.0 (2.0).

Use the least radius of gyration (or the one about the axis the column buckles about) and the effective length for that axis. The slenderness is compared with two boundaries: the tangent slenderness (KL/r)₁ = √(2π²E/S_y), above which Euler's curve applies and below which Johnson's parabola does (Shigley); and 4.71√(E/F_y), above which AISC 360 E3 treats buckling as elastic.

### Column

```calc {description="Effective length factor"}
K = 1.0_
```

```calc {description="Length between ends or braces"}
L = 4 m_
```

```calc {description="Radius of gyration about the buckling axis (least)"}
r = 2.02 in_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Yield stress"}
F_y = 345 MPa_
```

### Slenderness

```calc {description="Effective length"}
L_c = K L = ?? m
```

```calc {description="Slenderness ratio"}
λ = L_c / r = ??
```

```calc {description="Tangent slenderness between Johnson and Euler, Shigley Eq. 4-45"}
λ_1 = sqrt(2 π^2 E / F_y) = ??
```

```calc {description="Boundary of elastic buckling, AISC 360 E3"}
λ_e = 4.71 sqrt(E / F_y) = ??
```

### Check

The AISC 360 E2 User Note prefers compression members no more slender than 200.

```calc
@test λ ≤ 200
```
