+++
cruncha-template = 1
id = "mechanical.gearTrains.ordinary"
version = "1.0.0"
title = "Ordinary gear train: train value, speed and torque"
summary = "A simple or compound gear train with every axis fixed, of any number of meshes, external or internal: the signed train value (the product of the driving tooth numbers over the driven), the speed ratio, the output speed and the output torque after the mesh losses."
category = ["Mechanical engineering", "Gears"]
tags = ["gear train", "train value", "speed ratio", "gear ratio", "compound gear train", "idler", "reverted train", "reduction", "output torque", "Shigley", "Juvinall"]
from = "lacewing:mechanical.gearTrains"
license = "GPL-3.0-or-later"
inputs = ["N_dr", "N_dn", "i_m", "n_F", "T_F", "η"]
outputs = ["k", "e", "r_s", "n_L", "η_t", "T_L"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 13-13, Eqs. 13-29 to 13-31; Example 13-5"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 15.13, Eqs. 15.26 and 15.27; Sample Problem 15.2"

[[tests]]
name = "Juvinall Sample Problem 15.2: 12-tooth pinion at 600 rev/min through a 36-tooth idler to a 28-tooth gear"
source = "Juvinall 5th Sample Problem 15.2 (also Lacewing Lab gear_trains_test): the idler cancels, e = +12/28, and the output turns the same way at 257.1 rev/min"
inputs = { N_dr = "[12, 36]", N_dn = "[36, 28]", i_m = "[0, 0]", n_F = "600 rpm", T_F = "0 N·m", "η" = "1" }
expect = { e = "0.42857143", n_L = "257.142857 rpm" }
tolerance = "1e-5"

[[tests]]
name = "Shigley Ex. 13-5: compound reverted train 108/18 × 105/21"
source = "Shigley 10th Example 13-5 (also Lacewing Lab gear_trains_test): a reduction of exactly 30, both meshes external so the output turns with the input; an ideal train multiplies the torque by 30"
inputs = { N_dr = "[18, 21]", N_dn = "[108, 105]", i_m = "[0, 0]", n_F = "1800 rpm", T_F = "1 N·m", "η" = "1" }
expect = { e = "0.0333333333333", r_s = "30", n_L = "60 rpm", T_L = "30 N·m" }
tolerance = "1e-9"

[[tests]]
name = "Hand check: one internal mesh, 20 teeth driving a 60-tooth ring, 98 % efficient"
source = "Lacewing Lab gear_trains_test and by hand: an internal mesh keeps the sense, e = +20/60 = 1/3; output torque 10 × 0.98 × 3 = 29.4 N·m"
inputs = { N_dr = "[20]", N_dn = "[60]", i_m = "[1]", n_F = "900 rpm", T_F = "10 N·m", "η" = "0.98" }
expect = { k = "1", e = "0.33333333", n_L = "300 rpm", T_L = "29.4 N·m" }
tolerance = "1e-7"

[[tests]]
name = "Hand check: three external meshes 20/50, 18/45, 15/60 at 97 %"
source = "By hand: e = −(20 × 18 × 15)/(50 × 45 × 60) = −0.04, so 1750 rev/min in gives −70 rev/min out (the opposite sense); torque 12 × 0.97³/0.04 = 273.82 N·m"
inputs = { N_dr = "[20, 18, 15]", N_dn = "[50, 45, 60]", i_m = "[0, 0, 0]", n_F = "1750 rpm", T_F = "12 N·m", "η" = "0.97" }
expect = { k = "3", e = "-0.04", r_s = "25", n_L = "-70 rpm", "η_t" = "0.912673", T_L = "273.80190 N·m" }
tolerance = "1e-7"
+++

## Ordinary gear train

A train of meshes whose axes are all fixed in the frame (Shigley Sec. 13-13, Juvinall Sec. 15.13). List the meshes in order from the input: each mesh's driving gear sits on the shaft of the previous mesh's driven gear. An idler is driven in one mesh and drives the next, so its teeth appear in both lists; they cancel, and it changes only the sense of rotation. The train value e is the product of the driving tooth numbers over the product of the driven ones, negative when the last gear turns the opposite way to the first: each external mesh reverses the sense, an internal (ring gear) mesh keeps it. For a planetary train, where axes move, use the planetary template.

### Meshes

Each list has one entry per mesh, in order; `[12, 36]` is two meshes.

```calc {description="Driving gears' teeth, mesh by mesh"}
N_dr = [12, 36]_
```

```calc {description="Driven gears' teeth, mesh by mesh"}
N_dn = [36, 28]_
```

```calc {description="Internal mesh, mesh by mesh: 1 if one gear is a ring gear, 0 if external"}
i_m = [0, 0]_
```

```calc
> Number of meshes
k = length(N_dr) = ??
> Train value, Shigley Eq. 13-30, Juvinall Eq. 15.27
e = product(2 i_m - 1) product(N_dr / N_dn) = ??
> Speed ratio, input over output
r_s = 1 / abs(e) = ??
```

### Speed and torque

```calc {description="Input (first driver) speed"}
n_F = 600 rpm_
```

```calc {description="Input torque"}
T_F = 100 N·m_
```

```calc {description="Efficiency of each mesh (0.98 to 0.99 for a spur mesh; 1 ideal)"}
η = 0.98_
```

```calc
> Output speed, Eq. 13-31 (negative: the opposite sense)
n_L = e n_F = ?? rpm
> Efficiency of the train
η_t = η^k = ??
> Output torque
T_L = T_F η_t / abs(e) = ?? N·m
```
