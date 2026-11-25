+++
cruncha-template = 1
id = "mechanical.bevelGears.geometry"
version = "1.0.0"
title = "Straight bevel gear geometry and tooth forces"
summary = "A straight (Coniflex) bevel gear pair at any shaft angle: pitch cone angles, outer pitch diameters, cone distance, the largest face width of Eq. 15-24, the Gleason addenda of Shigley Table 13-3, clearance and outside diameters, and the tangential, radial and axial tooth forces on pinion and gear at the middle of the face."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "bevel gear", "straight bevel", "Coniflex", "miter gear", "pitch angle", "cone distance", "addendum", "tooth forces", "thrust", "Shigley", "Juvinall"]
from = "lacewing:mechanical.bevelGears"
license = "GPL-3.0-or-later"
inputs = ["m", "N_P", "N_G", "Σ", "F", "H", "n_P", "φ"]
outputs = ["m_G", "γ", "Γ", "d_P", "d_G", "A_0", "F_max", "m_90", "a_P", "a_G", "c", "D_(o,P)", "D_(o,G)", "d_av", "W_t", "W_(r,P)", "W_(a,P)", "W_(r,G)", "W_(a,G)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 13-9 and Table 13-3 (straight bevel tooth proportions); Sec. 13-15, Eqs. 13-37 and 13-38 and Example 13-8 (forces); Eqs. 15-24 and 15-25 (face width, cone distance)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 16.7, Eqs. 16.18 to 16.23 (forces at the mean diameter)"

[[tests]]
name = "Shigley Ex. 13-8: 15/45 teeth, P 5, 5 hp at 600 rev/min, mean pinion radius 1.293 in"
source = "Shigley 10th Example 13-8 (also Lacewing Lab bevel_test): γ = 18.43°, at the mean radius 1.293 in the pitch-line velocity is 406 ft/min, Wt = 406 lbf, and on the gear Wr = 46.6 lbf (46.75 lbf by these equations) and Wa = 140 lbf. The face width 1.309 in is chosen here to give the example's mean radius, (3 − 2 × 1.293)/sin γ"
inputs = { m = "0.2 in", N_P = "15", N_G = "45", "Σ" = "90°", F = "1.30917 in", H = "5 hp", n_P = "600 rpm", "φ" = "20°" }
expect = { "γ" = "18.43°", d_av = "2.586 in", W_t = "406 lbf", "W_(r,G)" = "46.6 lbf", "W_(a,G)" = "140 lbf" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 15-1 geometry: 25-tooth miters, P 5, 1.10 in face"
source = "Shigley 10th Example 15-1 (also Lacewing Lab bevel_test): pitch angles 45°, A0 = 2.5/sin 45° = 3.536 in, equal addenda 1/P = 0.200 in. The example's 1.10 in face is wider than 0.3 A0 = 1.061 in"
inputs = { m = "0.2 in", N_P = "25", N_G = "25", "Σ" = "90°", F = "1.10 in", H = "1 hp", n_P = "600 rpm", "φ" = "20°" }
expect = { "γ" = "45°", A_0 = "3.5355339 in", a_P = "0.2 in", a_G = "0.2 in" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: 18/40 teeth, P 6, 75° shaft angle, 1.5 in face, 3 hp at 1200 rev/min"
source = "By hand: mG = 2.2222, γ = atan(sin 75°/(mG + cos 75°)) = 21.272096°, Γ = 53.727904°, A0 = 3/(2 sin γ) = 4.1345387 in, m90 = √(mG cos γ/cos Γ) = 1.8708945, aG = 0.54/6 + 0.46/(6 m90²) = 0.11190322 in, aP = 0.22143011 in, DoP = 3.4126873 in, DoG = 6.7990752 in; dav = 3 − 1.5 sin γ = 2.4558039 in, Wt = 33 000 × 3/(π dav 1200/12) = 128.31920 lbf, pinion Wr = 43.522309, Wa = 16.944225 lbf; gear Wr = 27.631267, Wa = 37.653834 lbf. Face 1.5 in is over 0.3 A0 = 1.2403616 in"
inputs = { m = "0.16666667 in", N_P = "18", N_G = "40", "Σ" = "75°", F = "1.5 in", H = "3 hp", n_P = "1200 rpm", "φ" = "20°" }
expect = { "γ" = "21.272096°", "Γ" = "53.727904°", A_0 = "4.1345387 in", m_90 = "1.8708945", a_G = "0.11190322 in", a_P = "0.22143011 in", "D_(o,P)" = "3.4126873 in", "D_(o,G)" = "6.7990752 in", d_av = "2.4558039 in", W_t = "128.31920 lbf", "W_(r,P)" = "43.522309 lbf", "W_(a,P)" = "16.944225 lbf", "W_(r,G)" = "27.631267 lbf", "W_(a,G)" = "37.653834 lbf" }
tolerance = "2e-6"
checks = "fail"
+++

## Straight bevel gear geometry and tooth forces

A straight bevel (Coniflex) gear pair on intersecting shafts, sized at the outer (heel) end in the transverse plane, as Shigley Sec. 13-9 and Table 13-3 give the Gleason proportions. The pitch cones' angles add to the shaft angle. The gear's addendum is the short one and the pinion's the long one, by the equivalent 90° ratio m₉₀. AGMA limits the face width to the smaller of 0.3 A₀ and 10/P (Eq. 15-24).

The tooth forces are taken at the middle of the face, where the pitch diameter is d_P − F sin γ (Juvinall Eq. 16.18; Shigley Eqs. 13-37 and 13-38). On the pinion the radial (separating) force goes as cos γ and the axial thrust as sin γ, both pushing it away from the apex; on the gear Γ takes γ's place. At a 90° shaft angle the gear's thrust is the pinion's radial force, and the reverse. Use them for the shaft and bearing loads; for strength use the bevel gear rating template.

### Pair

```calc {description="Outer transverse module (1/P for a diametral pitch)"}
m = 0.2 in_
```

```calc {description="Pinion teeth"}
N_P = 15_
```

```calc {description="Gear teeth"}
N_G = 45_
```

```calc {description="Shaft angle"}
Σ = 90°_
```

```calc {description="Face width, along the cone"}
F = 1.30917 in_
```

```calc
> Gear ratio
m_G = N_G / N_P = ??
> Pinion pitch angle, Eq. 13-14 (tan γ = NP/NG at 90°)
γ = atan(sin(Σ) / (m_G + cos(Σ))) = ?? °
> Gear pitch angle
Γ = Σ - γ = ?? °
> Outer pitch diameters
d_P = N_P m = ?? in
d_G = N_G m = ?? in
> Outer cone distance, Eq. 15-25
A_0 = d_P / (2 sin(γ)) = ?? in
> Largest face width, Eq. 15-24
F_max = min(0.3 A_0, 10 m) = ?? in
```

### Tooth proportions

```calc
> Equivalent 90° ratio, Table 13-3
m_90 = sqrt(m_G cos(γ) / cos(Γ)) = ??
> Gear addendum, Table 13-3
a_G = 0.54 m + 0.460 m / m_90^2 = ?? in
> Pinion addendum (working depth 2m)
a_P = 2 m - a_G = ?? in
> Clearance, Table 13-3
c = 0.188 m + 0.002 'in' = ?? in
> Outside diameters
D_(o,P) = d_P + 2 a_P cos(γ) = ?? in
D_(o,G) = d_G + 2 a_G cos(Γ) = ?? in
```

### Tooth forces at the middle of the face

```calc {description="Power transmitted"}
H = 5 hp_
```

```calc {description="Pinion speed"}
n_P = 600 rpm_
```

```calc {description="Pressure angle"}
φ = 20°_
```

```calc
> Pinion pitch diameter at the middle of the face, Juvinall Eq. 16.18
d_av = d_P - F sin(γ) = ?? in
> Tangential force, Eq. 13-37
W_t = H / (n_P d_av / 2) = ?? lbf
> Pinion radial and axial forces, Eq. 13-38
W_(r,P) = W_t tan(φ) cos(γ) = ?? lbf
W_(a,P) = W_t tan(φ) sin(γ) = ?? lbf
> Gear radial and axial forces
W_(r,G) = W_t tan(φ) cos(Γ) = ?? lbf
W_(a,G) = W_t tan(φ) sin(Γ) = ?? lbf
```

### Check

```calc
@test F ≤ F_max
```
