+++
cruncha-template = 1
id = "mechanical.pressureHeads.torispherical"
version = "1.0.0"
title = "Torispherical (flanged and dished) head under internal pressure, ASME VIII-1"
summary = "Required thickness and maximum allowable working pressure of a torispherical head of any crown and knuckle radius under pressure on the concave side, to ASME VIII-1 Appendix 1-4(d) with M = [3 + √(L/r)]/4 (the ASME F&D head of UG-32(e) is the case L = D_o, r = 0.06 D_o), in the corroded condition, with its inside depth and the proportion limits of UG-32(j) checked."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["pressure vessel", "head", "torispherical head", "flanged and dished", "F&D head", "Klöpper", "Korbbogen", "dished end", "ASME VIII", "UG-32", "Appendix 1-4", "MAWP", "thickness"]
from = "lacewing:mechanical.pressureHeads"
license = "GPL-3.0-or-later"
inputs = ["D", "L", "r", "P", "S", "E", "c", "t_n"]
outputs = ["D_o", "L_c", "r_c", "M", "t", "t_c", "t_a", "MAWP", "h"]

[[references]]
work = "ASME Boiler and Pressure Vessel Code, Section VIII, Division 1"
edition = "2010 with 2011a addenda"
locator = "UG-16(e) (corroded dimensions), UG-32(e) eq. (2) (F&D head), UG-32(j) (crown and knuckle limits), Mandatory Appendix 1-4(d) eq. (3) and Examples 1 and 2, 1-4(f) (thin heads)"

[[references]]
work = "EN 13445-3, Unfired pressure vessels, Part 3: Design"
locator = "7.5.3.2, Note 3 (inside depth of a torispherical end)"

