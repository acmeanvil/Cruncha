+++
cruncha-template = 1
id = "mechanical.gearTrains.differential"
version = "1.0.0"
title = "Bevel gear differential: axle speeds and torques"
summary = "The bevel differential as a planetary train with a train value of −1: the carrier turns at the mean of the two axle speeds, so one axle's speed gives the other's, and with frictionless planet pinions each axle carries half the carrier torque."
category = ["Mechanical engineering", "Gears"]
tags = ["gear train", "differential", "bevel differential", "axle", "carrier", "planetary", "Willis", "Juvinall"]
from = "lacewing:mechanical.gearTrains"
license = "GPL-3.0-or-later"
inputs = ["ω_A", "ω_L", "T_A"]
outputs = ["ω_R", "T_axle"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 16.9, Figs. 16.17 and 16.18; Sec. 15.13, Eq. 15.28 (Willis' equation)"

[[tests]]
name = "Juvinall Sec. 16.9: one axle held, the other at twice the carrier speed"
source = "Juvinall 5th Sec. 16.9 (also Lacewing Lab gear_trains_test)"
inputs = { "ω_A" = "100 rpm", "ω_L" = "0 rpm", T_A = "500 N·m" }
expect = { "ω_R" = "200 rpm", T_axle = "250 N·m" }
tolerance = "1e-12"

[[tests]]
name = "Juvinall Sec. 16.9: one wheel at 101 % of the carrier, the other at 99 %"
source = "Juvinall 5th Sec. 16.9 (also Lacewing Lab gear_trains_test)"
inputs = { "ω_A" = "100 rpm", "ω_L" = "101 rpm", T_A = "500 N·m" }
expect = { "ω_R" = "99 rpm" }
tolerance = "1e-12"
+++

## Bevel gear differential

The bevel differential (Juvinall Sec. 16.9, Fig. 16.18) is a planetary train: the ring gear and case are the carrier, driven by the engine, and the two side gears on the axles are the sun and ring. With the carrier held the side gears turn equal and opposite, a train value of exactly −1, so Willis' equation makes the carrier speed the mean of the axle speeds. With one axle held the other turns at twice the carrier; when one wheel turns at 101 % of the carrier, the other turns at 99 %. A differential with frictionless planet pinions always divides the torque equally, which is why a wheel on ice limits the torque to both.

```calc {description="Carrier (ring gear and case) speed"}
ω_A = 100 rpm_
```

```calc {description="Left axle speed"}
ω_L = 101 rpm_
```

```calc {description="Torque into the carrier"}
T_A = 500 N·m_
```

```calc
> Right axle speed, Willis' equation with e = −1
ω_R = 2 ω_A - ω_L = ?? rpm
> Torque on each axle
T_axle = T_A / 2 = ?? N·m
```
