+++
cruncha-template = 1
id = "mechanical.fits.iso286Formulas"
version = "1.0.0"
title = "ISO 286 hole-basis clearance fit from the system's formulas"
summary = "An H hole on a d, e, f, g or h shaft, worked from the formulas the ISO system of limits and fits is built on rather than its tables: the standard tolerance unit at the size row's geometric mean, the standard tolerances IT5 to IT18, the shaft's fundamental deviation, both parts' limits, and the least and greatest clearance."
category = ["Mechanical engineering", "Fits and tolerances"]
tags = ["fit", "limits and fits", "ISO 286", "standard tolerance", "IT grade", "tolerance unit", "fundamental deviation", "clearance fit", "hole basis", "running fit", "sliding fit"]
from = "lacewing:mechanical.fits"
license = "GPL-3.0-or-later"
inputs = ["D_1", "D_2", "n_h", "n_s", "pos"]
outputs = ["D", "i", "IT_h", "IT_s", "es", "ei", "D_(h,max)", "D_(h,min)", "d_max", "d_min", "C_min", "C_max"]

[[references]]
work = "ISO 286-1:1988 ISO system of limits and fits - Part 1: Bases of tolerances, deviations and fits"
edition = "1988"
locator = "Annex A (bases of the system): the standard tolerance unit, the standard tolerance grades IT5 to IT18 as multiples of it, and the formulas for the fundamental deviations of shafts"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-8 Limits and Fits, Eqs. 7-36 to 7-38; Table 7-9 (preferred fits)"

[[references]]
work = "Machinery's Handbook"
edition = "26th"
locator = "BS 4500 Tables 22 and 23 (standard tolerances, shaft fundamental deviations), pp. 662-663: the rounded values the formulas are compared with"

