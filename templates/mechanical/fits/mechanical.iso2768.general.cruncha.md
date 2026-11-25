+++
cruncha-template = 1
id = "mechanical.iso2768.general"
version = "1.0.0"
title = "General tolerances (ISO 2768): limits and the derived geometrical tolerances"
summary = "What a title-block note such as ISO 2768-mK allows a feature without tolerances of its own: the limits of a linear dimension from its Part 1 deviation, the lateral deviation an angular tolerance allows along the shorter side, and the circularity and parallelism Part 2 derives from the size tolerance, the run-out and the straightness or flatness tolerances, with a measured size checked within the limits."
category = ["Mechanical engineering", "Fits and tolerances"]
tags = ["general tolerances", "ISO 2768", "ISO 2768-1", "ISO 2768-2", "title block", "linear dimension", "angular dimension", "circularity", "roundness", "parallelism", "run-out", "flatness", "straightness", "mK"]
from = "lacewing:mechanical.iso2768"
license = "GPL-3.0-or-later"
inputs = ["l", "Δ_l", "Δ_α", "l_s", "t_r", "t_st", "l_a"]
outputs = ["l_max", "l_min", "t_d", "e_α", "t_circ", "t_par"]

[[references]]
work = "ISO 2768-1:1989 General tolerances - Part 1: Tolerances for linear and angular dimensions without individual tolerance indications"
edition = "1989"
locator = "Clauses 1, 4, 5, 6; Tables 1 (linear dimensions) and 3 (angular dimensions); Annex A.2"

[[references]]
work = "ISO 2768-2:1989 General tolerances - Part 2: Geometrical tolerances for features without individual tolerance indications"
edition = "1989"
locator = "5.1.2 (circularity), 5.2.2 (parallelism), 5.2.5 (coaxiality), clause 7; Tables 1 (straightness and flatness) and 4 (circular run-out); Annex B.2 and B.4"

[[references]]
work = "Nielsen, The ISO Geometrical Product Specifications Handbook"
edition = "2012"
locator = "Ch. 8, General tolerances"

[[tests]]
name = "ISO 2768-2 Annex B.2, Example 2: Ø25 under ISO 2768-mK"
source = "Lacewing Lab iso2768_test, after ISO 2768-2:1989 Annex B.2: ±0.2 (class m) gives a diameter tolerance of 0.4, more than the 0.2 run-out of class K, so circularity is 0.2"
inputs = { l = "25 mm", "Δ_l" = "0.2 mm", t_r = "0.2 mm", t_st = "0.1 mm", l_a = "25.1 mm" }
expect = { l_max = "25.2 mm", l_min = "24.8 mm", t_d = "0.4 mm", t_circ = "0.2 mm", t_par = "0.4 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "ISO 2768-1 Annex A.2: Ø35 at customary medium accuracy"
source = "Lacewing Lab iso2768_test, after ISO 2768-1:1989 Annex A.2: class m, ±0.3 on 35 (Table 1, over 30 up to 120), limits 34.7 to 35.3"
inputs = { l = "35 mm", "Δ_l" = "0.3 mm", l_a = "34.8 mm" }
expect = { l_max = "35.3 mm", l_min = "34.7 mm", t_d = "0.6 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "ISO 2768-2 5.2.2: parallelism the greater of size and flatness tolerances"
source = "Lacewing Lab iso2768_test, after ISO 2768-2:1989 5.2.2 and Annex B.4: a size tolerance of 0.1 (±0.05) with a 0.4 flatness tolerance gives parallelism 0.4"
inputs = { l = "10 mm", "Δ_l" = "0.05 mm", t_r = "0.2 mm", t_st = "0.4 mm", l_a = "10 mm" }
expect = { t_d = "0.1 mm", t_par = "0.4 mm", t_circ = "0.1 mm" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: ±1° on a 10 mm shorter side"
source = "By hand: class m up to 10 mm is ±1° (ISO 2768-1 Table 3); 10 tan 1° = 0.174551 mm at the end of the side"
inputs = { "Δ_α" = "1°", l_s = "10 mm" }
expect = { "e_α" = "0.17455065 mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Ø35 m measured at 35.4: over its upper limit"
source = "ISO 2768-1 Annex A.2 limits 34.7 to 35.3; Part 1 clause 6: such a part is rejected only if its function is impaired"
inputs = { l = "35 mm", "Δ_l" = "0.3 mm", l_a = "35.4 mm" }
expect = { l_max = "35.3 mm" }
tolerance = "1e-9"
checks = "fail"
+++

## General tolerances (ISO 2768)

A drawing note such as **ISO 2768-mK** gives every dimension and feature that has no tolerance of its own a general tolerance: the Part 1 class (f fine, m medium, c coarse, v very coarse) for linear and angular dimensions, and the Part 2 class (H, K, L) for straightness and flatness, perpendicularity, symmetry and circular run-out. A trailing E adds the envelope requirement on every single feature of size.

The tolerance values are in the standards' tables, by class and size range; read them there and enter them here. A size on a range edge belongs to the range it is "up to". Part 2 gives no general tolerance for cylindricity, profile, angularity, position or total run-out; cylindricity's components are each limited by their own general tolerances. Coaxiality may in an extreme case reach the radial run-out of Part 2 Table 4 (5.2.5). Part 2 is written for the independency principle of ISO 8015.

A part that exceeds a general tolerance is not rejected automatically unless its function is impaired (Part 1 clause 6, Part 2 clause 7). The standards are from 1989; check their standing for the drawing's use, since some current practice prefers a general profile tolerance on the main datum system.

### Linear dimension

```calc {description="Nominal size of the dimension (an external or internal size, step, diameter, radius or distance)"}
l = 25 mm_
```

```calc {description="Permissible deviation ± for the class and size, ISO 2768-1 Table 1"}
Δ_l = 0.2 mm_
```

```calc {description="Measured size of the produced part"}
l_a = 25.1 mm_
```

```calc
> Upper and lower limits
l_max = l + Δ_l = ?? mm
l_min = l - Δ_l = ?? mm
> Size tolerance, the difference between the limits
t_d = 2 Δ_l = ?? mm
```

### Angular dimension

The angular tolerance depends on the length of the shorter side of the angle and controls the general orientation of the lines, not their form (Part 1 4.2). At the end of the shorter side it allows a lateral deviation of l_s tan Δα.

```calc {description="Permissible angular deviation ± for the class and shorter side, ISO 2768-1 Table 3"}
Δ_α = 1°_
```

```calc {description="Length of the shorter side of the angle"}
l_s = 10 mm_
```

```calc {description="Lateral deviation the angular tolerance allows at the end of the shorter side"}
e_α = l_s tan(Δ_α) = ?? mm
```

### Circularity and parallelism

Part 2 gives these no table of their own. Circularity is the diameter tolerance, but no more than the circular run-out of Table 4 (5.1.2, Annex B.2). Parallelism is the greater of the size tolerance and the straightness or flatness tolerance, the longer feature being the datum (5.2.2, Annex B.4). For a toleranced diameter, enter half its tolerance as Δ_l.

```calc {description="Circular run-out tolerance for the Part 2 class, ISO 2768-2 Table 4"}
t_r = 0.2 mm_
```

```calc {description="Straightness or flatness tolerance for the class and the feature's length, ISO 2768-2 Table 1"}
t_st = 0.1 mm_
```

```calc
> Circularity, 5.1.2
t_circ = min(t_d, t_r) = ?? mm
> Parallelism, 5.2.2
t_par = max(t_d, t_st) = ?? mm
```

### Check

```calc
@test l_min ≤ l_a ≤ l_max
```
