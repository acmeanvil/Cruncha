+++
cruncha-template = 1
id = "measurement.uncertainty.productOfPowers"
version = "1.0.0"
title = "Measurement uncertainty: density of a cylinder (GUM, first order)"
summary = "Combined and expanded standard uncertainty of a result that is a product of powers of measured quantities, worked for a cylinder's density from its mass, diameter and height."
category = ["Measurement", "Uncertainty"]
tags = ["uncertainty", "GUM", "propagation", "combined standard uncertainty", "expanded uncertainty", "coverage factor", "density", "metrology"]
license = "GPL-3.0-or-later"
inputs = ["m", "u_m", "d", "u_d", "h", "u_h", "k"]
outputs = ["ρ", "u_c", "U"]

[[references]]
work = "JCGM 100:2008, Evaluation of measurement data — Guide to the expression of uncertainty in measurement (GUM)"
edition = "2008"
locator = "Sec. 5.1.2 (Eq. 10, law of propagation), 5.1.6 (Eq. 12, product of powers), 6.2 and 6.3 (expanded uncertainty)"
url = "https://www.bipm.org/en/committees/jc/jcgm/publications"

[[tests]]
name = "100 g, 20 mm × 40 mm cylinder"
source = "By hand, GUM Eq. 12: relative uncertainties 0.1/100, 2 × 0.02/20 and 0.05/40 combine to 0.256 %"
inputs = { m = "100 g", u_m = "0.1 g", d = "20 mm", u_d = "0.02 mm", h = "40 mm", u_h = "0.05 mm", k = "2" }
expect = { "ρ" = "7957.747 kg/m^3", u_c = "20.3857 kg/m^3", U = "40.7713 kg/m^3" }
tolerance = "1e-5"

[[tests]]
name = "250 g, 30 mm × 45 mm cylinder, k = 2"
source = "By hand, GUM Eq. 12: 0.5/250, 2 × 0.05/30 and 0.1/45 combine to 0.448 %"
inputs = { m = "250 g", u_m = "0.5 g", d = "30 mm", u_d = "0.05 mm", h = "45 mm", u_h = "0.1 mm", k = "2" }
expect = { "ρ" = "7859.503 kg/m^3", u_c = "35.1921 kg/m^3", U = "70.3843 kg/m^3" }
tolerance = "1e-5"
+++

## Measurement uncertainty: density of a cylinder

The density of a solid cylinder from its measured mass, diameter and height, ρ = 4m / (π d² h), with the uncertainty that the three measurements carry into it.

The model is a product of powers (m¹ d⁻² h⁻¹), so the relative standard uncertainties combine in quadrature, each weighted by its exponent (GUM 5.1.6). That holds when the inputs are uncorrelated and their uncertainties small next to their values; otherwise use the full law of propagation (GUM 5.1.2, 5.2) or a Monte Carlo evaluation.

Each u is a **standard** uncertainty: a Type A standard deviation of the mean, or a Type B estimate (for a reading limited to ±a with a rectangular distribution, a/√3).

### Measurements

```calc {description="Measured mass"}
m = 100 g_
```

```calc {description="Standard uncertainty of the mass"}
u_m = 0.1 g_
```

```calc {description="Measured diameter"}
d = 20 mm_
```

```calc {description="Standard uncertainty of the diameter"}
u_d = 0.02 mm_
```

```calc {description="Measured height"}
h = 40 mm_
```

```calc {description="Standard uncertainty of the height"}
u_h = 0.05 mm_
```

```calc {description="Coverage factor (2 for about 95 % with a normal distribution)"}
k = 2_
```

### Result

```calc {description="Density, the measurement model"}
ρ = 4 m / (π d^2 h) = ?? kg/m³
```

```calc
> Relative combined standard uncertainty, GUM Eq. 12 with exponents 1, −2, −1
w_c = sqrt((u_m / m)^2 + (2 u_d / d)^2 + (u_h / h)^2) = ??
> Combined standard uncertainty
u_c = w_c ρ = ?? kg/m³
> Expanded uncertainty, GUM 6.2
U = k u_c = ?? kg/m³
```

Report the result as ρ ± U with k stated, rounding U to two significant figures and ρ to the same decimal place.
