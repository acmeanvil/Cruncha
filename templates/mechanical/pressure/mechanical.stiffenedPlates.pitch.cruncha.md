+++
cruncha-template = 1
id = "mechanical.stiffenedPlates.pitch"
version = "1.0.0"
title = "Stiffener pitch of a flat vessel wall, ASME VIII-1 Appendix 13-8(d)"
summary = "The plate between stiffeners on a flat wall of a rectangular vessel or tank under internal pressure: the basic stay pitch of UG-47, β and the plate parameter J of Table 13-8(d), the largest stiffener pitch 13-8(d)(1) allows for the corroded plate, and the bending stress and deflection of the panel between stiffeners by Den Hartog's plate coefficients, checked against 1.5 SE."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["stiffened plate", "stiffener pitch", "rectangular vessel", "noncircular vessel", "tank wall", "flat plate", "ASME VIII", "Appendix 13", "13-8", "UG-47", "stayed plate", "Den Hartog"]
from = "lacewing:mechanical.stiffenedPlates"
license = "GPL-3.0-or-later"
inputs = ["P", "L", "p", "t_n", "c", "S", "E", "J", "α_p", "β_p", "E_2"]
outputs = ["t", "C", "p_b", "β", "p_max", "σ_p", "w_p"]

[[references]]
work = "ASME Boiler and Pressure Vessel Code, Section VIII, Division 1"
edition = "2010 with 2011a addenda"
locator = "UG-25 (corrosion), UG-47(a) eq. (1) (stayed plates, C for welded stays), Mandatory Appendix 13: 13-4(b)(1) (plate bending limit 1.5SE), 13-8(d)(1) eqs. (1a)-(1d) and Table 13-8(d) (J), 13-8(d)(5) (pitch); Example 13-17(d)"

[[references]]
work = "Den Hartog, Advanced Strength of Materials"
edition = "1952"
locator = "§19 Catalogue of Results, Cases 19 and 24 (simply supported and clamped plates, uniform pressure); Eq. 68"

