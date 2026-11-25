+++
cruncha-template = 1
id = "mechanical.stiffenedPlates.stiffener"
version = "1.0.0"
title = "Stiffener on a flat vessel wall, simple or fixed ends, ASME VIII-1 Appendix 13"
summary = "One stiffener on a flat wall of a rectangular vessel or tank under internal pressure, with the plate acting with it: the effective width of 13-8(d) eq. (2), the composite section, moments with simply supported or fixed ends, the bending stresses at the plate surface and the stiffener's outer fibre at mid-span and at the ends against the limits of 13-4(b), the deflection, the web shear, the shear flow the welds carry and the section modulus needed."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["stiffened plate", "stiffener", "rectangular vessel", "noncircular vessel", "tank wall", "effective width", "composite section", "ASME VIII", "Appendix 13", "13-8", "flat bar", "Roark"]
from = "lacewing:mechanical.stiffenedPlates"
license = "GPL-3.0-or-later"
inputs = ["P", "L", "p", "t_n", "c", "ends", "S", "S_y", "E", "S_st", "S_yst", "Δ", "r_E", "E_2", "A_s", "I_s", "X_s", "d", "A_w"]
outputs = ["t", "w", "X_w", "I_w", "c_ow", "X_p", "I_p", "c_op", "q", "M_mid", "M_end", "S_(i,mid)", "S_(o,mid)", "S_(i,end)", "S_(o,end)", "S_ai", "S_ao", "δ", "V", "τ_w", "q_w", "Z_o", "Z_req"]

[[references]]
work = "ASME Boiler and Pressure Vessel Code, Section VIII, Division 1"
edition = "2010 with 2011a addenda"
locator = "UG-25; Mandatory Appendix 13: 13-4(b) (limits), 13-5 (sign of c), 13-8(b) (welds), 13-8(d)(2)-(3) eq. (2) and Table 13-8(e) (effective width), 13-17(d) (composite section)"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, cases 2e (simply supported) and 2d (fixed ends), uniform load, p. 193"

