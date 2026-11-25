+++
cruncha-template = 1
id = "mechanical.stiffenedPlates.frame"
version = "1.0.0"
title = "Stiffener frame of a rectangular vessel, ASME VIII-1 Appendix 13-8(e)"
summary = "One side of a rectangular vessel whose stiffeners are welded into closed frames round it (ASME VIII-1 13-8(e), Fig. 13-2(a) sketch (4)): the effective width and composite sections, the frame factor from both sides' lengths and stiffness, the membrane stress from the adjacent sides' pressure, the bending moments at mid-span and at the corners, and the total stresses at the plate surface and the stiffener's outer fibre against the limits of 13-4(b), with the deflection, the weld shear flow and the section modulus needed."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["stiffened plate", "stiffener", "frame", "rectangular vessel", "noncircular vessel", "reinforced vessel", "tank", "duct", "effective width", "composite section", "ASME VIII", "Appendix 13", "13-8", "13-17"]
from = "lacewing:mechanical.stiffenedPlates"
license = "GPL-3.0-or-later"
inputs = ["P", "L", "p", "t_n", "c", "L_1", "L_o", "L_o1", "I_o", "S", "S_y", "E", "S_st", "S_yst", "Δ", "r_E", "E_2", "A_s", "I_s", "X_s", "d"]
outputs = ["t", "w", "X_w", "I_w", "c_ow", "X_p", "I_p", "c_op", "α_1", "K", "f", "S_m", "q", "M_mid", "M_end", "S_(i,mid)", "S_(o,mid)", "S_(i,end)", "S_(o,end)", "σ_(i,mid)", "σ_(o,mid)", "σ_(i,end)", "σ_(o,end)", "S_ai", "S_ao", "δ", "V", "q_w", "Z_o", "Z_req"]

[[references]]
work = "ASME Boiler and Pressure Vessel Code, Section VIII, Division 1"
edition = "2010 with 2011a addenda"
locator = "UG-25; Mandatory Appendix 13: 13-4(b) (limits), 13-5 (nomenclature and sign of c), 13-8(b) (welds), 13-8(d)(2)-(3) eq. (2) and Table 13-8(e) (effective width), 13-8(e) eqs. (3)-(8) (membrane and bending stresses of a frame, Fig. 13-2(a) sketch (4)), Example 13-17(d)"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, cases 2e and 2d (deflection by superposition), p. 193"

