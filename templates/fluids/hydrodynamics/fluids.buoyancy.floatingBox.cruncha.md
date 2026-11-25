+++
cruncha-template = 1
id = "fluids.buoyancy.floatingBox"
version = "1.0.0"
title = "Floating box: draft and initial stability"
summary = "Displaced volume, draft and freeboard of a floating rectangular box (barge or pontoon), the centre of buoyancy KB, the metacentric radius BM = I/V and the metacentric height GM = KB + BM − KG for small angles of heel, checked to float and to be stable."
category = ["Fluids", "Hydrodynamics"]
tags = ["buoyancy", "flotation", "Archimedes", "barge", "pontoon", "draft", "freeboard", "stability", "metacentre", "metacenter", "metacentric height", "GM", "KB", "BM", "heel", "Marks"]
from = "lacewing:fluids.buoyancy"
license = "GPL-3.0-or-later"
inputs = ["ρ_f", "L", "B", "D", "m", "KG"]
outputs = ["V", "T", "f", "KB", "I", "BM", "GM"]

[[references]]
work = "Marks' Standard Handbook for Mechanical Engineers (Avallone, Baumeister)"
edition = "10th"
locator = "Sec. 3.3 (buoyancy, flotation); Sec. 11.3 (transverse metacentre, metacentric height and stability)"

[[tests]]
name = "Lacewing Lab: 20 × 6 m box of 60 t in fresh water, KG 1.5 m"
source = "Lacewing Lab fluids_test: V = 60 m³, T = 0.5 m, KB = 0.25 m, BM = (20 × 6³/12)/60 = 6 m, GM = 0.25 + 6 − 1.5 = 4.75 m"
inputs = { "ρ_f" = "1000 kg/m^3", L = "20 m", B = "6 m", D = "2 m", m = "60000 kg", KG = "1.5 m" }
expect = { V = "60 m^3", T = "0.5 m", f = "1.5 m", KB = "0.25 m", I = "360 m^4", BM = "6 m", GM = "4.75 m" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Hand check: the same box in sea water"
source = "By hand: V = 60 000/1025 = 58.536585 m³, T = 0.48780488 m, KB = 0.24390244 m, BM = 360/58.536585 = 6.15 m, GM = 4.8939024 m"
inputs = { "ρ_f" = "1025 kg/m^3", L = "20 m", B = "6 m", D = "2 m", m = "60000 kg", KG = "1.5 m" }
expect = { V = "58.536585 m^3", T = "0.48780488 m", BM = "6.15 m", GM = "4.8939024 m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Cargo stowed too high: negative GM"
source = "By hand: KG = 7 m (a deck load on a 2 m hull) gives GM = 0.25 + 6 − 7 = −0.75 m: it capsizes"
inputs = { "ρ_f" = "1000 kg/m^3", L = "20 m", B = "6 m", D = "2 m", m = "60000 kg", KG = "7 m" }
expect = { GM = "-0.75 m" }
tolerance = "1e-9"
checks = "fail"
+++

## Floating box: draft and initial stability

A floating body displaces its own mass of fluid (Archimedes; Marks Sec. 3.3). For a rectangular box (a barge or pontoon) that fixes its draft. Its initial stability is set by the metacentric height GM (Marks Sec. 11.3): positive is stable at small heel, with a righting moment of about m g GM sin φ; negative means it capsizes.

The metacentric radius BM = I/V, with I the second moment of the waterplane about the roll axis, follows from Marks' definition of the transverse metacentre. The method is for small angles of heel and a box shape only: ship hulls, large heel and free liquid surfaces in tanks (which reduce GM) are outside it. Marks quotes 0.46 to 1.07 m as usual for seagoing ships; a large GM rolls quickly and uncomfortably.

```calc {description="Density of the fluid (sea water about 1025 kg/m³, fresh water about 1000 kg/m³)"}
ρ_f = 1025 kg/m³_
```

```calc {description="Length of the box, along the axis it rolls about"}
L = 20 m_
```

```calc {description="Beam: the box's width at the waterline"}
B = 6 m_
```

```calc {description="Depth of the hull, keel to deck"}
D = 2 m_
```

```calc {description="Loaded mass: the box and everything in it"}
m = 60000 kg_
```

```calc {description="Height of the loaded centre of gravity above the keel"}
KG = 1.5 m_
```

### Draft

```calc
> Displaced volume
V = m / ρ_f = ?? m³
> Draft
T = V / (L B) = ?? m
> Freeboard
f = D - T = ?? m
```

### Stability

```calc
> Centre of buoyancy above the keel (half the draft for a box)
KB = T / 2 = ?? m
> Second moment of the waterplane about the roll axis
I = L B^3 / 12 = ?? m⁴
> Metacentric radius
BM = I / V = ?? m
> Metacentric height, Marks Sec. 11.3
GM = KB + BM - KG = ?? m
```

### Checks

The box floats (its draft is less than its hull depth) and is stable at small heel.

```calc
@test T < D
```

```calc
@test GM > 0 'm'
```
