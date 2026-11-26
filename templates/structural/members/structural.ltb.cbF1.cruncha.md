+++
cruncha-template = 1
id = "structural.ltb.cbF1"
version = "1.0.0"
title = "Lateral-torsional buckling modification factor Cb, AISC 360 Eq. F1-1"
summary = "Moment gradient factor Cb of an unbraced beam segment from the absolute moments at its quarter points, centre and maximum, by AISC 360 Eq. F1-1."
category = ["Structural engineering", "Members"]
tags = ["AISC 360", "F1", "Cb", "moment gradient", "lateral-torsional buckling", "LTB", "beam", "unbraced length", "Table 3-1"]
from = "lacewing:structural.lateralBuckling"
license = "GPL-3.0-or-later"
inputs = ["M_max", "M_A", "M_B", "M_C"]
outputs = ["C_b"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. F1(c), Eq. F1-1 and its User Note"

[[references]]
work = "AISC Steel Construction Manual"
edition = "15th"
locator = "Table 3-1 (values of Cb for simply supported beams), p. 3-18"

[[tests]]
name = "Uniform load, simple span, no intermediate brace: 1.14"
source = "AISC Manual 15th ed. Table 3-1: 1.14 (as cited by Lacewing Lab lateral_torsional_buckling_test). By hand, wL²/8 = 1: quarter points 0.75, centre 1; Cb = 12.5/(2.5 + 2.25 + 4 + 2.25) = 1.13636"
inputs = { M_max = "1 kip·ft", M_A = "0.75 kip·ft", M_B = "1 kip·ft", M_C = "0.75 kip·ft" }
expect = { C_b = "1.136364" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Midspan point load, simple span, no intermediate brace: 1.32"
source = "AISC Manual 15th ed. Table 3-1: 1.32. By hand, PL/4 = 1: quarter points 0.5, centre 1; Cb = 12.5/(2.5 + 1.5 + 4 + 1.5) = 1.31579"
inputs = { M_max = "100 kN·m", M_A = "50 kN·m", M_B = "100 kN·m", M_C = "50 kN·m" }
expect = { C_b = "1.315789" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "End moment at one end, zero at the other: 1.67"
source = "AISC 360-22 F1 User Note: 1.67 for a linear diagram with one end moment zero (Lacewing Lab lateral_torsional_buckling_test). By hand: moments 0.25, 0.5, 0.75 at the quarter points, maximum 1; Cb = 12.5/(2.5 + 0.75 + 2 + 2.25) = 1.66667"
inputs = { M_max = "1 kip·ft", M_A = "0.25 kip·ft", M_B = "0.5 kip·ft", M_C = "0.75 kip·ft" }
expect = { C_b = "1.666667" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Equal end moments, double curvature: 2.27"
source = "AISC 360-22 F1 User Note: 2.27 for equal end moments of opposite sign (reverse curvature). By hand: absolute moments 0.5, 0, 0.5 at the quarter points, maximum 1; Cb = 12.5/(2.5 + 1.5 + 0 + 1.5) = 2.27273"
inputs = { M_max = "1 kip·ft", M_A = "0.5 kip·ft", M_B = "0 kip·ft", M_C = "0.5 kip·ft" }
expect = { C_b = "2.272727" }
tolerance = "1e-6"
checks = "pass"
+++

## Lateral-torsional buckling modification factor C_b

C_b accounts for a moment that varies along an unbraced segment: a beam with its largest moment over only part of the segment buckles at a higher moment than one under uniform moment. Eq. F1-1 takes the absolute moments at the quarter point (M_A), centreline (M_B) and three-quarter point (M_C) of the segment, and the largest absolute moment anywhere in it (M_max).

It applies to doubly symmetric members and to singly symmetric members in single curvature. C_b = 1.0 is always conservative; the AISC 360 User Note gives 1.0 for uniform moment, 1.67 with one end moment zero and 2.27 for equal end moments in reverse curvature. For a cantilever with a free, unbraced tip, take C_b = 1.0. Tees and double angles in F9 take no C_b.

### Moments over the segment (absolute values)

```calc {description="Largest absolute moment in the unbraced segment"}
M_max = 1 kip·ft_
```

```calc {description="Absolute moment at the quarter point of the segment"}
M_A = 0.75 kip·ft_
```

```calc {description="Absolute moment at the centreline of the segment"}
M_B = 1 kip·ft_
```

```calc {description="Absolute moment at the three-quarter point of the segment"}
M_C = 0.75 kip·ft_
```

### Factor

```calc {description="Lateral-torsional buckling modification factor, Eq. F1-1"}
C_b = 12.5 M_max / (2.5 M_max + 3 M_A + 4 M_B + 3 M_C) = ??
```

### Check

M_max is the largest moment in the segment, so it is at least each of the three others.

```calc
@test M_max ≥ max(M_A, M_B, M_C)
```
