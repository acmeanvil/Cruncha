+++
cruncha-template = 1
id = "mechanical.weldGroup.circle"
version = "1.0.0"
title = "Fillet weld all round a circle, weld treated as a line"
summary = "Length, section modulus and polar moment of a circular fillet weld (a round bar or tube welded to a plate), the largest force per length under eccentric shear (twist and bending) and a direct pull, the leg it needs at an allowable throat shear, and the throat stress of the leg chosen."
category = ["Mechanical engineering", "Fastening"]
tags = ["weld", "fillet weld", "weld group", "weld as a line", "circular weld", "round bar", "tube", "eccentric load", "torsion", "bending", "throat", "leg size", "Schaum", "Shigley"]
from = "lacewing:mechanical.weldGroup"
license = "GPL-3.0-or-later"
inputs = ["d", "V", "e_V", "a_V", "P", "τ_all", "w"]
outputs = ["L_w", "Z_w", "J_w", "f_V", "f_T", "f_P", "f_M", "f", "w_req", "τ"]

[[references]]
work = "Schaum's Outline of Machine Design (Hall, Holowenko, Laughlin)"
edition = "1st"
locator = "Ch. 25, Welding: Fig. 25-7 (properties of a weld treated as a line, row 12; forces per length), pp. 301-311; solved problem 9"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 9-4 and 9-5; Tables 9-1 and 9-2 (unit properties of a circular weld line)"

