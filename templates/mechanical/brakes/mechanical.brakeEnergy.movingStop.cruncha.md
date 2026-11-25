+++
cruncha-template = 1
id = "mechanical.brakeEnergy.movingStop"
version = "1.0.0"
title = "Brake energy and temperature rise, stopping a moving mass"
summary = "Stopping a moving mass (a vehicle, or a load lowered on a hoist) with a vertical drop during the stop: kinetic and potential energy, the share one brake absorbs, the deceleration, stopping distance, average and peak braking power, and the temperature rise of the parts that store the heat."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "brake energy", "kinetic energy", "potential energy", "vehicle", "hoist", "stopping distance", "deceleration", "braking power", "temperature rise", "Juvinall", "Shigley"]
from = "lacewing:mechanical.brakeEnergy"
license = "GPL-3.0-or-later"
inputs = ["M", "v_0", "v_f", "h", "s_b", "t", "m", "c_p", "T_amb", "T_lim"]
outputs = ["KE", "PE", "E", "a", "a_g", "d", "P_avg", "P_peak", "ΔT", "T_end"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.4: Eqs. 18.10 (kinetic energy) and 18.12 (potential energy); Table 18.1"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-9: Eq. 16-55 (temperature rise); Table 16-3"

[[tests]]
name = "Hand check: 1000 kg from 20 m/s in 4 s, dropping 5 m, half the energy in this brake"
source = "Lacewing Lab brakes_test (a moving mass): KE = ½ × 1000 × 20² = 200 kJ, PE = 1000 × 9.80665 × 5 = 49033.25 J, E = 0.5 × 249033.25 = 124516.625 J, a = 5 m/s², d = 40 m; by hand P avg = 31129.156 W, P peak = 124516.625/40 × 20 = 62258.313 W; 50 kg at 460 J/(kg·K) rises 5.4137663 K"
inputs = { M = "1000 kg", v_0 = "20 m/s", v_f = "0 m/s", h = "5 m", s_b = "0.5", t = "4 s", m = "50 kg", c_p = "460 J/(kg·K)", T_amb = "20 °C", T_lim = "260 °C" }
expect = { KE = "200000 J", PE = "49033.25 J", E = "124516.625 J", a = "5 m/s²", d = "40 m", P_avg = "31129.156 W", P_peak = "62258.313 W", "ΔT" = "5.4137663 ΔK" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Hand check: the same stop into 2 kg of brake parts"
source = "By hand: ΔT = 124516.625/(2 × 460) = 135.34416 K, so 155.34 °C at the end, over a 150 °C limit"
inputs = { M = "1000 kg", v_0 = "20 m/s", v_f = "0 m/s", h = "5 m", s_b = "0.5", t = "4 s", m = "2 kg", c_p = "460 J/(kg·K)", T_amb = "20 °C", T_lim = "150 °C" }
expect = { "ΔT" = "135.34416 ΔK" }
tolerance = "1e-7"
checks = "fail"
+++

## Stopping a moving mass

A brake stopping (or slowing) a mass M moving at v_0 to v_f in the time t, at uniform deceleration, while the mass drops h (a vehicle going downhill, a load lowered on a hoist; zero on the level) (Juvinall Sec. 18.4). The energy to absorb is the change of kinetic energy plus the potential energy lost (Eqs. 18.10, 18.12). One brake takes the share s_b of it: a car's front brakes take most of a stop's energy, and Juvinall's rejected take-off of a jet airliner puts 80 % of it in the wheel brakes. The braking force is taken as constant, so the peak power, at the start of the stop, is that force times v_0; with a drop this assumes it is made at a uniform rate along the stopping distance.

For a short stop the heat is taken as stored in the mass m of the parts in contact with the lining, none lost while it is made (Shigley Eq. 16-55); about 500 J/(kg·K) for steel or cast iron.

### Inputs

```calc {description="Mass being stopped"}
M = 1000 kg_
```

```calc {description="Speed at the start of the stop"}
v_0 = 20 m/s_
```

```calc {description="Speed at the end of braking (0 to rest)"}
v_f = 0 m/s_
```

```calc {description="Height the mass drops during the stop (0 on the level)"}
h = 5 m_
```

```calc {description="Share of the energy this brake absorbs"}
s_b = 0.5_
```

```calc {description="Stop time"}
t = 4 s_
```

```calc {description="Mass of the parts that store the heat (drum, disk)"}
m = 50 kg_
```

```calc {description="Specific heat of those parts"}
c_p = 460 J/(kg·K)_
```

```calc {description="Temperature of the parts before the stop"}
T_amb = 20 °C_
```

```calc {description="Largest temperature of the lining, Juvinall Table 18.1 or Shigley Table 16-3"}
T_lim = 260 °C_
```

### Energy

```calc {description="Standard acceleration of gravity"}
g = 9.80665 m/s²
```

```calc
> Kinetic energy given up, Eq. 18.10
KE = M (v_0^2 - v_f^2) / 2 = ?? J
> Potential energy given up, Eq. 18.12
PE = M g h = ?? J
> Energy this brake absorbs
E = s_b (KE + PE) = ?? J
```

### Deceleration, distance and power

```calc
> Deceleration
a = (v_0 - v_f) / t = ?? m/s²
> In g
a_g = a / g = ??
> Stopping distance
d = (v_0 + v_f) t / 2 = ?? m
> Average braking power of this brake
P_avg = E / t = ?? W
> Peak braking power, at the start of the stop
P_peak = E / d v_0 = ?? W
```

### Temperature

```calc
> Temperature rise of the parts, Shigley Eq. 16-55
ΔT = E / (m c_p) = ?? ΔK
> Temperature at the end of the stop
T_end = T_amb + ΔT = ?? °C
```

### Checks

```calc
@test T_end ≤ T_lim
```
