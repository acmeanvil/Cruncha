+++
cruncha-template = 1
id = "mechanical.pressureHeads.ellipsoidal"
version = "1.0.0"
title = "Ellipsoidal head under internal pressure, ASME VIII-1"
summary = "Required thickness and maximum allowable working pressure of a 2:1 or other ellipsoidal head under pressure on the concave side, to ASME VIII-1 UG-32(d) and Appendix 1-4(c) with K = [2 + (D/2h)²]/6, in the corroded condition, with the head's inside depth and capacity and the limits of the rule checked."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["pressure vessel", "head", "ellipsoidal head", "2:1 head", "semi-ellipsoidal", "dished end", "ASME VIII", "UG-32", "Appendix 1-4", "MAWP", "thickness", "boiler and pressure vessel code"]
from = "lacewing:mechanical.pressureHeads"
license = "GPL-3.0-or-later"
inputs = ["D", "ratio", "P", "S", "E", "c", "t_n", "K_1", "h_1"]
outputs = ["D_c", "h_c", "ratio_c", "K", "t", "t_c", "t_a", "MAWP", "h", "V"]

[[references]]
work = "ASME Boiler and Pressure Vessel Code, Section VIII, Division 1"
edition = "2010 with 2011a addenda"
locator = "UG-16(e) (corroded dimensions), UG-32(d) eq. (1) (2:1 ellipsoidal head), Table UG-37 (K1), Mandatory Appendix 1-4(c) eq. (1) and Examples 1 and 2, 1-4(f) (thin heads), footnote to 1-4 (heads of materials over 70 ksi)"

