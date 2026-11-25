+++
cruncha-template = 1
id = "mechanical.boltTorque.simple"
version = "1.0.0"
title = "Bolt tightening torque from preload, VDI 2230"
summary = "The torque that tightens a 60° thread (ISO metric or Unified) to a given preload, with thread and head friction taken separately (VDI 2230 Part 1 Eq. 127), split into the part that stretches the bolt, thread friction and head friction, with the equivalent nut factor K."
category = ["Mechanical engineering", "Fastening"]
tags = ["bolt", "torque", "tightening torque", "preload", "clamp load", "friction", "nut factor", "VDI 2230", "R13", "metric thread", "UNC"]
from = "lacewing:mechanical.boltTorqueSimple"
license = "GPL-3.0-or-later"
inputs = ["d", "P", "d_W", "d_h", "μ_G", "μ_K", "F_M"]
outputs = ["d_2", "D_Km", "M_P", "M_G", "M_K", "M_A", "K"]

[[references]]
work = "VDI 2230 Part 1, Systematic calculation of highly stressed bolted joints: joints with one cylindrical bolt"
edition = "2015"
locator = "Sec. 5.4.3.1, Eqs. 119 to 129; calculation step R13 (Eq. R13/1); Table A1 (tightening torques) and Table A5 (friction classes)"

[[references]]
work = "ISO 273, Fasteners: clearance holes for bolts and screws"
edition = "1979"
locator = "Medium series"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Eq. 8-27 (torque coefficient K); Tables 8-1 and 8-2 (coarse pitches)"

[[tests]]
name = "M12 10.9 at μ = 0.10, ISO 4014 head: Table A1's 108 N·m"
source = "Lacewing Lab bolt_torque_test, after VDI 2230 Part 1:2015 Table A1: M12, FM,Tab 64.8 kN at μG = μK = 0.10, dW 16.63 mm (ISO 4014), dh 13.5 mm (ISO 273 medium): MA 108 N·m (Table A1 rounds to about 2 %)"
inputs = { d = "12 mm", P = "1.75 mm", d_W = "16.63 mm", d_h = "13.5 mm", "μ_G" = "0.10", "μ_K" = "0.10", F_M = "64.8 kN" }
expect = { M_A = "108 N·m" }
tolerance = "2%"
checks = "pass"

[[tests]]
name = "M16 at μ = 0.12, ISO 4014 head: Table A1's 302 N·m"
source = "Lacewing Lab bolt_torque_test, after VDI 2230 Part 1:2015 Table A1: M16, FM,Tab 118.8 kN at μ = 0.12, dW 22.49 mm, dh 17.5 mm: MA 302 N·m"
inputs = { d = "16 mm", P = "2 mm", d_W = "22.49 mm", d_h = "17.5 mm", "μ_G" = "0.12", "μ_K" = "0.12", F_M = "118.8 kN" }
expect = { M_A = "302 N·m" }
tolerance = "2%"
checks = "pass"

[[tests]]
name = "M20 at μ = 0.10, ISO 4014 head: Table A1's 517 N·m"
source = "Lacewing Lab bolt_torque_test, after VDI 2230 Part 1:2015 Table A1: M20, FM,Tab 190 kN at μ = 0.10, dW 28.19 mm, dh 22 mm: MA 517 N·m"
inputs = { d = "20 mm", P = "2.5 mm", d_W = "28.19 mm", d_h = "22 mm", "μ_G" = "0.10", "μ_K" = "0.10", F_M = "190 kN" }
expect = { M_A = "517 N·m" }
tolerance = "2%"
checks = "pass"

[[tests]]
name = "M24 at μ = 0.14, ISO 4014 head: Table A1's 1329 N·m"
source = "Lacewing Lab bolt_torque_test, after VDI 2230 Part 1:2015 Table A1: M24, FM,Tab 305 kN at μ = 0.14, dW 33.61 mm, dh 26 mm: MA 1329 N·m"
inputs = { d = "24 mm", P = "3 mm", d_W = "33.61 mm", d_h = "26 mm", "μ_G" = "0.14", "μ_K" = "0.14", F_M = "305 kN" }
expect = { M_A = "1329 N·m" }
tolerance = "2%"
checks = "pass"

