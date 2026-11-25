+++
cruncha-template = 1
id = "mechanical.compressionSprings.fatigue"
version = "1.0.0"
title = "Helical compression spring in fatigue (Shigley, Zimmerli data)"
summary = "A round-wire helical compression spring cycled between F_min and F_max: alternating and mean shear stresses with the Bergsträsser factor, the torsional ultimate strength, Zimmerli's endurance data reduced to a fully reversed endurance strength, and the fatigue factor of safety by Gerber, Goodman and Sines, with Gerber checked against a design factor."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "compression spring", "helical spring", "coil spring", "fatigue", "Zimmerli", "Gerber", "Goodman", "Sines", "shot-peened", "Bergsträsser", "endurance strength", "Shigley"]
from = "lacewing:mechanical.compressionSprings"
license = "GPL-3.0-or-later"
inputs = ["d", "OD", "A", "m", "d_A", "S_sa", "S_sm", "F_max", "F_min", "n_d"]
outputs = ["D", "C", "K_B", "S_ut", "S_su", "F_a", "F_m", "τ_a", "τ_m", "S_(se,Gerber)", "S_(se,Goodman)", "n_(f,Gerber)", "n_(f,Goodman)", "n_(f,Sines)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 10-8 and 10-9: Eqs. 10-1, 10-5, 10-14 and Table 10-4 (Sut = A/d^m), 10-28 and 10-29 (Zimmerli's endurance components, unpeened and peened), 10-30 (Ssu), 10-32 and 10-33 (alternating and mean stresses); Table 6-7 (Gerber and Goodman for fatigue); Example 10-4"

[[tests]]
name = "Shigley Ex. 10-4: unpeened music wire 0.092 in, OD 9/16 in, cycled 5 to 35 lbf"
source = "Lacewing Lab coil_springs_test, after Shigley 10th Example 10-4: KB = 1.287, τa = 29.7, τm = 39.6, Ssu = 190.3, Sse (Gerber) = 38.2 kpsi; nf = 1.21 by Gerber, 1.18 by Sines, 1.23 by Goodman"
inputs = { d = "0.092 in", OD = "0.5625 in", A = "201 kpsi", m = "0.145", d_A = "1 in", S_sa = "35 kpsi", S_sm = "55 kpsi", F_max = "35 lbf", F_min = "5 lbf", n_d = "1.2" }
expect = { K_B = "1.287", "τ_a" = "29.7 kpsi", "τ_m" = "39.6 kpsi", S_su = "190.3 kpsi", "S_(se,Gerber)" = "38.2 kpsi", "n_(f,Gerber)" = "1.21", "n_(f,Sines)" = "1.18", "n_(f,Goodman)" = "1.23" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 10-4 exactly"
source = "By hand: D = 0.4705 in, C = 5.1141304, KB = 1.2864259, Sut = 201/0.092^0.145 = 284.08403 kpsi, Fa = 15, Fm = 20 lbf, τa = 29.690143, τm = 39.586858 kpsi, Ssu = 0.67 Sut = 190.33630 kpsi; Sse = 35/(1 − (55/190.3363)²) = 38.188727 kpsi (Gerber), 35/(1 − 55/190.3363) = 49.223826 kpsi (Goodman); nf = 1.2053994 (Gerber), 1.2328178 (Goodman), 35/29.690143 = 1.1788424 (Sines)"
inputs = { d = "0.092 in", OD = "0.5625 in", A = "201 kpsi", m = "0.145", d_A = "1 in", S_sa = "35 kpsi", S_sm = "55 kpsi", F_max = "35 lbf", F_min = "5 lbf", n_d = "1.2" }
expect = { D = "0.4705 in", C = "5.1141304", K_B = "1.2864259", S_ut = "284.08403 kpsi", F_a = "15 lbf", F_m = "20 lbf", "τ_a" = "29.690143 kpsi", "τ_m" = "39.586858 kpsi", S_su = "190.33630 kpsi", "S_(se,Gerber)" = "38.188727 kpsi", "S_(se,Goodman)" = "49.223826 kpsi", "n_(f,Gerber)" = "1.2053994", "n_(f,Goodman)" = "1.2328178", "n_(f,Sines)" = "1.1788424" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 10-4 shot-peened"
source = "By hand, Ex. 10-4 with Zimmerli's peened components 57.5 and 77.5 kpsi: Sse = 57.5/(1 − (77.5/190.3363)²) = 68.927546 kpsi, Gerber nf = 1.9425943"
inputs = { d = "0.092 in", OD = "0.5625 in", A = "201 kpsi", m = "0.145", d_A = "1 in", S_sa = "57.5 kpsi", S_sm = "77.5 kpsi", F_max = "35 lbf", F_min = "5 lbf", n_d = "1.5" }
expect = { "S_(se,Gerber)" = "68.927546 kpsi", "n_(f,Gerber)" = "1.9425943" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 10-4 unpeened, against a design factor of 1.5"
source = "As Shigley 10th Example 10-4: the Gerber factor 1.21 is under 1.5"
inputs = { d = "0.092 in", OD = "0.5625 in", A = "201 kpsi", m = "0.145", d_A = "1 in", S_sa = "35 kpsi", S_sm = "55 kpsi", F_max = "35 lbf", F_min = "5 lbf", n_d = "1.5" }
expect = { "n_(f,Gerber)" = "1.21" }
tolerance = "0.4%"
checks = "fail"
+++

## Helical compression spring in fatigue

A round-wire helical compression spring whose load cycles between F_min and F_max, after Shigley Secs. 10-8 and 10-9. Both the alternating and the mean shear stress carry Bergsträsser's factor K_B. The endurance data are Zimmerli's: for spring steel wire under 10 mm, the endurance strength components are independent of size, material and tensile strength, so long as the wire is unpeened or shot-peened as entered. They are reduced to a fully reversed endurance strength S_se by the chosen criterion and then used on the load line through the origin. Sines's criterion ignores the mean stress and takes the Zimmerli amplitude directly. Rate, solid length, buckling and surge are in the compression spring static design template; set removal is not recommended for a spring in fatigue (Sec. 10-4).

### Wire

S_ut = A/d^m (Eq. 10-14) with A and m from Shigley Table 10-4; A is for d in the unit d_A. Zimmerli's components (Eqs. 10-28, 10-29) are an alternating S_sa at a mean S_sm: 35 and 55 kpsi (241 and 379 MPa) unpeened, 57.5 and 77.5 kpsi (398 and 534 MPa) shot-peened.

```calc {description="Wire diameter"}
d = 0.092 in_
```

```calc {description="Outside diameter of the coil"}
OD = 0.5625 in_
```

```calc {description="Table 10-4 constant A of Sut = A/d^m, for d in units of d_A"}
A = 201 kpsi_
```

```calc {description="Table 10-4 exponent m of Sut = A/d^m"}
m = 0.145_
```

```calc {description="Diameter unit Table 10-4's A is for (1 in for kpsi·in^m, 1 mm for MPa·mm^m)"}
d_A = 1 in_
```

```calc {description="Zimmerli's alternating endurance component, Shigley Eqs. 10-28, 10-29 (unpeened 35 kpsi, peened 57.5 kpsi)"}
S_sa = 35 kpsi_
```

```calc {description="Zimmerli's mean endurance component, Shigley Eqs. 10-28, 10-29 (unpeened 55 kpsi, peened 77.5 kpsi)"}
S_sm = 55 kpsi_
```

```calc
> Mean coil diameter
D = OD - d = ?? in
> Spring index, Eq. 10-1
C = D / d = ??
> Bergsträsser factor, Eq. 10-5
K_B = (4 C + 2) / (4 C - 3) = ??
> Minimum tensile strength, Eq. 10-14
S_ut = A / (d / d_A)^m = ?? kpsi
> Torsional ultimate strength, Eq. 10-30
S_su = 0.67 S_ut = ?? kpsi
```

### Loads and stresses

```calc {description="Largest load of the cycle"}
F_max = 35 lbf_
```

```calc {description="Smallest load of the cycle"}
F_min = 5 lbf_
```

```calc
> Alternating force
F_a = (F_max - F_min) / 2 = ?? lbf
> Mean force
F_m = (F_max + F_min) / 2 = ?? lbf
> Alternating shear stress, Eq. 10-32
τ_a = K_B 8 F_a D / (π d^3) = ?? kpsi
> Mean shear stress, Eq. 10-33
τ_m = K_B 8 F_m D / (π d^3) = ?? kpsi
```

### Fatigue factors of safety

The Gerber and Goodman lines through Zimmerli's point (S_sm, S_sa) meet the S_sa axis at S_se, and the factor of safety is where the load line through the origin with slope τ_a/τ_m meets the line (Ex. 10-4). Gerber fits the data best and is the check here.

```calc
> Fully reversed endurance strength, Gerber
S_(se,Gerber) = S_sa / (1 - (S_sm / S_su)^2) = ?? kpsi
> Fatigue factor of safety, Gerber, Table 6-7
n_(f,Gerber) = 0.5 (S_su / τ_m)^2 (τ_a / S_(se,Gerber)) (-1 + sqrt(1 + (2 τ_m S_(se,Gerber) / (S_su τ_a))^2)) = ??
```

```calc
> Fully reversed endurance strength, Goodman
S_(se,Goodman) = S_sa / (1 - S_sm / S_su) = ?? kpsi
> Fatigue factor of safety, Goodman
n_(f,Goodman) = 1 / (τ_a / S_(se,Goodman) + τ_m / S_su) = ??
```

```calc {description="Fatigue factor of safety, Sines (mean stress ignored)"}
n_(f,Sines) = S_sa / τ_a = ??
```

### Check

```calc {description="Design factor required against fatigue"}
n_d = 1.2_
```

```calc
@test n_(f,Gerber) ≥ n_d
```
