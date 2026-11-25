+++
cruncha-template = 1
id = "mechanical.keysSplines.pin"
version = "1.0.0"
title = "Cross pin through a shaft in double shear (straight or taper pin)"
summary = "A straight or taper pin driven across a shaft to carry torque to a hub or collar: the pin's diameters at its small end and at the two shear planes (Shigley Eq. 7-35 for the standard 1/4 in per foot taper), the torque it carries to shear yield in double shear (Juvinall Eq. 17.6), its shear stress and factor of safety."
category = ["Mechanical engineering", "Shafts"]
tags = ["pin", "cross pin", "taper pin", "dowel", "double shear", "shaft", "hub", "collar", "torque", "Shigley", "Juvinall"]
from = "lacewing:mechanical.keysSplines"
license = "GPL-3.0-or-later"
inputs = ["D", "d_L", "L_p", "k_t", "S_y", "T", "n_d"]
outputs = ["d_s", "d_p", "S_sy", "T_cap", "τ", "n"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-7, Eq. 7-35 (taper pin diameter), Table 7-5 (standard taper pins)"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.6, Eq. 17.6 (pin in double shear)"

[[tests]]
name = "Shigley Eq. 7-35: a No. 4 taper pin, 0.2513 in large end, 2 in long through a 1 in shaft"
source = "Lacewing Lab keys_splines_test, after Shigley 10th Eq. 7-35 and Table 7-5 (No. 4, commercial maximum 0.2513 in): small end 0.2513 − 0.0208 × 2 = 0.20963 in; at the shear planes, pin centred, 0.24088 and 0.22005 in. By hand: pin Sy 60 kpsi, Ssy = 34.62 kpsi, Juvinall Eq. 17.6 at the smaller plane T = π 0.22005² × 1 × 34 620/4 = 1316.62 lbf·in, so 1500 lbf·in is too much (n = 0.8777)"
inputs = { D = "1 in", d_L = "0.2513 in", L_p = "2 in", k_t = "0.25 in/ft", S_y = "60 kpsi", T = "1500 lbf·in", n_d = "1.5" }
expect = { d_s = "0.2096333 in", d_p = "0.22005 in", T_cap = "1316.6177 lbf·in", "τ" = "39441.973 psi", n = "0.8777451" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Juvinall Eq. 17.6: a straight 1/4 in pin through a 1 in shaft, Ssy 30 kpsi"
source = "Lacewing Lab keys_splines_test, after Juvinall 5th Eq. 17.6: T = π d² D Ssy/4 = π 0.0625 × 1 × 30 000/4 = 1472.62 lbf·in (Sy entered as 30 000/0.577 = 51 993.07 psi so that Ssy = 30 kpsi)"
inputs = { D = "1 in", d_L = "0.25 in", L_p = "1.25 in", k_t = "0 in/ft", S_y = "51993.0676 psi", T = "500 lbf·in", n_d = "1.5" }
expect = { d_p = "0.25 in", S_sy = "30 kpsi", T_cap = "1472.6216 lbf·in" }
tolerance = "1e-6"
checks = "pass"
+++

## Cross pin in double shear

A pin driven diametrically through a shaft and a hub or collar carries the torque in double shear: each of its two shear planes, at the shaft surface, takes a force T/D (Juvinall Eq. 17.6), so the pin reaches shear yield at T = π d² D S_sy/4. Shear yield is taken as S_sy = 0.577 S_y by distortion energy, as for keys. Pins are cheap and locate the hub axially as well, but the cross hole weakens the shaft (a stress concentration to include in its fatigue check), so they suit light torques; Shigley lists them with keys as a way to fix a hub.

Standard taper pins taper 1/4 in per foot on the diameter (Shigley Eq. 7-35); their large-end diameters by size are in Shigley Table 7-5. With the pin centred in the shaft, the smaller shear plane governs. Enter a taper of 0 for a straight solid pin.

```calc {description="Shaft diameter (the pin's length between its shear planes)"}
D = 1 in_
```

```calc {description="Pin diameter at its large end (Shigley Table 7-5 for taper pins)"}
d_L = 0.2513 in_
```

```calc {description="Pin length"}
L_p = 2 in_
```

```calc {description="Taper on the diameter per length (standard taper pins 1/4 in per foot; 0 for a straight pin)"}
k_t = 0.25 in/ft_
```

```calc {description="Tensile yield strength of the pin"}
S_y = 60 kpsi_
```

```calc {description="Torque transmitted"}
T = 1500 lbf·in_
```

```calc {description="Design factor"}
n_d = 1.5_
```

```calc
> Small-end diameter, Eq. 7-35
d_s = d_L - k_t L_p = ?? in
> Diameter at the smaller shear plane, the pin centred in the shaft
d_p = d_L - k_t (L_p + D) / 2 = ?? in
> Shear yield strength of the pin
S_sy = 0.577 S_y = ?? kpsi
> Torque at shear yield in double shear, Juvinall Eq. 17.6
T_cap = π d_p^2 D S_sy / 4 = ?? lbf·in
> Shear stress in the pin
τ = 4 abs(T) / (π d_p^2 D) = ?? psi
> Factor of safety in shear
n = S_sy / τ = ??
```

### Checks

```calc
@test n ≥ n_d
```

```calc
@test L_p ≥ D
```

```calc
@test d_s > 0 'in'
```
