+++
cruncha-template = 1
id = "mechanical.wear.pinOnDisk"
version = "1.0.0"
title = "Wear coefficient from a pin-on-disk test (Juvinall)"
summary = "The dimensionless wear coefficient K of a surface from a pin-on-disk test: the rubbing distance from the track radius, disk speed and time, K = W·H/(F·S) from the volume the surface lost, and the pin's linear wear."
category = ["Mechanical engineering", "Bearings"]
tags = ["wear", "wear coefficient", "pin-on-disk", "wear test", "Archard", "tribology", "Juvinall"]
from = "lacewing:mechanical.wear"
license = "GPL-3.0-or-later"
inputs = ["r", "n", "t", "F", "W", "H_n", "d"]
outputs = ["S", "H", "K", "h"]

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 9.12: Eq. 9.1a, footnote 2 (hardness number as a pressure), Fig. 9.12, Sample Problem 9.2"

[[tests]]
name = "Juvinall Sample Problem 9.2: the copper pin"
source = "Lacewing Lab wear_test, after Juvinall 5th Sample Problem 9.2: copper pin of 80 HV pressed with 20 N on a steel disk at 16 mm radius, 80 rev/min for 2 h, losing 2.7 mm³: S = 9.65 × 10⁵ mm, K = 1.10 × 10⁻⁴; a 4 mm pin wears 0.21 mm"
inputs = { r = "16 mm", n = "80 rpm", t = "2 h", F = "20 N", W = "2.7 mm³", H_n = "80", d = "4 mm" }
expect = { S = "9.65e5 mm", K = "1.10e-4", h = "0.21 mm" }
tolerance = "2.5%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 9.2: the steel disk"
source = "Lacewing Lab wear_test, after Juvinall 5th Sample Problem 9.2: the steel disk, 210 HB, lost 0.65 mm³ over the same distance: K = 6.94 × 10⁻⁵"
inputs = { r = "16 mm", n = "80 rpm", t = "2 h", F = "20 N", W = "0.65 mm³", H_n = "210", d = "4 mm" }
expect = { K = "6.94e-5" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Sample Problem 9.2's copper pin exactly"
source = "By hand: S = 2π × 0.016 × (80/60) × 7200 = 965.09726 m, H = 80 × 9.80665 = 784.532 MPa, K = 2.7e-9 × 784.532e6/(20 × 965.09726) = 1.0974212e-4, h = 2.7/(π 4²/4) = 0.21485917 mm"
inputs = { r = "16 mm", n = "80 rpm", t = "2 h", F = "20 N", W = "2.7 mm³", H_n = "80", d = "4 mm" }
expect = { S = "965.09726 m", H = "784.532 MPa", K = "1.0974212e-4", h = "0.21485917 mm" }
tolerance = "1e-6"
checks = "pass"
+++

## Wear coefficient from a pin-on-disk test

A pin pressed with a force F on a disk turning at speed n wears a circular track of radius r. The wear equation W = K F S/H (Juvinall Eq. 9.1a) turned round gives the wear coefficient of either surface from the volume it lost (its weight loss over its density) and its own hardness. The coefficient applies to the same pair of materials under the same lubrication, temperature and kind of wear as the test; use it in the Archard wear template.

```calc {description="Radius of the pin's track on the disk"}
r = 16 mm_
```

```calc {description="Disk speed"}
n = 80 rpm_
```

```calc {description="Duration of the test"}
t = 2 h_
```

```calc {description="Normal force on the pin"}
F = 20 N_
```

```calc {description="Volume the surface lost (weight loss over density)"}
W = 2.7 mm³_
```

```calc {description="Hardness number of the surface, HB or HV (kgf/mm²)"}
H_n = 80_
```

```calc {description="Diameter of the pin's worn face"}
d = 4 mm_
```

Cruncha's rpm is an angular speed in radians per unit time, so the track's sliding speed is r n.

```calc
> Rubbing distance, 2πr per revolution
S = r n t = ?? m
> Hardness as a pressure
H = H_n × 1 'kgf/mm²' = ?? MPa
> Wear coefficient, Eq. 9.1a
K = W H / (F S) = ??
> Linear wear of the pin, its worn face flat
h = W / (π d^2 / 4) = ?? mm
```
