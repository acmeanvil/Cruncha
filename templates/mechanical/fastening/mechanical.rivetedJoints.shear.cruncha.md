+++
cruncha-template = 1
id = "mechanical.rivetedJoints.shear"
version = "1.0.0"
title = "Riveted or bolted lap and butt joint in shear (Shigley)"
summary = "The capacity of a lap joint or a butt joint with one or two straps, riveted or with fitted bolts, in each way it can fail: shear of the rivets, bearing on the rivets and on the members, tearing of the plate and the straps across a row of holes, and the edge margin shearing out. The least is the joint's capacity; its ratio to the solid plate is the efficiency."
category = ["Mechanical engineering", "Fastening"]
tags = ["rivet", "riveted joint", "lap joint", "butt joint", "splice", "strap", "bolted joint", "shear", "bearing", "net section", "tearing", "edge distance", "joint efficiency", "Shigley"]
from = "lacewing:mechanical.rivetedJoints"
license = "GPL-3.0-or-later"
inputs = ["n_st", "w", "t", "t_s", "d", "d_h", "A_r", "n_r", "n_row", "n_e", "l_e", "S_(y,r)", "S_(y,m)", "n_d", "F"]
outputs = ["A_v", "t_b", "τ_r", "σ_(br,r)", "σ_(br,m)", "σ_t", "τ_m", "F_s", "F_(br,r)", "F_(br,m)", "F_(t,p)", "F_(t,s)", "F_e", "F_cap", "F_solid", "η", "r_e"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 8-12: Eqs. 8-53 (shear), 8-54 (tension on the net section), 8-55 (bearing), Figs. 8-23 and 8-25 (modes of failure, edge shear-out), the 1.5d edge distance; Example 8-6"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sample Problem 2.10 (force flow through a riveted butt joint); Sec. 10.10, Sample Problem 10.3; Sec. 11.2"

[[tests]]
name = "Shigley Ex. 8-6: 1 × 4 in bars spliced between 1/2 in plates, four 3/4 in grade 5 bolts"
source = "Lacewing Lab riveted_joints_test, after Shigley 10th Example 8-6: 1018 CD members (Sy 54 kpsi), grade 5 bolts (Sy 92 kpsi), nd = 1.5, two bolts each side, edge distance 1.5 in: bearing on the bolts 92 kip, on the members 54 kip, shear 62.5 kip, edge 93.5 kip, tearing of bar and splice plates 90 kip; member bearing governs at 54 kip"
inputs = { n_st = "2", w = "4 in", t = "1 in", t_s = "0.5 in", d = "0.75 in", d_h = "0.75 in", A_r = "0 in²", n_r = "2", n_row = "2", n_e = "2", l_e = "1.5 in", "S_(y,r)" = "92 ksi", "S_(y,m)" = "54 ksi", n_d = "1.5", F = "50 kip" }
expect = { "F_(br,r)" = "92 kip", "F_(br,m)" = "54 kip", F_s = "62.5 kip", F_e = "93.5 kip", "F_(t,p)" = "90 kip", "F_(t,s)" = "90 kip", F_cap = "54 kip", "η" = "0.375" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 8-6 with threads in the shear planes"
source = "Lacewing Lab riveted_joints_test, after Shigley 10th Example 8-6: on the minor-diameter area Ar = 0.351 in² the bolts shear at 49.7 kip, which then governs"
inputs = { n_st = "2", w = "4 in", t = "1 in", t_s = "0.5 in", d = "0.75 in", d_h = "0.75 in", A_r = "0.351 in²", n_r = "2", n_row = "2", n_e = "2", l_e = "1.5 in", "S_(y,r)" = "92 ksi", "S_(y,m)" = "54 ksi", n_d = "1.5", F = "45 kip" }
expect = { F_s = "49.7 kip", F_cap = "49.7 kip" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 10.3: a 1/2 in grade 5 bolt in double shear"
source = "Lacewing Lab riveted_joints_test, after Juvinall & Marshek 5th Sample Problem 10.3: yields in shear at about 21 000 lb (the book takes Ssy = 0.58 Sy; 0.577 gives 20 846 lb)"
inputs = { n_st = "2", w = "4 in", t = "1 in", t_s = "1 in", d = "0.5 in", d_h = "0.5 in", A_r = "0 in²", n_r = "1", n_row = "1", n_e = "1", l_e = "2 in", "S_(y,r)" = "92 ksi", "S_(y,m)" = "92 ksi", n_d = "1", F = "15 kip" }
expect = { F_s = "21.0 kip" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "Hand check: lap joint, three 12 mm rivets in 13 mm holes"
source = "Lacewing Lab riveted_joints_test (lap joint), by hand: w = 100 mm, plates 6 and 8 mm, Sy 300 (rivet) and 250 MPa (plate), nd = 2: shear 3 × 113.097 × 0.577 × 150 = 29 365.72 N; member bearing 3 × 12 × 6 × 125 = 27 000 N governs; tearing (100 − 39) × 6 × 125 = 45 750 N; efficiency 27 000/75 000 = 0.36"
inputs = { n_st = "1", w = "100 mm", t = "6 mm", t_s = "8 mm", d = "12 mm", d_h = "13 mm", A_r = "0 mm²", n_r = "3", n_row = "3", n_e = "3", l_e = "20 mm", "S_(y,r)" = "300 MPa", "S_(y,m)" = "250 MPa", n_d = "2", F = "20 kN" }
expect = { F_s = "29365.72317 N", "F_(br,m)" = "27000 N", "F_(t,p)" = "45750 N", F_cap = "27000 N", "η" = "0.36" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "One-strap butt joint with the rivets 1d from the edge: the margin shears out"
source = "Lacewing Lab riveted_joints_test (single strap: bearing on the 5 mm strap, 22 500 N), by hand: edge margin 12 − 6 = 6 mm, 2 × 3 × 6 × 5 × 0.577 × 125 = 12 982.5 N governs; e/d = 1, under 1.5"
inputs = { n_st = "1", w = "100 mm", t = "8 mm", t_s = "5 mm", d = "12 mm", d_h = "12 mm", A_r = "0 mm²", n_r = "3", n_row = "3", n_e = "3", l_e = "12 mm", "S_(y,r)" = "300 MPa", "S_(y,m)" = "250 MPa", n_d = "2", F = "10 kN" }
expect = { "F_(br,m)" = "22500 N", F_e = "12982.5 N", F_cap = "12982.5 N", r_e = "1" }
tolerance = "1e-8"
checks = "fail"
+++

## Riveted or bolted lap and butt joint in shear

The load a riveted joint can carry when the plates pull apart in their own plane, at given allowable stresses (Shigley Sec. 8-12). Fitted bolts that bear on their holes are treated the same way. The joint is a lap joint (two plates overlapping, each rivet in single shear) or a butt joint whose plates meet end to end and are joined by one strap (single shear) or two straps (double shear). A butt joint is analysed on one side of the butt line; the other side is the same.

Every rivet is taken to carry the same share, which holds for a ductile joint at its capacity because slight yielding brings them all in. Bending of the rivets, stress concentration at the holes and fatigue are not checked. For a rivet or bolt group loaded off its centroid, use the bolt group under eccentric shear template.

### Joint

For a lap joint enter n_st = 1, t as the thinner plate and t_s as the other plate. For a butt joint, t is the plate and t_s each strap's thickness. With a seam of many rows, w may be one pitch, with the rivets in it.

```calc {description="Straps: 1 for a lap or one-strap butt joint (single shear), 2 for two straps (double shear)"}
n_st = 2_
```

```calc {description="Width of the plates (or one pitch of a long seam)"}
w = 4 in_
```

```calc {description="Plate thickness (the thinner plate of a lap joint)"}
t = 1 in_
```

```calc {description="Thickness of each strap (the other plate of a lap joint)"}
t_s = 0.5 in_
```

```calc {description="Rivet (or fitted bolt) diameter"}
d = 0.75 in_
```

```calc {description="Hole diameter, for the net section"}
d_h = 0.75 in_
```

```calc {description="Shear area per plane if not πd²/4, such as a bolt's minor-diameter area (0: πd²/4)"}
A_r = 0 in²_
```

```calc {description="Rivets on one side of the joint"}
n_r = 2_
```

```calc {description="Holes in the critical row, where the whole load crosses the plate"}
n_row = 2_
```

```calc {description="Rivets in the end row, next to the plate's edge"}
n_e = 2_
```

```calc {description="Edge distance, centre of the end-row holes to the edge along the load"}
l_e = 1.5 in_
```

### Allowable stresses

As Shigley's Example 8-6 takes them, from the yield strengths and a design factor: shear 0.577 S_y / n_d (distortion energy), bearing and net-section tension S_y / n_d. Where a code or specification gives allowables, replace these five calcs with its values.

```calc {description="Yield strength of the rivets or bolts"}
S_(y,r) = 92 ksi_
```

```calc {description="Yield strength of the plates and straps"}
S_(y,m) = 54 ksi_
```

```calc {description="Design factor"}
n_d = 1.5_
```

```calc
> Allowable shear stress of the rivets
τ_r = 0.577 S_(y,r) / n_d = ?? ksi
> Allowable bearing stress of the rivets
σ_(br,r) = S_(y,r) / n_d = ?? ksi
> Allowable bearing stress of the members
σ_(br,m) = S_(y,m) / n_d = ?? ksi
> Allowable tensile stress of the members on the net section
σ_t = S_(y,m) / n_d = ?? ksi
> Allowable shear stress of the members, for the edge margin
τ_m = 0.577 S_(y,m) / n_d = ?? ksi
```

### Capacity in each mode

Shear is taken on the rivet's nominal diameter, not the hole. Bearing acts on the projected area d·t_b of the thinnest part: the plate, or the straps together. The plate tears across the critical row, and so do the straps, which carry the whole load across the butt line. The edge margin shears out along two planes of length l_e − d/2 beside each end-row rivet.

```calc
> Shear area of one rivet per shear plane
A_v = { π d^2 / 4 if A_r ≤ 0 'in²'; A_r otherwise } = ?? in²
> Thickness the rivets bear on
t_b = min(t, n_st t_s) = ?? in
```

```calc {description="Shear of the rivets, Eq. 8-53"}
F_s = n_r n_st A_v τ_r = ?? kip
```

```calc {description="Bearing on the rivets, Eq. 8-55"}
F_(br,r) = n_r d t_b σ_(br,r) = ?? kip
```

```calc {description="Bearing on the members, Eq. 8-55"}
F_(br,m) = n_r d t_b σ_(br,m) = ?? kip
```

```calc {description="Tearing of the plate across the critical row, Eq. 8-54"}
F_(t,p) = (w - n_row d_h) t σ_t = ?? kip
```

```calc {description="Tearing of the straps (the other plate of a lap joint)"}
F_(t,s) = (w - n_row d_h) n_st t_s σ_t = ?? kip
```

```calc {description="Shearing out of the edge margin, Fig. 8-25"}
F_e = 2 n_e (l_e - d / 2) t_b τ_m = ?? kip
```

### Joint capacity and efficiency

```calc
> Joint capacity, the least of the modes
F_cap = min(F_s, F_(br,r), F_(br,m), F_(t,p), F_(t,s), F_e) = ?? kip
> The solid plate's capacity in tension
F_solid = w t σ_t = ?? kip
> Joint efficiency
η = F_cap / F_solid = ??
> Edge distance in rivet diameters (at least 1.5 in structural practice)
r_e = l_e / d = ??
```

```calc {description="Load on the joint"}
F = 50 kip_
```

### Checks

```calc
@test F ≤ F_cap
```

```calc
@test r_e ≥ 1.5
```
