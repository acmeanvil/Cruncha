+++
cruncha-template = 1
id = "structural.grooveWelds.flare"
version = "1.0.0"
title = "Flare groove weld strength, AISC 360 J2 (LRFD or ASD)"
summary = "Available strength of a flare-bevel or flare-V groove weld, such as a bar or an HSS corner against a plate: effective throat from the radius of the joint surface (Table J2.2), and the weld metal and base metal at the fusion zone by Table J2.5 for tension, compression or shear, by LRFD or ASD."
category = ["Structural engineering", "Connections"]
tags = ["weld", "groove weld", "flare groove", "flare-bevel", "flare-V", "HSS", "round bar", "effective throat", "AISC 360", "J2", "Table J2.2", "Table J2.5", "LRFD", "ASD", "E70"]
from = "lacewing:structural.grooveWelds"
license = "GPL-3.0-or-later"
inputs = ["LRFD", "R", "k_f", "bevel", "n_f", "u", "l", "F_y", "F_u", "F_EXX", "F_(EXX,m)", "load", "P_r"]
outputs = ["E_f", "E", "A_we", "R_(a,w)", "R_(a,b)", "R_a"]

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Sec. J2.1a (effective area, underfill), Table J2.2 and its note a (p. 16.1-126), J2.4 (Eqs. J2-2, J2-3), Table J2.5 and its note b (pp. 16.1-131, 16.1-132), J4.1, J4.2, J4.4"

[[references]]
work = "AISC Design Guide 21, Welded Connections: A Primer for Engineers (Miller)"
edition = "2nd (2017)"
locator = "Sec. 3.4.8 (PJP fusion zone check)"

