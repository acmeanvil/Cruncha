+++
cruncha-template = 1
id = "mechanics.gyroscope.turning"
version = "1.0.0"
title = "Gyroscopic couple at a steady turning rate"
summary = "The gyroscopic couple M = IΩω on a rotor spinning at Ω whose axis is turned at a steady rate ω (a vehicle in a turn, a spindle swung while running), and the pair of forces M/a it puts on the shaft's bearings."
category = ["Mechanical engineering", "Mechanics"]
tags = ["gyroscope", "gyroscopic couple", "gyroscopic moment", "precession", "rotor", "flywheel", "spindle", "vehicle", "turn", "yaw", "bearing load", "Den Hartog"]
from = "lacewing:mechanics.gyroscope"
license = "GPL-3.0-or-later"
inputs = ["I", "Ω", "a", "ω"]
outputs = ["M", "F"]

[[references]]
work = "Den Hartog, J.P., Mechanics (Dover reprint)"
edition = "1948"
locator = "§60 (the principal theorem of the gyroscope, M = IΩω) and §61a (bearing forces)"

[[tests]]
name = "Den Hartog §61a: the destroyer's turbine, per unit mass, with ω = 0.5 rad/s"
source = "Lacewing Lab mechanics_test, after Den Hartog §61a: I = 64m lb·in² (k = 8 in), Ω = 1050 rad/s, ω = 0.5 rad/s give M = 33 600m lb·in²/s² (Den Hartog prints 33 500), and over bearings 8 in apart F = 4 200m"
inputs = { I = "64 lb·in^2", "Ω" = "1050 rad/s", a = "8 in", "ω" = "0.5 rad/s" }
expect = { M = "33600 lb·in^2/s^2", F = "4200 lb·in/s^2" }
tolerance = "1e-9"

[[tests]]
name = "Hand check: a 0.05 kg·m² flywheel at 6 000 rev/min in a car turning at 0.4 rad/s"
source = "By hand: Ω = 628.31853 rad/s, M = 0.05 × 628.31853 × 0.4 = 12.566371 N·m, F = 12.566371/0.15 = 83.775804 N"
inputs = { I = "0.05 kg·m^2", "Ω" = "6000 rpm", a = "0.15 m", "ω" = "0.4 rad/s" }
expect = { M = "12.566371 N·m", F = "83.775804 N" }
tolerance = "1e-7"
+++

## Gyroscopic couple at a steady turning rate

A rotor spinning fast at Ω about its axis carries an angular momentum IΩ along that axis. Turning the axis at a rate ω changes the angular momentum at the rate IΩω, which a couple of that size must supply (Den Hartog §60, the principal theorem of the gyroscope). The couple acts about the axis square to both the spin and the turning axes, and the bearings supply it as two equal and opposite forces M/a (§61a). These add to the static load on one bearing and subtract from it on the other.

The rotor spins about a principal axis, and the spin is much faster than the turning. For a vehicle in a steady turn, ω is its speed over the radius of the turn. For an axis that rocks back and forth (a ship rolling), use mechanics.gyroscope.roll.

```calc {description="Polar mass moment of inertia of the rotor about its spin axis (m·k², ½mr² for a solid disc)"}
I = 0.05 kg·m²_
```

```calc {description="Spin speed of the rotor"}
Ω = 6000 rpm_
```

```calc {description="Distance between the two bearings that carry the rotor's shaft"}
a = 0.15 m_
```

```calc {description="Turning rate of the spin axis (a vehicle's speed over its turn radius)"}
ω = 0.4 rad/s_
```

```calc
> Gyroscopic couple, §60
M = I Ω ω = ?? N·m
> Force on each bearing, in opposite directions, §61a
F = M / a = ?? N
```
