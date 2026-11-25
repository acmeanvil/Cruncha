+++
cruncha-template = 1
id = "mechanical.lubricantViscosity.walther"
version = "1.0.0"
title = "Oil viscosity through two points, Walther (ASTM D341)"
summary = "The kinematic viscosity of a petroleum oil at any temperature from two measured kinematic viscosities (a data sheet's 40 °C and 100 °C), by Walther's equation log log(ν + 0.7) = A − B log T as ASTM D341 uses it, with the constants A and B."
category = ["Mechanical engineering", "Bearings"]
tags = ["viscosity", "kinematic viscosity", "viscosity-temperature", "Walther", "ASTM D341", "lubricant", "oil", "two-point", "extrapolation", "data sheet"]
from = "lacewing:mechanical.lubricantViscosity"
license = "GPL-3.0-or-later"
inputs = ["T_1", "ν_1", "T_2", "ν_2", "T"]
outputs = ["A", "B", "ν"]

[[references]]
work = "ASTM D341, Standard Practice for Viscosity-Temperature Equations and Charts for Liquid Petroleum or Hydrocarbon Products"
edition = "current (Walther's equation with Z = ν + 0.7)"
locator = "The viscosity-temperature equation, log log Z = A − B log T, T in kelvins, ν in mm²/s"

[[references]]
work = "Hamrock, Schmid and Jacobson, Fundamentals of Fluid Film Lubrication (NASA RP-1255)"
edition = "1991"
locator = "Sec. 4.5, viscosity-temperature equations (Walther's equation)"

[[tests]]
name = "Hand check: an oil of 32 mm²/s at 40 °C and 5.4 mm²/s at 100 °C, at 60 °C"
source = "By hand: W = log log(ν + 0.7); W1 = 0.1802830, W2 = −0.1525574; B = (W1 − W2)/(log 373.15 − log 313.15) = 3.7465784; A = W1 + B log 313.15 = 9.5308151; at 333.15 K, ν = 10^10^(A − B log T) − 0.7 = 15.185893 mm²/s"
inputs = { T_1 = "40 °C", "ν_1" = "32 mm²/s", T_2 = "100 °C", "ν_2" = "5.4 mm²/s", T = "60 °C" }
expect = { A = "9.53081512", B = "3.74657842", "ν" = "15.1858928 mm²/s" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: the line passes through its first point"
source = "By hand: at T = T1 the equation returns ν1"
inputs = { T_1 = "40 °C", "ν_1" = "32 mm²/s", T_2 = "100 °C", "ν_2" = "5.4 mm²/s", T = "40 °C" }
expect = { "ν" = "32 mm²/s" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "The same oil at 170 °C: below 2 mm²/s, outside the simple form"
source = "By hand: at 443.15 K, ν = 10^10^(9.5308151 − 3.7465784 × 2.6465508) − 0.7 = 1.884563 mm²/s, under the 2 mm²/s down to which Z = ν + 0.7 holds"
inputs = { T_1 = "40 °C", "ν_1" = "32 mm²/s", T_2 = "100 °C", "ν_2" = "5.4 mm²/s", T = "170 °C" }
expect = { "ν" = "1.88456344 mm²/s" }
tolerance = "1e-6"
checks = "fail"
+++

## Oil viscosity through two points, Walther (ASTM D341)

Walther's equation makes the viscosity-temperature curve of a petroleum oil a straight line on ASTM D341's chart: log log(ν + 0.7) against log T, with ν the kinematic viscosity in mm²/s (cSt) and T the absolute temperature. Two measured points, usually a data sheet's 40 °C and 100 °C viscosities, fix the line, and the equation gives the viscosity at other temperatures. It is the usual way to interpolate, and to extrapolate modestly, an oil's viscosity. In the simple form used here, Z = ν + 0.7, it holds down to about 2 mm²/s; later editions of D341 add small correction terms below that.

For the absolute viscosity a bearing analysis needs, multiply by the density at the temperature.

```calc {description="First temperature of measurement"}
T_1 = 40 °C_
```

```calc {description="Kinematic viscosity at the first temperature"}
ν_1 = 32 mm²/s_
```

```calc {description="Second temperature of measurement"}
T_2 = 100 °C_
```

```calc {description="Kinematic viscosity at the second temperature"}
ν_2 = 5.4 mm²/s_
```

```calc {description="Temperature at which the viscosity is wanted"}
T = 60 °C_
```

```calc
> Walther's double logarithm at the first point
W_1 = log10(log10(ν_1 / (1 'mm²/s') + 0.7)) = ??
> At the second point
W_2 = log10(log10(ν_2 / (1 'mm²/s') + 0.7)) = ??
> Slope of the line
B = (W_1 - W_2) / (log10(T_2 / (1 'K')) - log10(T_1 / (1 'K'))) = ??
> Its intercept
A = W_1 + B log10(T_1 / (1 'K')) = ??
> Kinematic viscosity at T, ASTM D341
ν = (10^(10^(A - B log10(T / (1 'K')))) - 0.7) × 1 'mm²/s' = ?? mm²/s
```

```calc
@test ν ≥ 2 'mm²/s'
```
