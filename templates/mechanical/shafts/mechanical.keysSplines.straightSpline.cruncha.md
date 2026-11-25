+++
cruncha-template = 1
id = "mechanical.keysSplines.straightSpline"
version = "1.0.0"
title = "Straight-sided spline (SAE, ISO 14): side pressure and torque rating"
summary = "A straight-sided (parallel-key) spline by its number of splines, major and minor diameters, depth of engagement and length: the mean radius, the side pressure on the splines under a torque, the SAE torque rating at 1000 psi side pressure, and the torsional shear stress of the shaft at its minor diameter."
category = ["Mechanical engineering", "Shafts"]
tags = ["spline", "straight-sided spline", "parallel spline", "SAE spline", "ISO 14", "shaft", "hub", "torque capacity", "side pressure", "Machinery's Handbook"]
from = "lacewing:mechanical.keysSplines"
license = "GPL-3.0-or-later"
inputs = ["N", "D", "d_m", "h", "L", "T", "S_(s,all)"]
outputs = ["R_m", "p", "T_rated", "T_(rated,1)", "τ_root"]

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "Straight-sided splines: SAE standard proportions (Tables 1-5, pp. 2157-2160; T = 1000 N R h per inch of length)"

[[references]]
work = "ISO 14:1982 Straight-sided splines for cylindrical shafts with internal centering: dimensions, tolerances and verification"
edition = "1982"
locator = "Table 1 (light and medium series: N × d × D and B)"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.6 (a splined shaft about as strong as a round shaft of its minor diameter)"

[[tests]]
name = "SAE 4A spline on a 1 in shaft: 139 lb·in per inch of length"
source = "Lacewing Lab keys_splines_test, after Machinery's 26th Tables 5 and 1 (SAE): four splines, fit A, h = 0.075D, d = 0.850D: R = (1 + 0.85)/4 = 0.4625 in, T = 1000 × 4 × 0.4625 × 0.075 = 138.75 lb·in per inch (the table 139). At 100 lb·in over 1 in by hand: p = 720.72 psi, root shear 16 × 100/(π 0.85³) = 829.30 psi"
inputs = { N = "4", D = "1 in", d_m = "0.85 in", h = "0.075 in", L = "1 in", T = "100 lbf·in", "S_(s,all)" = "20000 psi" }
expect = { R_m = "0.4625 in", "T_(rated,1)" = "138.75 lbf·in", T_rated = "138.75 lbf·in", p = "720.7207 psi", "τ_root" = "829.3032 psi" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "SAE 10C spline on a 3 in shaft, overloaded at 7000 lb·in over 1.5 in"
source = "Lacewing Lab keys_splines_test (Machinery's 26th Tables 5 and 3): ten splines, fit C, h = 0.095D, d = 0.810D: 3869 lb·in per inch. By hand at 7000 lb·in over 1.5 in: R = 1.3575 in, p = 7000/(10 × 1.3575 × 0.285 × 1.5) = 1206.21 psi, over the 1000 psi rating (5803.31 lb·in); root shear 16 × 7000/(π 2.43³) = 2484.56 psi"
inputs = { N = "10", D = "3 in", d_m = "2.43 in", h = "0.285 in", L = "1.5 in", T = "7000 lbf·in", "S_(s,all)" = "20000 psi" }
expect = { "T_(rated,1)" = "3868.875 lbf·in", T_rated = "5803.3125 lbf·in", p = "1206.2077 psi", "τ_root" = "2484.5591 psi" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: ISO 14 light series 6 × 26 × 32, 200 N·m over 30 mm"
source = "By hand: ISO 14 Table 1, light series, 6 splines, d 26, D 32 mm (B 6 mm); h taken as (32 − 26)/2 = 3 mm without chamfers; R = 14.5 mm, p = 200/(6 × 0.0145 × 0.003 × 0.03) = 25.543 MPa, rating at 1000 psi 53.986 N·m (so the SAE basis fails), root shear 16 × 200/(π 0.026³) = 57.954 MPa"
inputs = { N = "6", D = "32 mm", d_m = "26 mm", h = "3 mm", L = "30 mm", T = "200 N·m", "S_(s,all)" = "140 MPa" }
expect = { R_m = "14.5 mm", p = "25.542784 MPa", T_rated = "53.985950 N·m", "τ_root" = "57.953552 MPa" }
tolerance = "1e-6"
checks = "fail"
+++

## Straight-sided spline

A straight-sided spline has N parallel-sided splines (keys) integral with the shaft, engaging grooves in the hub. Machinery's Handbook gives the SAE standard: proportions of the spline width, depth and minor diameter as fractions of the major diameter D for 4, 6, 10 and 16 splines in three fits (A permanent, B to slide without load, C to slide under load; Table 5), and torque ratings T = 1000 N R h per inch of engaged length, which is a side pressure of 1000 psi on the splines at their mean radius R = (D + d)/4. ISO 14 gives metric light- and medium-series sizes (N × d × D, with spline width B) for splines centred on the minor diameter; their depth of engagement is (D − d)/2 less the chamfers.

The side pressure assumes all splines share the load evenly. The shaft under the splines is checked here as a round shaft of the minor diameter, which Juvinall notes is about its strength in torsion; the splines' roots add stress concentration in fatigue.

```calc {description="Number of splines"}
N = 4_
```

```calc {description="Major diameter"}
D = 1 in_
```

```calc {description="Minor diameter (SAE Table 5 as a fraction of D; ISO 14 Table 1)"}
d_m = 0.85 in_
```

```calc {description="Depth of engagement (SAE Table 5 as a fraction of D; (D − d)/2 less chamfers)"}
h = 0.075 in_
```

```calc {description="Engaged length"}
L = 1 in_
```

```calc {description="Torque transmitted"}
T = 100 lbf·in_
```

```calc {description="Allowable shear stress of the shaft"}
S_(s,all) = 20000 psi_
```

```calc
> Mean radius of the splines
R_m = (D + d_m) / 4 = ?? in
> Side pressure on the splines
p = abs(T) / (N R_m h L) = ?? psi
> SAE torque rating per inch of length, at 1000 psi side pressure
T_(rated,1) = 1000 'psi' N R_m h × 1 'in' = ?? lbf·in
> SAE torque rating of the engaged length
T_rated = 1000 'psi' N R_m h L = ?? lbf·in
> Torsional shear stress at the minor diameter
τ_root = 16 abs(T) / (π d_m^3) = ?? psi
```

### Checks

```calc
@test abs(T) ≤ T_rated
```

```calc
@test τ_root ≤ S_(s,all)
```
