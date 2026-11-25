+++
cruncha-template = 1
id = "mechanical.powerScrews.torque"
version = "1.0.0"
title = "Power screw torque, efficiency and self-locking"
summary = "Torque to raise and to lower a load with a square or Acme power screw, thread and collar friction included, the overall efficiency in raising, and whether the thread alone holds the load (self-locking)."
category = ["Mechanical engineering", "Fastening"]
tags = ["power screw", "lead screw", "jack", "Acme", "square thread", "torque", "raise", "lower", "efficiency", "self-locking", "collar friction", "overhauling"]
from = "lacewing:mechanical.powerScrews"
license = "GPL-3.0-or-later"
inputs = ["d", "p", "N_s", "α", "F", "f", "f_c", "d_c"]
outputs = ["d_m", "l", "λ", "T_R", "T_L", "T_c", "T_raise", "T_lower", "e", "e_s", "r_lock"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-2, Fig. 8-3, Eqs. 8-1 to 8-6 (torques, self-locking, efficiency, collar); Table 8-5 (thread friction); Example 8-1"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 10.2 and 10.3, Eqs. 10.1, 10.4 to 10.8; Sample Problem 10.1"

[[tests]]
name = "Shigley Ex. 8-1: 32 mm square thread, 4 mm pitch, double, 6.4 kN"
source = "Lacewing Lab threads_test, after Shigley 10th Example 8-1 (f = fc = 0.08, 40 mm collar): raise 26.18 N·m, lower 9.77 N·m, e = 0.311; the lowering thread torque is negative, so the thread alone is not self-locking"
inputs = { d = "32 mm", p = "4 mm", N_s = "2", "α" = "0°", F = "6.4 kN", f = "0.08", f_c = "0.08", d_c = "40 mm" }
expect = { T_raise = "26.18 N·m", T_lower = "9.77 N·m", e = "0.311", T_c = "10.24 N·m" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 8-1 exactly"
source = "By hand: dm = 30 mm, l = 8 mm; TR = 6400·0.015·(8 + π·0.08·30)/(π·30 − 0.08·8) = 15.936955 N·m; TL = 6400·0.015·(π·0.08·30 − 8)/(π·30 + 0.08·8) = −0.465572 N·m; tan λ = 8/(30π) = 0.0848826; e = 6400·0.008/(2π·26.176955) = 0.311294"
inputs = { d = "32 mm", p = "4 mm", N_s = "2", "α" = "0°", F = "6.4 kN", f = "0.08", f_c = "0.08", d_c = "40 mm" }
expect = { d_m = "30 mm", l = "8 mm", T_R = "15.9369547 N·m", T_L = "-0.465571571 N·m", T_raise = "26.1769547 N·m", T_lower = "9.77442843 N·m", e = "0.311294158", e_s = "0.511310549", r_lock = "0.942477796" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Juvinall SP 10.1: 1 in double Acme jack, 1000 lb, starting friction"
source = "Lacewing Lab threads_test, after Juvinall & Marshek 5th Sample Problem 10.1(b): f = 0.16, fc = 0.12 on a 1.5 in collar, 5 threads per inch: 231.3 lb·in to raise (141.3 + 90), 100.4 lb·in to lower, λ = 8.05°. Shigley's sec α form is within 0.05 % of Juvinall's normal-plane angle here"
inputs = { d = "1 in", p = "0.2 in", N_s = "2", "α" = "14.5°", F = "1000 lbf", f = "0.16", f_c = "0.12", d_c = "1.5 in" }
expect = { T_raise = "231.3 lbf·in", T_R = "141.3 lbf·in", T_lower = "100.4 lbf·in", "λ" = "8.05°", T_c = "90 lbf·in" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Juvinall SP 10.1(b) comment: the same jack with a square thread"
source = "Lacewing Lab threads_test, after Juvinall & Marshek 5th Sample Problem 10.1 comment: Eqs. 10.4a and 10.5a give 228.8 and 98.2 lb·in"
inputs = { d = "1 in", p = "0.2 in", N_s = "2", "α" = "0°", F = "1000 lbf", f = "0.16", f_c = "0.12", d_c = "1.5 in" }
expect = { T_raise = "228.8 lbf·in", T_lower = "98.2 lbf·in" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Juvinall SP 10.1(c): running friction, efficiency, overhauling"
source = "Lacewing Lab threads_test, after Juvinall & Marshek 5th Sample Problem 10.1(c): f = 0.12, fc = 0.09: 189 lb·in to raise (121.5 + 67.5), e = 33.7 %, the screw alone 52 %; at running friction the screw overhauls"
inputs = { d = "1 in", p = "0.2 in", N_s = "2", "α" = "14.5°", F = "1000 lbf", f = "0.12", f_c = "0.09", d_c = "1.5 in" }
expect = { T_raise = "189.0 lbf·in", T_R = "121.5 lbf·in", e = "0.337", e_s = "0.52" }
tolerance = "1%"
checks = "pass"
+++

## Power screw torque, efficiency and self-locking

The torque to raise and to lower an axial load with a square or Acme power screw (a jack, press, vice or lead screw), the efficiency in raising, and whether the thread alone holds the load. The thread is treated as a block pushed along the unwrapped helix, with a thread depth of p/2 (Shigley Fig. 8-3). An Acme thread's flank angle raises the thread friction by sec α (Shigley Eq. 8-5); Juvinall's exact form, with the angle in the plane normal to the thread, differs by a fraction of a per cent at ordinary lead angles. Ball screws are outside this.

### Screw and load

```calc {description="Major (outside) diameter of the screw"}
d = 32 mm_
```

```calc {description="Pitch, the axial distance between adjacent threads"}
p = 4 mm_
```

```calc {description="Number of thread starts"}
N_s = 2_
```

```calc {description="Thread half-angle: 0° square, 14.5° Acme"}
α = 0°_
```

```calc {description="Axial load raised or held"}
F = 6.4 kN_
```

Shigley Table 8-5 gives running and starting values of the thread friction for steel screws on cast iron or bronze nuts; use the starting value for the torque to break the load away.

```calc {description="Coefficient of friction between screw and nut threads"}
f = 0.08_
```

```calc {description="Coefficient of friction of the thrust collar"}
f_c = 0.08_
```

Enter a collar diameter of 0 when the thrust is taken on a rolling bearing.

```calc {description="Mean diameter of the thrust collar's bearing face"}
d_c = 40 mm_
```

### Geometry

```calc
> Mean thread diameter, Fig. 8-3
d_m = d - p / 2 = ?? mm
> Lead: how far the nut moves in one turn
l = N_s p = ?? mm
> Lead angle at the mean diameter
λ = atan(l / (π d_m)) = ?? °
```

### Torques

```calc
> Thread torque to raise the load, Eqs. 8-1 and 8-5
T_R = F d_m / 2 (l + π f d_m sec(α)) / (π d_m - f l sec(α)) = ?? N·m
> Thread torque to lower the load, Eq. 8-2 with sec α
T_L = F d_m / 2 (π f d_m sec(α) - l) / (π d_m + f l sec(α)) = ?? N·m
> Collar friction torque, Eq. 8-6
T_c = F f_c d_c / 2 = ?? N·m
```

```calc {description="Total torque to raise the load"}
T_raise = T_R + T_c = ?? N·m
```

```calc {description="Total torque to lower the load"}
T_lower = T_L + T_c = ?? N·m
```

### Efficiency and self-locking

```calc {description="Overall efficiency in raising, collar friction included, Eq. 8-4"}
e = F l / (2 π T_raise) = ??
```

```calc {description="Efficiency of the screw alone, collar neglected (as on a ball thrust bearing)"}
e_s = F l / (2 π T_R) = ??
```

The thread alone holds the load when its friction exceeds the slope of the lead, f sec α > tan λ (Eq. 8-3): then T_L is positive. A ratio above 1 is self-locking; below 1 the load turns the screw (it overhauls) unless the collar or a brake holds it. A screw self-locking under static load can still creep under vibration.

```calc {description="Self-locking ratio f sec α / tan λ: above 1, the thread alone holds the load"}
r_lock = f sec(α) / tan(λ) = ??
```
