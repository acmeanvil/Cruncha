+++
cruncha-template = 1
id = "mechanical.rollingBearings.life"
version = "1.0.0"
title = "Rolling bearing life and selection (Shigley, ISO 281)"
summary = "A ball or roller bearing under radial and axial load: the equivalent load from the catalogue's e, X and Y, the rating life L10 in revolutions and hours, the catalogue rating C10 a design life needs at a reliability by Shigley's Weibull model, the reliability the bearing gives at that life, the adjusted life a1·aISO·L10 of ISO 281, and the static safety factor."
category = ["Mechanical engineering", "Bearings"]
tags = ["rolling bearing", "ball bearing", "roller bearing", "bearing life", "L10", "rating life", "equivalent load", "dynamic load rating", "C10", "static load rating", "reliability", "Weibull", "ISO 281", "ABMA", "Shigley"]
from = "lacewing:mechanical.rollingBearings"
license = "GPL-3.0-or-later"
inputs = ["kind", "C_10", "C_0", "L_R", "x_0", "θ", "b_W", "F_r", "F_a", "V", "e", "X_1", "Y_1", "X_2", "Y_2", "X_0", "Y_0", "a_f", "n", "L_D", "R_D", "a_1", "a_ISO", "s_(0,min)"]
outputs = ["p", "n_r", "X", "Y", "F_e", "F_D", "L_10", "L_(10,h)", "x_D", "C_req", "R", "L_nm", "L_(nm,h)", "P_0", "s_0"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 11-3 to 11-7: Eqs. 11-1 and 11-3 (load-life), 11-8 and 11-9 (Weibull life and the rating needed), 11-12 (equivalent load), 11-21 (reliability); Tables 11-1 (X, Y, e), 11-4 (design lives), 11-5 (application factors), 11-6 (Weibull parameters); Examples 11-3, 11-4, 11-9, 11-11"

[[references]]
work = "ISO 281 Rolling bearings — Dynamic load ratings and rating life"
edition = "ISO 281:2007"
locator = "Modified rating life Lnm = a1·aISO·L10; life modification factor for reliability a1 (as SKF Rolling bearings 17000/1 EN Table 3 and JTEKT Basic Bearing Knowledge Table 5-3 print it)"

[[references]]
work = "Machinery's Handbook"
edition = "26th"
locator = "Ball and Roller Bearings (ANSI/ABMA Std 9 and 11): Tables 26, 28, 31, 32 (X and Y), 34, 35 (X0, Y0) and Formulas 24 to 29 (static equivalent load)"

[[tests]]
name = "Shigley Ex. 11-4: SKF 6210 deep groove bearing, Fr 500 lbf, Fa 400 lbf, 720 rev/min"
source = "Lacewing Lab rolling_bearings_test, after Shigley 10th Example 11-4: C10 = 7900 lbf, C0 = 4450 lbf; Table 11-1 at Fa/C0 = 0.090 gives e = 0.285, X2 = 0.56, Y2 = 1.527; Fa/Fr = 0.8 > e so Fe = 0.56 × 500 + 1.527 × 400 = 890.8 lbf and L10 = (7900/890.8)³ × 10⁶ rev = 16 150 h at 720 rev/min (the book, 16 146 unrounded). A 10 000 h life at 90 % needs C10 = 6749 lbf"
inputs = { kind = "1", C_10 = "7900 lbf", C_0 = "4450 lbf", F_r = "500 lbf", F_a = "400 lbf", V = "1", e = "0.285", X_2 = "0.56", Y_2 = "1.527", a_f = "1", n = "720 rpm", L_D = "10000 h", R_D = "0.90" }
expect = { F_e = "890.8 lbf", "L_(10,h)" = "16150 h", C_req = "6749 lbf" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 11-3: 413 lbf radial, af 1.2, 30 kh at 300 rev/min, R 0.99"
source = "Lacewing Lab rolling_bearings_test, after Shigley 10th Example 11-3: xD = 540, C10 = 6696 lbf by Eq. 11-10 (the book's ln(1/R) ≈ 1 − R); the exact Eq. 11-9 gives 6689.5 lbf, 0.1 % less, so a bearing rated 19.5 kN (4384 lbf) fails the check (C0 does not enter the example)"
inputs = { kind = "1", C_10 = "4384 lbf", C_0 = "2540 lbf", F_r = "413 lbf", F_a = "0 lbf", a_f = "1.2", n = "300 rpm", L_D = "30000 h", R_D = "0.99" }
expect = { x_D = "540", C_req = "6696 lbf" }
tolerance = "0.2%"
checks = "fail"

[[tests]]
name = "Shigley Ex. 11-9: the 02-70 deep groove bearing at Ex. 11-3's duty"
source = "Lacewing Lab rolling_bearings_test, after Shigley 10th Example 11-9: C10 = 13 888 lbf (61.8 kN) under 1.2 × 413 lbf at xD = 540: R = 0.999 963 (Eq. 11-22; C0 does not enter the example)"
inputs = { kind = "1", C_10 = "13888 lbf", C_0 = "8000 lbf", F_r = "413 lbf", F_a = "0 lbf", a_f = "1.2", n = "300 rpm", L_D = "30000 h", R_D = "0.99" }
expect = { R = "0.999963" }
tolerance = "2e-6"
checks = "pass"

[[tests]]
name = "Shigley Ex. 11-11: thrust load 8000 N, 10 kh at 950 rev/min, R 0.95, Timken basis"
source = "Lacewing Lab rolling_bearings_test, after Shigley 10th Example 11-11: tapered roller bearing in pure thrust, Timken rating at 90 × 10⁶ rev (x0 = 0, θ = 4.48, b = 1.5): xD = 6.333, C10 = 16 159 N by Eq. 11-10 (16 076 N by the exact Eq. 11-9); the bearing chosen, C10 = 17 200 N, gives R = 0.963"
inputs = { kind = "2", C_10 = "17200 N", C_0 = "30000 N", L_R = "90e6", x_0 = "0", "θ" = "4.48", b_W = "1.5", F_r = "0 N", F_a = "8000 N", e = "0", X_2 = "0", Y_2 = "1", a_f = "1", n = "950 rpm", L_D = "10000 h", R_D = "0.95", "s_(0,min)" = "1.5" }
expect = { x_D = "6.3333", C_req = "16159 N", R = "0.963" }
tolerance = "0.6%"
checks = "pass"

[[tests]]
name = "Hand check: a 6210 under 5 kN radial at 1500 rev/min, 20 kh at 95 %"
source = "By hand: C10 = 35.1 kN, C0 = 19.6 kN (Shigley Table 11-2, 02-50), Fe = 5 kN: L10 = 7.02³ × 10⁶ = 345.94841e6 rev = 3843.8712 h; xD = 1800, C10 needed = 5000 × (1800/(0.02 + 4.439 ln(1/0.95)^(1/1.483)))^(1/3) = 71364.813 N; R = 0.28411459; with a1 = 0.64 (ISO 281, 95 %) Lnm = 221.40698e6 rev; P0 = 5 kN, s0 = 3.92. The bearing is far too small for the life"
inputs = { kind = "1", C_10 = "35.1 kN", C_0 = "19.6 kN", F_r = "5 kN", F_a = "0 kN", a_f = "1", n = "1500 rpm", L_D = "20000 h", R_D = "0.95", a_1 = "0.64" }
expect = { F_e = "5 kN", L_10 = "345.94841e6", "L_(10,h)" = "3843.8712 h", x_D = "1800", C_req = "71364.813 N", R = "0.28411459", L_nm = "221.40698e6", P_0 = "5 kN", s_0 = "3.92" }
tolerance = "1e-6"
checks = "fail"
+++

## Rolling bearing life and selection

A rolling (ball or roller) bearing under a radial load F_r and an axial load F_a at a steady speed n, after Shigley Ch. 11. Its life is fatigue of the rolling surfaces: L10 is the life that 90 % of a group of bearings reach. The rating life assumes a well-lubricated, aligned, rigidly supported bearing at moderate speed; ISO 281's factor a_ISO adjusts it for lubrication and contamination. For a load spectrum, use the duty-cycle template; for two tapered roller bearings, the tapered pair template; for the seats, the bearing fits template.

### Bearing

The basic dynamic load rating C_10 is the catalogue's load for a life of L_R revolutions at 90 % reliability: 10⁶ revolutions for most makers (Shigley Tables 11-2 and 11-3), 90 × 10⁶ for Timken. The Weibull parameters of the life measure x = L/L_R go with the catalogue (Shigley Table 11-6): x_0 = 0.02, θ = 4.459, b = 1.483 for the 10⁶ basis, and 0, 4.48, 1.5 for Timken's. The life exponent is 3 for ball and 10/3 for roller bearings (Eq. 11-1).

```calc {description="Bearing kind: 1 ball, 2 roller"}
kind = 1_
```

```calc {description="Basic dynamic load rating C10, from the catalogue"}
C_10 = 7900 lbf_
```

```calc {description="Basic static load rating C0, from the catalogue"}
C_0 = 4450 lbf_
```

```calc {description="Rating life the catalogue's C10 is for, in revolutions (10⁶, or 90·10⁶ for Timken)"}
L_R = 1e6_
```

```calc {description="Weibull guaranteed value x0 of the catalogue's life data, Shigley Table 11-6"}
x_0 = 0.02_
```

```calc {description="Weibull characteristic parameter θ, Shigley Table 11-6"}
θ = 4.459_
```

```calc {description="Weibull shape parameter b, Shigley Table 11-6"}
b_W = 1.483_
```

```calc {description="Life exponent, Eq. 11-1"}
p = {3 if kind == 1; 10 / 3 if kind == 2} = ??
```

### Loads and equivalent load

The dynamic equivalent load is F_e = X V F_r + Y F_a (Eq. 11-12), with one pair of factors (X_1, Y_1) while F_a/(V F_r) ≤ e and another (X_2, Y_2) above it. The factors are tables, not formulas: for deep groove ball bearings, Shigley Table 11-1, where e and Y_2 depend on F_a/C_0 (X_1 = 1, Y_1 = 0, X_2 = 0.56); for the other types the ANSI/ABMA tables Machinery's Handbook prints (Tables 26 to 32) or the maker's catalogue (its e, Y_1, Y_2). A cylindrical roller bearing takes F_e = F_r (Y = 0 throughout), a thrust bearing F_e = F_a (e = 0, X_2 = 0, Y_2 = 1). The rotation factor V is 1.2 when the outer ring turns relative to the load, and 1 otherwise and for self-aligning bearings.

```calc {description="Radial load"}
F_r = 500 lbf_
```

```calc {description="Axial (thrust) load"}
F_a = 400 lbf_
```

```calc {description="Rotation factor: 1 inner ring rotating, 1.2 outer ring rotating"}
V = 1_
```

```calc {description="Limiting ratio e of Fa/(V·Fr), Shigley Table 11-1 or the catalogue"}
e = 0.285_
```

```calc {description="Radial factor X1 for Fa/(V·Fr) ≤ e (Table 11-1: 1)"}
X_1 = 1_
```

```calc {description="Axial factor Y1 for Fa/(V·Fr) ≤ e (Table 11-1: 0)"}
Y_1 = 0_
```

```calc {description="Radial factor X2 for Fa/(V·Fr) > e (Table 11-1: 0.56)"}
X_2 = 0.56_
```

```calc {description="Axial factor Y2 for Fa/(V·Fr) > e, Table 11-1 at Fa/C0, or the catalogue"}
Y_2 = 1.527_
```

The application factor a_f allows for loads above the nominal (Shigley Table 11-5: 1.0 to 1.2 for precision gearing and machinery without impact, 1.1 to 1.3 for commercial gearing, 1.2 to 1.5 for light impact, 1.5 to 3 for moderate impact).

```calc {description="Application (load) factor, Shigley Table 11-5"}
a_f = 1_
```

```calc
> Radial factor in use
X = {X_1 if F_a ≤ e V F_r; X_2 otherwise} = ??
> Axial factor in use
Y = {Y_1 if F_a ≤ e V F_r; Y_2 otherwise} = ??
> Dynamic equivalent load, Eq. 11-12
F_e = X V F_r + Y F_a = ?? lbf
> Design load with the application factor
F_D = a_f F_e = ?? lbf
```

### Rating life

```calc {description="Speed"}
n = 720 rpm_
```

Cruncha's rpm is an angular speed (2π radians a revolution), so the revolutions are counted as n/(2π).

```calc
> Revolutions per unit time
n_r = n / (2 π) = ?? 1/min
> Rating life in revolutions, Eq. 11-3
L_10 = L_R (C_10 / F_D)^p = ??
> Rating life in hours at the speed
L_(10,h) = L_10 / n_r = ?? h
```

### Design life, reliability and the rating needed

The design life L_D is the hours the bearing must run (Shigley Table 11-4: 4 to 8 kh for intermittent short duty, 20 to 30 kh for 8 h service fully used, 50 to 60 kh continuous, 100 to 200 kh where reliability is critical). The rating needed is Eq. 11-9 with the exact ln(1/R_D); Shigley's worked examples use Eq. 11-10's 1 − R_D, which differs by a fraction of a percent at R_D ≥ 0.9. The reliability is Eq. 11-21 at the design life.

```calc {description="Design life, in hours at the speed"}
L_D = 10000 h_
```

```calc {description="Reliability wanted at the design life"}
R_D = 0.90_
```

```calc
> Design life in rating lives
x_D = L_D n_r / L_R = ??
> Catalogue rating the design life needs, Eq. 11-9
C_req = F_D (x_D / (x_0 + (θ - x_0) ln(1 / R_D)^(1 / b_W)))^(1 / p) = ?? lbf
> Life measure the bearing reaches at the design load
x_R = x_D (F_D / C_10)^p = ??
> Reliability at the design life, Eq. 11-21
R = exp(-(max(x_R - x_0, 0) / (θ - x_0))^b_W) = ??
```

### Adjusted life (ISO 281)

ISO 281:2007's modified rating life is a_1 a_ISO L_10. a_1 is the life modification factor for reliability (1 at 90 %, 0.64 at 95 %, 0.25 at 99 % in ISO 281:2007's table). a_ISO allows for lubrication, contamination and the fatigue load limit and is read from the standard's (or the maker's) diagram for the bearing type at the viscosity ratio κ and e_c·C_u/P; enter 1 to leave it out.

