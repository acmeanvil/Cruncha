+++
cruncha-template = 1
id = "hvac.psychrometrics.moistAir"
version = "1.0.0"
title = "Moist air from dry-bulb temperature and relative humidity"
summary = "Saturation and partial vapor pressure, humidity ratio, enthalpy, specific volume and dew point of moist air at a given barometric pressure."
category = ["HVAC", "Psychrometrics"]
tags = ["psychrometrics", "moist air", "humidity ratio", "relative humidity", "enthalpy", "dew point", "specific volume", "saturation pressure", "ASHRAE"]
license = "GPL-3.0-or-later"
inputs = ["T_db", "φ", "p"]
outputs = ["p_ws", "p_w", "W", "h", "v", "T_dp"]

[[references]]
work = "ASHRAE Handbook — Fundamentals"
edition = "2017"
locator = "Ch. 1, Psychrometrics: saturation pressure over water (Eq. 6, after Hyland and Wexler), humidity ratio, enthalpy, specific volume and dew point (Peppers) equations; Table 3"

[[references]]
work = "Hyland, R.W., and A. Wexler, Formulations for the thermodynamic properties of the saturated phases of H2O from 173.15 K to 473.15 K, ASHRAE Transactions 89(2A)"
edition = "1983"

[[tests]]
name = "Saturation pressure at 20 °C"
source = "ASHRAE Handbook — Fundamentals 2017, Ch. 1 Table 3: 2.3393 kPa at 20 °C"
inputs = { T_db = "20 °C", "φ" = "1", p = "101.325 kPa" }
expect = { p_ws = "2.3393 kPa" }
tolerance = "0.05%"

[[tests]]
name = "Saturation pressure at 40 °C"
source = "ASHRAE Handbook — Fundamentals 2017, Ch. 1 Table 3: 7.3851 kPa at 40 °C"
inputs = { T_db = "40 °C", "φ" = "1", p = "101.325 kPa" }
expect = { p_ws = "7.3851 kPa" }
tolerance = "0.05%"

[[tests]]
name = "25 °C, 50 % at sea level"
source = "By hand with the Handbook's equations; the psychrometric chart reads W ≈ 0.0099, h ≈ 50.3 kJ/kg, dew point ≈ 13.9 °C"
inputs = { T_db = "25 °C", "φ" = "0.5", p = "101.325 kPa" }
expect = { W = "0.0098810", h = "50.322 kJ/kg", v = "0.85804 m^3/kg", T_dp = "13.893 °C" }
tolerance = "0.01%"

[[tests]]
name = "20 °C, 60 % at 84 kPa (about 1600 m)"
source = "By hand with the Handbook's equations: lower pressure raises W for the same relative humidity"
inputs = { T_db = "20 °C", "φ" = "0.6", p = "84 kPa" }
expect = { W = "0.010567", h = "46.940 kJ/kg", v = "1.01876 m^3/kg" }
tolerance = "0.01%"
+++

## Moist air from dry-bulb temperature and relative humidity

Properties of moist air treated as an ideal-gas mixture of dry air and water vapor, from the dry-bulb temperature, the relative humidity and the barometric pressure. The saturation pressure is over liquid water, which holds from 0 to 200 °C; below 0 °C use the formulation over ice.

Enthalpy is per kilogram of **dry** air, from 0 °C dry air and 0 °C liquid water, as on the ASHRAE chart. The dew-point equation fits 0 to 93 °C.

### Air

```calc {description="Dry-bulb temperature"}
T_db = 25 °C_
```

```calc {description="Relative humidity, a fraction (0.5 for 50 %)"}
φ = 0.5_
```

```calc {description="Barometric pressure (101.325 kPa at sea level)"}
p = 101.325 kPa_
```

```calc
@test 0 ≤ φ ≤ 1
```

### Water vapor

```calc
> Absolute temperature as a number of kelvins
θ = T_db / (1 'K') = ??
> Saturation pressure over liquid water, Handbook Eq. 6
p_ws = exp(-5.8002206e3 / θ + 1.3914993 - 4.8640239e-2 θ + 4.1764768e-5 θ^2 - 1.4452093e-8 θ^3 + 6.5459673 ln(θ)) 'Pa' = ?? kPa
> Partial pressure of the water vapor
p_w = φ p_ws = ?? kPa
```

### Mixture

```calc {description="Humidity ratio, kg of water vapor per kg of dry air"}
W = 0.621945 p_w / (p - p_w) = ??
```

```calc
> Dry-bulb temperature as a number of degrees Celsius
t = θ - 273.15 = ??
> Specific enthalpy per kg of dry air
h = (1.006 t + W (2501 + 1.86 t)) 'kJ/kg' = ?? kJ/kg
> Specific volume per kg of dry air
v = 0.287042 θ (1 + 1.607858 W) 'm³/kg' / (p / (1 'kPa')) = ?? m³/kg
```

### Dew point

```calc
> ln of the vapor pressure in kPa
α = ln(p_w / (1 'kPa')) = ??
> Dew-point temperature in °C, 0 to 93 °C
t_d = 6.54 + 14.526 α + 0.7389 α^2 + 0.09486 α^3 + 0.4569 (p_w / (1 'kPa'))^0.1984 = ??
> Dew-point temperature
T_dp = (t_d + 273.15) 'K' = ?? °C
```
