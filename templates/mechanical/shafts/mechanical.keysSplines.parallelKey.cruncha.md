+++
cruncha-template = 1
id = "mechanical.keysSplines.parallelKey"
version = "1.0.0"
title = "Parallel key in shear and crushing (square or rectangular key)"
summary = "A square or rectangular parallel key between a shaft and a hub: the force at the shaft surface, the shear and crushing stresses and factors of safety at the key's length, the length needed for a design factor in shear and in crushing, and Juvinall's balanced length at which the key is as strong as the shaft in torsion."
category = ["Mechanical engineering", "Shafts"]
tags = ["key", "parallel key", "square key", "rectangular key", "keyway", "keyseat", "shear", "crushing", "bearing stress", "hub", "ANSI B17.1", "BS 4235", "DIN 6885", "Shigley", "Juvinall"]
from = "lacewing:mechanical.keysSplines"
license = "GPL-3.0-or-later"
inputs = ["d", "T", "w", "h", "t_2", "l", "n_d", "S_(y,key)", "S_(y,shaft)", "S_(y,hub)"]
outputs = ["F", "S_sy", "S_(y,c)", "h_c", "τ", "σ_c", "n_s", "n_c", "l_s", "l_c", "l_req", "T_y", "l_(bal,s)", "l_(bal,c)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-7, Fig. 7-19, Table 7-6 (key sizes); Example 7-6"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.6, Fig. 17.8 (key loading, balanced key length); Problem 17.35D"

[[references]]
work = "Machinery's Handbook"
edition = "26th"
locator = "Keys and keyseats: ANSI B17.1 key sizes (Table 1, p. 2342); BS 4235 Part 1 metric parallel keys (Table 1, pp. 2355-2356)"

[[tests]]
name = "Shigley Ex. 7-6: 1 7/16 in shaft, 4200 lbf·in, 3/8 in square key of 1018 CD, n = 1.5"
source = "Lacewing Lab keys_splines_test, after Shigley 10th Example 7-6: F = 4200/(1.4375/2) = 5843 lbf (the book rounds to 5850), Ssy = 0.577 × 54 = 31.2 kpsi; l = 0.75 in in shear, 0.866 in against crushing (the book 0.87 in): crushing governs. At a 7/8 in key by hand: τ = 17 809 psi, n_s = 1.7496; σ = 35 617 psi, n_c = 1.5161. Shaft and hub taken as 1035 at Sy 75 kpsi: Juvinall's balanced lengths 3.0055 and 3.4863 in"
inputs = { d = "1.4375 in", T = "4200 lbf·in", w = "0.375 in", h = "0.375 in", t_2 = "0.1875 in", l = "0.875 in", n_d = "1.5", "S_(y,key)" = "54 kpsi", "S_(y,shaft)" = "75 kpsi", "S_(y,hub)" = "75 kpsi" }
expect = { F = "5843.478 lbf", S_sy = "31.158 kpsi", l_s = "0.75017 in", l_c = "0.86570 in", l_req = "0.86570 in", "τ" = "17808.70 psi", "σ_c" = "35617.39 psi", n_s = "1.749595", n_c = "1.516113", "l_(bal,s)" = "3.005466 in", "l_(bal,c)" = "3.486341 in" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Shigley Ex. 7-6 with a 3/4 in key: too short in crushing"
source = "By hand from Ex. 7-6: at l = 0.75 in, σ = 5843.48/(0.1875 × 0.75) = 41 554 psi, n_c = 54/41.554 = 1.2995, under 1.5"
inputs = { d = "1.4375 in", T = "4200 lbf·in", w = "0.375 in", h = "0.375 in", t_2 = "0.1875 in", l = "0.75 in", n_d = "1.5", "S_(y,key)" = "54 kpsi", "S_(y,shaft)" = "75 kpsi", "S_(y,hub)" = "75 kpsi" }
expect = { n_c = "1.299526", n_s = "1.499653" }
tolerance = "1e-5"
checks = "fail"

[[tests]]
name = "Juvinall Sec. 17.6: a flat key w = d/4, h = 3d/16 of the shaft's steel balances at 2.4d"
source = "Lacewing Lab keys_splines_test, after Juvinall 5th Sec. 17.6 and Problem 17.35D: square key w = h = d/4 balances at 1.57d in shear and 1.82d in crushing; the flat key at 2.4d in crushing. By hand for d = 50 mm, Sy 400 MPa: T_y = π 0.05³ × 0.58 × 400e6/16 = 5694.137 N·m, balanced 1.5707963d (shear) and 2.4294983d (crushing)"
inputs = { d = "50 mm", T = "1000 N·m", w = "12.5 mm", h = "9.375 mm", t_2 = "4.6875 mm", l = "60 mm", n_d = "1.5", "S_(y,key)" = "400 MPa", "S_(y,shaft)" = "400 MPa", "S_(y,hub)" = "400 MPa" }
expect = { T_y = "5694.1367 N·m", "l_(bal,s)" = "78.539816 mm", "l_(bal,c)" = "121.47492 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: BS 4235 8 × 7 key on a 28 mm shaft, cast-iron hub"
source = "By hand: 150 N·m, key 8 × 7 with hub keyseat depth t2 = 3.3 mm (BS 4235 Table 1 for 22-30 mm, below h/2 = 3.5 mm, so the hub face crushes on 3.3 mm), l = 36 mm; key Sy 350, shaft 400, hub 250 MPa; F = 10 714.29 N, τ = 37.202 MPa, n_s = 0.577 × 350/37.202 = 5.4284, σ = 90.188 MPa on the hub's 250 MPa, n_c = 2.772; for n_d = 2: l_s 13.2635 mm, l_c 25.9740 mm"
inputs = { d = "28 mm", T = "150 N·m", w = "8 mm", h = "7 mm", t_2 = "3.3 mm", l = "36 mm", n_d = "2", "S_(y,key)" = "350 MPa", "S_(y,shaft)" = "400 MPa", "S_(y,hub)" = "250 MPa" }
expect = { F = "10714.286 N", "S_(y,c)" = "250 MPa", h_c = "3.3 mm", "τ" = "37.202381 MPa", "σ_c" = "90.187590 MPa", n_s = "5.428416", n_c = "2.772", l_s = "13.263538 mm", l_c = "25.974026 mm", l_req = "25.974026 mm" }
tolerance = "1e-6"
checks = "pass"
+++

## Parallel key in shear and crushing

A square or rectangular parallel key carrying a torque from a shaft to a hub, after Shigley Sec. 7-7 and Juvinall Sec. 17.6. Both take the force at the shaft surface, F = T/r, uniformly spread along the key. The key shears across its width at the shaft surface, and its sides bear (crush) on half its height in the shaft and in the hub. The shear strength is S_sy = 0.577 S_y of the key by distortion energy; the crushing face that yields first is the weakest of the key, the shaft and the hub. A hub keyseat shallower than h/2 (BS 4235's t₂) leaves less bearing height on the hub side.

The key is usually softer than the shaft and the hub so that it fails first, and is made to the hub's width (commonly 1.5d to 2d, Juvinall). Shigley warns that a key longer than about 1.5d twists with the shaft and loads unevenly: use two keys at 90° instead. The keyway also weakens the shaft in fatigue: see the shaft design templates (Shigley Table 7-1, Juvinall Fig. 17.9).

Juvinall's balanced length is the length at which the key carries the same torque as the shaft does at its torsional yield, πd³(0.58 S_y)/16: a key that long fails no sooner than the shaft. For a square key w = h = d/4 of the shaft's steel it is 1.57d in shear and 1.82d in crushing.

Standard key sizes for a shaft diameter are in ANSI B17.1 (Shigley Table 7-6, Machinery's Handbook) and BS 4235 / ISO R773 / DIN 6885; enter the key's width, height and the hub keyseat depth from the standard.

### Shaft, key and torque

```calc {description="Shaft diameter"}
d = 1.4375 in_
```

```calc {description="Torque transmitted"}
T = 4200 lbf·in_
```

```calc {description="Key width (ANSI B17.1 or BS 4235 for the shaft diameter)"}
w = 0.375 in_
```

```calc {description="Key height"}
h = 0.375 in_
```

```calc {description="Keyseat depth in the hub (ANSI: h/2; BS 4235: t2)"}
t_2 = 0.1875 in_
```

```calc {description="Key length in engagement"}
l = 0.875 in_
```

```calc {description="Design factor"}
n_d = 1.5_
```

### Strengths

```calc {description="Tensile yield strength of the key"}
S_(y,key) = 54 kpsi_
```

```calc {description="Tensile yield strength of the shaft"}
S_(y,shaft) = 75 kpsi_
```

```calc {description="Tensile yield strength (or compressive strength) of the hub"}
S_(y,hub) = 75 kpsi_
```

```calc
> Force at the shaft surface, F = T/r, Shigley Fig. 7-19
F = 2 abs(T) / d = ?? lbf
> Shear yield strength of the key, 0.577 Sy by distortion energy
S_sy = 0.577 S_(y,key) = ?? kpsi
> Crushing strength: the weakest of key, shaft and hub
S_(y,c) = min(S_(y,key), S_(y,shaft), S_(y,hub)) = ?? kpsi
> Bearing height of the weaker face: half the key, or a shallower hub keyseat
h_c = min(h / 2, t_2) = ?? in
```

### At the key's length

```calc
> Shear stress across the key
τ = F / (w l) = ?? psi
> Factor of safety in shear
n_s = S_sy / τ = ??
> Crushing (bearing) stress on the key's side
σ_c = F / (h_c l) = ?? psi
> Factor of safety in crushing
n_c = S_(y,c) / σ_c = ??
```

### Length for the design factor

```calc
> Length needed in shear, Shigley Ex. 7-6
l_s = n_d F / (S_sy w) = ?? in
> Length needed against crushing
l_c = n_d F / (S_(y,c) h_c) = ?? in
> Length needed: the longer of the two
l_req = max(l_s, l_c) = ?? in
```

### Juvinall's balanced length

```calc
> Torque at the shaft's torsional yield, Juvinall Sec. 17.6
T_y = π d^3 (0.58 S_(y,shaft)) / 16 = ?? lbf·in
> Key length as strong as the shaft, in shear
l_(bal,s) = T_y / (0.58 S_(y,key) w d / 2) = ?? in
> Key length as strong as the shaft, in crushing
l_(bal,c) = T_y / (S_(y,c) h_c d / 2) = ?? in
```

### Checks

```calc
@test n_s ≥ n_d
```

```calc
@test n_c ≥ n_d
```

```calc
@test l ≤ 1.5 d
```
