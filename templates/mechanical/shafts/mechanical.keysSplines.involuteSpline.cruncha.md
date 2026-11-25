+++
cruncha-template = 1
id = "mechanical.keysSplines.involuteSpline"
version = "1.0.0"
title = "Involute spline, 30° (ANSI B92.1): basic dimensions and Dudley's stresses"
summary = "A 30° involute spline by its number of teeth and diametral pitch: the basic dimensions of ANSI B92.1 (pitch, major and minor diameters, tooth thickness, depth of engagement) and Dudley's stresses as Machinery's Handbook gives them, with application, load-distribution, fatigue and wear factors: shear under the roots, shear at the pitch line, compression on the tooth sides, and the bursting stress of the internal member, each against its allowable."
category = ["Mechanical engineering", "Shafts"]
tags = ["spline", "involute spline", "ANSI B92.1", "Dudley", "shaft", "hub", "torque capacity", "tooth shear", "compressive stress", "bursting", "Machinery's Handbook"]
from = "lacewing:mechanical.keysSplines"
license = "GPL-3.0-or-later"
inputs = ["N", "P", "form", "T", "L_e", "D_h", "K_a", "K_m", "K_f", "K_w", "flex", "c_sp", "D_oi", "L", "n", "S_(s,all)", "S_(c,all)", "S_(t,all)"]
outputs = ["D", "D_re", "D_ri", "t", "h", "S_(s,root)", "S_(s,pitch)", "S_c", "L_(e,max)", "t_w", "S_1", "S_2", "S_3", "S_t", "n_root", "n_pitch", "n_c", "n_t"]

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "Involute splines: ANSI B92.1 basic dimensions (Table 2, pp. 2131-2132); \"Formulas for Torque Capacity of Involute Splines\" after Dudley (pp. 2141-2145, Eqs. 1-8; Tables 1-4 factors, Tables 5-7 allowable stresses)"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.6 (splines; a splined shaft about as strong as a round shaft of its minor diameter)"