[[tests]]
name = "VIII-1 Appendix 1-4(c) Example 2: 2:1 head, D 30 in, 1/2 in plate, S 10 200 psi"
source = "Lacewing Lab pressure_heads_test, after ASME VIII-1 (2010, 2011a) Appendix 1-4(c) Example 2: MAWP = 2SEt/(KD + 0.2t) = 339 psi (seamless, K = 1 for the uncorroded 2:1 head, the same as UG-32(d)); by hand 338.87043 psi"
inputs = { D = "30 in", ratio = "2", P = "200 psi", S = "10200 psi", E = "1", c = "0 in", t_n = "0.5 in", K_1 = "0.9", h_1 = "0 in" }
expect = { K = "1", MAWP = "339 psi" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "VIII-1 Appendix 1-4(c) Example 1: D 40 in, h 9 in, 200 psi, S 13 750 psi"
source = "ASME VIII-1 (2010, 2011a) Appendix 1-4(c) Example 1 gives t = 0.33 in with K = 1.14 read from Table 1-4.1 at the nearest D/2h (2.2); the formula's K = [2 + (40/18)²]/6 = 1.1563786 at D/2h = 2.2222 gives, by hand, t = 200 × 40 × 1.1563786/(27 500 − 40) = 0.33689107 in (as Lacewing Lab's pressure_heads_test). At 3/8 in plate MAWP = 2 × 13 750 × 0.375/(1.1563786 × 40 + 0.075) = 222.58726 psi; inside capacity πD²h/6 = 7539.8224 in³"
inputs = { D = "40 in", ratio = "2.2222222222", P = "200 psi", S = "13750 psi", E = "1", c = "0 in", t_n = "0.375 in", K_1 = "1.0", h_1 = "0 in" }
expect = { K = "1.1563786", t = "0.33689107 in", MAWP = "222.58726 psi", h = "9 in", V = "7539.8224 in³" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: 2:1 head, 1000 mm, 1 MPa, E 0.85, 3 mm corrosion allowance"
source = "By hand: corroded D = 1006 mm, h = 253 mm, D/2h = 1.9881423, K = 0.99211830, t = 1 × 1006 × 0.99211830/(2 × 138 × 0.85 − 0.2) = 4.2579821 mm, t + c = 7.2579821 mm; at 8 mm plate, t = 5 mm, MAWP = 2 × 117.3 × 5/(0.99211830 × 1006 + 1) = 1.1740907 MPa; capacity π × 1000² × 250/6 + π/4 × 1000² × 50 = 1.3089969e8 mm³ + 3.9269908e7 mm³ = 0.17016960 m³"
inputs = { D = "1000 mm", ratio = "2", P = "1 MPa", S = "138 MPa", E = "0.85", c = "3 mm", t_n = "8 mm", K_1 = "0.9", h_1 = "50 mm" }
expect = { D_c = "1006 mm", h_c = "253 mm", ratio_c = "1.9881423", K = "0.99211830", t = "4.2579821 mm", t_c = "7.2579821 mm", MAWP = "1.1740907 MPa", V = "0.17016960 m³" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Example 1's head in 5/16 in plate: too thin"
source = "By hand: t = 0.33689107 in is more than the 0.3125 in plate"
inputs = { D = "40 in", ratio = "2.2222222222", P = "200 psi", S = "13750 psi", E = "1", c = "0 in", t_n = "0.3125 in", K_1 = "1.0", h_1 = "0 in" }
expect = { t = "0.33689107 in" }
tolerance = "1e-6"
checks = "fail"
+++

## Ellipsoidal head under internal pressure, ASME VIII-1

The minimum thickness and the maximum allowable working pressure (MAWP) of a semi-ellipsoidal head with pressure on its concave side, to ASME Section VIII Division 1. A 2:1 head (inside depth D/4) is UG-32(d); other proportions are Appendix 1-4(c), whose factor K is 1 for the 2:1 head, so the one formula covers both. Every dimension is in the corroded condition (UG-16(e)): the inside diameter grows by 2c and the depth by c, which takes a corroded 2:1 head slightly below D/2h = 2.

The rule is for heads with t/L ≥ 0.002, L being the equivalent spherical radius K₁D; a thinner head must also meet the knuckle-buckling rules of 1-4(f), which this template does not do. Where K > 1 and the material's specified tensile strength is over 70 ksi (482 MPa), S is limited to 20 ksi (138 MPa) at room temperature (footnote to 1-4). Supplementary loads, nozzles, external pressure and the code's material, welding and testing rules are not checked; this is an aid to the code, not a substitute for it.

### Head

```calc {description="Inside diameter of the head's skirt (straight flange), new"}
D = 1000 mm_
```

```calc {description="Axis ratio D/2h of the new head: major over minor axis (2 for a 2:1 head)"}
ratio = 2_
```

```calc {description="Straight flange (skirt) below the tangent line, for the capacity only"}
h_1 = 50 mm_
```

### Design conditions

S is the allowable stress at the design temperature from ASME Section II Part D (Tables 1A and 1B). E is the lowest efficiency of the joints in the head (UW-12); 1 for a seamless head.

```calc {description="Design (internal, gauge) pressure at the design temperature, including any static head"}
P = 1 MPa_
```

```calc {description="Allowable stress at the design temperature, ASME II-D Table 1A or 1B (for example SA-240 304 at room temperature, 138 MPa)"}
S = 138 MPa_
```

```calc {description="Joint efficiency, UW-12 (1 for a seamless head)"}
E = 1_
```

```calc {description="Corrosion allowance"}
c = 0 mm_
```

```calc {description="Nominal thickness of the head after forming"}
t_n = 5 mm_
```

```calc {description="Spherical radius factor K₁ at the head's D/2h, ASME VIII-1 Table UG-37 (0.9 for a 2:1 head)"}
K_1 = 0.9_
```

### Required thickness

```calc
> Inside diameter, corroded
D_c = D + 2 c = ?? mm
> Inside depth, corroded
h_c = D / (2 ratio) + c = ?? mm
> Axis ratio, corroded
ratio_c = D_c / (2 h_c) = ??
> Appendix 1-4(c) factor (1 for a 2:1 head)
K = (2 + ratio_c^2) / 6 = ??
> Required thickness, 1-4(c) eq. (1) (UG-32(d) eq. (1) when K = 1)
t = P D_c K / (2 S E - 0.2 P) = ?? mm
> Thickness with the corrosion allowance (before forming allowances)
t_c = t + c = ?? mm
```

### Maximum allowable working pressure

```calc
> Thickness available, corroded
t_a = t_n - c = ?? mm
> MAWP at the corroded thickness, 1-4(c) eq. (1)
MAWP = 2 S E t_a / (K D_c + 0.2 t_a) = ?? MPa
```

### Geometry

The new head's inside depth from the tangent line and its inside capacity, dish and straight flange.

```calc
> Inside depth
h = D / (2 ratio) = ?? mm
> Inside capacity, dish (πD²h/6) and straight flange
V = π D^2 h / 6 + π D^2 / 4 h_1 = ?? m³
```

### Checks

The nominal plate must reach the required thickness; the axis ratio must be within Tables 1-4.1 and UG-37 (1.0 to 3.0); and t/L must be at least 0.002, or 1-4(f) applies.

```calc
@test t_a ≥ t
```

```calc
@test 1 ≤ ratio_c ≤ 3
```

```calc
@test t_a / (K_1 D_c) ≥ 0.002
```
