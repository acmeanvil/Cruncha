+++
cruncha-template = 1
id = "mechanical.brakeEnergy.clutchEngagement"
version = "1.0.0"
title = "Clutch engagement: slip time, heat and temperature rise (Shigley)"
summary = "A friction clutch slipping at constant torque until a driving inertia (or a driver held at constant speed) and a driven inertia reach a common speed: the slip time, the common speed, the heat made in the clutch and the temperature rise of the parts that store it."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["clutch", "engagement", "slip", "slip time", "heat", "energy", "temperature rise", "inertia", "Shigley", "Juvinall"]
from = "lacewing:mechanical.brakeEnergy"
license = "GPL-3.0-or-later"
inputs = ["driver", "I_1", "I_2", "n_1", "n_2", "T", "m", "c_p", "T_amb", "T_lim"]
outputs = ["t_1", "n", "E", "ΔKE", "ΔT", "T_end"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 16-8 and 16-9: Fig. 16-1a, Eqs. 16-50 to 16-52 (slip time, heat), 16-55 (temperature rise); Table 16-3"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.4; Probs. 18.18 to 18.20; Table 18.1"

[[tests]]
name = "Juvinall Prob. 18.19: 0.7 kg·m² brought to 600 rpm by 6 N·m from a constant-speed drive"
source = "Lacewing Lab brakes_test and help, after Juvinall & Marshek 5th Prob. 18.19: 7.33 s, 1382 J delivered to the driven machine and 1382 J of heat in the clutch"
inputs = { driver = "2", I_1 = "1 kg·m²", I_2 = "0.7 kg·m²", n_1 = "600 rpm", n_2 = "0 rpm", T = "6 N·m", m = "5 kg", c_p = "500 J/(kg·K)", T_amb = "20 °C", T_lim = "260 °C" }
expect = { t_1 = "7.33 s", "ΔKE" = "1381.7 J", E = "1381.7 J", n = "600 rpm" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: equal inertias lose half the relative kinetic energy"
source = "Lacewing Lab brakes_test (Shigley Eqs. 16-51, 16-52): I1 = I2 = 2 kg·m², 100 and 20 rad/s, 50 N·m: t1 = 2 × 2 × 80/(50 × 4) = 1.6 s, common speed 60 rad/s, E = 2 × 2 × 80²/(2 × 4) = 3200 J; into 10 kg at 500 J/(kg·K), 0.64 K"
inputs = { driver = "1", I_1 = "2 kg·m²", I_2 = "2 kg·m²", n_1 = "100 rad/s", n_2 = "20 rad/s", T = "50 N·m", m = "10 kg", c_p = "500 J/(kg·K)", T_amb = "20 °C", T_lim = "260 °C" }
expect = { t_1 = "1.6 s", n = "60 rad/s", E = "3200 J", "ΔT" = "0.64 ΔK" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: a heavy engagement into a light clutch"
source = "By hand: I1 = 50, I2 = 30 kg·m² from 3000 rpm to rest: E = 50 × 30 × 314.15927²/(2 × 80) = 925275.1 J; 2 kg at 500 J/(kg·K) rises 925.3 K, far past a 260 °C lining"
inputs = { driver = "1", I_1 = "50 kg·m²", I_2 = "30 kg·m²", n_1 = "3000 rpm", n_2 = "0 rpm", T = "500 N·m", m = "2 kg", c_p = "500 J/(kg·K)", T_amb = "20 °C", T_lim = "260 °C" }
expect = { E = "925275.1 J" }
tolerance = "1e-6"
checks = "fail"
+++

## Clutch engagement

A friction clutch joining a driving side of inertia I_1 turning at n_1 to a driven side of inertia I_2 at n_2 (Shigley Sec. 16-8, Fig. 16-1a). While it slips it carries a constant torque T, which slows the driver and speeds the driven side until they reach a common speed. Enter the driver as a number: 1 for an inertia I_1 that slows, 2 for a driver held at constant speed (an engine or motor that keeps its speed, the limit as I_1 grows without bound; I_1 is then ignored).

The heat made in the clutch is the kinetic energy lost, I_1 I_2 (ω_1 − ω_2)²/(2 (I_1 + I_2)) (Eq. 16-52): it does not depend on the torque, which sets only how long the slip lasts. With a constant-speed driver it is ½ I_2 (ω_1 − ω_2)², as much again as the driven side gains when it starts from rest. The temperature rise lumps the heat into the mass m of the parts in contact with the lining (Eq. 16-55); about 500 J/(kg·K) for steel or cast iron.

### Inputs

```calc {description="Driver: 1 an inertia I_1, 2 held at constant speed"}
driver = 2_
```

```calc {description="Inertia of the driving side (ignored for a constant-speed driver)"}
I_1 = 1 kg·m²_
```

```calc {description="Inertia of the driven side"}
I_2 = 0.7 kg·m²_
```

```calc {description="Speed of the driving side at engagement"}
n_1 = 600 rpm_
```

```calc {description="Speed of the driven side at engagement"}
n_2 = 0 rpm_
```

```calc {description="Torque the clutch carries while it slips"}
T = 6 N·m_
```

```calc {description="Mass of the parts that store the heat (plates, pressure plate)"}
m = 5 kg_
```

```calc {description="Specific heat of those parts"}
c_p = 500 J/(kg·K)_
```

```calc {description="Temperature of the parts before the engagement"}
T_amb = 20 °C_
```

```calc {description="Largest temperature of the lining, Juvinall Table 18.1 or Shigley Table 16-3"}
T_lim = 260 °C_
```

### Slip and heat

```calc
> Slip time, Eq. 16-51
t_1 = {I_1 I_2 (n_1 - n_2) / (T (I_1 + I_2)) if driver == 1; I_2 (n_1 - n_2) / T if driver == 2} = ?? s
> Common speed at the end of slipping
n = {(I_1 n_1 + I_2 n_2) / (I_1 + I_2) if driver == 1; n_1 if driver == 2} = ?? rpm
> Heat made in the clutch, Eq. 16-52
E = {I_1 I_2 (n_1 - n_2)^2 / (2 (I_1 + I_2)) if driver == 1; I_2 (n_1 - n_2)^2 / 2 if driver == 2} = ?? J
> Kinetic energy the driven side gains
ΔKE = I_2 (n^2 - n_2^2) / 2 = ?? J
```

### Temperature

```calc
> Temperature rise of the parts, Eq. 16-55
ΔT = E / (m c_p) = ?? ΔK
> Temperature at the end of the engagement
T_end = T_amb + ΔT = ?? °C
```

### Checks

```calc
@test n_1 > n_2
```

```calc
@test T_end ≤ T_lim
```
