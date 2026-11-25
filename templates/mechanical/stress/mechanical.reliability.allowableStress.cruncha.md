+++
cruncha-template = 1
id = "mechanical.reliability.allowableStress"
version = "1.0.0"
title = "Largest mean stress for a reliability goal (normal)"
summary = "The largest mean service stress, load or torque that keeps failures to a goal when strength and stress are both normally distributed: the standard deviations of margin needed, the margin's deviation and mean, and the allowable mean stress."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["reliability", "probability of failure", "stress-strength interference", "normal distribution", "allowable stress", "margin of safety", "wrench setting", "statistics", "Python"]
from = "lacewing:mechanical.reliability"
license = "GPL-3.0-or-later"
inputs = ["μ_S", "s_S", "s_σ", "R"]
outputs = ["s_z", "k", "μ_z", "μ_σ"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 6.14-6.15 (Eqs. 6.14, 6.15; Fig. 6.20; Sample Problem 6.4); Appendix H (Table H.1)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 1-12 (Eq. 1-10); Table A-10"

[[tests]]
name = "Juvinall Sample Problem 6.4: bolts twist off at 20 ± 1 N·m, wrenches scatter 1.5 N·m, 1 in 500"
source = "Lacewing Lab reliability_test, after Juvinall & Marshek 5th Sample Problem 6.4: σz = 1.80 N·m; k = 2.878 (the book reads 2.9 off Fig. 6.20, Appendix H 2.88); mean setting 14.81 N·m (book 14.78, Appendix H 14.8)"
inputs = { "μ_S" = "20 N·m", "s_S" = "1 N·m", "s_σ" = "1.5 N·m", R = "0.998" }
expect = { s_z = "1.80 N·m", k = "2.878", "μ_σ" = "14.81 N·m" }
tolerance = "0.3%"

[[tests]]
name = "Sample Problem 6.4 to solver precision"
source = "An independent evaluation with scipy.stats.norm.ppf: k = Φ⁻¹(0.998) = 2.8781617, σz = √(1 + 2.25) = 1.8027756, μσ = 20 − 2.8781617 × 1.8027756 = 14.8113201 N·m"
inputs = { "μ_S" = "20 N·m", "s_S" = "1 N·m", "s_σ" = "1.5 N·m", R = "0.998" }
expect = { s_z = "1.80277564 N·m", k = "2.87816174", "μ_z" = "5.18867986 N·m", "μ_σ" = "14.8113201 N·m" }
tolerance = "1e-7"

[[tests]]
name = "Hand check: three standard deviations"
source = "By hand, with Juvinall Fig. 6.16's distributions: R = Φ(3) = 0.99865010196837 needs k = 3; σz = √(6² + 8²) = 10, so the mean may be 70 − 30 = 40 (here as torques, N·m)"
inputs = { "μ_S" = "70 N·m", "s_S" = "6 N·m", "s_σ" = "8 N·m", R = "0.99865010196837" }
expect = { k = "3", s_z = "10 N·m", "μ_σ" = "40 N·m" }
tolerance = "1e-7"
+++

## Largest mean stress for a reliability goal

The inverse of the interference calculation: given the strength's distribution and the stress's scatter, how high may the mean stress be set so that no more than a share 1 − R of parts fail? Strength and stress are normal and independent, so the margin of safety is normal, and its mean must be k of its standard deviations, where k is the standard normal value whose cumulative probability is R.

The values can be stresses, loads or torques: in Juvinall's Sample Problem 6.4 (the defaults) they are the torque at which bolts twist off and the scatter of the wrenches that tighten them. The result is the mean wrench setting. The normal model fits poorly far out in the tails.

### Distributions and goal

```calc {description="Mean strength of production parts"}
μ_S = 20 N·m_
```

```calc {description="Standard deviation of the strength"}
s_S = 1 N·m_
```

```calc {description="Standard deviation of the service stress"}
s_σ = 1.5 N·m_
```

```calc {description="Reliability goal: the share of parts that must survive (0.998 is 1 failure in 500)"}
R = 0.998_
```

### Standard deviations of margin

The standard normal quantile is computed (SciPy), not read from Fig. 6.20 or Table H.1.

```python {description="Standard deviations of margin needed: k = Φ⁻¹(R)"}
from scipy.stats import norm

k = float(norm.ppf(float(R)))
```

### Allowable mean stress

```calc
> Standard deviation of the margin, Juvinall Eq. 6.15
s_z = hypot(s_S, s_σ) = ?? N·m
> Mean margin needed, Sample Problem 6.4
μ_z = k s_z = ?? N·m
> Largest mean stress, Eq. 6.14
μ_σ = μ_S - μ_z = ?? N·m
```