[[tests]]
name = "ASME VIII-1 13-17(d), long sides: 8 in I-beams at 14 in on 0.375 in plate, 15 psi"
source = "Lacewing Lab stiffened_plates_test, after ASME VIII-1 (2010, 2011a) Example 13-17(d): SA-285 C plate (S 13 800, Sy 25 700 psi), SA-36 beams (S 14 500, Sy 30 800 psi), 8 in 18.4 lb/ft beams (A 5.34 in², I 56.9 in⁴) on the long sides h 83.625 in, h1 90.375 in; short sides H 61.625, H1 70.375 in with I11 = 43.60 in⁴. The example gives w = 14 in (eq. (2) 14.03 in limited to the pitch), Sm = 611 psi, (Sb) at mid-span 5413 psi outer, −2049 inner, at the corners −5374 outer, 2034 inner, and the limits (2/3)Sy = 17 133 and 20 533 psi. It rounds α1 to 0.78 and K to 1.85, so the exact equations differ from its stresses by up to 0.4 %"
inputs = { P = "15 psi", L = "83.625 in", p = "14 in", t_n = "0.375 in", c = "0 in", L_1 = "90.375 in", L_o = "61.625 in", L_o1 = "70.375 in", I_o = "43.60 in^4", S = "13800 psi", S_y = "25700 psi", E = "1", S_st = "14500 psi", S_yst = "30800 psi", "r_E" = "1", E_2 = "27.7 Mpsi", A_s = "5.34 in²", I_s = "56.9 in^4", X_s = "4 in", d = "8 in" }
expect = { w = "14 in", S_m = "611 psi", "S_(o,mid)" = "5413 psi", "S_(i,mid)" = "-2049 psi", "S_(o,end)" = "-5374 psi", "S_(i,end)" = "2034 psi", S_ai = "17133 psi", S_ao = "20533 psi" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "ASME VIII-1 13-17(d), short sides: 6 in I-beams"
source = "Lacewing Lab stiffened_plates_test, after ASME VIII-1 (2010, 2011a) Example 13-17(d): 6 in 12.5 lb/ft beams (A 3.61 in², I 21.8 in⁴) on the short sides, I21 = 103.39 in⁴: X = 1.486 in, I11 = 43.60 in⁴, co = −4.889 in, Sm = 991 psi, (Sb) at the corners −10 234 psi outer, 3111 inner. (Its mid-span 944 and −287 psi are a small difference of large terms with the rounded α1 and K; the exact equations give 960 and −292, in the next case)"
inputs = { P = "15 psi", L = "61.625 in", p = "14 in", t_n = "0.375 in", c = "0 in", L_1 = "70.375 in", L_o = "83.625 in", L_o1 = "90.375 in", I_o = "103.39 in^4", S = "13800 psi", S_y = "25700 psi", E = "1", S_st = "14500 psi", S_yst = "30800 psi", "r_E" = "1", E_2 = "27.7 Mpsi", A_s = "3.61 in²", I_s = "21.8 in^4", X_s = "3 in", d = "6 in" }
expect = { X_w = "1.486 in", I_w = "43.60 in^4", c_ow = "-4.889 in", S_m = "991 psi", "S_(o,end)" = "-10234 psi", "S_(i,end)" = "3111 psi" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Hand check: 13-17(d) short sides exactly"
source = "By hand (Python, the same equations): I11 = 43.595199, I21 = 103.382497 in⁴, α1 = 70.375/90.375 = 0.77870, K = 1.8466, f = 0.74465185; Sm = 991.04120 psi; mid-span −291.74516 inner, 959.64757 outer; corners 3106.8132 inner, −10 219.350 outer; δ = −0.0031672539 in (the corners bend it inward), weld shear flow 1012.0234 lbf/in, Z needed 4.6632434 in³"
inputs = { P = "15 psi", L = "61.625 in", p = "14 in", t_n = "0.375 in", c = "0 in", L_1 = "70.375 in", L_o = "83.625 in", L_o1 = "90.375 in", I_o = "103.382497 in^4", S = "13800 psi", S_y = "25700 psi", E = "1", S_st = "14500 psi", S_yst = "30800 psi", "r_E" = "1", E_2 = "27.7 Mpsi", A_s = "3.61 in²", I_s = "21.8 in^4", X_s = "3 in", d = "6 in" }
expect = { I_w = "43.595199 in^4", f = "0.74465185", S_m = "991.04120 psi", "S_(i,mid)" = "-291.74516 psi", "S_(o,mid)" = "959.64757 psi", "S_(i,end)" = "3106.8132 psi", "S_(o,end)" = "-10219.350 psi", "δ" = "-0.0031672539 in", q_w = "1012.0234 lbf/in", Z_req = "4.6632434 in³" }
tolerance = "2e-6"
checks = "pass"

[[tests]]
name = "13-17(d) short sides at 40 psi: the corners overstressed"
source = "By hand: every stress scales with P; Sm = 991.04120 × 40/15 = 2642.7765 psi, and the total at the corners' outer fibre is (40/15)(991.04 − 10 219.35) = −24 608.8 psi, over 20 533 psi"
inputs = { P = "40 psi", L = "61.625 in", p = "14 in", t_n = "0.375 in", c = "0 in", L_1 = "70.375 in", L_o = "83.625 in", L_o1 = "90.375 in", I_o = "103.382497 in^4", S = "13800 psi", S_y = "25700 psi", E = "1", S_st = "14500 psi", S_yst = "30800 psi", "r_E" = "1", E_2 = "27.7 Mpsi", A_s = "3.61 in²", I_s = "21.8 in^4", X_s = "3 in", d = "6 in" }
expect = { S_m = "2642.7765 psi" }
tolerance = "2e-6"
checks = "fail"
+++

## Stiffener frame of a rectangular vessel, ASME VIII-1 13-8(e)

One side of a rectangular vessel whose stiffeners, at pitch p on all four sides, are welded at the corners into closed rectangular frames (ASME Section VIII Division 1 Appendix 13-8(e), Fig. 13-2(a) sketch (4)). The corner moment depends on the stiffness of both sides; the pressure on the adjacent sides pulls this side along its length, a membrane stress S_m added to the bending. Work the template once for each side: for a long side, L is h and the adjacent side is the short one (H, H₁, I₁₁); for a short side, L is H and the adjacent side is the long one. I_o is the adjacent side's composite second moment, with its own effective width: the I_w this template gives when set up for that side.

The moments are 13-8(e) eqs. (5) to (8) without their c/I: with h the long side's inside length, α₁ = H₁/h₁ (short over long centroidal length) and K = (I₂₁/I₁₁)α₁, the frame factor is f = (1 + α₁²K)/(1 + K), the corner moment is P p h² f/12 and the mid-span moment P p (−3L² + 2h² f)/24. The effective width, composite sections, signs and limits are as in the stiffener template: only w of plate acts where it is in compression (mid-span), the whole pitch where it is in tension (the corners), c_i positive towards the plate's inside surface and c_o negative, tension positive, and the limits of 13-4(b) are the lesser of 1.5 SE and (2/3)S_y, with S E for the membrane stress alone. Check the plate between the stiffeners with the pitch template. Local stresses at the corners can be well above these (13-8(d)(4)); the code asks for a closer look there. This is an aid to the code, not a substitute for it.

### Wall

```calc {description="Internal design pressure, UG-21"}
P = 15 psi_
```

```calc {description="Inside length of this side (h for a long side, H for a short side)"}
L = 83.625 in_
```

```calc {description="Stiffener pitch, centre to centre (within the 13-8(d) limit)"}
p = 14 in_
```

```calc {description="Nominal plate thickness"}
t_n = 0.375 in_
```

```calc {description="Corrosion allowance, taken off the plate (the stiffener is outside)"}
c = 0 in_
```

### Frame

```calc {description="Centroidal length of this side's stiffener (h₁ or H₁)"}
L_1 = 90.375 in_
```

```calc {description="Inside length of the adjacent side (H or h)"}
L_o = 61.625 in_
```

```calc {description="Centroidal length of the adjacent side's stiffener (H₁ or h₁)"}
L_o1 = 70.375 in_
```

```calc {description="Composite second moment of the adjacent side's stiffener and plate (I₁₁ or I₂₁)"}
I_o = 43.60 in^4_
```

### Materials

```calc {description="Allowable stress of the plate at the design temperature, ASME II-D"}
S = 13800 psi_
```

```calc {description="Yield strength of the plate at the design temperature, ASME II-D Table Y-1"}
S_y = 25700 psi_
```

```calc {description="Joint efficiency of the plate's butt welds, UW-12 (1 away from welds)"}
E = 1_
```

```calc {description="Allowable stress of the stiffener material at the design temperature (the plate's if the same)"}
S_st = 14500 psi_
```

```calc {description="Yield strength of the stiffener material at the design temperature (the plate's if the same)"}
S_yst = 30800 psi_
```

```calc {description="Effective width coefficient Δ of the plate material, ASME VIII-1 Table 13-8(e) (6000 √psi for carbon steel)"}
Δ = 6000 √psi_
```

```calc {description="Modulus ratio E₂/E₃, at the design temperature over ambient (Table 13-8(e) note); 1 at room temperature"}
r_E = 1_
```

```calc {description="Young's modulus at the design temperature, for the deflection"}
E_2 = 27.7 Mpsi_
```

### Stiffener

The stiffener's own section, measured from the face welded to the plate.

```calc {description="Stiffener's own area (without plate)"}
A_s = 5.34 in²_
```

```calc {description="Stiffener's own second moment of area about its centroidal axis parallel to the plate"}
I_s = 56.9 in^4_
```

```calc {description="Height of the stiffener's centroid above the face welded to the plate"}
X_s = 4 in_
```

```calc {description="Stiffener's overall height above the plate"}
d = 8 in_
```

### Effective width and composite sections

```calc
> Plate thickness as corroded
t = t_n - c = ?? in
> Effective width, 13-8(d) eq. (2) with the Table 13-8(e) note
w_2 = t Δ sqrt(r_E) / sqrt(S_y) = ?? in
> Effective width, not more than the pitch, 13-8(d)(3)
w = min(w_2, p) = ?? in
```

```calc
function compositeX(A, Xs, t, b)
    return (A (t + Xs) + b t t / 2) / (A + b t)
end
```

```calc
function compositeI(A, Is, Xs, t, b)
    X = (A (t + Xs) + b t t / 2) / (A + b t)
    return Is + A (t + Xs - X)^2 + b t^3 / 12 + b t (X - t / 2)^2
end
```

```calc
> Neutral axis from the plate's inside surface, plate width w (13-17(d))
X_w = compositeX(A_s, X_s, t, w) = ?? in
> Composite second moment, width w (this side's I₁₁ or I₂₁)
I_w = compositeI(A_s, I_s, X_s, t, w) = ?? in^4
> Distance to the stiffener's outer fibre, width w (negative, 13-5)
c_ow = -(t + d - X_w) = ?? in
> Neutral axis, the whole pitch acting
X_p = compositeX(A_s, X_s, t, p) = ?? in
> Composite second moment, the whole pitch
I_p = compositeI(A_s, I_s, X_s, t, p) = ?? in^4
> Distance to the outer fibre, the whole pitch
c_op = -(t + d - X_p) = ?? in
```

### Frame factor and membrane stress

```calc
> Short side's centroidal length over the long side's, α₁ = H₁/h₁
α_1 = {L_o1 / L_1 if L ≥ L_o; L_1 / L_o1 otherwise} = ??
> Stiffness ratio K = (I₂₁/I₁₁) α₁
K = {I_w / I_o α_1 if L ≥ L_o; I_o / I_w α_1 otherwise} = ??
> Frame factor (1 + α₁²K)/(1 + K)
f = (1 + α_1^2 K) / (1 + K) = ??
> Long side's inside length h
h = max(L, L_o) = ?? in
> Membrane stress from the adjacent sides' pressure, 13-8(e) eqs. (3), (4)
S_m = P L_o p / (2 (A_s + p t)) = ?? psi
```

### Moments and stresses

```calc
> Load per length on one stiffener
q = P p = ?? lbf/in
> Moment at mid-span, 13-8(e) eqs. (5), (7) without c/I
M_mid = q (-3 L^2 + 2 h^2 f) / 24 = ?? lbf·in
> Moment at the corners, 13-8(e) eqs. (6), (8) without c/I
M_end = q h^2 f / 12 = ?? lbf·in
```

At mid-span under internal pressure the plate is in compression and the section of width w acts; at the corners it is in tension and the whole pitch acts.

```calc
> Bending stress at mid-span, plate's inside surface
S_(i,mid) = M_mid X_w / I_w = ?? psi
> Bending stress at mid-span, stiffener's outer fibre
S_(o,mid) = M_mid c_ow / I_w = ?? psi
> Bending stress at the corners, plate's inside surface
S_(i,end) = M_end X_p / I_p = ?? psi
> Bending stress at the corners, stiffener's outer fibre
S_(o,end) = M_end c_op / I_p = ?? psi
> Total stresses, membrane plus bending
σ_(i,mid) = S_m + S_(i,mid) = ?? psi
σ_(o,mid) = S_m + S_(o,mid) = ?? psi
σ_(i,end) = S_m + S_(i,end) = ?? psi
σ_(o,end) = S_m + S_(o,end) = ?? psi
> Limit at the plate, 13-4(b)
S_ai = min(1.5 S E, 2 / 3 S_y) = ?? psi
> Limit at the stiffener's outer fibre, 13-4(b)
S_ao = min(1.5 S_st E, 2 / 3 S_yst) = ?? psi
```

### Deflection, shear and section modulus

```calc
> Mid-span deflection: the simply supported beam less the corner couples' share (Roark Table 8.1)
δ = 5 q L^4 / (384 E_2 I_w) - M_end L^2 / (8 E_2 I_w) = ?? in
> End shear
V = q L / 2 = ?? lbf
> Shear flow the stiffener-to-plate welds carry at the corners, both sides together (13-8(b))
q_w = V p t (X_p - t / 2) / I_p = ?? lbf/in
> Section modulus at the outer fibre, width w
Z_o = I_w / abs(c_ow) = ?? in³
> Section modulus the largest moment needs, with the membrane stress taken off the limit
Z_req = max(abs(M_mid), abs(M_end)) / (S_ao - abs(S_m)) = ?? in³
```

### Checks

```calc
@test abs(S_m) ≤ S E
```

```calc
@test abs(σ_(i,mid)) ≤ S_ai
```

```calc
@test abs(σ_(o,mid)) ≤ S_ao
```

```calc
@test abs(σ_(i,end)) ≤ S_ai
```

```calc
@test abs(σ_(o,end)) ≤ S_ao
```
