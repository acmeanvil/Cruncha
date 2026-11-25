+++
cruncha-template = 1
id = "mechanics.gyroscope.roll"
version = "1.0.0"
title = "Gyroscopic couple on a rotor in a rolling vessel"
summary = "The largest gyroscopic couple M = IΩω on a rotor spinning at Ω whose axis rocks harmonically ±φ0 with period T (a turbine in a rolling ship), with ω = 2πφ0/T at mid-roll, and the pair of forces M/a it puts on the shaft's bearings."
category = ["Mechanical engineering", "Mechanics"]
tags = ["gyroscope", "gyroscopic couple", "gyroscopic moment", "precession", "rotor", "turbine", "ship", "roll", "pitch", "bearing load", "Den Hartog"]
from = "lacewing:mechanics.gyroscope"
license = "GPL-3.0-or-later"
inputs = ["I", "Ω", "a", "φ_0", "T"]
outputs = ["ω", "M", "F"]

[[references]]
work = "Den Hartog, J.P., Mechanics (Dover reprint)"
edition = "1948"
locator = "§60 (the principal theorem of the gyroscope, M = IΩω) and §61a (the turbine in a rolling destroyer)"

[[tests]]
name = "Den Hartog §61a in SI: 50 kg turbine disk, k = 8 in, 10 000 rev/min, bearings 8 in apart, rolling ±45° in 10 s"
source = "Lacewing Lab's opening case, Den Hartog §61a's example in SI. By hand: I = 50 × 0.2032² = 2.064512 kg·m², ω = 2π(π/4)/10 = 0.49348022 rad/s (Den Hartog rounds to 0.5), M = 2.064512 × 1047.1976 × 0.49348022 = 1 066.8805 N·m, F = M/0.2032 = 5 250.3962 N, over 20 times the static 245 N on each bearing"
inputs = { I = "2.064512 kg·m^2", "Ω" = "10000 rpm", a = "0.2032 m", "φ_0" = "45°", T = "10 s" }
expect = { "ω" = "0.49348022 rad/s", M = "1066.8805 N·m", F = "5250.3962 N" }
tolerance = "1e-7"
+++

## Gyroscopic couple in a rolling vessel

A rotor spinning fast at Ω about its axis carries an angular momentum IΩ along that axis. Turning the axis at a rate ω changes the angular momentum at the rate IΩω, which a couple of that size must supply (Den Hartog §60, the principal theorem of the gyroscope). The couple acts about the axis square to both the spin and the turning axes, and the bearings supply it as two equal and opposite forces M/a (§61a). These add to the static load on one bearing and subtract from it on the other.

Here the axis rocks as φ = φ₀ sin(2πt/T), as a ship rolls or pitches with the rotor athwartships or fore and aft. The turning rate is largest at mid-roll, ω = 2πφ₀/T. The rotor spins about a principal axis, and the spin is much faster than the rolling. For a steady rate of turn (a vehicle turning, a spindle swung while running), use mechanics.gyroscope.turning.

```calc {description="Polar mass moment of inertia of the rotor about its spin axis (m·k², ½mr² for a solid disc)"}
I = 2.06 kg·m²_
```

```calc {description="Spin speed of the rotor"}
Ω = 10000 rpm_
```

```calc {description="Distance between the two bearings that carry the rotor's shaft"}
a = 0.2032 m_
```

```calc {description="Roll amplitude, each way from upright"}
φ_0 = 45°_
```

```calc {description="Roll period: one full roll, there and back"}
T = 10 s_
```

```calc
> Largest turning rate of the axis, at mid-roll
ω = 2 π φ_0 / T = ?? rad/s
> Gyroscopic couple, §60
M = I Ω ω = ?? N·m
> Force on each bearing, in opposite directions, §61a
F = M / a = ?? N
```
