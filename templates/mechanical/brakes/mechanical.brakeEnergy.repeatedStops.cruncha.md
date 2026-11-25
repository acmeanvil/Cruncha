+++
cruncha-template = 1
id = "mechanical.brakeEnergy.repeatedStops"
version = "1.0.0"
title = "Brake temperatures under repeated stops (Shigley)"
summary = "A brake stopping a rotating inertia again and again at a fixed interval, its parts cooling by Newton's law between stops: the energy and temperature rise of one stop, the overall heat-transfer coefficient, the cooling rate, the steady-state largest and smallest temperatures, the average heat rate and the mean surface temperature that sheds it."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "clutch", "temperature rise", "heat", "cooling", "repeated stops", "Newtonian cooling", "heat dissipation", "Shigley", "Juvinall"]
from = "lacewing:mechanical.brakeEnergy"
license = "GPL-3.0-or-later"
inputs = ["I", "n_0", "t_c", "h_r", "h_c", "f_v", "A", "m", "c_p", "T_amb", "T_lim"]
outputs = ["E", "h_CR", "ΔT", "β", "T_max", "T_min", "H", "T_s"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-9: Eqs. 16-55 to 16-60, Fig. 16-24 (heat-transfer coefficients, ventilation factor), Table 16-3 (lining temperatures); Example 16-5"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.4: Eq. 18.9 (heat-dissipation balance), Table 18.1"

[[tests]]
name = "Shigley Ex. 16-5: 289 lbf·in·s² from 250 rpm, 24 stops an hour"
source = "Lacewing Lab brakes_test, after Shigley 10th Example 16-5: hr 3.0e-6 and hc 2.0e-6 Btu/(in²·s·°F) from Fig. 16-24a, fv 4.8 from Fig. 16-24b, A 50 in², 1.99 lbm of cast iron at 0.108 Btu/(lbm·°F), 70 °F ambient: E 10.6 Btu, ΔT 49.3 °F, β 2.93e-3 1/s, Tmax 209 °F, Tmin 160 °F (the book takes 209 − 49.3 from the rounded Tmax)"
inputs = { I = "289 lbf·in·s²", n_0 = "250 rpm", t_c = "150 s", h_r = "3.0e-6 Btu/(in²·s·Δ°F)", h_c = "2.0e-6 Btu/(in²·s·Δ°F)", f_v = "4.8", A = "50 in²", m = "1.99 lbm", c_p = "0.108 Btu/(lbm·Δ°F)", T_amb = "70 °F", T_lim = "350 °F" }
expect = { E = "10.6 Btu", "ΔT" = "49.3 Δ°F", "β" = "2.93e-3 1/s", T_max = "209 °F", T_min = "159.4 °F" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 16-5 unrounded, with a 200 °F lining limit"
source = "By hand: E = ½ × 32.6526 N·m·s² × 26.17994² = 11189.87 J = 10.605954 Btu; ΔT = 49.348379 °F; h̄CR = 12.6e-6 Btu/(in²·s·°F) = 37.089513 W/(m²·K); β = 2.9313233e-3 1/s; Tmax = 208.70890 °F, over 200 °F; H = 11189.87/150 = 74.599 W, Ts = 70 °F + 74.599/(37.089513 × 0.032258 m²) = 182.23231 °F"
inputs = { I = "289 lbf·in·s²", n_0 = "250 rpm", t_c = "150 s", h_r = "3.0e-6 Btu/(in²·s·Δ°F)", h_c = "2.0e-6 Btu/(in²·s·Δ°F)", f_v = "4.8", A = "50 in²", m = "1.99 lbm", c_p = "0.108 Btu/(lbm·Δ°F)", T_amb = "70 °F", T_lim = "200 °F" }
expect = { E = "10.605954 Btu", "ΔT" = "49.348379 Δ°F", h_CR = "37.089513 W/(m²·K)", "β" = "2.9313233e-3 1/s", T_max = "208.70890 °F", T_s = "182.23231 °F" }
tolerance = "1e-6"
checks = "fail"
+++

## Brake temperatures under repeated stops

A brake that stops an inertia I from the speed n_0 every t_c seconds (Shigley Sec. 16-9). Each stop is short, so its heat is taken as stored in the mass m of the parts in contact with the lining (the drum, disk or plates), raising them by ΔT = E/(m c_p) (Eq. 16-55). Between stops the parts cool by Newton's law toward the ambient T_amb, as exp(−β t) (Eq. 16-56). After enough cycles the temperature swings between a steady largest value just after a stop and a smallest one just before the next (Eq. 16-60).

The overall heat-transfer coefficient is radiation plus convection, h̄_CR = h_r + f_v h_c (Eq. 16-57): Shigley Fig. 16-24a gives h_r and h_c against the temperature rise and Fig. 16-24b the ventilation factor f_v against the rubbing speed. Because they depend on the temperature found, iterate if the result is far from the rise assumed (Example 16-5 assumes 200 °F and notes iteration gives about 220 °F). Shigley warns that so many variables are involved that the result shows which ones matter rather than predicting the temperature.

### Inputs

```calc {description="Mass moment of inertia stopped, referred to the brake shaft"}
I = 289 lbf·in·s²_
```

```calc {description="Speed at the start of each stop"}
n_0 = 250 rpm_
```

```calc {description="Time from one stop to the next"}
t_c = 150 s_
```

```calc {description="Radiation heat-transfer coefficient, Shigley Fig. 16-24a"}
h_r = 3.0e-6 Btu/(in²·s·Δ°F)_
```

```calc {description="Convection heat-transfer coefficient, Shigley Fig. 16-24a"}
h_c = 2.0e-6 Btu/(in²·s·Δ°F)_
```

```calc {description="Ventilation factor, Shigley Fig. 16-24b"}
f_v = 4.8_
```

```calc {description="Lateral area of the parts that shed the heat"}
A = 50 in²_
```

```calc {description="Mass of the parts that store the heat"}
m = 1.99 lbm_
```

```calc {description="Specific heat of those parts (about 500 J/(kg·K) for steel or cast iron)"}
c_p = 0.108 Btu/(lbm·Δ°F)_
```

```calc {description="Ambient temperature"}
T_amb = 70 °F_
```

```calc {description="Largest temperature of the lining, Juvinall Table 18.1 or Shigley Table 16-3"}
T_lim = 350 °F_
```

### One stop

```calc
> Energy of each stop, Eq. 16-58
E = I n_0^2 / 2 = ?? Btu
> Temperature rise of one stop, Eq. 16-59
ΔT = E / (m c_p) = ?? Δ°F
```

### Cooling and the steady state

```calc
> Overall heat-transfer coefficient, Eq. 16-57
h_CR = h_r + f_v h_c = ?? W/(m²·K)
> Cooling rate constant, Eq. 16-56
β = h_CR A / (m c_p) = ?? 1/s
> Largest temperature, just after a stop, Eq. 16-60
T_max = T_amb + ΔT / (1 - exp(-β t_c)) = ?? °F
> Smallest temperature, just before the next stop
T_min = T_max - ΔT = ?? °F
> Average heat rate the surfaces must shed
H = E / t_c = ?? W
> Mean surface temperature that sheds it, Juvinall Eq. 18.9
T_s = T_amb + H / (h_CR A) = ?? °F
```

### Checks

```calc
@test T_max ≤ T_lim
```