[[tests]]
name = "Hand check: the Lacewing Lab opening tank wall, 80 × 10 mm flat bars, simply supported"
source = "By hand, as Lacewing Lab stiffened_plates_test (simply supported: M = −qL²/8, δ = 5qL⁴/(384EI), V = qL/2): 6 mm plate, 1.2 m span, 300 mm pitch, 0.5 bar, S 138, Sy 250 MPa, carbon steel Δ = 6000 √psi = 498 208.05 √Pa; w = 0.006 × 498 208.05/√250e6 = 189.05666 mm; with w: X = 20.783844 mm, I = 1 297 505.46 mm⁴, co = −65.216156 mm; q = 15 N/mm, M = −2700 N·m, Si = −43.249435, So = 135.70935 MPa against (2/3)Sy = 166.66667 MPa; δ = 1.5606871 mm; τ = 9000/800 = 11.25 MPa; with the pitch X = 16.230769 mm, I = 1 456 128.21 mm⁴, weld shear flow 147.19752 N/mm; Z = 19 895.461 mm³, needed 16 200 mm³"
inputs = { P = "0.05 MPa", L = "1200 mm", p = "300 mm", t_n = "6 mm", c = "0 mm", ends = "1", S = "138 MPa", S_y = "250 MPa", E = "1", S_st = "138 MPa", S_yst = "250 MPa", "r_E" = "1", E_2 = "200 GPa", A_s = "800 mm²", I_s = "426666.6667 mm^4", X_s = "40 mm", d = "80 mm", A_w = "800 mm²" }
expect = { w = "189.05666 mm", X_w = "20.783844 mm", I_w = "1297505.46 mm^4", c_ow = "-65.216156 mm", X_p = "16.230769 mm", I_p = "1456128.21 mm^4", M_mid = "-2700 N·m", "S_(i,mid)" = "-43.249435 MPa", "S_(o,mid)" = "135.70935 MPa", S_ao = "166.66667 MPa", "δ" = "1.5606871 mm", "τ_w" = "11.25 MPa", q_w = "147.19752 N/mm", Z_o = "19895.461 mm³", Z_req = "16200 mm³" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: the same wall with fixed ends"
source = "By hand, as Lacewing Lab stiffened_plates_test (fixed: qL²/12 at the ends with the whole pitch acting, −qL²/24 at mid-span, δ = qL⁴/(384EI)): M ends = 1800 N·m, Si = 20.063745, So = −86.245576 MPa; M mid = −900 N·m, Si = −14.416478, So = 45.236450 MPa; δ = 0.31213741 mm; Z needed 10 800 mm³"
inputs = { P = "0.05 MPa", L = "1200 mm", p = "300 mm", t_n = "6 mm", c = "0 mm", ends = "2", S = "138 MPa", S_y = "250 MPa", E = "1", S_st = "138 MPa", S_yst = "250 MPa", "r_E" = "1", E_2 = "200 GPa", A_s = "800 mm²", I_s = "426666.6667 mm^4", X_s = "40 mm", d = "80 mm", A_w = "800 mm²" }
expect = { M_end = "1800 N·m", M_mid = "-900 N·m", "S_(i,end)" = "20.063745 MPa", "S_(o,end)" = "-86.245576 MPa", "S_(i,mid)" = "-14.416478 MPa", "S_(o,mid)" = "45.236450 MPa", "δ" = "0.31213741 mm", Z_req = "10800 mm³" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Simply supported over 1.5 m: the stiffener is overstressed"
source = "By hand: M = −15 × 1500²/8 = −4218.75 N·m, So = 135.70935 × (1.5/1.2)² = 212.04586 MPa, over 166.67 MPa"
inputs = { P = "0.05 MPa", L = "1500 mm", p = "300 mm", t_n = "6 mm", c = "0 mm", ends = "1", S = "138 MPa", S_y = "250 MPa", E = "1", S_st = "138 MPa", S_yst = "250 MPa", "r_E" = "1", E_2 = "200 GPa", A_s = "800 mm²", I_s = "426666.6667 mm^4", X_s = "40 mm", d = "80 mm", A_w = "800 mm²" }
expect = { M_mid = "-4218.75 N·m", "S_(o,mid)" = "212.04586 MPa" }
tolerance = "1e-6"
checks = "fail"
+++

## Stiffener on a flat vessel wall, simple or fixed ends

One of the stiffeners welded at a pitch p on the outside of a flat wall (the side of a rectangular vessel, tank or duct) under uniform internal pressure P, after ASME Section VIII Division 1 Appendix 13-8. The stiffener carries the pressure on one pitch as a beam of span L, with a width of the plate acting with it. Its ends are simply supported (Roark Table 8.1 case 2e: M = qL²/8 at mid-span) or fixed (case 2d: qL²/12 at the ends, qL²/24 at mid-span). For stiffeners welded into closed frames round a rectangular vessel (13-8(e)), use the frame template. Check the plate between the stiffeners, and that p is within 13-8(d)'s largest pitch, with the pitch template.

Where the plate is in compression (at mid-span under internal pressure) only the effective width w of 13-8(d) eq. (2) acts with the stiffener: w = tΔ/√S_y, with Δ from Table 13-8(e) for the plate's material, scaled by √(E₂/E₃) when the design temperature is not ambient. Where the plate is in tension (at fixed ends) the whole pitch may act (13-8(d)(3)). Stresses follow Appendix 13's signs (13-5): distances from the composite's neutral axis are positive towards the plate's inside surface (c_i) and negative to the stiffener's outer fibre (c_o); a negative moment puts the plate in compression; tension is positive. The limits are those of 13-4(b): the lesser of 1.5 SE and (2/3)S_y for the composite section, for the plate and for the stiffener's own material.

The stiffener-to-plate welds must develop the shear flow given (13-8(b)). The code sets no deflection limit. This is an aid to the code, not a substitute for it.

### Wall

```calc {description="Internal design pressure, UG-21"}
P = 0.05 MPa_
```

```calc {description="Span of the stiffener between supports"}
L = 1200 mm_
```

```calc {description="Stiffener pitch, centre to centre (within the 13-8(d) limit)"}
p = 300 mm_
```

```calc {description="Nominal plate thickness"}
t_n = 6 mm_
```

```calc {description="Corrosion allowance, taken off the plate (the stiffener is outside)"}
c = 0 mm_
```

```calc {description="Stiffener ends: 1 simply supported (Roark Table 8.1 case 2e), 2 fixed (case 2d)"}
ends = 1_
```

### Materials

```calc {description="Allowable stress of the plate at the design temperature, ASME II-D"}
S = 138 MPa_
```

```calc {description="Yield strength of the plate at the design temperature, ASME II-D Table Y-1"}
S_y = 250 MPa_
```

```calc {description="Joint efficiency of the plate's butt welds, UW-12 (1 away from welds)"}
E = 1_
```

```calc {description="Allowable stress of the stiffener material at the design temperature (the plate's if the same)"}
S_st = 138 MPa_
```

```calc {description="Yield strength of the stiffener material at the design temperature (the plate's if the same)"}
S_yst = 250 MPa_
```

```calc {description="Effective width coefficient Δ of the plate material, ASME VIII-1 Table 13-8(e) (6000 √psi for carbon steel)"}
Δ = 6000 √psi_
```

```calc {description="Modulus ratio E₂/E₃, at the design temperature over ambient (Table 13-8(e) note); 1 at room temperature"}
r_E = 1_
```

```calc {description="Young's modulus at the design temperature, for the deflection"}
E_2 = 200 GPa_
```

### Stiffener

The stiffener's own section, measured from the face welded to the plate. For a flat bar h × b on edge: A = hb, I = bh³/12, X_s = h/2, d = h, A_w = hb.

```calc {description="Stiffener's own area (without plate)"}
A_s = 800 mm²_
```

```calc {description="Stiffener's own second moment of area about its centroidal axis parallel to the plate"}
I_s = 426666.6667 mm^4_
```

```calc {description="Height of the stiffener's centroid above the face welded to the plate"}
X_s = 40 mm_
```

```calc {description="Stiffener's overall height above the plate"}
d = 80 mm_
```

```calc {description="Area of the stiffener's web, for the average shear stress"}
A_w = 800 mm²_
```

### Effective width and composite sections

```calc
> Plate thickness as corroded
t = t_n - c = ?? mm
> Effective width, 13-8(d) eq. (2) with the Table 13-8(e) note
w_2 = t Δ sqrt(r_E) / sqrt(S_y) = ?? mm
> Effective width, not more than the pitch, 13-8(d)(3)
w = min(w_2, p) = ?? mm
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
X_w = compositeX(A_s, X_s, t, w) = ?? mm
> Composite second moment, width w
I_w = compositeI(A_s, I_s, X_s, t, w) = ?? mm^4
> Distance to the stiffener's outer fibre, width w (negative, 13-5)
c_ow = -(t + d - X_w) = ?? mm
> Neutral axis, the whole pitch acting
X_p = compositeX(A_s, X_s, t, p) = ?? mm
> Composite second moment, the whole pitch
I_p = compositeI(A_s, I_s, X_s, t, p) = ?? mm^4
> Distance to the outer fibre, the whole pitch
c_op = -(t + d - X_p) = ?? mm
```

### Moments and stresses

```calc
> Load per length on one stiffener
q = P p = ?? N/mm
> Moment at mid-span: −qL²/8 simply supported, −qL²/24 fixed
M_mid = {-q L^2 / 8 if ends == 1; -q L^2 / 24 if ends == 2} = ?? N·m
> Moment at the ends: 0 simply supported, qL²/12 fixed
M_end = {0 'N·m' if ends == 1; q L^2 / 12 if ends == 2} = ?? N·m
```

```calc
> Mid-span, plate in compression (width w): stress at the plate's inside surface
S_(i,mid) = M_mid X_w / I_w = ?? MPa
> Mid-span, at the stiffener's outer fibre
S_(o,mid) = M_mid c_ow / I_w = ?? MPa
> Ends, plate in tension (the whole pitch): stress at the plate's inside surface
S_(i,end) = M_end X_p / I_p = ?? MPa
> Ends, at the stiffener's outer fibre
S_(o,end) = M_end c_op / I_p = ?? MPa
> Limit at the plate, 13-4(b)
S_ai = min(1.5 S E, 2 / 3 S_y) = ?? MPa
> Limit at the stiffener's outer fibre, 13-4(b)
S_ao = min(1.5 S_st E, 2 / 3 S_yst) = ?? MPa
```

### Deflection, shear and section modulus

```calc
> Mid-span deflection: the simply supported beam less the end couples' share (Roark Table 8.1)
δ = 5 q L^4 / (384 E_2 I_w) - M_end L^2 / (8 E_2 I_w) = ?? mm
> End shear
V = q L / 2 = ?? N
> Average shear stress in the web
τ_w = V / A_w = ?? MPa
> Shear flow the stiffener-to-plate welds carry at the ends, both sides together (13-8(b))
q_w = V p t (X_p - t / 2) / I_p = ?? N/mm
> Section modulus at the outer fibre, width w
Z_o = I_w / abs(c_ow) = ?? mm³
> Section modulus the largest moment needs at the stiffener's limit
Z_req = max(abs(M_mid), abs(M_end)) / S_ao = ?? mm³
```

### Checks

```calc
@test abs(S_(i,mid)) ≤ S_ai
```

```calc
@test abs(S_(o,mid)) ≤ S_ao
```

```calc
@test abs(S_(i,end)) ≤ S_ai
```

```calc
@test abs(S_(o,end)) ≤ S_ao
```
