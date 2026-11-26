+++
cruncha-template = 1
id = "structural.basePlates.concentric"
version = "1.0.0"
title = "Column base plate, concentric axial load, AISC 360 J8 and Manual Part 14 (LRFD or ASD)"
summary = "Concrete bearing strength under a column base plate (J8) and the plate's minimum thickness by the cantilever method of the AISC Manual Part 14 (m, n and λn′), by LRFD or ASD, checked against the required axial compression and the plate supplied."
category = ["Structural engineering", "Connections"]
tags = ["base plate", "column base", "concrete bearing", "AISC 360", "J8", "AISC Manual Part 14", "Design Guide 1", "Thornton", "cantilever", "LRFD", "ASD", "W-shape"]
from = "lacewing:structural.basePlates"
license = "GPL-3.0-or-later"
inputs = ["LRFD", "P_r", "B", "N", "d", "b_f", "f_c′", "A_2", "F_y", "t_p"]
outputs = ["A_1", "P_p", "P_(p,a)", "m", "n", "X", "λ", "λn′", "l", "t_min"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J8 (Eqs. J8-1, J8-2; ϕc = 0.65, Ωc = 2.31), pp. 16.1-148, 16.1-149"

[[references]]
work = "AISC Steel Construction Manual"
edition = "15th"
locator = "Part 14, pp. 14-5 and 14-6: Eqs. 14-2 to 14-5, 14-6a and 14-6b, 14-7a and 14-7b (after Thornton, 1990)"

[[references]]
work = "AISC Design Guide 1, Column Base Plates (DeWolf and Ricker)"
edition = "1st (1990; third printing 2003)"
locator = "Example 2 (LRFD), p. 6: W10×100 on a 14 × 17 in. plate"

[[tests]]
name = "DG1 Example 2: W10×100 on a 14 × 17 in. plate, A2 = 4 A1, 728 kips (LRFD)"
source = "AISC Design Guide 1, 1st ed., Example 2, p. 6 (verified): Pu = 1.2(180) + 1.6(320) = 728 kips, m = 3.23 in, n = 2.86 in, tp = 1.40 in (1 1/2 in. used), A2 = 952 in². The example's bearing used the 1986 ϕc = 0.60; ϕP_p here is AISC 360-22 by hand: 0.65 × 0.85 × 3 × 238 × 2 = 789.0 kips; X = 0.922, so λ = 1"
inputs = { LRFD = "1", P_r = "728 kips", B = "14 in", N = "17 in", d = "11.1 in", b_f = "10.34 in", "f_c′" = "3 ksi", A_2 = "952 in²", F_y = "36 ksi", t_p = "1.5 in" }
expect = { m = "3.23 in", n = "2.86 in", t_min = "1.40 in", "P_(p,a)" = "788.97 kips", "λ" = "1" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Same plate, ASD, 500 kips service (180 D + 320 L)"
source = "By hand, AISC 360-22 J8 and Manual 15th Eqs. 14-6b and 14-7b (verified): P_p/Ωc = 0.85 × 3 × 238 × 2 / 2.31 = 525.45 kips; X = 0.99876 × 500 / 525.45 = 0.9504, λ = 1; l = m = 3.2275 in; t_min = 3.2275 √(1.67 × 2 × 500 / (36 × 238)) = 1.4249 in"
inputs = { LRFD = "0", P_r = "500 kips", B = "14 in", N = "17 in", d = "11.1 in", b_f = "10.34 in", "f_c′" = "3 ksi", A_2 = "952 in²", F_y = "36 ksi", t_p = "1.5 in" }
expect = { "P_(p,a)" = "525.45 kips", "λ" = "1", t_min = "1.4249 in" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Same plate on a pier no larger than the plate: bearing fails (LRFD)"
source = "By hand, Eq. J8-1 (A2 = A1): ϕP_p = 0.65 × 0.85 × 3 × 238 = 394.5 kips < 728 kips"
inputs = { LRFD = "1", P_r = "728 kips", B = "14 in", N = "17 in", d = "11.1 in", b_f = "10.34 in", "f_c′" = "3 ksi", A_2 = "238 in²", F_y = "36 ksi", t_p = "1.5 in" }
expect = { "P_(p,a)" = "394.485 kips", "X" = "1" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Same plate at 300 kips: λ < 1 (LRFD)"
source = "By hand, Manual 15th Eqs. 14-4 to 14-7a: X = 4 × 11.1 × 10.34 / 21.44² × 300 / 788.97 = 0.37977; λ = 2√X / (1 + √(1 − X)) = 0.6895; λn′ = 0.6895 × √(11.1 × 10.34) / 4 = 1.847 in; l = m = 3.2275 in; t_min = 3.2275 × √(600 / (0.9 × 36 × 238)) = 0.9003 in"
inputs = { LRFD = "1", P_r = "300 kips", B = "14 in", N = "17 in", d = "11.1 in", b_f = "10.34 in", "f_c′" = "3 ksi", A_2 = "952 in²", F_y = "36 ksi", t_p = "1 in" }
expect = { "X" = "0.37977", "λ" = "0.6895", "λn′" = "1.847 in", l = "3.2275 in", t_min = "0.9003 in" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "728 kips on a 1 1/4 in. plate: too thin (LRFD)"
source = "As DG1 Example 2: t_min = 1.40 in > 1.25 in"
inputs = { LRFD = "1", P_r = "728 kips", B = "14 in", N = "17 in", d = "11.1 in", b_f = "10.34 in", "f_c′" = "3 ksi", A_2 = "952 in²", F_y = "36 ksi", t_p = "1.25 in" }
expect = { t_min = "1.4025 in" }
tolerance = "0.05%"
checks = "fail"
+++

## Column base plate under concentric compression

A base plate under an axially loaded W-shape column: the concrete's bearing strength under the plate (AISC 360-22 J8) and the least plate thickness by the cantilever method of the AISC Manual Part 14, which takes the plate as bending about the column's footprint under uniform bearing. LRFD compares ϕ_c P_p with the factored load; ASD compares P_p / Ω_c with the service load.

Axial compression only. Moments, uplift and anchor rods, shear transfer and the column-to-plate weld (AISC Design Guide 1) are separate checks, as is the footing or pier below.

### Method

```calc {description="Design method: 1 LRFD, 0 ASD"}
LRFD = 1_
```

```calc
function avail(R, ϕ, Ω, m)
    return { ϕ R if m == 1; R / Ω otherwise }
end
```

### Column, plate and concrete

```calc {description="Required axial compression (factored for LRFD, service for ASD)"}
P_r = 728 kips_
```

```calc {description="Plate dimension parallel to the column's flanges (across b_f)"}
B = 14 in_
```

```calc {description="Plate dimension parallel to the column's web (along d)"}
N = 17 in_
```

```calc {description="Column depth"}
d = 11.1 in_
```

```calc {description="Column flange width"}
b_f = 10.34 in_
```

```calc {description="Specified compressive strength of the concrete"}
f_c′ = 3 ksi_
```

A_2 is the largest area of the supporting surface that is geometrically similar to the plate and concentric with it. Enter the plate's own area (B N) when the support is no larger.

```calc {description="Supporting concrete area, similar and concentric (at least B N)"}
A_2 = 952 in²_
```

```calc {description="Yield stress of the plate"}
F_y = 36 ksi_
```

```calc {description="Plate thickness supplied"}
t_p = 1.5 in_
```

### Concrete bearing (J8)

A larger supporting area raises the bearing strength by √(A_2 / A_1), up to twice the strength on the plate's area alone (A_2 ≥ 4 A_1).

```calc
> Plate area
A_1 = B N = ?? in²
> Confinement factor √(A_2 / A_1), 1 to 2
r_A = min(sqrt(max(A_2, A_1) / A_1), 2) = ??
> Nominal bearing strength, Eqs. J8-1 and J8-2
P_p = 0.85 f_c′ A_1 r_A = ?? kips
```

```calc {description="Available bearing strength (ϕ_c = 0.65, Ω_c = 2.31)"}
P_(p,a) = avail(P_p, 0.65, 2.31, LRFD) = ?? kips
```

### Plate thickness (Manual Part 14)

The plate cantilevers m beyond 0.95 d and n beyond 0.8 b_f; between the flanges, λn′ covers a lightly loaded plate that bends between the flanges and web (Thornton). The largest of the three governs. λ = 1 is always conservative.

```calc
> Cantilever along N, Eq. 14-2
m = (N - 0.95 d) / 2 = ?? in
> Cantilever along B, Eq. 14-3
n = (B - 0.8 b_f) / 2 = ?? in
> Eq. 14-6a or 14-6b (at most 1)
X = min(4 d b_f / (d + b_f)^2 P_r / P_(p,a), 1) = ??
> Eq. 14-5 (at most 1)
λ = min(2 sqrt(X) / (1 + sqrt(1 - X)), 1) = ??
> Eq. 14-4
λn′ = λ sqrt(d b_f) / 4 = ?? in
> Critical cantilever length
l = max(m, n, λn′) = ?? in
> Minimum plate thickness, Eq. 14-7a (LRFD) or 14-7b (ASD)
t_min = { l sqrt(2 P_r / (0.90 F_y B N)) if LRFD == 1; l sqrt(1.67 × 2 P_r / (F_y B N)) otherwise } = ?? in
```

### Checks

```calc
@test P_r ≤ P_(p,a)
```

```calc
@test t_p ≥ t_min
```

```calc
@test N ≥ d and B ≥ b_f
```