[[tests]]
name = "VIII-1 Appendix 1-4(d) Example 1: D 40 in, L 40 in, r 4 in, 200 psi, S 13 750 psi"
source = "Lacewing Lab pressure_heads_test, after ASME VIII-1 (2010, 2011a) Appendix 1-4(d) Example 1: M = 1.54 (Table 1-4.2 at L/r = 10), t = 0.45 in. By hand M = (3 + √10)/4 = 1.5405694, t = 200 × 40 × 1.5405694/(27 500 − 40) = 0.44881847 in"
inputs = { D = "40 in", L = "40 in", r = "4 in", P = "200 psi", S = "13750 psi", E = "1", c = "0 in", t_n = "0.5 in" }
expect = { M = "1.54", t = "0.45 in" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "VIII-1 Appendix 1-4(d) Example 2: D 30 in, L 24 in, r 2 in, 1/2 in plate, S 6500 psi"
source = "Lacewing Lab pressure_heads_test, after ASME VIII-1 (2010, 2011a) Appendix 1-4(d) Example 2: M = 1.62, MAWP = 167 psi. By hand M = (3 + √12)/4 = 1.6160254, MAWP = 2 × 6500 × 0.5/(24 × 1.6160254 + 0.1) = 167.16125 psi; the 200 psi design pressure needs t = 0.59852793 in, more than the plate"
inputs = { D = "30 in", L = "24 in", r = "2 in", P = "200 psi", S = "6500 psi", E = "1", c = "0 in", t_n = "0.5 in" }
expect = { M = "1.6160254", MAWP = "167.16125 psi", t = "0.59852793 in" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: F&D-proportioned head, 1000 mm, 1 MPa, E 0.85, 2 mm corrosion allowance"
source = "By hand: 8 mm plate, Do = 1016 mm, L = Do = 1016, r = 61 mm (just over 0.06 Do = 60.96); corroded L = 1018, r = 63, M = (3 + √(1018/63))/4 = 1.7549481, t = 1 × 1018 × 1.7549481/(2 × 117.3 − 0.2) = 7.6217455 mm, t + c = 9.6217455 mm, MAWP at 6 mm = 2 × 117.3 × 6/(1018 × 1.7549481 + 1.2) = 0.78736407 MPa; inside depth h = 1016 − √(516 × 394) = 167.88208 mm"
inputs = { D = "1000 mm", L = "1016 mm", r = "61 mm", P = "1 MPa", S = "138 MPa", E = "0.85", c = "2 mm", t_n = "8 mm" }
expect = { D_o = "1016 mm", L_c = "1018 mm", r_c = "63 mm", M = "1.7549481", t = "7.6217455 mm", t_c = "9.6217455 mm", MAWP = "0.78736407 MPa", h = "167.88208 mm" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: the same head at 0.75 MPa"
source = "By hand: t = 0.75 × 1018 × 1.7549481/(234.6 − 0.15) = 5.7150900 mm, under the 6 mm available; every UG-32(j) limit met"
inputs = { D = "1000 mm", L = "1016 mm", r = "61 mm", P = "0.75 MPa", S = "138 MPa", E = "0.85", c = "2 mm", t_n = "8 mm" }
expect = { t = "5.7150900 mm" }
tolerance = "1e-6"
checks = "pass"
+++

## Torispherical head under internal pressure, ASME VIII-1

The minimum thickness and the maximum allowable working pressure (MAWP) of a torispherical head (a spherical crown of inside radius L joined to the skirt by a toroidal knuckle of inside radius r) with pressure on its concave side, to ASME Section VIII Division 1 Appendix 1-4(d). The ASME flanged and dished head of UG-32(e) is the case L = D_o, r = 0.06 D_o: UG-32(e)'s 0.885 is M/2 at L/r = 16⅔, rounded, so this template gives a thickness 0.04 % above UG-32(e)'s. DIN 28011 (Klöpper) and 28013 (Korbbogen) ends can be entered by their radii, but VIII-1's proportion limits then decide whether the rule applies.

Every dimension is in the corroded condition (UG-16(e)): the inside diameter grows by 2c and both radii by c. The limits of UG-32(j) are on the head as made. The rule is for heads with t/L ≥ 0.002; a thinner head must also meet the knuckle-buckling rules of 1-4(f), which this template does not do. Supplementary loads, nozzles, external pressure and the code's material, welding and testing rules are not checked; this is an aid to the code, not a substitute for it.

### Head

```calc {description="Inside diameter of the head's skirt (straight flange), new"}
D = 1000 mm_
```

```calc {description="Inside crown radius, new"}
L = 1000 mm_
```

```calc {description="Inside knuckle radius, new"}
r = 100 mm_
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
t_n = 8 mm_
```

### Required thickness

```calc
> Outside diameter of the skirt
D_o = D + 2 t_n = ?? mm
> Inside crown radius, corroded
L_c = L + c = ?? mm
> Inside knuckle radius, corroded
r_c = r + c = ?? mm
> Appendix 1-4(d) factor
M = (3 + sqrt(L_c / r_c)) / 4 = ??
> Required thickness, 1-4(d) eq. (3)
t = P L_c M / (2 S E - 0.2 P) = ?? mm
> Thickness with the corrosion allowance (before forming allowances; the knuckle thins in forming)
t_c = t + c = ?? mm
```

### Maximum allowable working pressure

```calc
> Thickness available, corroded
t_a = t_n - c = ?? mm
> MAWP at the corroded thickness, 1-4(d) eq. (3)
MAWP = 2 S E t_a / (L_c M + 0.2 t_a) = ?? MPa
```

### Geometry

```calc {description="Inside depth of the new head from the tangent line (EN 13445-3 7.5.3.2 Note 3)"}
h = L - sqrt((L - D / 2) (L + D / 2 - 2 r)) = ?? mm
```

### Checks

The nominal plate must reach the required thickness. UG-32(j): the crown radius no more than the skirt's outside diameter, the knuckle radius at least 6 % of it and at least three times the head thickness. And t/L must be at least 0.002, or 1-4(f) applies.

```calc
@test t_a ≥ t
```

```calc
@test L ≤ D_o
```

```calc
@test r ≥ 0.06 D_o
```

```calc
@test r ≥ 3 t_n
```

```calc
@test t_a / L_c ≥ 0.002
```
