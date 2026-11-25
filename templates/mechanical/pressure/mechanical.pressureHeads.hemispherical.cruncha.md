+++
cruncha-template = 1
id = "mechanical.pressureHeads.hemispherical"
version = "1.0.0"
title = "Hemispherical head under internal pressure, ASME VIII-1"
summary = "Required thickness and maximum allowable working pressure of a hemispherical head under pressure on the concave side, to ASME VIII-1 UG-32(f) and, for high pressure or a thick head, the thick-wall formulas of Appendix 1-3, in the corroded condition, with its inside capacity."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["pressure vessel", "head", "hemispherical head", "hemi head", "sphere", "dished end", "ASME VIII", "UG-32", "UG-27", "Appendix 1-3", "MAWP", "thickness"]
from = "lacewing:mechanical.pressureHeads"
license = "GPL-3.0-or-later"
inputs = ["D", "P", "S", "E", "c", "t_n", "h_1"]
outputs = ["L", "t", "t_c", "t_a", "MAWP", "V"]

[[references]]
work = "ASME Boiler and Pressure Vessel Code, Section VIII, Division 1"
edition = "2010 with 2011a addenda"
locator = "UG-16(e) (corroded dimensions), UG-32(c) (joint efficiency of a hemispherical head), UG-32(f) eq. (3) (P ≤ 0.665SE, t ≤ 0.356L), Mandatory Appendix 1-3 eqs. (1) and (2) (thick spherical shells)"

[[tests]]
name = "Hand check: D 60 in, 200 psi, S 13 750 psi, 1/4 in plate"
source = "By hand, UG-32(f), as Lacewing Lab's pressure_heads_test: t = 200 × 30/(27 500 − 40) = 0.21849964 in; at 1/4 in MAWP = 2 × 13 750 × 0.25/(30 + 0.05) = 228.78536 psi; capacity πD³/12 = 56 548.668 in³"
inputs = { D = "60 in", P = "200 psi", S = "13750 psi", E = "1", c = "0 in", t_n = "0.25 in", h_1 = "0 in" }
expect = { L = "30 in", t = "0.21849964 in", MAWP = "228.78536 psi", V = "56548.668 in³" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: the same head at 0.7 SE, Appendix 1-3"
source = "By hand, as Lacewing Lab's pressure_heads_test: P = 9625 psi > 0.665SE, so t = L[exp(0.5P/SE) − 1] = 30(e^0.35 − 1) = 12.572026 in; at 14 in (t > 0.356L) MAWP = 2SE ln((L + t)/L) = 27 500 ln(44/30) = 10 532.287 psi"
inputs = { D = "60 in", P = "9625 psi", S = "13750 psi", E = "1", c = "0 in", t_n = "14 in", h_1 = "0 in" }
expect = { t = "12.572026 in", MAWP = "10532.287 psi" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: 1/8 in corrosion allowance, 3/8 in plate"
source = "By hand, as Lacewing Lab's pressure_heads_test: corroded L = 30.125 in, t = 200 × 30.125/27 460 = 0.21941005 in, t + c = 0.34441005 in; at 0.25 in available MAWP = 27 500 × 0.25/(30.125 + 0.05) = 227.83761 psi"
inputs = { D = "60 in", P = "200 psi", S = "13750 psi", E = "1", c = "0.125 in", t_n = "0.375 in", h_1 = "0 in" }
expect = { L = "30.125 in", t = "0.21941005 in", t_c = "0.34441005 in", MAWP = "227.83761 psi" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: 3/16 in plate is too thin"
source = "By hand: t = 0.21849964 in is more than 0.1875 in"
inputs = { D = "60 in", P = "200 psi", S = "13750 psi", E = "1", c = "0 in", t_n = "0.1875 in", h_1 = "0 in" }
expect = { t = "0.21849964 in" }
tolerance = "1e-6"
checks = "fail"
+++

## Hemispherical head under internal pressure, ASME VIII-1

The minimum thickness and the maximum allowable working pressure (MAWP) of a hemispherical head with pressure on its concave side, to ASME Section VIII Division 1. Up to P = 0.665 SE (thickness up to 0.356 L) it is UG-32(f); above, the thick spherical shell formulas of Appendix 1-3. The joint efficiency E includes the head-to-shell joint (UG-32(c)). Dimensions are in the corroded condition (UG-16(e)): the inside radius grows by c.

Supplementary loads, nozzles, external pressure and the code's material, welding and testing rules are not checked; this is an aid to the code, not a substitute for it.

### Head and design conditions

S is the allowable stress at the design temperature from ASME Section II Part D (Tables 1A and 1B).

```calc {description="Inside diameter of the head, new"}
D = 1000 mm_
```

```calc {description="Straight flange (skirt) below the tangent line, for the capacity only"}
h_1 = 0 mm_
```

```calc {description="Design (internal, gauge) pressure at the design temperature, including any static head"}
P = 1 MPa_
```

```calc {description="Allowable stress at the design temperature, ASME II-D Table 1A or 1B (for example SA-240 304 at room temperature, 138 MPa)"}
S = 138 MPa_
```

```calc {description="Joint efficiency, UW-12, including the head-to-shell joint (1 for a seamless head)"}
E = 1_
```

```calc {description="Corrosion allowance"}
c = 0 mm_
```

```calc {description="Nominal thickness of the head after forming"}
t_n = 3 mm_
```

### Required thickness

```calc
> Inside radius, corroded
L = (D + 2 c) / 2 = ?? mm
> Required thickness: UG-32(f) eq. (3) to 0.665 SE, Appendix 1-3 eq. (1) above
t = {P L / (2 S E - 0.2 P) if P ≤ 0.665 S E; L (exp(0.5 P / (S E)) - 1) otherwise} = ?? mm
> Thickness with the corrosion allowance (before forming allowances)
t_c = t + c = ?? mm
```

### Maximum allowable working pressure

```calc
> Thickness available, corroded
t_a = t_n - c = ?? mm
> MAWP: UG-32(f) eq. (3) to t = 0.356 L, Appendix 1-3 eq. (2) above
MAWP = {2 S E t_a / (L + 0.2 t_a) if t_a ≤ 0.356 L; 2 S E ln((L + t_a) / L) otherwise} = ?? MPa
```

### Geometry

```calc {description="Inside capacity of the new head, dish (πD³/12) and straight flange"}
V = π D^3 / 12 + π D^2 / 4 h_1 = ?? m³
```

### Check

```calc
@test t_a ≥ t
```
