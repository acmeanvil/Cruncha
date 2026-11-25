+++
cruncha-template = 1
id = "mechanical.extensionSprings.static"
version = "1.0.0"
title = "Helical extension spring with hook ends, static (Shigley)"
summary = "A close-wound helical extension spring with full-loop hook ends under a static load: active coils Na = Nb + G/E, rate, free length inside the loops, extension, the initial-tension stress against the range springmakers can hold, and the static factors of safety of the body in torsion and of the hook in bending (point A) and torsion (point B) against Table 10-7's allowables."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "extension spring", "tension spring", "helical spring", "coil spring", "hook", "initial tension", "spring rate", "Bergsträsser", "curved beam", "Shigley"]
from = "lacewing:mechanical.extensionSprings"
license = "GPL-3.0-or-later"
inputs = ["d", "OD", "N_b", "r_1", "r_2", "F_i", "F_max", "A", "m", "d_A", "G", "E", "f_body", "f_A", "f_B", "n_d"]
outputs = ["D", "C", "K_B", "N_a", "k", "L_0", "y", "τ_i", "τ_(i,lo)", "τ_(i,hi)", "S_ut", "τ_max", "n_body", "C_1", "K_hookA", "σ_A", "n_A", "C_2", "K_hookB", "τ_B", "n_B"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 10-11: Eqs. 10-34 and 10-35 (hook bending at A), 10-36 and 10-37 (hook torsion at B), 10-38 (extension), 10-39 (free length), 10-40 (Na = Nb + G/E), 10-41 and Fig. 10-7c (preferred initial tension), Table 10-7 (allowables); Eqs. 10-1, 10-5, 10-9, 10-14 with Tables 10-4 and 10-5; Example 10-6"

[[tests]]
name = "Shigley Ex. 10-6: hard-drawn wire 0.035 in, OD 0.248 in, 12.17 body turns, 5.25 lbf"
source = "Lacewing Lab coil_springs_test, after Shigley 10th Example 10-6: Na = 12.57, k = 17.91 lbf/in, L0 = 0.817 in, y = 0.227 in, τi = 15.1 kpsi (in the preferred range), n body = 1.45, (K)A = 1.14, σA = 156.9 kpsi, nA = 1.27, nB = 1.35"
inputs = { d = "0.035 in", OD = "0.248 in", N_b = "12.17", r_1 = "0.106 in", r_2 = "0.089 in", F_i = "1.19 lbf", F_max = "5.25 lbf", A = "140 kpsi", m = "0.190", d_A = "1 in", G = "11.6 Mpsi", E = "28.7 Mpsi", f_body = "0.45", f_A = "0.75", f_B = "0.40", n_d = "1" }
expect = { N_a = "12.57", k = "17.91 lbf/in", L_0 = "0.817 in", y = "0.227 in", "τ_i" = "15.1 kpsi", n_body = "1.45", K_hookA = "1.14", "σ_A" = "156.9 kpsi", n_A = "1.27", n_B = "1.35" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 10-6 exactly"
source = "By hand: D = 0.213 in, C = 6.0857143, KB = 1.2342704, Sut = 140/0.035^0.190 = 264.70071 kpsi, Na = 12.17 + 11.6/28.7 = 12.574181, k = 17.906951 lbf/in, L0 = (2C − 1 + Nb)d = 0.81695 in, y = 0.22672759 in, τi = 8 × 1.19 × 0.213/(π 0.035³) = 15.054369 kpsi, preferred range 33 500/e^(0.105C) ∓ 1000(4 − (C − 3)/6.5) psi = 14.156695 to 21.207244 kpsi, τmax = 81.975714 kpsi, n body = 1.4530562; C1 = 6.0571429, (K)A = 1.1401436, σA = 156.90506 kpsi, nA = 1.2652590; C2 = 5.0857143, (K)B = 1.1835664, τB = 78.608141 kpsi, nB = 1.3469379"
inputs = { d = "0.035 in", OD = "0.248 in", N_b = "12.17", r_1 = "0.106 in", r_2 = "0.089 in", F_i = "1.19 lbf", F_max = "5.25 lbf", A = "140 kpsi", m = "0.190", d_A = "1 in", G = "11.6 Mpsi", E = "28.7 Mpsi", f_body = "0.45", f_A = "0.75", f_B = "0.40", n_d = "1" }
expect = { D = "0.213 in", C = "6.0857143", K_B = "1.2342704", S_ut = "264.70071 kpsi", N_a = "12.574181", k = "17.906951 lbf/in", L_0 = "0.81695 in", y = "0.22672759 in", "τ_i" = "15.054369 kpsi", "τ_(i,lo)" = "14.156695 kpsi", "τ_(i,hi)" = "21.207244 kpsi", "τ_max" = "81.975714 kpsi", n_body = "1.4530562", C_1 = "6.0571429", K_hookA = "1.1401436", "σ_A" = "156.90506 kpsi", n_A = "1.2652590", C_2 = "5.0857143", K_hookB = "1.1835664", "τ_B" = "78.608141 kpsi", n_B = "1.3469379" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 10-6 wound with 2 lbf initial tension"
source = "By hand: τi = 8 × 2 × 0.213/(π 0.035³) = 25.301 kpsi, above the preferred range's 21.21 kpsi"
inputs = { d = "0.035 in", OD = "0.248 in", N_b = "12.17", r_1 = "0.106 in", r_2 = "0.089 in", F_i = "2 lbf", F_max = "5.25 lbf", A = "140 kpsi", m = "0.190", d_A = "1 in", G = "11.6 Mpsi", E = "28.7 Mpsi", f_body = "0.45", f_A = "0.75", f_B = "0.40", n_d = "1" }
expect = { "τ_i" = "25.301 kpsi" }
tolerance = "0.01%"
checks = "fail"
+++

## Helical extension spring, static

A close-wound helical extension spring with a full loop at each end, after Shigley Sec. 10-11. The body is a helix in torsion like a compression spring's, with Bergsträsser's factor, but it does not extend until the load passes the initial tension F_i wound into it. The ends are usually the weak point: where the loop leaves the body (point A, seen side on) the wire is a curved beam in bending plus direct tension, and where it bends into the loop (point B, seen end on) it is in torsion with a curvature factor. The allowables are fractions of S_ut (Table 10-7), lower at the ends than in the body. A broken extension spring lets go of its load, so Shigley advises larger design factors than for compression springs. For a load that cycles, use the extension spring fatigue template.

### Wire

S_ut = A/d^m (Eq. 10-14) with A and m from Shigley Table 10-4, A for d in the unit d_A; G and E from Table 10-5. The static allowables (Table 10-7, low ends of its ranges) are, as fractions of S_ut, body torsion 0.45, end bending 0.75 and end torsion 0.40 for steels, and 0.35, 0.55 and 0.30 for austenitic stainless and nonferrous wire.

```calc {description="Wire diameter"}
d = 0.035 in_
```

```calc {description="Table 10-4 constant A of Sut = A/d^m, for d in units of d_A"}
A = 140 kpsi_
```

```calc {description="Table 10-4 exponent m of Sut = A/d^m"}
m = 0.190_
```

```calc {description="Diameter unit Table 10-4's A is for (1 in for kpsi·in^m, 1 mm for MPa·mm^m)"}
d_A = 1 in_
```

```calc {description="Shear modulus of the wire, Shigley Table 10-5"}
G = 11.6 Mpsi_
```

```calc {description="Young's modulus of the wire, Shigley Table 10-5"}
E = 28.7 Mpsi_
```

```calc {description="Body torsion allowable as a fraction of Sut, Shigley Table 10-7"}
f_body = 0.45_
```

```calc {description="End (hook) bending allowable as a fraction of Sut, Shigley Table 10-7"}
f_A = 0.75_
```

```calc {description="End (hook) torsion allowable as a fraction of Sut, Shigley Table 10-7"}
f_B = 0.40_
```

```calc {description="Minimum tensile strength, Eq. 10-14"}
S_ut = A / (d / d_A)^m = ?? kpsi
```

### Body

```calc {description="Outside diameter of the coil"}
OD = 0.248 in_
```

```calc {description="Body turns between the hooks"}
N_b = 12.17_
```

```calc
> Mean coil diameter
D = OD - d = ?? in
> Spring index, Eq. 10-1
C = D / d = ??
> Bergsträsser factor, Eq. 10-5
K_B = (4 C + 2) / (4 C - 3) = ??
> Active coils, the loops adding a little flexibility, Eq. 10-40
N_a = N_b + G / E = ??
> Spring rate once the initial tension is overcome, Eq. 10-9
k = d^4 G / (8 D^3 N_a) = ?? lbf/in
> Free length inside the end loops, Eq. 10-39
L_0 = (2 C - 1 + N_b) d = ?? in
```

### Initial tension

The initial tension is set in winding. Springmakers can hold the uncorrected stress it causes within a band that falls with the index (Eq. 10-41, Fig. 10-7c): 33 500/e^(0.105C) ± 1000(4 − (C − 3)/6.5) psi.

```calc {description="Initial tension"}
F_i = 1.19 lbf_
```

```calc
> Uncorrected shear stress from the initial tension
τ_i = 8 F_i D / (π d^3) = ?? kpsi
> Preferred range, low end, Eq. 10-41
τ_(i,lo) = (33500 / exp(0.105 C) - 1000 (4 - (C - 3) / 6.5)) × 1 'psi' = ?? kpsi
> Preferred range, high end, Eq. 10-41
τ_(i,hi) = (33500 / exp(0.105 C) + 1000 (4 - (C - 3) / 6.5)) × 1 'psi' = ?? kpsi
```

### Load and body stress

```calc {description="Largest load"}
F_max = 5.25 lbf_
```

```calc
> Extension at the largest load, Eq. 10-38
y = (F_max - F_i) / k = ?? in
> Body shear stress
τ_max = K_B 8 F_max D / (π d^3) = ?? kpsi
> Body factor of safety, Table 10-7
n_body = f_body S_ut / τ_max = ??
```

### Hooks

r_1 is the mean radius of the bend at A, where the loop leaves the body, seen from the side; r_2 the mean radius of the bend at B into the loop, seen from the end. For a standard full loop r_1 is about D/2.

```calc {description="Mean radius of the hook bend at A (bending)"}
r_1 = 0.106 in_
```

```calc {description="Mean radius of the hook bend at B (torsion)"}
r_2 = 0.089 in_
```

```calc
> Index of the bend at A
C_1 = 2 r_1 / d = ??
> Bending stress-correction factor at A, Eq. 10-35
K_hookA = (4 C_1^2 - C_1 - 1) / (4 C_1 (C_1 - 1)) = ??
> Bending and direct tension at A, Eq. 10-34
σ_A = F_max (K_hookA 16 D / (π d^3) + 4 / (π d^2)) = ?? kpsi
> Hook bending factor of safety, Table 10-7
n_A = f_A S_ut / σ_A = ??
```

```calc
> Index of the bend at B
C_2 = 2 r_2 / d = ??
> Torsional stress-correction factor at B, Eq. 10-37
K_hookB = (4 C_2 - 1) / (4 C_2 - 4) = ??
> Torsion at B, Eq. 10-36
τ_B = K_hookB 8 F_max D / (π d^3) = ?? kpsi
> Hook torsion factor of safety, Table 10-7
n_B = f_B S_ut / τ_B = ??
```

### Checks

```calc {description="Design factor required against yield"}
n_d = 1_
```

```calc
@test n_body ≥ n_d
```

```calc
@test n_A ≥ n_d
```

```calc
@test n_B ≥ n_d
```

```calc
@test τ_(i,lo) ≤ τ_i ≤ τ_(i,hi)
```

```calc
@test F_max > F_i
```