```calc {description="Life modification factor for reliability a1, ISO 281 table"}
a_1 = 1_
```

```calc {description="Life modification factor aISO, from ISO 281's diagram (1 to omit)"}
a_ISO = 1_
```

```calc
> Modified rating life, ISO 281
L_nm = a_1 a_ISO L_10 = ??
> In hours at the speed
L_(nm,h) = L_nm / n_r = ?? h
```

### Static load

The static equivalent load is the larger of F_r and X_0 F_r + Y_0 F_a (Machinery's Handbook Formulas 24 to 29, X_0 and Y_0 from its Tables 34 and 35 or the catalogue; deep groove ball bearings 0.6 and 0.5). The static safety factor s_0 = C_0/P_0 should be at least about 1 for ball and 1.5 for roller bearings in normal rotation, more for quiet running or shock, and at least 4 for spherical roller thrust bearings (JTEKT Basic Bearing Knowledge Table 5-10).

```calc {description="Static radial factor X0, Machinery's Table 34 or the catalogue"}
X_0 = 0.6_
```

```calc {description="Static axial factor Y0, Machinery's Table 34 or the catalogue"}
Y_0 = 0.5_
```

```calc {description="Smallest acceptable static safety factor for the duty"}
s_(0,min) = 1_
```

```calc
> Static equivalent load
P_0 = max(F_r, X_0 F_r + Y_0 F_a) = ?? lbf
> Static safety factor
s_0 = C_0 / P_0 = ??
```

### Checks

```calc
@test C_10 ≥ C_req
```

```calc
@test s_0 ≥ s_(0,min)
```
