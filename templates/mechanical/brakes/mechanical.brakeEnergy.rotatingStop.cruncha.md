+++
cruncha-template = 1
id = "mechanical.brakeEnergy.rotatingStop"
version = "1.0.0"
title = "Brake energy and temperature rise, stopping a rotating inertia"
summary = "Stopping (or slowing) a rotating inertia with a constant brake torque: the energy absorbed, the stop time and revolutions, the average and peak braking power, and the temperature rise of the parts that store the heat against the lining's limit."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "brake energy", "kinetic energy", "stopping time", "braking power", "temperature rise", "heat", "flywheel", "Juvinall", "Shigley"]
from = "lacewing:mechanical.brakeEnergy"
license = "GPL-3.0-or-later"
inputs = ["I", "n_0", "n_f", "T", "m", "c_p", "T_amb", "T_lim"]
outputs = ["E", "t", "N_rev", "P_avg", "P_peak", "ΔT", "T_end"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.4: Eq. 18.11 (rotating kinetic energy); Table 18.1; Probs. 18.15 and 18.22"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 16-8 and 16-9: Eqs. 16-55 (temperature rise) and 16-58; Table 16-3"

[[tests]]
name = "Juvinall Prob. 18.15: 6.5 N·m·s² from 600 rpm on a 212 N·m brake"
source = "Lacewing Lab help for Brake energy (the rotating case opens on Juvinall & Marshek 5th Prob. 18.15, answer 1.93 s); E = ½ × 6.5 × 62.831853² = 12830.486 J by hand; 20 kg of steel at 500 J/(kg·K) rises 1.2830486 K"
inputs = { I = "6.5 kg·m²", n_0 = "600 rpm", n_f = "0 rpm", T = "212 N·m", m = "20 kg", c_p = "500 J/(kg·K)", T_amb = "20 °C", T_lim = "260 °C" }
expect = { t = "1.93 s", E = "12830.486 J", "ΔT" = "1.2830486 ΔK" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Juvinall Prob. 18.22: 236 N·m, 4 s, 110 turns"
source = "Lacewing Lab brakes_test, after Juvinall & Marshek 5th Prob. 18.22 (answer): 110 turns in 4 s at uniform deceleration give ω0 = 2 × 110 × 2π/4 = 345.575 rad/s (3300 rpm), I = 236 × 4/345.575 = 2.731678 kg·m²; 163 kJ at 41 kW average. Stored in only 1 kg of steel the heat would raise it 326 K, past a 260 °C lining"
inputs = { I = "2.731678 kg·m²", n_0 = "3300 rpm", n_f = "0 rpm", T = "236 N·m", m = "1 kg", c_p = "500 J/(kg·K)", T_amb = "20 °C", T_lim = "260 °C" }
expect = { t = "4 s", N_rev = "110", E = "163 kJ", P_avg = "41 kW" }
tolerance = "0.6%"
checks = "fail"

[[tests]]
name = "Hand check: Prob. 18.15 unrounded"
source = "By hand: ω0 = 62.831853 rad/s, t = 6.5 × 62.831853/212 = 1.9264483 s, revolutions = 62.831853 × 1.9264483/(4π) = 9.6322416, P avg = 12830.486/1.9264483 = 6660.1764 W, P peak = 212 × 62.831853 = 13320.353 W"
inputs = { I = "6.5 kg·m²", n_0 = "600 rpm", n_f = "0 rpm", T = "212 N·m", m = "20 kg", c_p = "500 J/(kg·K)", T_amb = "20 °C", T_lim = "260 °C" }
expect = { t = "1.9264483 s", N_rev = "9.6322416", P_avg = "6660.1764 W", P_peak = "13320.353 W", T_end = "21.283049 °C" }
tolerance = "1e-7"
checks = "pass"
+++

## Stopping a rotating inertia

A brake that stops (or slows) a rotating inertia I, referred to the brake shaft, from n_0 to n_f with a constant torque T, so the deceleration is uniform (Juvinall Sec. 18.4). The energy turned into heat is the change of kinetic energy, ½ I (ω_0² − ω_f²) (Eq. 18.11, Shigley Eq. 16-58). Heat is made fastest at the start of the stop.

For a short stop the heat is taken as stored in the mass m of the parts in contact with the lining, none lost while it is made: ΔT = E/(m c_p) (Shigley Eq. 16-55). Shigley suggests c_p = 500 J/(kg·K) for steel or cast iron. For repeated stops, use the repeated-stops template, which adds the cooling between them.

### Inputs

```calc {description="Mass moment of inertia, referred to the brake shaft"}
I = 6.5 kg·m²_
```

```calc {description="Speed at the start of the stop"}
n_0 = 600 rpm_
```

```calc {description="Speed at the end of braking (0 to rest)"}
n_f = 0 rpm_
```

```calc {description="Constant brake torque"}
T = 212 N·m_
```

```calc {description="Mass of the parts that store the heat (drum, disk, plates)"}
m = 20 kg_
```

```calc {description="Specific heat of those parts"}
c_p = 500 J/(kg·K)_
```

```calc {description="Temperature of the parts before the stop"}
T_amb = 20 °C_
```

```calc {description="Largest temperature of the lining, Juvinall Table 18.1 or Shigley Table 16-3"}
T_lim = 260 °C_
```

### Energy, time and power

```calc
> Energy absorbed, Eq. 18.11
E = I (n_0^2 - n_f^2) / 2 = ?? J
> Stop time at constant torque
t = I (n_0 - n_f) / T = ?? s
> Revolutions made while braking
N_rev = (n_0 + n_f) t / 2 / (1 'rev') = ??
> Average braking power
P_avg = E / t = ?? W
> Peak braking power, at the start of the stop
P_peak = T n_0 = ?? W
```

### Temperature

```calc
> Temperature rise of the parts, Eq. 16-55
ΔT = E / (m c_p) = ?? ΔK
> Temperature at the end of the stop
T_end = T_amb + ΔT = ?? °C
```

### Checks

```calc
@test T_end ≤ T_lim
```
