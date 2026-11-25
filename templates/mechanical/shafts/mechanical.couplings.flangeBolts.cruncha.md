+++
cruncha-template = 1
id = "mechanical.couplings.flangeBolts"
version = "1.0.0"
title = "Rigid flanged coupling: bolts in shear and bearing"
summary = "The bolts of a rigid flanged coupling carrying torque on their bolt circle: the force on each bolt, the bolt shear stress and the flange bearing stress, the torque at which the bolts reach their shear yield strength and the flange its bearing yield, and the factors of safety against each."
category = ["Mechanical engineering", "Shafts"]
tags = ["coupling", "flanged coupling", "rigid coupling", "flange bolts", "bolt circle", "bolt shear", "bearing stress", "torque capacity", "Juvinall"]
from = "lacewing:mechanical.couplings"
license = "GPL-3.0-or-later"
inputs = ["n_b", "d_b", "D_c", "t_f", "S_(sy,b)", "S_(y,f)", "T", "n_d"]
outputs = ["F_b", "τ_b", "σ_br", "T_s", "T_br", "n_s", "n_br"]

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.7, Fig. 17.10 (rigid flanged coupling and its force flow: key, hub, flange, bolts in shear and bearing)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-12 (shear joints: bolt shear and member bearing)"

[[tests]]
name = "Six 1/2 in bolts on a 6 in circle, 3/4 in flanges, 50 000 lbf·in"
source = "Lacewing Lab fits_couplings_test: shear capacity 6 × π 0.25/4 × 50 000 × 3 = 176 714.6 lbf·in (Ssy 50 kpsi), bearing 6 × 0.5 × 0.75 × 60 000 × 3 = 405 000 lbf·in (Sy 60 kpsi). By hand at 50 000 lbf·in: 2777.78 lbf per bolt, τ = 14 147.1 psi, bearing 7407.41 psi, n 3.5343 in shear and 8.1 in bearing"
inputs = { n_b = "6", d_b = "0.5 in", D_c = "6 in", t_f = "0.75 in", "S_(sy,b)" = "50 kpsi", "S_(y,f)" = "60 kpsi", T = "50000 lbf·in", n_d = "2" }
expect = { T_s = "176714.59 lbf·in", T_br = "405000 lbf·in", F_b = "2777.7778 lbf", "τ_b" = "14147.106 psi", "σ_br" = "7407.4074 psi", n_s = "3.5342917", n_br = "8.1" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: four M12 class 8.8 bolts on a 100 mm circle, 15 mm steel flanges, overloaded"
source = "By hand: Ssy = 0.577 × 640 = 369.28 MPa (proof strength basis), flange Sy 250 MPa; T_s = 4 × π 0.012²/4 × 369.28e6 × 0.05 = 8352.92 N·m, T_br = 4 × 0.012 × 0.015 × 250e6 × 0.05 = 9000 N·m; at 5000 N·m the shear factor 8352.92/5000 = 1.6706 and the bearing factor 1.8 are under the design factor 2"
inputs = { n_b = "4", d_b = "12 mm", D_c = "100 mm", t_f = "15 mm", "S_(sy,b)" = "369.28 MPa", "S_(y,f)" = "250 MPa", T = "5000 N·m", n_d = "2" }
expect = { T_s = "8352.9168 N·m", T_br = "9000 N·m", n_s = "1.6705834", n_br = "1.8" }
tolerance = "1e-6"
checks = "fail"
+++

## Rigid flanged coupling: bolts

A rigid flanged coupling joins two shaft ends by flanges bolted together; the torque flows from one shaft through its key into the hub, through the flange into the bolts, and back out the other side (Juvinall Fig. 17.10). Each link in that path needs a check; this template checks the bolts, which share the torque as forces on their circle: each fitted bolt carries F = 2T/(n D_c) in shear across the joint plane and bears with that force on the flange holes. The key and hub are checked with the parallel-key template.

All bolts are assumed to bear equally, as with fitted (reamed-hole) bolts; with clearance holes the torque goes by friction from the bolt preload until it slips, and only then into shear. Use the shear yield strength of the bolt (0.577 S_y by distortion energy, or 0.577 times the proof strength for a property class) and the yield strength of the weaker flange in bearing.

```calc {description="Number of bolts"}
n_b = 6_
```

```calc {description="Bolt diameter in the shear plane"}
d_b = 0.5 in_
```

```calc {description="Bolt-circle diameter"}
D_c = 6 in_
```

```calc {description="Thickness of the thinner flange (bearing length)"}
t_f = 0.75 in_
```

```calc {description="Shear yield strength of the bolts"}
S_(sy,b) = 50 kpsi_
```

```calc {description="Yield strength of the weaker flange, in bearing"}
S_(y,f) = 60 kpsi_
```

```calc {description="Torque transmitted"}
T = 50000 lbf·in_
```

```calc {description="Design factor"}
n_d = 2_
```

```calc
> Force on each bolt at the bolt circle
F_b = 2 abs(T) / (n_b D_c) = ?? lbf
> Shear stress in each bolt
τ_b = F_b / (π d_b^2 / 4) = ?? psi
> Bearing stress on the flange
σ_br = F_b / (d_b t_f) = ?? psi
> Torque at which the bolts reach shear yield
T_s = n_b π d_b^2 / 4 S_(sy,b) D_c / 2 = ?? lbf·in
> Torque at which the flange reaches bearing yield
T_br = n_b d_b t_f S_(y,f) D_c / 2 = ?? lbf·in
> Factor of safety of the bolts in shear
n_s = T_s / abs(T) = ??
> Factor of safety of the flange in bearing
n_br = T_br / abs(T) = ??
```

### Checks

```calc
@test n_s ≥ n_d
```

```calc
@test n_br ≥ n_d
```
