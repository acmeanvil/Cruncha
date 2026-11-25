+++
cruncha-template = 1
id = "mechanical.boltedJoint.fatigue"
version = "1.0.0"
title = "Preloaded bolt in fatigue: Goodman, Gerber, ASME-elliptic (Shigley)"
summary = "Alternating and mean stress of a preloaded bolt under an external load fluctuating between P_min and P_max, and its fatigue factor of safety on the load line from the preload stress by the Goodman, Gerber and ASME-elliptic criteria, checked against a design factor."
category = ["Mechanical engineering", "Fastening"]
tags = ["bolt", "bolted joint", "fatigue", "preload", "joint constant", "Goodman", "Gerber", "ASME-elliptic", "endurance strength", "fluctuating load", "Shigley"]
from = "lacewing:mechanical.boltedJoint"
license = "GPL-3.0-or-later"
inputs = ["C", "F_i", "A_t", "S_p", "S_ut", "S_e", "P_max", "P_min", "n_d"]
outputs = ["σ_i", "σ_a", "σ_m", "n_f", "n_(f,Gerber)", "n_(f,ASME)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-11: Eqs. 8-35, 8-36 (alternating and mean bolt stress), 8-38 (Goodman on the load line from σi), 8-43, 8-44 (Gerber, ASME-elliptic), 8-46, 8-47 (their closed forms for P_min = 0); Table 8-17 (endurance strengths); Example 8-5"

[[tests]]
name = "Shigley Ex. 8-5: 5/8-11 grade 5 cap screw, 0 to 5 kip"
source = "Lacewing Lab bolted_joint_test, after Shigley 10th Example 8-5: C = 0.2803, Fi = 14.41 kip, At = 0.226 in², Sp 85, Sut 120 kpsi, Se 18.6 kpsi (Table 8-17, grade 5): σa = 3.10 kpsi, Goodman nf = 2.44 (2.435 unrounded), Gerber 3.65"
inputs = { C = "0.2803", F_i = "14.4075 kip", A_t = "0.226 in²", S_p = "85 ksi", S_ut = "120 ksi", S_e = "18.6 ksi", P_max = "5 kip", P_min = "0 kip", n_d = "2" }
expect = { "σ_a" = "3.10 ksi", n_f = "2.44", "n_(f,Gerber)" = "3.65" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 8-5, ASME-elliptic by Eq. 8-47"
source = "By hand: σi = 14.4075/0.226 = 63.75 kpsi, σa = 0.2803 × 5/(2 × 0.226) = 3.100664 kpsi, σm = 66.850664 kpsi; Eq. 8-38 nf = 18.6(120 − 63.75)/(120 × 3.100664 + 18.6 × 3.100664) = 2.434544; Eq. 8-46 Gerber 3.651049; Eq. 8-47 ASME-elliptic 3.048792"
inputs = { C = "0.2803", F_i = "14.4075 kip", A_t = "0.226 in²", S_p = "85 ksi", S_ut = "120 ksi", S_e = "18.6 ksi", P_max = "5 kip", P_min = "0 kip", n_d = "2" }
expect = { "σ_i" = "63.75 ksi", "σ_a" = "3.100664 ksi", "σ_m" = "66.850664 ksi", n_f = "2.434544", "n_(f,Gerber)" = "3.651049", "n_(f,ASME)" = "3.048792" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 8-5's joint with the load fluctuating from 2 to 5 kip"
source = "By hand, the load line from σi through (σm, σa): σa = 1.860398, σm = 68.090929 kpsi; Goodman Eq. 8-38 3.441736; Gerber and ASME-elliptic as the roots of Sa/Se + ((σi + Sa(σm − σi)/σa)/Sut)² = 1 and (Sa/Se)² + ((σi + Sa(σm − σi)/σa)/Sp)² = 1, over σa: 4.952305, 3.589638"
inputs = { C = "0.2803", F_i = "14.4075 kip", A_t = "0.226 in²", S_p = "85 ksi", S_ut = "120 ksi", S_e = "18.6 ksi", P_max = "5 kip", P_min = "2 kip", n_d = "2" }
expect = { "σ_a" = "1.860398 ksi", "σ_m" = "68.090929 ksi", n_f = "3.441736", "n_(f,Gerber)" = "4.952305", "n_(f,ASME)" = "3.589638" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 8-5 against a design factor of 3: not enough margin"
source = "Shigley 10th Example 8-5: Goodman nf = 2.44, under 3"
inputs = { C = "0.2803", F_i = "14.4075 kip", A_t = "0.226 in²", S_p = "85 ksi", S_ut = "120 ksi", S_e = "18.6 ksi", P_max = "5 kip", P_min = "0 kip", n_d = "3" }
expect = { n_f = "2.434544" }
tolerance = "1e-6"
checks = "fail"
+++

## Preloaded bolt in fatigue

The fatigue factor of safety of a preloaded bolt in a tension joint whose external load per bolt fluctuates between P_min and P_max (Shigley Sec. 8-11). The joint stays closed, so the bolt sees only the share C of the load swing on top of its preload; take C and F_i from the bolted joint in tension template. With a preload the bolt's stress starts at σ_i, not zero, so the load line runs from (σ_i, 0) through the working point (σ_m, σ_a), and each factor of safety is how far along that line the failure curve lies. P_max must exceed P_min: a steady load has no fatigue factor.

S_e is the bolt's fully corrected endurance strength at the thread, with the thread's stress concentration already in it: for rolled threads of SAE grades 5 to 8 and ISO classes 8.8 to 12.9 see Shigley Table 8-17, otherwise derive it from the material's S′_e with the Marin factors and K_f.

### Joint and bolt

```calc {description="Joint constant, the bolt's share of the external load"}
C = 0.2803_
```

```calc {description="Preload"}
F_i = 14.4075 kip_
```

```calc {description="Tensile-stress area of the thread"}
A_t = 0.226 in²_
```

```calc {description="Minimum proof strength of the bolt grade"}
S_p = 85 ksi_
```

```calc {description="Minimum tensile strength of the bolt grade"}
S_ut = 120 ksi_
```

```calc {description="Fully corrected endurance strength of the threaded bolt (e.g. Shigley Table 8-17)"}
S_e = 18.6 ksi_
```

### Load

```calc {description="Largest external tensile load per bolt"}
P_max = 5 kip_
```

```calc {description="Least external tensile load per bolt"}
P_min = 0 kip_
```

```calc {description="Design factor the fatigue factor of safety must reach"}
n_d = 2_
```

### Bolt stresses

```calc
> Preload stress
σ_i = F_i / A_t = ?? ksi
> Alternating stress, Eq. 8-35
σ_a = C (P_max - P_min) / (2 A_t) = ?? ksi
> Mean stress, Eq. 8-36
σ_m = C (P_max + P_min) / (2 A_t) + σ_i = ?? ksi
> Slope of the load line, ΔSm/ΔSa
s_L = (σ_m - σ_i) / σ_a = ??
```

### Factors of safety

Goodman has a closed form on this load line (Eq. 8-38). For Gerber and ASME-elliptic the point S_a on the load line where S_m = σ_i + s_L S_a meets the curve is the positive root of a quadratic; for P_min = 0 these are Shigley's Eqs. 8-46 and 8-47.

```calc {description="Fatigue factor of safety, Goodman, Eq. 8-38"}
n_f = S_e (S_ut - σ_i) / (S_ut σ_a + S_e (σ_m - σ_i)) = ??
```

```calc
> Gerber: S_a/S_e + ((σ_i + s_L S_a)/S_ut)² = 1, as a S_a² + b S_a + c = 0
a_G = s_L^2 / S_ut^2 = ?? 1/ksi²
b_G = 1 / S_e + 2 σ_i s_L / S_ut^2 = ?? 1/ksi
c_G = σ_i^2 / S_ut^2 - 1 = ??
> Fatigue factor of safety, Gerber, Eq. 8-43 on the load line
n_(f,Gerber) = (sqrt(b_G^2 - 4 a_G c_G) - b_G) / (2 a_G σ_a) = ??
```

```calc
> ASME-elliptic: (S_a/S_e)² + ((σ_i + s_L S_a)/S_p)² = 1, as a S_a² + b S_a + c = 0
a_A = 1 / S_e^2 + s_L^2 / S_p^2 = ?? 1/ksi²
b_A = 2 σ_i s_L / S_p^2 = ?? 1/ksi
c_A = σ_i^2 / S_p^2 - 1 = ??
> Fatigue factor of safety, ASME-elliptic, Eq. 8-44 on the load line
n_(f,ASME) = (sqrt(b_A^2 - 4 a_A c_A) - b_A) / (2 a_A σ_a) = ??
```

Goodman is the most conservative of the three on this load line; Gerber and ASME-elliptic follow test data more closely. The proof, load and separation factors are in the bolted joint in tension template.

### Check

```calc
@test n_f ≥ n_d
```
