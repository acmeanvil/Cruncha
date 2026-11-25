+++
cruncha-template = 1
id = "mechanical.fatigue.notchFactor"
version = "1.0.0"
title = "Fatigue: notch sensitivity and fatigue stress-concentration factors (Neuber, steels)"
summary = "Neuber's notch sensitivity q and qs of a steel from its ultimate strength and the notch radius, and the fatigue stress-concentration factors Kf = 1 + q(Kt − 1) and Kfs = 1 + qs(Kts − 1) for any notch whose Kt and Kts are known."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["fatigue", "notch sensitivity", "Neuber", "Kf", "Kfs", "Kt", "stress concentration", "Shigley"]
from = "lacewing:mechanical.fatigue"
license = "GPL-3.0-or-later"
inputs = ["S_ut", "r", "K_t", "K_ts"]
outputs = ["q", "q_s", "K_f", "K_fs"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 6-10: Eqs. 6-32 and 6-33 (Kf, Kfs), Eqs. 6-34 and 6-35 (Neuber's equation and its constant, Sut in kpsi); Example 6-6"

[[tests]]
name = "Shigley Ex. 6-6: 690 MPa steel shaft, 3 mm shoulder fillet, Kt 1.65"
source = "Lacewing Lab stress_test (fatigue, Shigley ch. 6 examples): Neuber gives Kf = 1.55 at Sut 690 MPa, r = 3 mm, Kt = 1.65 (Shigley reads q = 0.84 off Fig. 6-20; Eq. 6-34 gives 0.847). Torsion by hand: √a = 0.04725 √in, qs = 0.879, Kfs = 1 + 0.879 × 0.35 = 1.308"
inputs = { S_ut = "690 MPa", r = "3 mm", K_t = "1.65", K_ts = "1.35" }
expect = { q = "0.8467", q_s = "0.8791", K_f = "1.550", K_fs = "1.3077" }
tolerance = "0.05%"

[[tests]]
name = "No notch: Kt = Kts = 1"
source = "By hand: Kf = 1 + q(1 − 1) = 1 whatever q"
inputs = { S_ut = "400 MPa", r = "1 mm", K_t = "1", K_ts = "1" }
expect = { K_f = "1", K_fs = "1" }
+++

## Notch sensitivity and fatigue stress-concentration factors

A notch reduces the fatigue strength by less than its geometric (theoretical) stress-concentration factor Kt suggests, more so for small radii and softer steels. The notch sensitivity q, from 0 (no notch effect) to 1 (fully sensitive), gives the fatigue stress-concentration factor K_f = 1 + q (Kt − 1) (Shigley Eq. 6-32), and the same in torsion (Eq. 6-33). K_f and K_fs multiply the nominal alternating and mean stresses in the fluctuating-stress template.

Neuber's equation (Eq. 6-34) with Shigley's fit of the Neuber constant √a to Sut (Eq. 6-35, Sut in kpsi, √a in √in) is for steels; Shigley fits it to the curves of Figs. 6-20 and 6-21. For cast irons q is about 0.2 (Sec. 6-10), and for other materials q comes from test data: use K_f = 1 + q (Kt − 1) with that q instead. When in doubt, K_f = Kt is conservative.

### Material and notch

Kt and Kts come from a stress-concentration chart for the notch (Peterson's charts, or Shigley Table A-15), or from the shaft-shoulder template.

```calc {description="Tensile ultimate strength (steel)"}
S_ut = 690 MPa_
```

```calc {description="Notch root radius"}
r = 3 mm_
```

```calc {description="Geometric stress-concentration factor in bending or axial load"}
K_t = 1.65_
```

```calc {description="Geometric stress-concentration factor in torsion"}
K_ts = 1.35_
```

### Neuber's notch sensitivity

```calc
> Ultimate strength as a number of kpsi, for Eq. 6-35
s_u = S_ut / (1 'ksi') = ??
> Neuber constant a for bending or axial load, Eq. 6-35a (√a squared)
a_N = (0.246 - 3.08e-3 s_u + 1.51e-5 s_u^2 - 2.67e-8 s_u^3)^2 × 1 'in' = ?? in
> Neuber constant a for torsion, Eq. 6-35b (√a squared)
a_Ns = (0.190 - 2.51e-3 s_u + 1.35e-5 s_u^2 - 2.67e-8 s_u^3)^2 × 1 'in' = ?? in
```

```calc {description="Notch sensitivity in bending or axial load, Eq. 6-34"}
q = 1 / (1 + sqrt(a_N / r)) = ??
```

```calc {description="Notch sensitivity in torsion, Eq. 6-34"}
q_s = 1 / (1 + sqrt(a_Ns / r)) = ??
```

### Fatigue stress-concentration factors

```calc {description="Fatigue stress-concentration factor in bending or axial load, Eq. 6-32"}
K_f = 1 + q (K_t - 1) = ??
```

```calc {description="Fatigue stress-concentration factor in torsion, Eq. 6-33"}
K_fs = 1 + q_s (K_ts - 1) = ??
```
