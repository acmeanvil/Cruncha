+++
cruncha-template = 1
id = "mechanical.weldGroup.rectangle"
version = "1.0.0"
title = "Fillet weld group on a rectangular outline, weld treated as a line"
summary = "Length, centroid, section modulus and polar moment of a fillet weld laid on any sides of a b × d rectangle (one line, two lines, L, C, three sides, all round), the largest force per length under eccentric shear (twist and bending) and a direct pull, the leg it needs at an allowable throat shear, and the throat stress of the leg chosen."
category = ["Mechanical engineering", "Fastening"]
tags = ["weld", "fillet weld", "weld group", "weld as a line", "eccentric load", "torsion", "bending", "bracket", "throat", "leg size", "Schaum", "Shigley"]
from = "lacewing:mechanical.weldGroup"
license = "GPL-3.0-or-later"
inputs = ["n_t", "n_b", "n_l", "n_r", "b", "d", "V", "e_V", "a_V", "P", "τ_all", "w"]
outputs = ["L_w", "x_c", "y_c", "I_w", "J_w", "Z_w", "T", "M", "f", "w_req", "τ"]

[[references]]
work = "Schaum's Outline of Machine Design (Hall, Holowenko, Laughlin)"
edition = "1st"
locator = "Ch. 25, Welding: Fig. 25-7 (properties of a weld treated as a line, forces per length), pp. 301-311; solved problems 8, 11 and 15"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 9-4 (welds in torsion: primary and secondary shear), Sec. 9-5 (welds in bending), Table 9-1 and 9-2 (unit properties of weld lines)"

