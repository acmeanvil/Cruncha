+++
cruncha-template = 1
id = "mechanical.reliability.designFactor"
version = "1.0.0"
title = "Design factor for a reliability goal (normal)"
summary = "Shigley's design factor, mean strength over mean stress, that gives a reliability goal when strength and stress are normal with known coefficients of variation; checks that the strength's scatter alone can reach the goal."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["reliability", "design factor", "factor of safety", "coefficient of variation", "stress-strength interference", "normal distribution", "statistics", "Python"]
from = "lacewing:mechanical.reliability"
license = "GPL-3.0-or-later"
inputs = ["R", "C_S", "C_σ"]
outputs = ["z", "a_S", "n_d"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 1-13 (Eqs. 1-11, 1-12; Example 1-6); Table A-10"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 6.13-6.15 (the same interference of normal distributions)"

[[tests]]
name = "Shigley Ex. 1-6: CS 0.0753, Cσ 0.082, R = 0.999"
source = "Lacewing Lab reliability_test, after Shigley 10th Example 1-6: σ̂S/μS = 5.90/78.4 = 0.0753, Cσ = 0.082 (the load's), z = −3.09 (Table A-10), nd = 1.416"
inputs = { R = "0.999", C_S = "0.0752551", "C_σ" = "0.082" }
expect = { z = "-3.09", n_d = "1.416" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 1-6 to solver precision"
source = "An independent evaluation with scipy.stats.norm.ppf: z = Φ⁻¹(0.001) = −3.0902323; 1 − z²CS² = 0.9459178; nd = (1 + √(1 − 0.9459178 × (1 − z²Cσ²)))/0.9459178 = 1.4153998"
inputs = { R = "0.999", C_S = "0.0752551020408", "C_σ" = "0.082" }
expect = { z = "-3.09023231", a_S = "0.94591782", n_d = "1.41539975" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Hand check: no scatter in the stress"
source = "By hand: with Cσ = 0, Eq. 1-12 reduces to nd = 1/(1 + z CS) for z < 0; R = Φ(2) = 0.977250 gives z = −2, and CS = 0.1 gives nd = 1/(1 − 0.2) = 1.25"
inputs = { R = "0.97724986805182", C_S = "0.1", "C_σ" = "0" }
expect = { z = "-2", n_d = "1.25" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "R = 0.999999 with CS 0.25: the strength's scatter alone is too wide"
source = "Lacewing Lab reliability_test: z = −4.7534, 1 − z²CS² = 1 − 22.595 × 0.0625 = −0.4122 < 0, so no design factor exists"
inputs = { R = "0.999999", C_S = "0.25", "C_σ" = "0.1" }
expect = { a_S = "-0.41219017" }
tolerance = "1e-6"
checks = "fail"
+++

## Design factor for a reliability goal

The design factor n_d = μ_S/μ_σ (mean strength over mean stress) that gives reliability R, when strength and stress are normal and independent and their scatter is known as coefficients of variation: C_S = σ̂_S/μ_S for the strength, C_σ = σ̂_σ/μ_σ for the stress (for a stress proportional to a load, the load's). Shigley's Eq. 1-12 comes from setting the standard normal variable of the margin to the value z whose cumulative probability is 1 − R.

The goal must be over 0.5 (the formula takes the root for a positive margin). If 1 − z²C_S² is not positive, the strength scatters so widely that no mean stress, however small, reaches the goal; the check below then fails.

### Goal and scatter

```calc {description="Reliability goal: the share of parts that must survive"}
R = 0.999_
```

```calc {description="Coefficient of variation of the strength, σ̂S/μS"}
C_S = 0.0752551_
```

```calc {description="Coefficient of variation of the stress, σ̂σ/μσ"}
C_σ = 0.082_
```

### Standard normal variable

The standard normal quantile is computed (SciPy), not read from Table A-10.

```python {description="Standard normal variable for the failure probability: z = Φ⁻¹(1 − R), negative for R > 0.5"}
from scipy.stats import norm

z = float(norm.ppf(1 - float(R)))
```

### Design factor

```calc
> Strength term of Eq. 1-12; it must be positive
a_S = 1 - z^2 C_S^2 = ??
> Design factor for the reliability goal, Shigley Eq. 1-12
n_d = (1 + sqrt(1 - a_S (1 - z^2 C_σ^2))) / a_S = ??
```

### Checks

```calc
@test R > 0.5
```

```calc
@test a_S > 0
```
