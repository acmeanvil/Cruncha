+++
cruncha-template = 1
id = "mechanical.boltGroups.tension"
version = "1.0.0"
title = "Bolt group in tension and bending, rigid plate"
summary = "The tension on each fastener of a pattern under an out-of-plane load acting off the centroid and bending moments about two axes, with the members rigid and the fasteners elastic: the load shared by area plus a linear distribution about the pattern's centroidal axes (product of area included for an unsymmetric pattern), and the largest tension checked against an allowable."
category = ["Mechanical engineering", "Fastening"]
tags = ["bolt group", "bolt pattern", "tension", "bending", "moment", "out of plane", "centroid", "rigid plate", "flange", "Shigley"]
from = "lacewing:mechanical.boltGroups"
license = "GPL-3.0-or-later"
inputs = ["x_b", "y_b", "d_b", "P_g", "p_x", "p_y", "M_x", "M_y", "F_(t,all)"]
outputs = ["x_c", "y_c", "M_x′", "M_y′", "I_xx", "I_yy", "I_xy", "F_t", "F_(t,max)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-12, Eq. 8-56 (centroid of the pattern); the rigid-member, elastic-fastener assumption of Eq. 8-57 applied about the centroidal axes"

[[tests]]
name = "Four bolts at ±100 mm, 4 kN of tension and 400 N·m about x"
source = "Lacewing Lab bolted_joint_test (bolt group tension): each row takes M/(2 · 2y), so the top bolts carry 1000 + 400/(4 × 0.1) = 2000 N and the bottom ones nothing"
inputs = { x_b = "[-100; 100; -100; 100] mm", y_b = "[100; 100; -100; -100] mm", d_b = "12 mm", P_g = "4 kN", p_x = "0 mm", p_y = "0 mm", M_x = "400 N·m", M_y = "0 N·m", "F_(t,all)" = "5 kN" }
expect = { "F_(t,max)" = "2000 N", "M_x′" = "400 N·m" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "The same tension 100 mm off the centroid, with no applied moment"
source = "Lacewing Lab bolted_joint_test: an offset tension is the tension plus its moment, P·py = 4000 × 0.1 = 400 N·m, so again 2000 N"
inputs = { x_b = "[-100; 100; -100; 100] mm", y_b = "[100; 100; -100; -100] mm", d_b = "12 mm", P_g = "4 kN", p_x = "0 mm", p_y = "100 mm", M_x = "0 N·m", M_y = "0 N·m", "F_(t,all)" = "5 kN" }
expect = { "F_(t,max)" = "2000 N", "M_x′" = "400 N·m" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: an L of three bolts, 1 kN·m about x (unsymmetric)"
source = "By hand: bolts at (0, 0), (100, 0), (0, 100) mm, each 1 cm²; centroid (33.33, 33.33) mm; Ixx = Iyy = 100 × (2 × 33.33² + 66.67²) = 666 667 mm⁴, Ixy = −333 333 mm⁴; with My = 0 the bolts carry A(Mx·Iyy·dy − Mx·Ixy·dx)/det = −10, 0, +10 kN"
inputs = { x_b = "[0; 100; 0] mm", y_b = "[0; 0; 100] mm", d_b = "11.283792 mm", P_g = "0 kN", p_x = "0 mm", p_y = "0 mm", M_x = "1 kN·m", M_y = "0 N·m", "F_(t,all)" = "12 kN" }
expect = { x_c = "33.33333 mm", I_xx = "666666.7 mm^4", I_xy = "-333333.3 mm^4", "F_(t,max)" = "10 kN" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Hand check: the L with 6 kN at (50, 50) mm"
source = "By hand: the moments about the centroidal axes are 6000 × 0.01667 = 100 N·m about x and −100 N·m about y; the corner bolt is unloaded and the other two take 3 kN each"
inputs = { x_b = "[0; 100; 0] mm", y_b = "[0; 0; 100] mm", d_b = "11.283792 mm", P_g = "6 kN", p_x = "50 mm", p_y = "50 mm", M_x = "0 N·m", M_y = "0 N·m", "F_(t,all)" = "4 kN" }
expect = { "M_x′" = "100 N·m", "M_y′" = "-100 N·m", "F_(t,max)" = "3 kN" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "One row of three bolts along x, 3 kN and 300 N·m about y: the end bolt exceeds its allowable"
source = "By hand: a single row along x has Ixx = 0, so the one-axis form carries My: Iyy = 2A(0.1 m)², and the bolts at x = −100, 0, 100 mm carry 1000 + 300 × 0.1/(2 × 0.01) = 2500, 1000 and −500 N, over the 2 kN allowable"
inputs = { x_b = "[-100; 0; 100] mm", y_b = "[0; 0; 0] mm", d_b = "12 mm", P_g = "3 kN", p_x = "0 mm", p_y = "0 mm", M_x = "0 N·m", M_y = "300 N·m", "F_(t,all)" = "2 kN" }
expect = { "F_(t,max)" = "2500 N" }
tolerance = "1e-9"
checks = "fail"
+++

## Bolt group in tension and bending

The tension on each fastener of a pattern loaded out of its plane: a tension pulling the joint apart, acting anywhere, and bending moments about the two axes in the plane. As for a pattern in shear (Shigley Sec. 8-12), the connected parts are rigid and the fasteners elastic, so the members rotate about the pattern's centroidal axes and each fastener's load varies linearly with its position, in proportion to its area. An unsymmetric pattern's product of area is included, so the result is right for any layout.

This is an elastic share-out with the joint closed: preload, prying (heel-and-toe action at a flange edge, which can raise bolt loads well above these), and the shift of the neutral axis once part of the joint opens are not modelled. A negative result is a fastener the moment unloads; check that the preload keeps the joint closed there. For shear in the plane of the pattern, use the bolt group under eccentric shear template; for one bolt's preload and its factors of safety, the bolted joint in tension template.

### Pattern

Fastener positions as column vectors, in any axes in the plane of the joint. Fasteners share the load by the area of the diameter entered (one value, or one per fastener).

```calc {description="x coordinate of each fastener"}
x_b = [-100; 100; -100; 100] mm_
```

```calc {description="y coordinate of each fastener"}
y_b = [100; 100; -100; -100] mm_
```

```calc {description="Fastener diameter (one value, or one per fastener)"}
d_b = 12 mm_
```

### Loads

The moments follow the right-hand rule: a positive M_x puts the fasteners at +y in tension, a positive M_y those at −x.

```calc {description="Out-of-plane tension on the pattern"}
P_g = 4 kN_
```

```calc {description="x coordinate where the tension acts"}
p_x = 0 mm_
```

```calc {description="y coordinate where the tension acts"}
p_y = 0 mm_
```

```calc {description="Applied moment about the x axis"}
M_x = 400 N·m_
```

```calc {description="Applied moment about the y axis"}
M_y = 0 N·m_
```

```calc {description="Allowable tension on one fastener"}
F_(t,all) = 5 kN_
```

### Centroid and section properties of the pattern

```calc
> Number of fasteners
n_b = length(x_b) = ??
> Area of each fastener
A_i = π d_b^2 / 4 ∘ ones(n_b, 1) = ?? mm²
> Centroid of the pattern, Eq. 8-56
x_c = sum(A_i ∘ x_b) / sum(A_i) = ?? mm
y_c = sum(A_i ∘ y_b) / sum(A_i) = ?? mm
> Each fastener's offset from the centroid
Δx = x_b - x_c = ?? mm
Δy = y_b - y_c = ?? mm
> Moments about the centroidal axes, including the tension's offset
M_x′ = M_x + P_g (p_y - y_c) = ?? N·m
M_y′ = M_y - P_g (p_x - x_c) = ?? N·m
> Second moments and product of the fastener areas about the centroid
I_xx = sum(A_i ∘ Δy^2) = ?? mm^4
I_yy = sum(A_i ∘ Δx^2) = ?? mm^4
I_xy = sum(A_i ∘ Δx ∘ Δy) = ?? mm^4
```

### Tension on each fastener

The load on fastener i is P A_i / ΣA plus A_i (a Δy_i + b Δx_i), with a and b chosen so the fasteners' loads balance both moments: a = (M_x′ I_yy + M_y′ I_xy)/D and b = −(M_y′ I_xx + M_x′ I_xy)/D, D = I_xx I_yy − I_xy². When the fasteners lie on one line (D = 0) only the moment about that line's own axis can be carried, and the function uses the one-axis form.

```calc
function plateTension(A, dx, dy, P, Mx, My, ixx, iyy, ixy)
    s = ixx + iyy
    D = ixx iyy - ixy^2
    f = P A / sum(A)
    if D > 1e-9 s^2
        return f + (Mx iyy + My ixy) A ∘ dy / D - (My ixx + Mx ixy) A ∘ dx / D
    end
    if ixx > 1e-9 s
        return f + Mx A ∘ dy / ixx
    end
    return f - My A ∘ dx / iyy
end
```

```calc {description="Tension on each fastener (negative: unloaded by the moment)"}
F_t = plateTension(A_i, Δx, Δy, P_g, M_x′, M_y′, I_xx, I_yy, I_xy) = ?? kN
```

```calc {description="Largest tension on one fastener"}
F_(t,max) = max(F_t) = ?? kN
```

### Check

```calc
@test F_(t,max) ≤ F_(t,all)
```