[[tests]]
name = "ASME VIII-1 13-17(d): 0.375 in plate, 83.625 in span, 15 psi, S 13 800 psi, 14 in pitch"
source = "Lacewing Lab stiffened_plates_test, after ASME VIII-1 (2010, 2011a) Example 13-17(d): basic pitch 16.48 in, β = 5.1, J = 2.0 (Table 13-8(d) from 4.0 on); eq. (1c) as written gives 16.09 in (the example prints 16.03). Panel 14 × 83.625 in clamped (Den Hartog Case 24 interpolated at a/b = 0.1674: 0.028232, 0.083166), by hand σ = 6 × 0.083166 × 15 × 14²/0.375² = 10 432 psi, w = 0.011137 in with E 27.7 Mpsi"
inputs = { P = "15 psi", L = "83.625 in", p = "14 in", t_n = "0.375 in", c = "0 in", S = "13800 psi", E = "1", J = "2.0", "α_p" = "0.028232", "β_p" = "0.083166", E_2 = "27.7 Mpsi" }
expect = { C = "2.1", p_b = "16.48 in", "β" = "5.07", p_max = "16.09 in", "σ_p" = "10432 psi", w_p = "0.011137 in" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: the Lacewing Lab opening tank wall, 6 mm plate, 1.2 m span, 300 mm pitch, 0.5 bar"
source = "By hand: basic pitch 0.006√(138 × 2.1/0.05) = 456.78879 mm, β = 2.6270347, J = 2.4 − 0.3 × 0.6270347 = 2.2118896 (Table 13-8(d), linear between 2.0 and 3.0), eq. (1a) p max = 0.006√(138 × 2.2118896/0.05) = 468.79990 mm; panel 300 × 1200 mm clamped (Case 24 at b/a = 4: 0.0281, 0.0831): σ = 6 × 0.0831 × 0.05 × 300²/6² = 62.325 MPa, w = 0.0281 × 0.05 × 300⁴/(200 000 × 6³) = 0.2634375 mm"
inputs = { P = "0.05 MPa", L = "1200 mm", p = "300 mm", t_n = "6 mm", c = "0 mm", S = "138 MPa", E = "1", J = "2.2118896", "α_p" = "0.0281", "β_p" = "0.0831", E_2 = "200 GPa" }
expect = { p_b = "456.78879 mm", "β" = "2.6270347", p_max = "468.79990 mm", "σ_p" = "62.325 MPa", w_p = "0.2634375 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same wall at 500 mm pitch: over the 13-8(d) limit"
source = "By hand: J and p max as above (β does not depend on the pitch), 468.79990 mm, under the 500 mm pitch; the panel 500 × 1200 mm (Case 24 at b/a = 2.4: 0.02783, 0.08297) is within 1.5S at 172.9 MPa"
inputs = { P = "0.05 MPa", L = "1200 mm", p = "500 mm", t_n = "6 mm", c = "0 mm", S = "138 MPa", E = "1", J = "2.2118896", "α_p" = "0.02783", "β_p" = "0.08297", E_2 = "200 GPa" }
expect = { p_max = "468.79990 mm", "σ_p" = "172.85417 MPa" }
tolerance = "1e-6"
checks = "fail"
+++

## Stiffener pitch of a flat vessel wall, ASME VIII-1 13-8(d)

A flat wall with stiffeners welded on its outside at a regular pitch p (the side of a rectangular vessel, tank or duct) under uniform internal pressure, to ASME Section VIII Division 1 Appendix 13-8 (reinforced vessels of rectangular cross section). This template checks the plate between the stiffeners: the largest pitch 13-8(d)(1) allows for the plate as corroded, and, beside it, the bending stress and deflection of one panel (pitch × span) by Den Hartog's plate coefficients against the 1.5 SE limit of 13-4(b)(1). The stiffener itself, with the plate acting with it, is the stiffener template (simple or fixed ends) or the frame template (13-8(e)).

The pitch is the distance between stiffener centrelines (13-8(d)(5)). The stiffeners span L between supports (the vessel's ends or a frame), which is also the panel's length. 13-8(d)(1) first finds the basic pitch of a welded-stayed plate from UG-47 eq. (1), with C = 2.1 for plate up to 11 mm (7/16 in) and 2.2 over; then β = L/basic pitch, and J from Table 13-8(d) at β (or 1/β, whichever is larger). Enter J for the β shown, interpolating between the rows; it is 2.0 from β = 4.0 on.

This is an aid to the code, not a substitute for it: the stiffener welds (13-8(b)), corners and the code's other rules are not checked.

### Wall

```calc {description="Internal design pressure, UG-21"}
P = 0.05 MPa_
```

```calc {description="Span of the stiffeners between supports, the panel's length (for a frame, the inside length of this side)"}
L = 1200 mm_
```

```calc {description="Stiffener pitch, centre to centre, 13-8(d)(5)"}
p = 300 mm_
```

```calc {description="Nominal plate thickness"}
t_n = 6 mm_
```

```calc {description="Corrosion allowance, taken off the plate"}
c = 0 mm_
```

```calc {description="Allowable stress of the plate at the design temperature, UG-23 and ASME II-D"}
S = 138 MPa_
```

```calc {description="Joint efficiency of the plate's butt welds, UW-12 (1 away from welds)"}
E = 1_
```

### Largest pitch, 13-8(d)(1)

```calc
> Plate thickness as corroded, UG-25
t = t_n - c = ?? mm
> UG-47 constant for welded stays
C = {2.1 if t ≤ 11 'mm'; 2.2 otherwise} = ??
> Basic stay pitch, UG-47 eq. (1)
p_b = t sqrt(S C / P) = ?? mm
> Span over the basic pitch, 13-8(d)(1)
β = L / p_b = ??
```

```calc {description="Plate parameter J at β (or 1/β if larger), ASME VIII-1 Table 13-8(d), interpolated linearly"}
J = 2.2118896_
```

```calc {description="Largest stiffener pitch: 13-8(d) eqs. (1a)/(1c) when L ≥ basic pitch, (1b)/(1d) otherwise"}
p_max = {t sqrt(S J / P) if L ≥ p_b; t / β sqrt(S J / P) otherwise} = ?? mm
```

### Panel between stiffeners

One panel, pitch × span, as a flat plate under the pressure. A plate continuous over the stiffeners is clamped at them (Den Hartog Case 24, the usual case); a simply supported panel is Case 19. Read both coefficients at b/a = longer/shorter side of the panel, interpolating linearly in a/b, where a is the shorter side.

```calc {description="Panel deflection coefficient, Den Hartog §19 Case 24 (μ) or Case 19 (α) at the panel's b/a"}
α_p = 0.0281_
```

```calc {description="Panel moment coefficient, Den Hartog §19 Case 24 (ν) or Case 19 (β) at the panel's b/a"}
β_p = 0.0831_
```

```calc {description="Young's modulus at the design temperature"}
E_2 = 200 GPa_
```

```calc
> Panel's shorter side
a_p = min(p, L) = ?? mm
> Largest bending stress of the panel, Den Hartog Eq. 68
σ_p = 6 β_p P a_p^2 / t^2 = ?? MPa
> Largest deflection of the panel
w_p = α_p P a_p^4 / (E_2 t^3) = ?? mm
```

### Checks

```calc
@test p ≤ p_max
```

```calc
@test σ_p ≤ 1.5 S E
```
