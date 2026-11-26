+++
cruncha-template = 1
id = "structural.frames.portalPinned"
version = "1.0.0"
title = "Portal frame, pinned bases: reactions, corner moments and drift"
summary = "A single-bay rigid portal frame on pinned bases under a uniform load on the beam and a lateral load at beam level: base reactions, the moments at the knees and mid-span, and the sway, in closed form."
category = ["Structural engineering", "Analysis"]
tags = ["frame", "portal frame", "rigid frame", "moment frame", "pinned base", "sway", "drift", "slope-deflection", "stiffness method"]
from = "lacewing:structural.frames"
license = "GPL-3.0-or-later"
inputs = ["L", "h", "I_b", "I_c", "E", "w", "P"]
outputs = ["k", "H_g", "H_A", "H_D", "V_A", "V_D", "M_B", "M_C", "M_mid", "Δ"]

[[references]]
work = "McGuire, Gallagher and Ziemian, Matrix Structural Analysis"
edition = "2nd (2000)"
locator = "Ch. 4 (the plane-frame member stiffness, used to check these closed forms)"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2d (fixed-end moment wL²/12 of the beam, the limit k → 0)"

[[tests]]
name = "Lacewing Lab: pinned-base portal under a lateral load splits it P/2 at each base"
source = "Lacewing Lab frame2d_test (pinned-base portal, h 4 m, b 6 m, P 20 kN, beam I twice the columns', beam axially rigid): H = −P/2 = −10 kN at each base, V = ∓Ph/b = ∓13.333 kN. Moments Ph/2 and drift Ph³(2k + 1)/(12EIc·k) by hand, checked against a stiffness solution"
inputs = { L = "6 m", h = "4 m", I_b = "1.6e8 mm^4", I_c = "8e7 mm^4", E = "200 GPa", w = "0 kN/m", P = "20 kN" }
expect = { k = "1.3333333", H_A = "-10 kN", H_D = "-10 kN", V_A = "-13.333333 kN", V_D = "13.333333 kN", M_B = "40 kN·m", M_C = "-40 kN·m", "Δ" = "18.333333 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "8 m × 4 m frame, k = 1, 10 kN/m and 20 kN"
source = "By hand: k = (2e8 × 4)/(1e8 × 8) = 1; Hg = 10 × 64/(4 × 4 × 5) = 8 kN; MB = −32 + 40 = 8 kN·m, MC = −32 − 40 = −72 kN·m, mid-span 80 − 32 = 48 kN·m; VA = 40 − 10 = 30 kN; Δ = 20 000 × 64 × 3/(12 × 200e9 × 1e-4) = 16 mm. A plane-frame stiffness solution (40 beam elements, axially rigid members) gives the same to 1e-6"
inputs = { L = "8 m", h = "4 m", I_b = "2e8 mm^4", I_c = "1e8 mm^4", E = "200 GPa", w = "10 kN/m", P = "20 kN" }
expect = { k = "1", H_g = "8 kN", H_A = "-2 kN", H_D = "-18 kN", V_A = "30 kN", V_D = "50 kN", M_B = "8 kN·m", M_C = "-72 kN·m", M_mid = "48 kN·m", "Δ" = "16 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Gravity only: knee moment wL²/(4(2k + 3))"
source = "By hand: k = 1, w = 10 kN/m over 8 m: Hg = 640/(16 × 5) = 8 kN, MB = MC = −32 kN·m, VA = VD = 40 kN, no sway"
inputs = { L = "8 m", h = "4 m", I_b = "2e8 mm^4", I_c = "1e8 mm^4", E = "200 GPa", w = "10 kN/m", P = "0 kN" }
expect = { H_A = "8 kN", H_D = "-8 kN", V_A = "40 kN", M_B = "-32 kN·m", M_C = "-32 kN·m", M_mid = "48 kN·m", "Δ" = "0 mm" }
tolerance = "1e-9"
checks = "pass"
+++

## Portal frame, pinned bases

A single-bay, single-storey rigid frame: two columns of height h on pinned bases A and D, joined rigidly at the knees B and C by a beam of span L. The beam carries a uniform load w over its whole span, and a lateral load P acts at beam level, from A's side towards D. The full frame calculator (any nodes, members, hinges and loads, by the stiffness method) is a solver this template doesn't hold; this is the closed form for the commonest case.

Assumptions: prismatic members, both columns alike; linear elastic, small displacements, no P-Δ effect; axial and shear deformation neglected (so the knees don't move vertically and the lateral load divides equally between the bases). The gravity case is symmetric and doesn't sway; the lateral case is antisymmetric. Their results are added.

Signs: reactions positive to the right (H) and upward (V); moments positive when they put the inside face of the frame in tension. The knees are hogging (negative) under gravity.

### Frame and loads

```calc {description="Span of the beam, column centre lines"}
L = 8 m_
```

```calc {description="Height of the columns, base to beam centre line"}
h = 4 m_
```

```calc {description="Second moment of area of the beam"}
I_b = 2e8 mm⁴_
```

```calc {description="Second moment of area of each column"}
I_c = 1e8 mm⁴_
```

```calc {description="Modulus of elasticity (for the drift only)"}
E = 200 GPa_
```

```calc {description="Uniform load on the beam, downward"}
w = 10 kN/m_
```

```calc {description="Lateral load at beam level, towards D"}
P = 20 kN_
```

### Relative stiffness

```calc {description="Stiffness of the beam relative to the columns, (I_b/L)/(I_c/h)"}
k = I_b h / (I_c L) = ??
```

### Gravity load on the beam

The frame spreads under the beam load, and the bases push back inwards with the thrust H_g. Each knee carries H_g h, hogging. As k grows (a stiff beam on slender columns) the beam tends to simply supported; as k → 0, to fixed-ended (wL²/12).

```calc
> Horizontal thrust at each base
H_g = w L^2 / (4 h (2 k + 3)) = ?? kN
> Moment at each knee from gravity (hogging)
M_(B,g) = -H_g h = ?? kN·m
> Moment at mid-span of the beam (sagging)
M_mid = w L^2 / 8 + M_(B,g) = ?? kN·m
```

### Lateral load

With pinned bases the shear in each column is P/2, so each knee carries Ph/2; overturning is resisted by a couple of vertical reactions.

```calc
> Vertical reaction from overturning, down at A and up at D
V_lat = P h / L = ?? kN
> Moment at each knee from the lateral load
M_(B,lat) = P h / 2 = ?? kN·m
> Sway of the beam (drift)
Δ = P h^3 (2 k + 1) / (12 E I_c k) = ?? mm
```

### Combined

```calc
> Horizontal reaction at A (positive to the right)
H_A = H_g - P / 2 = ?? kN
> Horizontal reaction at D (positive to the right)
H_D = -H_g - P / 2 = ?? kN
> Vertical reaction at A
V_A = w L / 2 - V_lat = ?? kN
> Vertical reaction at D
V_D = w L / 2 + V_lat = ?? kN
> Moment at knee B, windward (positive: tension inside)
M_B = M_(B,g) + M_(B,lat) = ?? kN·m
> Moment at knee C, leeward (positive: tension inside)
M_C = M_(B,g) - M_(B,lat) = ?? kN·m
```

The lateral load adds nothing at mid-span, but it moves the beam's largest sagging moment towards the windward knee; with a large P, check the beam's moment diagram between the knees. A negative V_A means the base must hold the column down.

### Checks

```calc
@test h > 0 'm' and L > 0 'm'
```