[[tests]]
name = "16/32 pitch, 24 teeth, flat root side fit, 10 000 lb·in on 1 in, fixed, Ka 1.2"
source = "Lacewing Lab keys_splines_test (Machinery's 26th Table 2 and Eqs. 1, 3, 5): D = 1.5 in, Dre = 22.65/16, Dri = 25.35/16, t = π/32, h = 0.9/16 in; Ss root = 16 × 10 000 × 1.2/(π Dre³) = 21 543.02 psi, Ss pitch = 4 × 10 000 × 1.2/(1.5 × 24 × 1 × π/32) = 13 581.22 psi, Sc = 2 × 10 000 × 1.2/(9 × 1.5 × 24 × 1 × 0.05625) = 1316.87 psi, Le max = 5000 × 1.5^3.5/10 000 = 2.0668 in. Bursting by hand (sleeve 2.5 in OD, 1 in long, 3000 rev/min): tw 0.45781 in, S1 2676.15, S2 101.081, S3 11 851.85, St 17 534.68 psi. Allowables for 302-351 HB steel (Tables 5-7): 40 000 shear, 3000 compressive, 45 000 tensile psi"
inputs = { N = "24", P = "16 1/in", form = "1", T = "10000 lbf·in", L_e = "1 in", D_h = "0 in", K_a = "1.2", K_m = "1", K_f = "1", K_w = "1", flex = "0", c_sp = "4", D_oi = "2.5 in", L = "1 in", n = "3000 rpm", "S_(s,all)" = "40000 psi", "S_(c,all)" = "3000 psi", "S_(t,all)" = "45000 psi" }
expect = { D = "1.5 in", D_re = "1.415625 in", D_ri = "1.584375 in", t = "0.09817477 in", h = "0.05625 in", "S_(s,root)" = "21543.025 psi", "S_(s,pitch)" = "13581.222 psi", S_c = "1316.8724 psi", "L_(e,max)" = "2.066757 in", t_w = "0.4578125 in", S_1 = "2676.1509 psi", S_2 = "101.08149 psi", S_3 = "11851.852 psi", S_t = "17534.685 psi", n_root = "1.856749", n_c = "2.278125" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: flexible 8/16 pitch fillet-root spline, hollow shaft, too little compressive margin"
source = "By hand (Machinery's 26th Table 2, Eqs. 2, 3, 4, 6-8): 20 teeth, P = 8, fillet root: D = 2.5, Dre = (20 − 1.8)/8 = 2.275, Dri = 21.8/8 = 2.725, t = π/16, h = 1/8 in; 30 000 lb·in, Le 1.5 in, bore 1 in, Ka 1.5, Km 1.5, Kw 1.4, flexible: Ss root = 16 × 30 000 × 2.275 × 1.5/(π(2.275⁴ − 1)) = 20 219.09 psi, Ss pitch = 18 334.65 psi, Sc = 2 × 30 000 × 1.5 × 1.5/(2.5 × 20 × 1.5 × 0.125 × 1.4) = 10 285.71 psi; sleeve 3.8 in OD, 2 in long, 1800 rev/min: S1 2051.456, S2 85.9234, S3 8533.333, St 23 901.70 psi. Case-hardened allowables 50 000, 20 000 (crowned), 55 000 psi: n_c = 1.944; with a compressive allowable of 5000 psi for straight teeth the check fails (n_c 0.486)"
inputs = { N = "20", P = "8 1/in", form = "3", T = "30000 lbf·in", L_e = "1.5 in", D_h = "1 in", K_a = "1.5", K_m = "1.5", K_f = "1", K_w = "1.4", flex = "1", c_sp = "4", D_oi = "3.8 in", L = "2 in", n = "1800 rpm", "S_(s,all)" = "50000 psi", "S_(c,all)" = "5000 psi", "S_(t,all)" = "55000 psi" }
expect = { D_re = "2.275 in", D_ri = "2.725 in", "S_(s,root)" = "20219.091 psi", "S_(s,pitch)" = "18334.649 psi", S_c = "10285.714 psi", S_1 = "2051.4564 psi", S_2 = "85.923404 psi", S_3 = "8533.3333 psi", S_t = "23901.700 psi", n_c = "0.4861111" }
tolerance = "1e-6"
checks = "fail"
+++

## Involute spline, 30°

A 30° involute spline joining a shaft (the external member) and a hub or sleeve (the internal member), sized by its number of teeth N and diametral pitch P (teeth per inch of pitch diameter; a 16/32 pitch spline has P = 16). The basic dimensions follow ANSI B92.1 as Machinery's Handbook tabulates them, and the stresses Dudley's formulas as Machinery's gives them (Eqs. 1-8). The factors are read from Machinery's tables: the application factor K_a by power source and load (Table 1), the load-distribution factor K_m of flexible splines by misalignment and face width (Table 2; 1 for fixed splines), the fatigue-life factor K_f by the number of torque cycles (Table 3) and the wear-life factor K_w of flexible splines by revolutions (Table 4). The allowable stresses by material and hardness are in Tables 5 to 7.

A **fixed** spline does not slide or rock in service; a **flexible** one accommodates misalignment, so its tooth sides wear and are checked with K_w. Machinery's limits the effective length of a fixed spline without helix modification to L_e ≤ 5000 D^3.5/T (inches, lb·in): a longer engagement does not share the load, so do not count it in the stresses. Dudley takes half the teeth as carrying the load because of spacing errors (factor 4 in the pitch-line shear; use 6 for poor accuracy). The bursting check is on the internal member: radial tooth load, centrifugal stress and tooth beam loading on the wall t_w under its tooth roots, over its full length L.

### Spline

```calc {description="Number of teeth (B92.1: 6 or more)"}
N = 24_
```

```calc {description="Diametral pitch, teeth per inch of pitch diameter (16/32 pitch: 16 per inch)"}
P = 16 1/in_
```

```calc {description="Tooth form: 1 flat root side fit, 2 flat root major-diameter fit, 3 fillet root side fit"}
form = 1_
```

```calc
> Pitch diameter
D = N / P = ?? in
> Minor (root) diameter of the external spline
D_re = { (N - 1.35) / P if form ≤ 2; (N - 1.8) / P if form == 3 and P ≤ 12 '1/in'; (N - 2) / P if form == 3 } = ?? in
> Major diameter of the internal spline (the root of its teeth)
D_ri = { (N + 1.35) / P if form == 1; (N + 1) / P if form == 2; (N + 1.8) / P if form == 3 } = ?? in
> Circular tooth thickness at the pitch line
t = π / (2 P) = ?? in
> Depth of engagement
h = { 0.9 / P if form ≤ 2; 1 / P if form == 3 } = ?? in
```

### Load and service

```calc {description="Torque transmitted"}
T = 10000 lbf·in_
```

```calc {description="Effective length of engagement"}
L_e = 1 in_
```

```calc {description="Bore of a hollow external member (0 for solid)"}
D_h = 0 in_
```

```calc {description="Application factor, Machinery's Table 1 (spline torque capacity)"}
K_a = 1.2_
```

```calc {description="Load-distribution factor, Machinery's Table 2 (1 for fixed splines)"}
K_m = 1_
```

```calc {description="Fatigue-life factor by torque cycles, Machinery's Table 3"}
K_f = 1_
```

```calc {description="Wear-life factor of flexible splines by revolutions, Machinery's Table 4"}
K_w = 1_
```

```calc {description="Spline type: 0 fixed, 1 flexible"}
flex = 0_
```

```calc {description="Spacing factor of the pitch-line shear: 4 for normal accuracy, 6 for poor"}
c_sp = 4_
```

### Stresses in the teeth and the shaft

```calc
> Shear stress under the roots of the external teeth, Eqs. 1 and 2
S_(s,root) = 16 T D_re K_a / (π (D_re^4 - D_h^4) K_f) = ?? psi
> Shear stress at the pitch line of the teeth, Eq. 3
S_(s,pitch) = c_sp T K_a K_m / (D N L_e t K_f) = ?? psi
> Compressive stress on the tooth sides, Eq. 4 (flexible) or Eq. 5 (fixed)
S_c = { 2 T K_m K_a / (D N L_e h K_w) if flex == 1; 2 T K_m K_a / (9 D N L_e h K_f) otherwise } = ?? psi
> Longest effective length of a fixed spline without helix modification, Machinery's p. 2141 (in, lb·in)
L_(e,max) = 5000 (D / 1 'in')^3.5 / (T / 1 'lbf·in') × 1 'in' = ?? in
```

### Bursting of the internal member

```calc {description="Outside diameter of the internal member (sleeve or hub)"}
D_oi = 2.5 in_
```

```calc {description="Full length of the internal member"}
L = 1 in_
```

```calc {description="Speed"}
n = 3000 rpm_
```

```calc
> Wall under the roots of the internal teeth
t_w = (D_oi - D_ri) / 2 = ?? in
> Radial-load tensile stress, Eq. 6 (30° pressure angle)
S_1 = T tan(30°) / (π D t_w L) = ?? psi
> Centrifugal tensile stress, Eq. 7 (empirical: rev/min, in, psi)
S_2 = 1.656 (n / 1 'rpm')^2 ((D_oi / 1 'in')^2 + 0.212 (D_ri / 1 'in')^2) / 10^6 × 1 'psi' = ?? psi
> Beam-loading tensile stress of the teeth, Eq. 8 (Y = 1.5 for 30° internal splines)
S_3 = 4 T / (D^2 L_e 1.5) = ?? psi
> Total tensile stress bursting the rim
S_t = (K_a K_m (S_1 + S_3) + S_2) / K_f = ?? psi
```

### Allowable stresses and checks

```calc {description="Allowable shear stress, Machinery's Table 5 (spline torque capacity)"}
S_(s,all) = 40000 psi_
```

```calc {description="Allowable compressive stress, Machinery's Table 6 (straight or crowned teeth)"}
S_(c,all) = 3000 psi_
```

```calc {description="Allowable tensile stress, Machinery's Table 7"}
S_(t,all) = 45000 psi_
```

```calc
> Margin on shear under the roots
n_root = S_(s,all) / S_(s,root) = ??
> Margin on shear at the pitch line
n_pitch = S_(s,all) / S_(s,pitch) = ??
> Margin on compression on the tooth sides
n_c = S_(c,all) / S_c = ??
> Margin on bursting
n_t = S_(t,all) / S_t = ??
```

```calc
@test S_(s,root) ≤ S_(s,all)
```

```calc
@test S_(s,pitch) ≤ S_(s,all)
```

```calc
@test S_c ≤ S_(c,all)
```

```calc
@test S_t ≤ S_(t,all)
```

```calc
@test flex == 1 or L_e ≤ L_(e,max)
```