[[tests]]
name = "Flare-bevel by GMAW on an HSS corner (3/8 in. wall, design 0.349 in.), 6 in., A500 Gr C, E70, shear (LRFD)"
source = "Throat: AISC 360-22 Table J2.2 (verified: flare-bevel by GMAW 5/8 R; R may be taken as 2t for HSS), R = 0.698 in, E = 0.43625 in, as Lacewing Lab groove_welds_test. Strength by hand, Table J2.5 and J4.2: A_we = 2.6175 in²; weld 0.75 × 0.60 × 70 × 2.6175 = 82.45 kips; fusion zone min(1.00 × 0.60 × 50, 0.75 × 0.60 × 62) × 2.6175 = 73.03 kips"
inputs = { LRFD = "1", R = "0.698 in", k_f = "0.625", bevel = "1", n_f = "1", u = "0 in", l = "6 in", F_y = "50 ksi", F_u = "62 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "60 kips" }
expect = { E = "0.43625 in", A_we = "2.6175 in²", "R_(a,w)" = "82.45125 kips", "R_(a,b)" = "73.02825 kips", R_a = "73.02825 kips" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Same HSS flare-bevel in shear, ASD"
source = "By hand, Table J2.5 and J4.2 (verified Ω): weld 0.60 × 70 × 2.6175 / 2.00 = 54.97 kips; fusion zone 0.60 × 50 × 2.6175 / 1.50 = 52.35 and 0.60 × 62 × 2.6175 / 2.00 = 48.686 kips"
inputs = { LRFD = "0", R = "0.698 in", k_f = "0.625", bevel = "1", n_f = "1", u = "0 in", l = "6 in", F_y = "50 ksi", F_u = "62 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "40 kips" }
expect = { "R_(a,w)" = "54.9675 kips", R_a = "48.6855 kips" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Flare-V both sides by SMAW, R = 1 in., 4 in., 50 ksi steel, E70, tension (LRFD)"
source = "Throat: Table J2.2 (verified: flare-V by SMAW 5/8 R), 2 × 5/8 × 1 = 1.25 in, as Lacewing Lab groove_welds_test. By hand, Table J2.5: A_we = 5 in²; weld 0.80 × 0.60 × 70 × 5 = 168 kips; fusion zone 0.75 × 65 × 5 = 243.75 kips"
inputs = { LRFD = "1", R = "1 in", k_f = "0.625", bevel = "0", n_f = "2", u = "0 in", l = "4 in", F_y = "50 ksi", F_u = "65 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "1", P_r = "150 kips" }
expect = { E = "1.25 in", R_a = "168 kips" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Flare-bevel on R = 1/4 in.: no groove throat"
source = "AISC 360-22 Table J2.2 note a (verified): a flare-bevel with R < 3/8 in. takes only a reinforcing fillet weld; as Lacewing Lab groove_welds_test"
inputs = { LRFD = "1", R = "0.25 in", k_f = "0.625", bevel = "1", n_f = "1", u = "0 in", l = "6 in", F_y = "50 ksi", F_u = "62 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "10 kips" }
expect = { E = "0 in" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "Flare-V by GMAW on R = 1/4 in. still counts"
source = "AISC 360-22 Table J2.2 (verified): note a is for flare-bevels only; flare-V by GMAW 3/4 R = 0.1875 in, as Lacewing Lab groove_welds_test"
inputs = { LRFD = "1", R = "0.25 in", k_f = "0.75", bevel = "0", n_f = "1", u = "0 in", l = "6 in", F_y = "50 ksi", F_u = "62 ksi", F_EXX = "70 ksi", "F_(EXX,m)" = "70 ksi", load = "3", P_r = "10 kips" }
expect = { E = "0.1875 in" }
tolerance = "1e-9"
checks = "pass"
+++

## Flare groove weld strength (AISC 360 J2)

Available strength of a **flare groove** weld of length l: a flare-bevel, where a round surface (a bar, a bent plate, the corner of an HSS) meets a flat one, or a flare-V, between two round surfaces. The effective throat is a fraction of the radius of the joint surface (Table J2.2). The weld metal works on it, and the base metal is checked at the fusion zone beside it, each by Table J2.5 for the direction of the load. LRFD compares ϕ R_n with the factored force; ASD compares R_n / Ω with the service force.

Not covered: the reinforcing fillet a flare-bevel on a small radius needs (design that as a fillet weld); whether the detail is prequalified by AWS D1.1; combined loads; fatigue. Loads parallel to the weld axis, and compression in a joint designed to bear, may be neglected in the design of the weld (Table J2.5).

### Method

```calc {description="Design method: 1 LRFD, 0 ASD"}
LRFD = 1_
```

```calc
function avail(R, ϕ, Ω, m)
    return { ϕ R if m == 1; R / Ω otherwise }
end
```

### Joint

The throat factor depends on the groove (flare-bevel or flare-V) and the welding process (Table J2.2). For HSS, R may be taken as twice the design wall thickness.

```calc {description="Radius of the joint surface (HSS: 2 t_des)"}
R = 0.698 in_
```

```calc {description="Effective throat factor of R, AISC 360 Table J2.2"}
k_f = 0.625_
```

```calc {description="1 for a flare-bevel, 0 for a flare-V"}
bevel = 1_
```

```calc {description="Number of flare welds in the joint (1, or 2 for both sides)"}
n_f = 1_
```

```calc {description="Underfill below flush, each weld"}
u = 0 in_
```

```calc {description="Effective length of the weld"}
l = 6 in_
```

### Materials and load

```calc {description="Yield stress of the base metal"}
F_y = 50 ksi_
```

```calc {description="Tensile strength of the base metal"}
F_u = 62 ksi_
```

```calc {description="Filler metal classification strength"}
F_EXX = 70 ksi_
```

```calc {description="Matching filler metal strength for the base metal (AISC 360 J2.6 User Note; AWS D1.1)"}
F_(EXX,m) = 70 ksi_
```

```calc {description="Load: 1 tension normal to the weld axis, 2 compression normal (not designed to bear), 3 shear"}
load = 3_
```

```calc {description="Required force on the joint (factored for LRFD, service for ASD)"}
P_r = 60 kips_
```

### Effective throat

A flare-bevel on a radius under 3/8 in. counts no groove throat (Table J2.2, note a).

```calc
> Effective throat of one flare weld, Table J2.2, less any underfill (J2.1a)
E_f = { 0 'in' if bevel == 1 and R < 0.375 'in'; max(k_f R - u, 0 'in') otherwise } = ?? in
> Effective throat of the joint
E = n_f E_f = ?? in
> Effective area of the weld (J2.1a)
A_we = E l = ?? in²
```

### Strength (Table J2.5)

As for a PJP weld: the weld metal on its effective area, and the base metal at the fusion zone, taken as the same area (Design Guide 21, Sec. 3.4.8).

```calc
> Weld metal, Eq. J2-3: 0.60 F_EXX in tension (ϕ 0.80, Ω 1.88), 0.90 F_EXX in compression (ϕ 0.80, Ω 1.88), 0.60 F_EXX in shear (ϕ 0.75, Ω 2.00)
R_(a,w) = { avail(0.60 F_EXX A_we, 0.80, 1.88, LRFD) if load == 1; avail(0.90 F_EXX A_we, 0.80, 1.88, LRFD) if load == 2; avail(0.60 F_EXX A_we, 0.75, 2.00, LRFD) if load == 3 } = ?? kips
> Base metal at the fusion zone, Eq. J2-2 with J4: F_u in tension (ϕ 0.75, Ω 2.00), F_y in compression (ϕ 0.90, Ω 1.67), the lesser of 0.60 F_y (ϕ 1.00, Ω 1.50) and 0.60 F_u (ϕ 0.75, Ω 2.00) in shear
R_(a,b) = { avail(F_u A_we, 0.75, 2.00, LRFD) if load == 1; avail(F_y A_we, 0.90, 1.67, LRFD) if load == 2; min(avail(0.60 F_y A_we, 1.00, 1.50, LRFD), avail(0.60 F_u A_we, 0.75, 2.00, LRFD)) if load == 3 } = ?? kips
```

```calc {description="Available strength of the joint"}
R_a = min(R_(a,w), R_(a,b)) = ?? kips
```

### Checks

```calc
@test E > 0 'in'
```

```calc
@test P_r ≤ R_a
```

Filler metal may be equal to or weaker than matching, or one strength level (10 ksi) stronger (Table J2.5 and its note b).

```calc
@test F_EXX ≤ F_(EXX,m) + 10 'ksi'
```