[[tests]]
name = "Hand check: M12 at 64.8 kN, μ = 0.10, dW 16.63 mm"
source = "By hand: d2 = 12 − 0.649519 × 1.75 = 10.86334 mm; DKm = (16.63 + 13.5)/2 = 15.065 mm; MP = 64 800 × 0.16 × 1.75 mm = 18.144 N·m; MG = 64 800 × 0.58 × 10.86334 mm × 0.10 = 40.82878 N·m; MK = 64 800 × 15.065/2 mm × 0.10 = 48.8106 N·m; MA = 107.78338 N·m; K = 107.78338/(64 800 × 12 mm) = 0.138610"
inputs = { d = "12 mm", P = "1.75 mm", d_W = "16.63 mm", d_h = "13.5 mm", "μ_G" = "0.10", "μ_K" = "0.10", F_M = "64.8 kN" }
expect = { d_2 = "10.86334 mm", D_Km = "15.065 mm", M_P = "18.144 N·m", M_G = "40.82878 N·m", M_K = "48.8106 N·m", M_A = "107.78338 N·m", K = "0.138610" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "1/2-13 UNC at 10 000 lbf, μ = 0.15, 1.5d head"
source = "By hand: d2 = 0.5 − 0.649519/13 = 0.450037 in; dh 17/32 in, dW 0.75 in (1.5d), DKm = 0.640625 in; MA = 10 000 lbf × (0.16/13 + 0.58 × 0.450037 × 0.15 + 0.3203125 × 0.15) in = 10 000 × 0.0995078 lbf·in = 995.08 lbf·in; K = 995.08/(10 000 × 0.5) = 0.19902, near Shigley's 0.20 for unlubricated steel"
inputs = { d = "0.5 in", P = "0.07692308 in", d_W = "0.75 in", d_h = "0.53125 in", "μ_G" = "0.15", "μ_K" = "0.15", F_M = "10000 lbf" }
expect = { d_2 = "0.450037 in", D_Km = "0.640625 in", M_A = "995.078 lbf·in", K = "0.199016" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Hole larger than the bearing face: the geometry check fails"
source = "As the hand check, with dW 13 mm under the 13.5 mm hole (Lacewing refuses it: the head would have no bearing face)"
inputs = { d = "12 mm", P = "1.75 mm", d_W = "13 mm", d_h = "13.5 mm", "μ_G" = "0.10", "μ_K" = "0.10", F_M = "64.8 kN" }
expect = { d_2 = "10.86334 mm" }
tolerance = "1e-5"
checks = "fail"
+++

## Bolt tightening torque from preload (VDI 2230)

The torque that brings one bolt to the preload F_M when it is tightened by turning the nut or head, by VDI 2230 Part 1:2015, Section 5.4.3.1 and step R13. Friction in the thread and friction under the head are separate coefficients. The thread can be ISO metric or Unified: both have the same 60° basic profile, so the pitch diameter is found the same way.

This is the quick check. It does not include the scatter of preload for one torque (the tightening factor α_A, Table A8), prevailing-torque (locking) elements, or whether the bolt can carry the preload; the VDI 2230 preload and working-checks template does those.

### Bolt and bearing face

Give the nominal diameter and the pitch (the coarse pitch for a standard thread: 1.75 mm for M12, 1/13 in for 1/2-13 UNC). The head's bearing diameter d_W is the outside of the bearing face of the head, nut or washer that turns; a hexagon head bears on roughly 1.4d to 1.5d (see the bolt standard, such as ISO 4014 or ISO 4762). The hole d_h is the clearance hole, or the outside of its chamfer if that is larger; ISO 273's medium series is a common choice (13.5 mm for M12).

```calc {description="Nominal (major) thread diameter"}
d = 12 mm_
```

```calc {description="Thread pitch (1/TPI for inch threads)"}
P = 1.75 mm_
```

```calc {description="Outside diameter of the head's (or nut's) bearing face"}
d_W = 16.63 mm_
```

```calc {description="Clearance hole, or the outside of its chamfer if larger"}
d_h = 13.5 mm_
```

### Friction and preload

Use the smallest friction expected: the torque is then the one that does not overload the bolt, and a larger friction only gives less preload. VDI 2230 Table A5 groups coefficients into classes by surface and lubrication; oiled steel is typically in the 0.10 to 0.16 range. The head friction is often taken equal to the thread friction.

```calc {description="Friction coefficient in the thread"}
μ_G = 0.10_
```

```calc {description="Friction coefficient under the head or nut"}
μ_K = 0.10_
```

```calc {description="Assembly preload the bolt must reach"}
F_M = 64.8 kN_
```

### Torque

The thread moment comes from the inclined plane of the thread, with the flank friction raised by 1/cos 30° for the 60° flanks; VDI 2230 simplifies it to two terms in P and d₂ (Eqs. 121 to 125). The head moment acts at the mean friction diameter of the bearing face, an annulus from the hole to d_W (Eqs. 126, 128, 129). The total is their sum (Eq. 119), which is Eq. 127.

```calc {description="Pitch diameter of the 60° basic profile"}
d_2 = d - 0.649519 P = ?? mm
```

```calc {description="Mean friction diameter under the head, Eqs. 128 and 129"}
D_Km = (d_W + d_h) / 2 = ?? mm
```

```calc
> Part that stretches the bolt (the thread lead, no friction), Eq. 125
M_P = F_M × 0.16 P = ?? N·m
> Thread friction on the 60° flanks, Eqs. 123 to 125
M_G = F_M × 0.58 d_2 μ_G = ?? N·m
> Friction under the head or nut, Eq. 126
M_K = F_M D_Km / 2 × μ_K = ?? N·m
> Tightening torque, Eq. 127 (R13/1)
M_A = M_P + M_G + M_K = ?? N·m
```

The torque is proportional to the preload, so a torque you tighten to gives F_M = M_A / (0.16 P + 0.58 d₂ μ_G + D_Km μ_K / 2). With the largest friction expected, that is the least preload the torque can be relied on for.

```calc {description="Equivalent nut factor (torque coefficient), T = K F d (Shigley Eq. 8-27)"}
K = M_A / (F_M d) = ??
```

The nut factor is for comparison with handbook values: roughly 0.2 for dry steel and 0.15 to 0.18 lubricated.

### Checks

```calc
@test d ≤ d_h < d_W
```

```calc
@test 0 'mm' < P < d / 2
```
