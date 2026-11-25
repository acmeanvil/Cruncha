+++
cruncha-template = 1
id = "mechanical.rollingBearings.taperedPair"
version = "1.0.0"
title = "Tapered roller bearing pair: induced thrust and life (Shigley, Timken)"
summary = "Two tapered roller bearings mounted as a pair, with radial loads on both and an external thrust: each bearing's induced thrust, which bearing carries the net thrust, both equivalent loads, the catalogue rating each needs for a design life at the pair's reliability, and the reliability each bearing and the pair reach."
category = ["Mechanical engineering", "Bearings"]
tags = ["rolling bearing", "tapered roller bearing", "induced thrust", "direct mounting", "indirect mounting", "Timken", "equivalent load", "bearing life", "reliability", "Weibull", "Shigley"]
from = "lacewing:mechanical.rollingBearings"
license = "GPL-3.0-or-later"
inputs = ["F_rA", "F_rB", "F_ae", "K_A", "K_B", "C_10A", "C_10B", "a_f", "n", "L_D", "R_D", "L_R", "x_0", "θ", "b_W"]
outputs = ["F_iA", "F_iB", "F_eA", "F_eB", "x_D", "R_i", "C_(req,A)", "C_(req,B)", "R_A", "R_B", "R"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 11-9 Tapered Roller Bearings: Eqs. 11-18 to 11-20, Fig. 11-21 (mounting); Eqs. 11-9 and 11-21 with Table 11-6's Timken parameters; Examples 11-8 and 11-10"

[[tests]]
name = "Shigley Ex. 11-8: direct-mounted pair, Fae 1690 N, K = 1.5 trial"
source = "Lacewing Lab rolling_bearings_test, after Shigley 10th Example 11-8: FrA = 2170 N, FrB = 2654 N, external thrust 1690 N toward A, first trial K = 1.5 for both: FiA = 680 N, FiB = 832 N, A carries the thrust, FeA = 4651 N, FeB = 2654 N; 5000 h at 800 rev/min is xD = 2.67 Timken rating lives, and at R = √0.99 = 0.995 each, C10 = 11 486 N for A and 6554 N for B (Eq. 11-10; the exact Eq. 11-9 is 0.1 % lower)"
inputs = { F_rA = "2170 N", F_rB = "2654 N", F_ae = "1690 N", K_A = "1.5", K_B = "1.5", C_10A = "12100 N", C_10B = "12100 N", n = "800 rpm", L_D = "5000 h", R_D = "0.99" }
expect = { F_iA = "680 N", F_iB = "832 N", F_eA = "4651 N", F_eB = "2654 N", x_D = "2.667", "C_(req,A)" = "11486 N", "C_(req,B)" = "6554 N" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 11-8 and 11-10: cone 15100 and cup 15245 (K = 1.67, C10 = 12 100 N) in both places"
source = "Lacewing Lab rolling_bearings_test, after Shigley 10th Examples 11-8 (second trial, K = 1.67: FiB = 747 N, FeA = 4938 N, C10 needed 12 195 N) and 11-10 (with C10 = 12 100 N: RA = 0.994 791, RB = 0.999 766 by Eq. 11-24; the exact Eq. 11-21 gives 0.994 817 and 0.999 767). The pair reaches R = 0.9946, above 0.99"
inputs = { F_rA = "2170 N", F_rB = "2654 N", F_ae = "1690 N", K_A = "1.67", K_B = "1.67", C_10A = "12100 N", C_10B = "12100 N", n = "800 rpm", L_D = "5000 h", R_D = "0.99" }
expect = { F_iB = "747 N", F_eA = "4938 N", "C_(req,A)" = "12195 N", R_A = "0.994791", R_B = "0.999766" }
tolerance = "0.15%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 11-8 with 9000 N bearings: too small"
source = "By hand: K = 1.67, FeA = 0.4 × 2170 + 1.67 × (746.93413 + 1690) = 4937.68 N, xD = 2.6666667: RA = exp(−(2.6666667 (4937.68/9000)^(10/3)/4.48)^1.5) = 0.97743211, RB = 0.99897646, R = 0.97643167, under 0.99; the rating needed at R = √0.99 is 12181.480 N for A"
inputs = { F_rA = "2170 N", F_rB = "2654 N", F_ae = "1690 N", K_A = "1.67", K_B = "1.67", C_10A = "9000 N", C_10B = "9000 N", n = "800 rpm", L_D = "5000 h", R_D = "0.99" }
expect = { F_eA = "4937.68 N", F_eB = "2654 N", R_A = "0.97743211", R_B = "0.99897646", R = "0.97643167", "C_(req,A)" = "12181.480 N", "C_(req,B)" = "6547.5379 N" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: bearing B carries the thrust"
source = "By hand: FrA = 4000 N, FrB = 2000 N, Fae = 200 N, K = 1.5 both: FiA = 0.47 × 4000/1.5 = 1253.3333 N exceeds FiB + Fae = 826.66667 N, so FeA = FrA = 4000 N and FeB = 0.4 × 2000 + 1.5 × (1253.3333 − 200) = 2380 N; 20 000 h at 1200 rev/min, xD = 16: C10 needed 16892.054 N (A) and 10050.772 N (B); with 30 and 20 kN, R = 0.99955462"
inputs = { F_rA = "4000 N", F_rB = "2000 N", F_ae = "200 N", K_A = "1.5", K_B = "1.5", C_10A = "30000 N", C_10B = "20000 N", n = "1200 rpm", L_D = "20000 h", R_D = "0.99" }
expect = { F_iA = "1253.3333 N", F_eA = "4000 N", F_eB = "2380 N", x_D = "16", "C_(req,A)" = "16892.054 N", "C_(req,B)" = "10050.772 N", R = "0.99955462" }
tolerance = "1e-6"
checks = "pass"
+++

## Tapered roller bearing pair

A tapered roller bearing's radial load pushes its rollers along their taper and so induces a thrust, F_i = 0.47 F_r/K (Shigley Eq. 11-18), that its partner must react. The pair is mounted direct (back to back of the cups, Shigley Fig. 11-21) or indirect; either way, call **A** the bearing that the external thrust F_ae pushes toward (the one it squeezes) and B the other. If A's own induced thrust is no more than B's plus F_ae, A carries the net thrust F_iB + F_ae (Eq. 11-19); otherwise B carries F_iA − F_ae (Eq. 11-20). A bearing's equivalent load is never taken below its radial load. Life follows Timken's rating basis: ratings at L_R = 90 × 10⁶ revolutions, life exponent 10/3, Weibull x_0 = 0, θ = 4.48, b = 1.5 (Shigley Table 11-6).

### Loads and K factors

K is the ratio of the bearing's radial to its thrust rating, the catalogue's K (Timken) or Y. To start a selection Shigley suggests K = 1.5, or about 0.75 for a steep-angle bearing; repeat with the chosen bearings' K.

```calc {description="Radial load on bearing A (the one the external thrust squeezes)"}
F_rA = 2170 N_
```

```calc {description="Radial load on bearing B"}
F_rB = 2654 N_
```

```calc {description="External thrust, toward bearing A"}
F_ae = 1690 N_
```

```calc {description="K factor of bearing A, radial over thrust rating (catalogue)"}
K_A = 1.5_
```

```calc {description="K factor of bearing B, radial over thrust rating (catalogue)"}
K_B = 1.5_
```

```calc
> Induced thrust of bearing A, Eq. 11-18
F_iA = 0.47 F_rA / K_A = ?? N
> Induced thrust of bearing B, Eq. 11-18
F_iB = 0.47 F_rB / K_B = ?? N
> Equivalent load of A, Eqs. 11-19 and 11-20
F_eA = {max(F_rA, 0.4 F_rA + K_A (F_iB + F_ae)) if F_iA ≤ F_iB + F_ae; F_rA otherwise} = ?? N
> Equivalent load of B, Eqs. 11-19 and 11-20
F_eB = {F_rB if F_iA ≤ F_iB + F_ae; max(F_rB, 0.4 F_rB + K_B (F_iA - F_ae)) otherwise} = ?? N
```

### Life and reliability

The design life L_D is at the speed n, and R_D is the reliability wanted of the pair. Each bearing is sized for √R_D, so that the pair, whose reliability is the product of the two, reaches R_D (Shigley Ex. 11-8). The rating needed is the exact Eq. 11-9 (Shigley's examples use Eq. 11-10's 1 − R, a fraction of a percent apart); the reliabilities are Eq. 11-21 with the bearings' catalogue ratings.

```calc {description="Catalogue rating C10 of bearing A (at L_R revolutions)"}
C_10A = 12100 N_
```

```calc {description="Catalogue rating C10 of bearing B (at L_R revolutions)"}
C_10B = 12100 N_
```

```calc {description="Application factor, Shigley Table 11-5"}
a_f = 1_
```

```calc {description="Speed"}
n = 800 rpm_
```

```calc {description="Design life, in hours at the speed"}
L_D = 5000 h_
```

```calc {description="Reliability wanted of the pair"}
R_D = 0.99_
```

```calc {description="Rating life of the catalogue's C10, in revolutions (Timken: 90·10⁶)"}
L_R = 90e6_
```

```calc {description="Weibull guaranteed value x0, Shigley Table 11-6 (Timken: 0)"}
x_0 = 0_
```

```calc {description="Weibull characteristic parameter θ, Shigley Table 11-6 (Timken: 4.48)"}
θ = 4.48_
```

```calc {description="Weibull shape parameter b, Shigley Table 11-6 (Timken: 1.5)"}
b_W = 1.5_
```

Cruncha's rpm is an angular speed (2π radians a revolution), so the revolutions are counted as n/(2π).

```calc
> Design life in rating lives
x_D = L_D n / (2 π L_R) = ??
> Reliability each bearing is sized for
R_i = sqrt(R_D) = ??
> Weibull life measure at R_i, Eq. 11-8
x_R = x_0 + (θ - x_0) ln(1 / R_i)^(1 / b_W) = ??
> Rating bearing A needs, Eq. 11-9
C_(req,A) = a_f F_eA (x_D / x_R)^(3 / 10) = ?? N
> Rating bearing B needs, Eq. 11-9
C_(req,B) = a_f F_eB (x_D / x_R)^(3 / 10) = ?? N
> Reliability of bearing A at the design life, Eq. 11-21
R_A = exp(-(max(x_D (a_f F_eA / C_10A)^(10 / 3) - x_0, 0) / (θ - x_0))^b_W) = ??
> Reliability of bearing B at the design life, Eq. 11-21
R_B = exp(-(max(x_D (a_f F_eB / C_10B)^(10 / 3) - x_0, 0) / (θ - x_0))^b_W) = ??
> Reliability of the pair
R = R_A R_B = ??
```

### Checks

```calc
@test R ≥ R_D
```
