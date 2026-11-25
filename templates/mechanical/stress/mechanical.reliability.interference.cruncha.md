+++
cruncha-template = 1
id = "mechanical.reliability.interference"
version = "1.0.0"
title = "Reliability by stress-strength interference (normal)"
summary = "Reliability of a part whose strength and service stress are both normally distributed: the margin of safety's mean and standard deviation, the standard normal variable z, the reliability and failure rate, and the mean safety factor, checked against a reliability goal."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["reliability", "probability of failure", "stress-strength interference", "normal distribution", "margin of safety", "standard deviation", "statistics", "Python"]
from = "lacewing:mechanical.reliability"
license = "GPL-3.0-or-later"
inputs = ["μ_S", "s_S", "μ_σ", "s_σ", "R_goal"]
outputs = ["μ_z", "s_z", "z", "R", "P_f", "N_f", "n_mean"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 6.13-6.15 (Eqs. 6.14, 6.15; Figs. 6.16, 6.20); Appendix H (Table H.1, Eq. H.3)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 1-12 and 1-13 (Eq. 1-10, Example 1-6); Table A-10"

[[tests]]
name = "Juvinall Fig. 6.16: strength 70 (σ 6), stress 40 (σ 8): three standard deviations of margin"
source = "Lacewing Lab reliability_test, after Juvinall & Marshek 5th Fig. 6.16 (its numbers, here taken as ksi): μz = 30, σz = 10, z = −3, R = Φ(3) = 0.99865 (Table H.1)"
inputs = { "μ_S" = "70 ksi", "s_S" = "6 ksi", "μ_σ" = "40 ksi", "s_σ" = "8 ksi", R_goal = "0.998" }
expect = { "μ_z" = "30 ksi", s_z = "10 ksi", z = "-3", R = "0.998650102", P_f = "0.00134989803", n_mean = "1.75" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Shigley Ex. 1-6 in reverse: the book's nd = 1.416 meets R = 0.999"
source = "Shigley 10th Example 1-6 (Lacewing Lab reliability_test checks the same round trip): μS 78.4, σ̂S 5.90 kpsi; stress mean 78.4/1.416 = 55.3672 kpsi with Cσ 0.082 (σ̂σ 4.54011 kpsi); z = −3.0939, R = 0.999012"
inputs = { "μ_S" = "78.4 ksi", "s_S" = "5.90 ksi", "μ_σ" = "55.3672 ksi", "s_σ" = "4.54011 ksi", R_goal = "0.999" }
expect = { z = "-3.093879", R = "0.9990122" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Juvinall Fig. 6.16 against a 1-in-1000 goal: not reliable enough"
source = "As the Fig. 6.16 case: R = 0.99865 is under 0.999"
inputs = { "μ_S" = "70 ksi", "s_S" = "6 ksi", "μ_σ" = "40 ksi", "s_σ" = "8 ksi", R_goal = "0.999" }
expect = { R = "0.998650102" }
tolerance = "1e-8"
checks = "fail"

[[tests]]
name = "Hand check: one standard deviation of margin"
source = "By hand: μz = 50 − 40 = 10 MPa, σz = √(6² + 8²) = 10 MPa, z = −1, R = Φ(1) = 0.841345 (Table H.1: 0.8413), one failure in 1/0.158655 = 6.30297"
inputs = { "μ_S" = "50 MPa", "s_S" = "6 MPa", "μ_σ" = "40 MPa", "s_σ" = "8 MPa", R_goal = "0.8" }
expect = { "μ_z" = "10 MPa", s_z = "10 MPa", z = "-1", R = "0.8413447", N_f = "6.302974" }
tolerance = "1e-6"
checks = "pass"
+++

## Reliability by stress-strength interference

Both the strength of production parts and the stress they see in service scatter. When both are normally distributed and independent, the margin of safety z = strength − stress is normal too, with mean μ_S − μ_σ and standard deviation √(s_S² + s_σ²). The reliability is the probability that the margin is positive.

The values can be stresses, loads or torques, as long as strength and stress are the same kind of quantity. Use the significant strength for the failure mode (yield, ultimate, fatigue at the life) of production parts, not specimens. The normal model fits poorly far out in the tails; lognormal or Weibull models are not covered.

### Distributions

```calc {description="Mean strength of production parts"}
μ_S = 70 ksi_
```

```calc {description="Standard deviation of the strength"}
s_S = 6 ksi_
```

```calc {description="Mean stress in service"}
μ_σ = 40 ksi_
```

```calc {description="Standard deviation of the stress (from the load's scatter)"}
s_σ = 8 ksi_
```

```calc {description="Reliability goal: the share of parts that must survive (0.998 is 1 failure in 500)"}
R_goal = 0.998_
```

### Margin of safety

```calc
> Mean margin of safety, Juvinall Eq. 6.14
μ_z = μ_S - μ_σ = ?? ksi
> Standard deviation of the margin, Eq. 6.15
s_z = hypot(s_S, s_σ) = ?? ksi
> Standard normal variable at zero margin, Shigley Eq. 1-10
z = -μ_z / s_z = ??
```

### Reliability

The normal integral is computed (SciPy's normal distribution), not read from Table H.1 or Table A-10, so it stays accurate far into the tail.

```python {description="Probability of failure: the standard normal probability below z, P(margin < 0)"}
from scipy.stats import norm

P_f = float(norm.cdf(float(z)))
```

```calc
> Reliability, P(margin > 0)
R = 1 - P_f = ??
> One failure in this many parts
N_f = 1 / P_f = ??
> Mean safety factor, Shigley's average design factor
n_mean = μ_S / μ_σ = ??
```

### Check

```calc
@test R ≥ R_goal
```