[[tests]]
name = "Schaum problem 8: bracket welded top, side and bottom, 8000 lb at 23.75 in"
source = "Lacewing Lab weld_test, after Schaum's Machine Design ch. 25 problem 8: Jw = 385.4 in³, centroid 1.25 in from the side, f = 3330 lb/in, leg 0.346 in at 13,600 psi on the throat"
inputs = { n_t = "1", n_b = "1", n_l = "1", n_r = "0", b = "5 in", d = "10 in", V = "8000 lbf", e_V = "23.75 in", a_V = "0 in", P = "0 lbf", "τ_all" = "13600 psi", w = "0.375 in" }
expect = { J_w = "385.4 in³", x_c = "1.25 in", f = "3330 lbf/in", w_req = "0.346 in" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Schaum problem 11: box weld 2 × 3 in, 3000 lb at 6 in out of the plane, fatigue allowable"
source = "Lacewing Lab weld_test, after Schaum's Machine Design ch. 25 problem 11: Zw = 9 in², f = 2020 lb/in; at the fatigue allowable of 2750 lb/in per inch of leg (τ = 2750/0.7071 = 3889 psi) the leg is 0.734 in"
inputs = { n_t = "1", n_b = "1", n_l = "1", n_r = "1", b = "2 in", d = "3 in", V = "3000 lbf", e_V = "0 in", a_V = "6 in", P = "0 lbf", "τ_all" = "3889.1 psi", w = "0.75 in" }
expect = { Z_w = "9 in²", f = "2020 lbf/in", w_req = "0.734 in" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Schaum problem 15: channel outline 2 × 6 in, 5000 lb at 10 in out of the plane"
source = "Lacewing Lab weld_test, after Schaum's Machine Design ch. 25 problem 15: Zw = bd + d²/6 = 18 in², f = √(2778² + 500²) = 2822 lb/in (Schaum rounds to 2830), leg 0.294 in at 9600 lb/in per inch"
inputs = { n_t = "1", n_b = "1", n_l = "1", n_r = "0", b = "2 in", d = "6 in", V = "5000 lbf", e_V = "0 in", a_V = "10 in", P = "0 lbf", "τ_all" = "13600 psi", w = "0.3125 in" }
expect = { Z_w = "18 in²", f = "2822 lbf/in", w_req = "0.294 in" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Hand check: L outline (top and side), twist, bending and a pull together"
source = "By hand: b = 5, d = 10 in, top and left side; Lw = 15 in, centroid 0.8333 in from the side and 3.3333 in below the top; Iw = 166.667 in³, Zw = Iw/6.6667 = 25 in² (Schaum's d²(4b + d)/6(2b + d)), Jw = 197.917 in³ (Schaum's [(b + d)⁴ − 6b²d²]/12(b + d)); V = 8000 lbf at e = 10 in, a = 3 in, P = 1000 lbf: at the bottom of the side, T·Y/Jw = 2694.7, V/Lw + T·X/Jw = 870.2 and P/Lw + M·Y/Iw = 1026.7 lb/in, f = 3012.118 lb/in"
inputs = { n_t = "1", n_b = "0", n_l = "1", n_r = "0", b = "5 in", d = "10 in", V = "8000 lbf", e_V = "10 in", a_V = "3 in", P = "1000 lbf", "τ_all" = "21 ksi", w = "0.25 in" }
expect = { L_w = "15 in", x_c = "0.833333333 in", y_c = "6.666666667 in", I_w = "166.6666667 in³", Z_w = "25 in²", J_w = "197.9166667 in³", f = "3012.118256 lbf/in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Schaum problem 8 with a 5/16 in leg: the throat is overstressed"
source = "As Schaum problem 8: the 0.3125 in leg is under the 0.346 in needed, τ = 3336/(0.7071 × 0.3125) = 15.1 ksi > 13.6 ksi"
inputs = { n_t = "1", n_b = "1", n_l = "1", n_r = "0", b = "5 in", d = "10 in", V = "8000 lbf", e_V = "23.75 in", a_V = "0 in", P = "0 lbf", "τ_all" = "13600 psi", w = "0.3125 in" }
expect = { "τ" = "15.10 ksi" }
tolerance = "0.3%"
checks = "fail"
+++

## Fillet weld group on a rectangular outline

A group of fillet welds laid along the sides of a b × d rectangle, treated as lines of unit width (Schaum ch. 25, Shigley Secs. 9-4 and 9-5): its properties are those of the line, and the loads give forces per length of weld. Dividing by the throat of the leg gives the shear stress on the throat.

The outline is set by how many weld lines run along each side: 1 for a weld, 2 for welds on both faces of a thin plate on that line, 0 for none. Some common patterns:

- **Single vertical line:** left 1, the others 0 (b has no effect).
- **Two vertical lines:** left 1, right 1.
- **Two horizontal lines:** top 1, bottom 1.
- **L (top and one side):** top 1, left 1.
- **C (top, side and bottom),** the bracket of Shigley's and Schaum's examples: top 1, bottom 1, left 1.
- **Three sides (top and both sides):** top 1, left 1, right 1.
- **All round:** every side 1.

The welds of a tee or I outline (a stem in the middle) and a circular weld are not in this family; the circular weld has its own template.

### Loads and directions

The shear V acts in the plane of the weld, downward, along d. Its line is e_V from the weld's centroid in that plane, which twists the group (T = V·e_V), and a_V out of the plane, which bends it about the horizontal axis through the centroid (M = V·a_V). P pulls normal to the plane. The direct shear V/Lw and the pull P/Lw are taken as spread evenly over the weld's length, as both books do.

```calc {description="Weld lines along the top side (0, 1 or 2)"}
n_t = 1_
```

```calc {description="Weld lines along the bottom side (0, 1 or 2)"}
n_b = 1_
```

```calc {description="Weld lines along the left side (0, 1 or 2)"}
n_l = 1_
```

```calc {description="Weld lines along the right side (0, 1 or 2)"}
n_r = 0_
```

```calc {description="Width of the outline, horizontal (across the shear)"}
b = 5 in_
```

```calc {description="Depth of the outline, vertical (along the shear)"}
d = 10 in_
```

```calc {description="Shear load in the plane of the weld, downward"}
V = 8000 lbf_
```

```calc {description="Arm of V in the weld's plane, from the centroid (twist)"}
e_V = 23.75 in_
```

```calc {description="Distance of V from the weld's plane (bending about the horizontal axis)"}
a_V = 0 in_
```

```calc {description="Pull normal to the weld's plane"}
P = 0 lbf_
```

The allowable is the shear stress on the throat a design code gives (0.30 F_EXX for steel fillets to AWS D1.1 and AISC 360 ASD; see the fillet weld capacity template), or a fatigue allowable. The throat of an equal-leg fillet is taken on 45° (0.707 w) for every direction of load, as AWS does.

```calc {description="Allowable shear stress on the fillet's throat"}
τ_all = 13.6 ksi_
```

```calc {description="Fillet leg size chosen"}
w = 0.375 in_
```

### Properties of the weld line

The origin is the outline's bottom-left corner. The second moments are of the line about its centroid, per unit width (length³); the polar moment is their sum.

```calc
> Total length of weld
L_w = (n_t + n_b) b + (n_l + n_r) d = ?? in
> Centroid from the left side
x_c = ((n_t + n_b) b^2 / 2 + n_r b d) / L_w = ?? in
> Centroid from the bottom
y_c = (n_t b d + (n_l + n_r) d^2 / 2) / L_w = ?? in
> Second moment of the line about the horizontal centroidal axis
I_w = n_t b d^2 + (n_l + n_r) d^3 / 3 - L_w y_c^2 = ?? in³
> Second moment of the line about the vertical centroidal axis
I_wy = (n_t + n_b) b^3 / 3 + n_r d b^2 - L_w x_c^2 = ?? in³
> Polar moment of the line about the centroid, Jw = Iw + Iwy
J_w = I_w + I_wy = ?? in³
> Section modulus of the line, at the fibre farther from the axis
Z_w = I_w / max(y_c, d - y_c) = ?? in²
```

### Forces per length

```calc {description="Twisting moment about the centroid, V·e_V"}
T = V e_V = ?? lbf·in
```

```calc {description="Bending moment about the horizontal centroidal axis, V·a_V"}
M = V a_V = ?? lbf·in
```

```calc
> Direct (primary) shear per length, V/Lw
f_V = V / L_w = ?? lbf/in
> Direct pull per length, P/Lw
f_P = P / L_w = ?? lbf/in
> Largest bending force per length, M/Zw
f_M = M / Z_w = ?? lbf/in
```

At a point (X, Y) from the centroid, the twist adds a force per length T·r/Jw at right angles to the radius: T·Y/Jw horizontally and T·X/Jw vertically, where the vertical part adds to the direct shear on one side. The bending adds M·Y/Iw normal to the plane, with the pull. The three directions are at right angles, so they combine as a vector. On a straight weld the resultant is largest at an end, so it is found at the corners of the outline where a weld runs, for either sense of the twist; a corner with no weld through it gives zero.

```calc
function weldPoint(X, Y, on, V, P, T, M, L, I, J)
    f_h = T Y / J
    f_v = abs(V) / L + abs(T X) / J
    f_n = abs(P) / L + abs(M Y / I)
    return min(on, 1) sqrt(f_h^2 + f_v^2 + f_n^2)
end
```

```calc
> Resultant force per length at the bottom-left corner
f_1 = weldPoint(-x_c, -y_c, n_b + n_l, V, P, T, M, L_w, I_w, J_w) = ?? lbf/in
> At the bottom-right corner
f_2 = weldPoint(b - x_c, -y_c, n_b + n_r, V, P, T, M, L_w, I_w, J_w) = ?? lbf/in
> At the top-left corner
f_3 = weldPoint(-x_c, d - y_c, n_t + n_l, V, P, T, M, L_w, I_w, J_w) = ?? lbf/in
> At the top-right corner
f_4 = weldPoint(b - x_c, d - y_c, n_t + n_r, V, P, T, M, L_w, I_w, J_w) = ?? lbf/in
> Largest resultant force per length on the group
f = max(f_1, f_2, f_3, f_4) = ?? lbf/in
```

### Leg and throat stress

```calc {description="Leg that carries f at the allowable throat shear, f / (0.707 τ_all)"}
w_req = f / (sqrt(2) / 2 τ_all) = ?? in
```

```calc {description="Shear stress on the throat of the leg chosen, f / (0.707 w)"}
τ = f / (sqrt(2) / 2 w) = ?? ksi
```

Round the leg up to a size that can be welded and check it against the code's minimum fillet size for the thicker part joined (AWS D1.1 Table 5.8, AISC 360 Table J2.4). The base metal at the weld needs its own check. Long welds don't share load evenly; Schaum reduces the allowable to about 90 %.

### Check

```calc
@test τ ≤ τ_all
```