[[tests]]
name = "Hand check: 40 mm H7/g6 sliding fit, unrounded"
source = "By hand: D = √(30 × 50) = 38.72983 mm, i = 0.45 × 38.72983^(1/3) + 0.001 × 38.72983 = 1.561243 µm, IT7 = 16i = 24.97989, IT6 = 10i = 15.61243, es = −2.5 × 38.72983^0.34 = −8.667134, ei = −24.27956 µm; clearance 8.667 to 49.259 µm"
inputs = { D_1 = "30 mm", D_2 = "50 mm", n_h = "7", n_s = "6", pos = "4" }
expect = { D = "38.72983346 mm", i = "1.5612430 µm", IT_h = "24.979888 µm", IT_s = "15.612430 µm", es = "-8.6671342 µm", ei = "-24.279564 µm", C_min = "8.6671342 µm", C_max = "49.259452 µm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "40 mm H7/g6 against the tables"
source = "Machinery's 26th Tables 22 and 23 (as Lacewing Lab's iso_fits holds them), row 30-50 mm: IT7 25, IT6 16, g es −9 µm; so H7/g6 has clearance 9 to 50 µm. The formulas give 8.7 to 49.3 before the standard's rounding."
inputs = { D_1 = "30 mm", D_2 = "50 mm", n_h = "7", n_s = "6", pos = "4" }
expect = { IT_h = "25 µm", IT_s = "16 µm", es = "-9 µm", C_min = "9 µm", C_max = "50 µm" }
tolerance = "4%"
checks = "pass"

[[tests]]
name = "16 mm H9/d9 free running fit against the tables"
source = "Machinery's 26th Tables 22 and 23, row 10-18 mm: IT9 43 µm, d es −50 µm; H9/d9 clearance 50 to 136 µm (formulas: 50.15 to 136.77)"
inputs = { D_1 = "10 mm", D_2 = "18 mm", n_h = "9", n_s = "9", pos = "1" }
expect = { IT_h = "43 µm", es = "-50 µm", ei = "-93 µm", C_min = "50 µm", C_max = "136 µm" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "40 mm H8/f7 close running fit against the tables"
source = "Machinery's 26th Tables 22 and 23, row 30-50 mm: IT8 39, IT7 25, f es −25 µm; H8/f7 clearance 25 to 89 µm (formulas: 24.63 to 88.64)"
inputs = { D_1 = "30 mm", D_2 = "50 mm", n_h = "8", n_s = "7", pos = "3" }
expect = { IT_h = "39 µm", IT_s = "25 µm", es = "-25 µm", C_min = "25 µm", C_max = "89 µm" }
tolerance = "1.6%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 7-7: IT11 at 34 mm against the tables"
source = "Shigley 10th Example 7-7 and Machinery's 26th Table 22: IT11 over 30 to 50 mm is 0.160 mm; the formula gives 100i = 156.1 µm before rounding. H11/h11 then has clearance 0 to 2 IT11."
inputs = { D_1 = "30 mm", D_2 = "50 mm", n_h = "11", n_s = "11", pos = "5" }
expect = { IT_h = "160 µm", IT_s = "160 µm", es = "0 µm", C_max = "320 µm" }
tolerance = "2.5%"
checks = "pass"

[[tests]]
name = "Hand check: H8/g7 over 500 to 630 mm, the large-size tolerance unit"
source = "By hand: D = √(500 × 630) = 561.2486 mm, I = 0.004 × 561.2486 + 2.1 = 4.344994 µm, IT8 = 25I = 108.6249, IT7 = 16I = 69.51991 µm (ISO 286 tabulates 110 and 70); es = −2.5 × 561.2486^0.34 = −21.51061 µm"
inputs = { D_1 = "500 mm", D_2 = "630 mm", n_h = "8", n_s = "7", pos = "4" }
expect = { D = "561.24861 mm", i = "4.3449944 µm", IT_h = "108.62486 µm", IT_s = "69.519911 µm", es = "-21.510614 µm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "A 3150 to 4000 mm row: beyond the system's sizes"
source = "ISO 286-1: the standard tolerances are defined up to 3150 mm"
inputs = { D_1 = "3150 mm", D_2 = "4000 mm", n_h = "7", n_s = "6", pos = "4" }
expect = { D = "3549.6479 mm" }
tolerance = "1e-6"
checks = "fail"
+++

## ISO 286 hole-basis clearance fit from the formulas

The ISO system of limits and fits (ISO 286; ANSI B4.2 and BS 4500 follow it) is built from a few formulas, which its tables round. This template works a hole-basis clearance fit from those formulas: an H hole (its lower deviation zero) on a shaft of position d, e, f, g or h, the positions that give clearance. For the values to put on a drawing, use the tables of ISO 286-2 (or Machinery's Handbook's copy) with the limits and fit template: the standard rounds each tolerance and deviation, so the formulas here differ from the tables by up to a few per cent (more in the smallest size rows).

The formulas are for sizes from 1 to 3150 mm. Every size within a row takes the values at the row's geometric mean D. Rows are "over D₁ up to D₂": a size on a boundary belongs to the lower row. The first row, up to 3 mm, uses 1 mm as its lower limit.

### Size row and grades

The main size rows up to 500 mm are 3, 6, 10, 18, 30, 50, 80, 120, 180, 250, 315, 400 and 500 mm; above that 630, 800, 1000, 1250, 1600, 2000, 2500 and 3150 mm.

```calc {description="Lower limit of the size row (over)"}
D_1 = 30 mm_
```

```calc {description="Upper limit of the size row (up to and including)"}
D_2 = 50 mm_
```

```calc {description="Hole tolerance grade, 5 to 18 (7 for H7)"}
n_h = 7_
```

```calc {description="Shaft tolerance grade, 5 to 18 (6 for g6)"}
n_s = 6_
```

```calc {description="Shaft position: 1 d, 2 e, 3 f, 4 g, 5 h"}
pos = 4_
```

### Standard tolerances

The standard tolerance unit is i = 0.45∛D + 0.001D µm up to 500 mm (D in mm) and I = 0.004D + 2.1 µm above. Grades IT5 to IT18 are fixed multiples of it, rising by about 1.6 (the R5 series) from IT6.

```calc
function gradeFactor(n)
    return { 7 if n == 5; 10 if n == 6; 16 if n == 7; 25 if n == 8; 40 if n == 9; 64 if n == 10; 100 if n == 11; 160 if n == 12; 250 if n == 13; 400 if n == 14; 640 if n == 15; 1000 if n == 16; 1600 if n == 17; 2500 if n == 18 }
end
```

```calc
> Geometric mean of the size row
D = sqrt(max(D_1, 1 'mm') D_2) = ?? mm
> Standard tolerance unit
i = { (0.45 (D / 1 'mm')^(1/3) + 0.001 D / 1 'mm') × 1 'µm' if D ≤ 500 'mm'; (0.004 D / 1 'mm' + 2.1) × 1 'µm' otherwise } = ?? µm
> Hole standard tolerance
IT_h = gradeFactor(n_h) i = ?? µm
> Shaft standard tolerance
IT_s = gradeFactor(n_s) i = ?? µm
```

### Shaft deviations

The fundamental deviation of a clearance shaft is its upper deviation es, a function of D alone (D in mm, es in µm): −16D^0.44 for d, −11D^0.41 for e, −5.5D^0.41 for f, −2.5D^0.34 for g, and zero for h. The lower deviation is one standard tolerance below (Shigley Eq. 7-37). Holes D to G mirror them, EI = −es.

```calc
> Upper deviation of the shaft, its fundamental deviation
es = { -16 (D / 1 'mm')^0.44 if pos == 1; -11 (D / 1 'mm')^0.41 if pos == 2; -5.5 (D / 1 'mm')^0.41 if pos == 3; -2.5 (D / 1 'mm')^0.34 if pos == 4; 0 if pos == 5 } × 1 'µm' = ?? µm
> Lower deviation of the shaft
ei = es - IT_s = ?? µm
```

### Limits and fit

The basic size is taken at the row's geometric mean; the limits of any size D₀ in the row are D₀ plus the same deviations.

```calc
> Largest and smallest hole, H: EI = 0, ES = IT
D_(h,max) = D + IT_h = ?? mm
D_(h,min) = D = ?? mm
> Largest and smallest shaft
d_max = D + es = ?? mm
d_min = D + ei = ?? mm
> Least clearance: the smallest hole on the largest shaft
C_min = -es = ?? µm
> Greatest clearance: the largest hole on the smallest shaft
C_max = IT_h - ei = ?? µm
```

### Check

The formulas are defined up to 3150 mm.

```calc
@test D_2 ≤ 3150 'mm'
```