[[tests]]
name = "Schaum problem 9: round bar 2 in, 2000 lb at 8 in out of the plane"
source = "Lacewing Lab weld_test, after Schaum's Machine Design ch. 25 problem 9: f = 5110 lb/in, leg 0.53 in at 9600 lb/in per inch of leg (13,600 psi on the throat)"
inputs = { d = "2 in", V = "2000 lbf", e_V = "0 in", a_V = "8 in", P = "0 lbf", "τ_all" = "13600 psi", w = "0.5625 in" }
expect = { f = "5110 lbf/in", w_req = "0.53 in" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Hand check: problem 9 exactly"
source = "By hand: Lw = 2π = 6.28319 in, Zw = π d²/4 = 3.14159 in², f_V = 2000/6.28319 = 318.310 lb/in, f_M = 16000/3.14159 = 5092.958 lb/in, at the top f = √(318.310² + 5092.958²) = 5102.896 lb/in"
inputs = { d = "2 in", V = "2000 lbf", e_V = "0 in", a_V = "8 in", P = "0 lbf", "τ_all" = "13600 psi", w = "0.5625 in" }
expect = { L_w = "6.283185307 in", Z_w = "3.141592654 in²", J_w = "6.283185307 in³", f_V = "318.3098862 lbf/in", f_M = "5092.958179 lbf/in", f = "5102.895668 lbf/in" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Hand check: twist, bending and pull together"
source = "By hand: d = 3 in, V = 5000 lbf at e = 6 in and a = 4 in, P = 2000 lbf: f_V = 530.516, f_T = T(d/2)/Jw = 2122.066, f_P = 212.207, f_M = 2829.421 lb/in; the largest of √[(f_T sin θ)² + (f_V + f_T cos θ)² + (f_P + f_M |sin θ|)²] round the circle, found numerically, is 3766.077 lb/in"
inputs = { d = "3 in", V = "5000 lbf", e_V = "6 in", a_V = "4 in", P = "2000 lbf", "τ_all" = "21 ksi", w = "0.3125 in" }
expect = { f_V = "530.5164770 lbf/in", f_T = "2122.065908 lbf/in", f_P = "212.2065908 lbf/in", f_M = "2829.421211 lbf/in", f = "3766.077005 lbf/in" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Schaum problem 9 with a 1/2 in leg: the throat is overstressed"
source = "As Schaum problem 9: the 0.5 in leg is under the 0.53 in needed, τ = 5102.9/(0.7071 × 0.5) = 14.43 ksi > 13.6 ksi"
inputs = { d = "2 in", V = "2000 lbf", e_V = "0 in", a_V = "8 in", P = "0 lbf", "τ_all" = "13600 psi", w = "0.5 in" }
expect = { "τ" = "14.43 ksi" }
tolerance = "0.1%"
checks = "fail"
+++

## Fillet weld all round a circle

A fillet weld all round a round bar or tube of diameter d welded to a plate, treated as a line of unit width (Schaum ch. 25, Shigley Secs. 9-4 and 9-5). Its properties are those of the circle; the loads give forces per length of weld, and dividing by the throat of the leg gives the shear stress on the throat.

### Loads and directions

The shear V acts in the plane of the weld. Its line is e_V from the centre in that plane, which twists the weld (T = V·e_V), and a_V out of the plane, which bends it (M = V·a_V, as for a bar loaded at its end). P pulls along the bar's axis. The direct shear V/Lw and the pull P/Lw are taken as spread evenly round the circle.

```calc {description="Diameter of the circular weld (the bar's or tube's outside diameter)"}
d = 2 in_
```

```calc {description="Shear load in the plane of the weld"}
V = 2000 lbf_
```

```calc {description="Arm of V in the weld's plane, from the centre (twist)"}
e_V = 0 in_
```

```calc {description="Distance of V from the weld's plane (bending)"}
a_V = 8 in_
```

```calc {description="Pull along the bar's axis, normal to the weld's plane"}
P = 0 lbf_
```

The allowable is the shear stress on the throat a design code gives (0.30 F_EXX for steel fillets to AWS D1.1 and AISC 360 ASD; see the fillet weld capacity template), or a fatigue allowable. The throat of an equal-leg fillet is taken on 45° (0.707 w) for every direction of load.

```calc {description="Allowable shear stress on the fillet's throat"}
τ_all = 13.6 ksi_
```

```calc {description="Fillet leg size chosen"}
w = 0.5625 in_
```

### Properties of the weld line

```calc
> Total length of weld
L_w = π d = ?? in
> Section modulus of the line, π d²/4
Z_w = π d^2 / 4 = ?? in²
> Polar moment of the line about the centre, π d³/4
J_w = π d^3 / 4 = ?? in³
```

### Forces per length

```calc
> Direct shear per length, V/Lw
f_V = V / L_w = ?? lbf/in
> Twist per length at the weld, T·(d/2)/Jw, at right angles to the radius
f_T = V e_V (d / 2) / J_w = ?? lbf/in
> Direct pull per length, P/Lw
f_P = P / L_w = ?? lbf/in
> Largest bending force per length, M/Zw, at the top and bottom
f_M = V a_V / Z_w = ?? lbf/in
```

At an angle θ round the circle from the side where the twist adds to V, the twist gives f_T sin θ across V and f_T cos θ along it, and the bending f_M |sin θ| normal to the plane, with the pull. The three directions are at right angles, so the resultant is √[(f_T sin θ)² + (f_V + f_T cos θ)² + (f_P + f_M |sin θ|)²]. With only one of twist and bending its largest value is at the side (twist) or the top (bending); with both it lies between, so the function searches the circle: every degree, then every hundredth of a degree round the best.

```calc
function circleWeldForce(f_V, f_T, f_P, f_M)
    best = 0 f_V
    θ_b = 0
    k = 0
    while k < 360
        θ = 2 π k / 360
        g = sqrt((f_T sin(θ))^2 + (f_V + f_T cos(θ))^2 + (f_P + f_M abs(sin(θ)))^2)
        if g > best
            best = g
            θ_b = θ
        end
        k = k + 1
    end
    k = -100
    while k ≤ 100
        θ = θ_b + π k / 18000
        g = sqrt((f_T sin(θ))^2 + (f_V + f_T cos(θ))^2 + (f_P + f_M abs(sin(θ)))^2)
        if g > best
            best = g
        end
        k = k + 1
    end
    return best
end
```

```calc {description="Largest resultant force per length round the weld"}
f = circleWeldForce(f_V, f_T, f_P, f_M) = ?? lbf/in
```

### Leg and throat stress

```calc {description="Leg that carries f at the allowable throat shear, f / (0.707 τ_all)"}
w_req = f / (sqrt(2) / 2 τ_all) = ?? in
```

```calc {description="Shear stress on the throat of the leg chosen, f / (0.707 w)"}
τ = f / (sqrt(2) / 2 w) = ?? ksi
```

Round the leg up to a size that can be welded and check it against the code's minimum fillet size for the thicker part joined (AWS D1.1 Table 5.8, AISC 360 Table J2.4). The bar and plate at the weld need their own checks.

### Check

```calc
@test τ ≤ τ_all
```
