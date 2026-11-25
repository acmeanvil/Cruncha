+++
cruncha-template = 1
id = "mechanical.adhesiveJoints.doubleLap"
version = "1.0.0"
title = "Adhesive double-lap joint, Volkersen shear lag with thermal mismatch"
summary = "Average and peak shear in the adhesive of a symmetric double-lap joint by Volkersen's elastic shear-lag solution (Shigley Eq. 9-7), with the residual shear from the adherends' thermal mismatch, and the lap-shear strength over the average shear checked against a design factor."
category = ["Mechanical engineering", "Fastening"]
tags = ["adhesive", "bonded joint", "double lap", "lap shear", "shear lag", "Volkersen", "thermal mismatch", "epoxy", "Shigley"]
from = "lacewing:mechanical.adhesiveJoints"
license = "GPL-3.0-or-later"
inputs = ["E_o", "t_o", "α_o", "E_i", "t_i", "α_i", "G", "h", "b", "l", "P", "ΔT", "S_s", "n_d"]
outputs = ["τ_av", "ω", "τ_P0", "τ_Pl", "τ_Pf", "τ_th", "τ_l", "τ_f", "τ_max", "n_av"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 9-9 (adhesive bonding: stress distributions), Eq. 9-7 and Fig. 9-26 (double lap, after Volkersen), Example 9-7, Table 9-7 (lap-shear strengths by adhesive chemistry)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 11.8 (adhesives: design guidance, average lap shear)"

[[tests]]
name = "Shigley Ex. 9-7: aluminium outers, steel inner, cured at 200 °F, 2000 lbf at 70 °F"
source = "Lacewing Lab adhesive_joints_test, after Shigley 10th Example 9-7 (pp. 494-496): ω = 3.65 in^-1; load shear 1922 psi at the ends and 604 psi in the middle; thermal shear ±2466 psi; 4388 psi at x = +l/2; average 1000 psi"
inputs = { E_o = "10 Mpsi", t_o = "0.150 in", "α_o" = "0.0000133 Δ°F^-1", E_i = "30 Mpsi", t_i = "0.100 in", "α_i" = "0.000006 Δ°F^-1", G = "0.2 Mpsi", h = "0.020 in", b = "1 in", l = "1 in", P = "2000 lbf", "ΔT" = "-130 Δ°F", S_s = "3000 psi", n_d = "2" }
expect = { "ω" = "3.65 in^-1", "τ_P0" = "604 psi", "τ_Pl" = "1922 psi", "τ_th" = "2466 psi", "τ_max" = "4388 psi", "τ_av" = "1000 psi" }
tolerance = "0.15%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 9-7 exactly"
source = "By hand from Eq. 9-7: ω = √[(0.2e6/0.02)(1/1.5e6 + 2/3e6)] = 3.651484 1/in, ωl/2 = 1.825742; balanced (2Eo·to = Ei·ti), so the sinh load term vanishes; τ_P(±l/2) = (2000 × 3.651484/4) coth(1.825742) = 1923.032 psi; τ_th = (6.0 − 13.3)e-6 × (−130) × 3.651484 tanh(1.825742)/1.33333e-6 = 2467.457 psi; 4390.490 psi at x = +l/2, −544.425 psi at −l/2"
inputs = { E_o = "10 Mpsi", t_o = "0.150 in", "α_o" = "0.0000133 Δ°F^-1", E_i = "30 Mpsi", t_i = "0.100 in", "α_i" = "0.000006 Δ°F^-1", G = "0.2 Mpsi", h = "0.020 in", b = "1 in", l = "1 in", P = "2000 lbf", "ΔT" = "-130 Δ°F", S_s = "3000 psi", n_d = "2" }
expect = { "ω" = "3.651483717 in^-1", "τ_P0" = "603.9203439 psi", "τ_Pl" = "1923.032271 psi", "τ_Pf" = "1923.032271 psi", "τ_th" = "2467.457292 psi", "τ_l" = "4390.489563 psi", "τ_f" = "-544.4250212 psi", "τ_max" = "4390.489563 psi", n_av = "3" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Hand check: unbalanced joint, thinner outers"
source = "By hand from Eq. 9-7 with t_o = 0.100 in: ω = 4.082483 1/in, (2Eo·to − Ei·ti)/(2Eo·to + Ei·ti) = −0.2; τ_P = 1716.570 psi at +l/2 and 2505.982 psi at −l/2; τ_th = 2247.456 psi; peak 3964.026 psi at +l/2"
inputs = { E_o = "10 Mpsi", t_o = "0.100 in", "α_o" = "0.0000133 Δ°F^-1", E_i = "30 Mpsi", t_i = "0.100 in", "α_i" = "0.000006 Δ°F^-1", G = "0.2 Mpsi", h = "0.020 in", b = "1 in", l = "1 in", P = "2000 lbf", "ΔT" = "-130 Δ°F", S_s = "3000 psi", n_d = "2" }
expect = { "ω" = "4.082482905 in^-1", "τ_Pl" = "1716.569858 psi", "τ_Pf" = "2505.981926 psi", "τ_th" = "2247.456156 psi", "τ_max" = "3964.026014 psi" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Ex. 9-7 with a weaker adhesive: lap-shear strength 1.5 times the average shear"
source = "As Shigley Ex. 9-7: τav = 1000 psi against a lap-shear strength of 1500 psi, under the design factor of 2"
inputs = { E_o = "10 Mpsi", t_o = "0.150 in", "α_o" = "0.0000133 Δ°F^-1", E_i = "30 Mpsi", t_i = "0.100 in", "α_i" = "0.000006 Δ°F^-1", G = "0.2 Mpsi", h = "0.020 in", b = "1 in", l = "1 in", P = "2000 lbf", "ΔT" = "-130 Δ°F", S_s = "1500 psi", n_d = "2" }
expect = { n_av = "1.5" }
tolerance = "1e-9"
checks = "fail"
+++

## Adhesive double-lap joint (Shigley Sec. 9-9)

A symmetric double-lap joint: an inner adherend bonded between two outer ones over an overlap l and width b, carrying a load P through the inner adherend. Volkersen's shear-lag analysis (Shigley Eq. 9-7) treats the adherends as elastic bars in tension and the adhesive as a thin elastic layer in shear. The shear gathers at the ends of the overlap, and a mismatch in the adherends' thermal expansion adds a residual shear, there before any load, whenever the service temperature differs from the temperature at which the bond is stress free (usually the cure).

The analysis ignores the adherends' bending, which is small in a symmetric double lap, and the adhesive's plasticity, which lowers the peaks in practice. x is measured from the middle of the overlap, with its sense as in Shigley's Fig. 9-26; the imbalance and thermal terms change sign between the two ends.

### Adherends

```calc {description="Modulus of each outer adherend"}
E_o = 10 Mpsi_
```

```calc {description="Thickness of each outer adherend"}
t_o = 0.150 in_
```

```calc {description="Thermal expansion coefficient of the outer adherends"}
α_o = 0.0000133 Δ°F^-1_
```

```calc {description="Modulus of the inner adherend"}
E_i = 30 Mpsi_
```

```calc {description="Whole thickness of the inner adherend"}
t_i = 0.100 in_
```

```calc {description="Thermal expansion coefficient of the inner adherend"}
α_i = 0.000006 Δ°F^-1_
```

### Adhesive, bond and load

```calc {description="Shear modulus of the adhesive"}
G = 0.2 Mpsi_
```

```calc {description="Bond line thickness"}
h = 0.020 in_
```

```calc {description="Width of the bond"}
b = 1 in_
```

```calc {description="Overlap length, along the load"}
l = 1 in_
```

```calc {description="Load on the inner adherend (shared by the two bond lines)"}
P = 2000 lbf_
```

```calc {description="Service temperature less the bond's stress-free temperature"}
ΔT = -130 Δ°F_
```

The lap-shear strength is the apparent (average) shear strength a lap-shear test reports, from the adhesive's data sheet; Shigley Table 9-7 gives typical ranges by adhesive chemistry.

```calc {description="Lap-shear strength of the adhesive"}
S_s = 3000 psi_
```

```calc {description="Design factor on the lap-shear strength"}
n_d = 2_
```

### Shear along the overlap, Eq. 9-7

```calc
> Average shear on the two bond lines, P/(2bl)
τ_av = P / (2 b l) = ?? psi
> Shear-lag parameter
ω = sqrt(G / h (1 / (E_o t_o) + 2 / (E_i t_i))) = ?? in^-1
> Half the overlap in shear-lag lengths: under about 1 the shear is nearly uniform
β_l = ω l / 2 = ??
> Stiffness imbalance: zero for a balanced joint (2Eo·to = Ei·ti)
r_s = (2 E_o t_o - E_i t_i) / (2 E_o t_o + E_i t_i) = ??
```

```calc
> Load shear in the middle of the overlap
τ_P0 = P ω / (4 b sinh(β_l)) = ?? psi
> Load shear at the end x = +l/2
τ_Pl = P ω / (4 b) (coth(β_l) + r_s tanh(β_l)) = ?? psi
> Load shear at the end x = −l/2
τ_Pf = P ω / (4 b) (coth(β_l) - r_s tanh(β_l)) = ?? psi
> Thermal-mismatch shear at x = +l/2 (the negative of it at −l/2)
τ_th = (α_i - α_o) ΔT ω tanh(β_l) / (1 / (E_o t_o) + 2 / (E_i t_i)) = ?? psi
```

```calc
> Shear at x = +l/2, load and thermal together
τ_l = τ_Pl + τ_th = ?? psi
> Shear at x = −l/2
τ_f = τ_Pf - τ_th = ?? psi
> Largest shear in the adhesive, at an end
τ_max = max(abs(τ_l), abs(τ_f)) = ?? psi
```

### Strength

Lap-shear strengths are average stresses from tests whose own peaks are like these, so they are compared with the average shear. A peak far above the average (τ_max/τ_av large) means a longer overlap adds little strength: more width, or thicker or stiffer adherends, help more.

```calc {description="Lap-shear strength over the average shear"}
n_av = S_s / τ_av = ??
```

```calc
@test n_av ≥ n_d
```
