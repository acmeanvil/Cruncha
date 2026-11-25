+++
cruncha-template = 1
id = "mechanical.lubricantViscosity.viscosityIndex"
version = "1.0.0"
title = "Viscosity index of an oil (ASTM D2270)"
summary = "The viscosity index of a petroleum oil from its kinematic viscosities at 40 °C and 100 °C, by ASTM D2270: procedure A for an index of 100 or less, procedure B (the logarithmic form) above 100, with the reference viscosities L and H from the standard's table."
category = ["Mechanical engineering", "Bearings"]
tags = ["viscosity index", "VI", "ASTM D2270", "ISO 2909", "kinematic viscosity", "lubricant", "oil", "viscosity-temperature"]
from = "lacewing:mechanical.lubricantViscosity"
license = "GPL-3.0-or-later"
inputs = ["U", "Y", "L_r", "H_r"]
outputs = ["VI"]

[[references]]
work = "ASTM D2270, Standard Practice for Calculating Viscosity Index from Kinematic Viscosity at 40 °C and 100 °C (also ISO 2909)"
edition = "current"
locator = "Procedure A (VI up to 100) and procedure B (VI above 100); Table 1 (L and H against the 100 °C viscosity); the calculation examples"

[[tests]]
name = "Procedure A: 73.30 mm²/s at 40 °C, 8.86 mm²/s at 100 °C"
source = "ASTM D2270's example of procedure A, checked by hand: L = 119.94 and H = 69.48 mm²/s at 8.86 mm²/s (Table 1, interpolated); VI = 100 (119.94 − 73.30)/(119.94 − 69.48) = 92.4296, reported as 92"
inputs = { U = "73.30 mm²/s", Y = "8.86 mm²/s", L_r = "119.94 mm²/s", H_r = "69.48 mm²/s" }
expect = { VI = "92.4296472" }
tolerance = "1e-6"

[[tests]]
name = "Procedure B: 22.83 mm²/s at 40 °C, 5.05 mm²/s at 100 °C"
source = "ASTM D2270's example of procedure B, checked by hand: H = 28.97 mm²/s at 5.05 mm²/s (Table 1); N = (log 28.97 − log 22.83)/log 5.05 = 0.1470835, VI = (10^N − 1)/0.00715 + 100 = 156.375, reported as 156. Procedure B does not use L"
inputs = { U = "22.83 mm²/s", Y = "5.05 mm²/s", H_r = "28.97 mm²/s" }
expect = { VI = "156.375324" }
tolerance = "1e-6"
+++

## Viscosity index of an oil (ASTM D2270)

The viscosity index measures how little an oil's viscosity changes with temperature: the higher the index, the flatter its curve. ASTM D2270 (ISO 2909) compares the oil's kinematic viscosity at 40 °C, U, with two reference oils that have the same viscosity at 100 °C, Y: L, of index 0, and H, of index 100. Up to an index of 100 the index is linear between them (procedure A); above 100 a logarithmic form takes over (procedure B), which needs only H.

L and H come from the standard's Table 1 against Y, interpolated linearly between its rows; for Y above 70 mm²/s the standard gives them by equations instead. The table is not reproduced here. Both viscosities should be measured, not read off an equation, and Y must be at least 2 mm²/s.

```calc {description="Kinematic viscosity of the oil at 40 °C"}
U = 73.30 mm²/s_
```

```calc {description="Kinematic viscosity of the oil at 100 °C"}
Y = 8.86 mm²/s_
```

```calc {description="L: 40 °C viscosity of the index-0 oil with the same 100 °C viscosity, ASTM D2270 Table 1"}
L_r = 119.94 mm²/s_
```

```calc {description="H: 40 °C viscosity of the index-100 oil with the same 100 °C viscosity, ASTM D2270 Table 1"}
H_r = 69.48 mm²/s_
```

```calc
> Exponent of procedure B
N_B = log10(H_r / U) / log10(Y / (1 'mm²/s')) = ??
> Viscosity index: procedure A when U is at least H, procedure B otherwise
VI = {100 (L_r - U) / (L_r - H_r) if U ≥ H_r; (10^N_B - 1) / 0.00715 + 100 otherwise} = ??
```
