+++
cruncha-template = 1
id = "mechanical.fits.limits"
version = "1.0.0"
title = "Limits and fit of a hole and shaft"
summary = "A hole and a shaft of the same basic size, each with its upper and lower deviations (an ISO 286 tolerance zone such as H7 or s6, or an ANSI B4.1 class): the limits and tolerance of each part, the greatest and least clearance (negative for interference), and the kind of fit."
category = ["Mechanical engineering", "Fits and tolerances"]
tags = ["fit", "limits and fits", "clearance", "interference", "transition fit", "hole basis", "shaft basis", "ISO 286", "ANSI B4.2", "ANSI B4.1", "tolerance zone", "deviation", "Shigley"]
from = "lacewing:mechanical.fits"
license = "GPL-3.0-or-later"
inputs = ["D", "ES", "EI", "es", "ei"]
outputs = ["D_(h,max)", "D_(h,min)", "T_h", "d_max", "d_min", "T_s", "C_max", "C_min", "fit"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-8 Limits and Fits, Eqs. 7-36 to 7-38, Fig. 7-20, Table 7-9 (preferred fits), Example 7-7"

[[references]]
work = "ISO 286-1:2010 and ISO 286-2:2010, Geometrical product specifications (GPS) - ISO code system for tolerances on linear sizes"
edition = "2010"
locator = "Part 1: deviations, fits; Part 2: tables of standard tolerance classes and limit deviations for holes and shafts"

[[references]]
work = "Machinery's Handbook"
edition = "26th"
locator = "ANSI B4.1 Tables 3 to 9, pp. 631-641 (inch classes, Example 1 p. 632); ANSI B4.2 Table 11 (preferred fits), p. 647"

[[tests]]
name = "Shigley Ex. 7-7: 34 mm H11/c11 loose running fit"
source = "Lacewing Lab iso_fits_test, after Shigley 10th Example 7-7: H11 0/+0.160, c11 −0.120/−0.280 mm (IT11 0.160, c's fundamental deviation −0.120): hole 34.000 to 34.160, shaft 33.720 to 33.880; clearance 0.120 to 0.440 mm"
inputs = { D = "34 mm", ES = "160 µm", EI = "0 µm", es = "-120 µm", ei = "-280 µm" }
expect = { "D_(h,max)" = "34.160 mm", "D_(h,min)" = "34.000 mm", d_max = "33.880 mm", d_min = "33.720 mm", T_h = "0.160 mm", T_s = "0.160 mm", C_max = "0.440 mm", C_min = "0.120 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "40 mm H7/s6 medium drive fit"
source = "Lacewing Lab iso_fits_test (Machinery's 26th Table 13 at 40 mm): H7 0/+25 µm, s6 +43/+59 µm, interference 0.018 to 0.059 mm"
inputs = { D = "40 mm", ES = "25 µm", EI = "0 µm", es = "59 µm", ei = "43 µm" }
expect = { C_max = "-0.018 mm", C_min = "-0.059 mm", d_max = "40.059 mm", "D_(h,max)" = "40.025 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Machinery's Example 1: RC 1 at 2 in"
source = "Lacewing Lab inch_bearing_fits_test, after Machinery's 26th p. 632 and ANSI B4.1 Table 3: hole +0.5/0, shaft −0.4/−0.7 thousandths; hole 2.0000 to 2.0005, shaft 1.9993 to 1.9996 in; clearance 0.0004 to 0.0012 in"
inputs = { D = "2 in", ES = "0.0005 in", EI = "0 in", es = "-0.0004 in", ei = "-0.0007 in" }
expect = { "D_(h,max)" = "2.0005 in", d_max = "1.9996 in", d_min = "1.9993 in", C_min = "0.0004 in", C_max = "0.0012 in" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "40 mm H7/k6 locational transition fit"
source = "Lacewing Lab iso_fits (Machinery's 26th Tables 22 and 23, row 30-50 mm): H7 0/+25 µm, k6 +2/+18 µm; clearance 0.023 to interference 0.018 mm"
inputs = { D = "40 mm", ES = "25 µm", EI = "0 µm", es = "18 µm", ei = "2 µm" }
expect = { C_max = "0.023 mm", C_min = "-0.018 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Deviations entered the wrong way round"
source = "By hand: an upper deviation below the lower one is a typing slip"
inputs = { D = "34 mm", ES = "0 µm", EI = "160 µm", es = "-120 µm", ei = "-280 µm" }
expect = { T_h = "-0.160 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## Limits and fit of a hole and shaft

The limits of a hole and a shaft that share a basic size, and the fit they make. Each part's tolerance zone is given by its two deviations from the basic size (Shigley Sec. 7-8):

- **ISO 286** (ANSI B4.2 and BS 4500 are the same system): a zone such as H7 or g6. Read its upper and lower deviations from the limit-deviation tables of ISO 286-2 (or Machinery's Handbook's tables of them) for the size row. Equivalently, take the fundamental deviation (the one nearer the basic size) from ISO 286-1 and place the other one standard tolerance IT away (Eqs. 7-36 to 7-38). A size on a row boundary belongs to the lower row. The ISO 286 formulas template computes IT and the clearance-shaft deviations without the tables.
- **ANSI B4.1** inch classes (RC, LC, LT, LN, FN): the tables give each class's hole and shaft limits by size row in thousandths of an inch; enter them as deviations.

A hole's deviations are ES (upper) and EI (lower), a shaft's es and ei. A deviation above the basic size is positive.

### Basic size and deviations

```calc {description="Basic size shared by hole and shaft"}
D = 34 mm_
```

```calc {description="Upper deviation of the hole, ES (ISO 286-2 table or B4.1 table)"}
ES = 160 µm_
```

```calc {description="Lower deviation of the hole, EI"}
EI = 0 µm_
```

```calc {description="Upper deviation of the shaft, es"}
es = -120 µm_
```

```calc {description="Lower deviation of the shaft, ei"}
ei = -280 µm_
```

### Limits

```calc
> Largest hole
D_(h,max) = D + ES = ?? mm
> Smallest hole
D_(h,min) = D + EI = ?? mm
> Hole tolerance
T_h = ES - EI = ?? mm
> Largest shaft
d_max = D + es = ?? mm
> Smallest shaft
d_min = D + ei = ?? mm
> Shaft tolerance
T_s = es - ei = ?? mm
```

### Fit

Clearance is positive and interference negative (Shigley's Eqs. 7-42 and 7-43 give the interference the other way round, shaft less hole). A clearance fit has clearance even with the tightest pair; an interference fit has interference even with the loosest; a transition fit can have either.

```calc
> Greatest clearance: the largest hole on the smallest shaft
C_max = D_(h,max) - d_min = ?? mm
> Least clearance (negative: the greatest interference): the smallest hole on the largest shaft
C_min = D_(h,min) - d_max = ?? mm
> Kind of fit
fit = { "clearance" if C_min ≥ 0 'mm'; "interference" if C_max ≤ 0 'mm'; "transition" otherwise } = ??
```

### Check

Each upper deviation is above its lower one.

```calc
@test T_h > 0 'mm'
```

```calc
@test T_s > 0 'mm'
```
