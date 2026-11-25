+++
cruncha-template = 1
id = "mechanical.weldCapacity.filletFatigue"
version = "1.0.0"
title = "Fatigue of a fillet weld, Shigley's stress-concentration method"
summary = "Alternating and mean shear on a fillet weld's throat with the fatigue stress-concentration factor, the weldment's endurance limit in shear estimated from the parent metal (as-forged surface), and the Gerber and Goodman fatigue factors of safety, checked against a design factor."
category = ["Mechanical engineering", "Fastening"]
tags = ["weld", "fillet weld", "fatigue", "fluctuating load", "Gerber", "Goodman", "endurance limit", "fatigue stress concentration", "Kfs", "throat", "Shigley", "Juvinall"]
from = "lacewing:mechanical.weldCapacity"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "K_fs", "F_a", "F_m", "S_ut", "a_f", "b_f", "n_d"]
outputs = ["A", "τ_a", "τ_m", "S_e′", "k_a", "S_se", "S_su", "n_Gerber", "n_Goodman", "L_req"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 9-7 (fatigue loading of welds), Table 9-5 (fatigue stress-concentration factors), Examples 9-5 and 9-6; Eqs. 6-8, 6-18, 6-19 and Table 6-2 (surface factor), 6-26 (load factor), 6-46 (Goodman), 6-54 (Ssu), Table 6-7 (Gerber)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 11.6, Table 11.2 (the same fatigue stress-concentration factors)"

[[tests]]
name = "Shigley Ex. 9-5: 1018 HR strap, two 3/8 in fillets 2 in long, 1000 lbf fully reversed"
source = "Lacewing Lab weld_fatigue_test, after Shigley 10th Example 9-5 (p. 488): Kfs = 2.7 at the end of a parallel fillet, τa = 2545 psi, Sse = 12.0 kpsi, nf = 4.72 by Gerber and Goodman"
inputs = { w = "0.375 in", L = "4 in", K_fs = "2.7", F_a = "1000 lbf", F_m = "0 lbf", S_ut = "58 kpsi", a_f = "39.9", b_f = "-0.995", n_d = "2" }
expect = { "τ_a" = "2545 psi", S_se = "12.0 kpsi", n_Gerber = "4.72", n_Goodman = "4.72" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 9-6: the same welds, 0 to 2000 lbf, Kfs = 2"
source = "Lacewing Lab weld_fatigue_test, after Shigley 10th Example 9-6 (p. 489): τa = τm = 1885 psi, Gerber nf = 5.85"
inputs = { w = "0.375 in", L = "4 in", K_fs = "2", F_a = "1000 lbf", F_m = "1000 lbf", S_ut = "58 kpsi", a_f = "39.9", b_f = "-0.995", n_d = "2" }
expect = { "τ_a" = "1885 psi", "τ_m" = "1885 psi", n_Gerber = "5.85" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 9-6 exactly"
source = "By hand: A = 0.375 × 4/√2 = 1.06066 in², τa = τm = 2 × 1000/1.06066 = 1885.618 psi; ka = 39.9 × 58^−0.995 = 0.702040, Sse = 0.702040 × 0.59 × 29 = 12.01191 kpsi; Ssu = 38.86 kpsi; Gerber 5.85593, Goodman 1/(1885.618/12011.91 + 1885.618/38860) = 4.86612; length for nf = 8: 4 × 8/5.85593 = 5.46454 in"
inputs = { w = "0.375 in", L = "4 in", K_fs = "2", F_a = "1000 lbf", F_m = "1000 lbf", S_ut = "58 kpsi", a_f = "39.9", b_f = "-0.995", n_d = "8" }
expect = { A = "1.06066017 in²", "τ_a" = "1885.61808 psi", k_a = "0.70204030", S_se = "12.0119095 kpsi", S_su = "38.86 kpsi", n_Gerber = "5.85593429", n_Goodman = "4.86612291", L_req = "5.46454 in" }
tolerance = "1e-6"
checks = "fail"
+++

## Fatigue of a fillet weld (Shigley Sec. 9-7)

The fatigue factor of safety of a fillet weld (or a group of fillets sharing a load evenly) under a fluctuating load, by the conventional method of Shigley Sec. 9-7: the nominal shear on the throat is raised by a fatigue stress-concentration factor, and compared with the endurance limit in shear of the weldment, estimated from the parent metal with an as-forged surface. Structures designed to a code (AISC 360 Appendix 3, AWS D1.1 fatigue provisions) use the code's detail categories and stress ranges instead.

### Weld and load

```calc {description="Fillet leg size (equal legs)"}
w = 0.375 in_
```

```calc {description="Total effective length of the fillets sharing the load"}
L = 4 in_
```

The fatigue stress-concentration factor depends on the weld detail and applies to both the alternating and the mean stress, as Shigley's examples do. Shigley Table 9-5 (and Juvinall Table 11.2) give values for the toe of a transverse fillet, the end of a parallel fillet, a reinforced butt weld and a T-butt with sharp corners.

```calc {description="Fatigue stress-concentration factor, Shigley Table 9-5"}
K_fs = 2.7_
```

```calc {description="Alternating (amplitude) force on the welds"}
F_a = 1000 lbf_
```

```calc {description="Mean (midrange) force on the welds"}
F_m = 0 lbf_
```

### Parent metal

The endurance limit is estimated from the ultimate strength of the parent (attachment) metal, with Shigley's surface factor k_a = a·Sut^b for an as-forged surface (Eq. 6-19, coefficients from Table 6-2 for Sut in kpsi), unless a better finish is specified and obtained.

```calc {description="Ultimate tensile strength of the parent metal"}
S_ut = 58 kpsi_
```

```calc {description="Surface factor coefficient a, Shigley Table 6-2, for Sut in kpsi"}
a_f = 39.9_
```

```calc {description="Surface factor exponent b, Shigley Table 6-2"}
b_f = -0.995_
```

```calc {description="Design factor the fatigue factor of safety must reach"}
n_d = 2_
```

### Stresses on the throat

```calc
> Throat area of the fillets, 0.707 w L
A = w L / sqrt(2) = ?? in²
> Alternating shear on the throat, with K_fs
τ_a = K_fs F_a / A = ?? psi
> Mean shear on the throat, with K_fs (its sign doesn't matter in shear)
τ_m = K_fs abs(F_m) / A = ?? psi
```

### Endurance limit of the weldment in shear

The size factor is 1 for the uniform stress on the throat, and the load factor 0.59 for shear. Temperature and reliability factors are taken as 1.

```calc
> Rotating-beam endurance limit, Eq. 6-8
S_e′ = { 0.5 S_ut if S_ut ≤ 1400 'MPa'; 700 'MPa' otherwise } = ?? kpsi
> Surface factor, as forged, Eq. 6-19
k_a = a_f (S_ut / 1 'kpsi')^b_f = ??
> Endurance limit in shear, Eq. 6-18 with k_b = 1 and k_c = 0.59
S_se = k_a 0.59 S_e′ = ?? kpsi
> Ultimate shear strength, Eq. 6-54
S_su = 0.67 S_ut = ?? kpsi
```

### Fatigue factors of safety

Shigley recommends Gerber for welds; Goodman is the more conservative line in common use.

```calc {description="Fatigue factor of safety, Gerber, Table 6-7"}
n_Gerber = { S_se / τ_a if τ_m ≤ 0 'psi'; S_su / τ_m if τ_a ≤ 0 'psi'; 0.5 (S_su / τ_m)^2 (τ_a / S_se) (-1 + sqrt(1 + (2 τ_m S_se / (S_su τ_a))^2)) otherwise } = ??
```

```calc {description="Fatigue factor of safety, modified Goodman, Eq. 6-46"}
n_Goodman = 1 / (τ_a / S_se + τ_m / S_su) = ??
```

The factor of safety grows in proportion to the weld length, so the length that reaches the design factor is L·n_d/n_f.

```calc {description="Weld length for the design factor, by Gerber"}
L_req = L n_d / n_Gerber = ?? in
```

The parent metal at the weld's ends and toes sees the same concentration, and needs its own fatigue check.

### Check

```calc
@test n_Gerber ≥ n_d
```
