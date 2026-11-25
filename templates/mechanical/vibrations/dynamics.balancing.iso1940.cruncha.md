+++
cruncha-template = 1
id = "dynamics.balancing.iso1940"
version = "1.0.0"
title = "Permissible residual unbalance, ISO 21940-11 (ISO 1940-1)"
summary = "The permissible residual unbalance of a rigid rotor from its balance quality grade G, its mass and its highest service speed: the permissible specific unbalance (centre-of-mass eccentricity) e_per = G/Ω and U_per = e_per m, allocated to the two bearing planes by the centre of mass's position, and the residual unbalance measured in each checked against it."
category = ["Mechanical engineering", "Vibrations"]
tags = ["balancing", "balance quality grade", "G 6.3", "G 2.5", "ISO 1940", "ISO 21940-11", "residual unbalance", "permissible unbalance", "eccentricity", "rigid rotor", "tolerance"]
from = "lacewing:dynamics.balancing"
license = "GPL-3.0-or-later"
inputs = ["G", "m", "n", "L_A", "L_B", "U_A", "U_B"]
outputs = ["e_per", "U_per", "U_(per,A)", "U_(per,B)"]

[[references]]
work = "ISO 21940-11, Mechanical vibration — Rotor balancing — Part 11: Procedures and tolerances for rotors with rigid behaviour (replaces ISO 1940-1)"
edition = "2016"
locator = "Balance tolerances: the permissible residual unbalance from the balance quality grade (U_per = 1000 e_per m with e_per = G/Ω); Table 1 (balance quality grades for representative rigid rotors); allocation of U_per to the tolerance planes for a rotor with its centre of mass between the bearings"

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "§47 and §48 (unbalance of rigid rotors)"

[[tests]]
name = "Hand check: G 6.3 at 3000 rpm, 100 kg rotor, centre of mass mid-way"
source = "By hand: Ω = 3000 × 2π/60 = 314.15927 rad/s, e_per = 6.3/314.15927 = 0.020053523 mm (the Lacewing Lab critical speeds page's 'about 0.02 mm'), U_per = 0.020053523 × 100 000 g = 2005.3523 g·mm, half to each bearing plane: 1002.6761 g·mm"
inputs = { G = "6.3 mm/s", m = "100 kg", n = "3000 rpm", L_A = "0.5 m", L_B = "0.5 m", U_A = "900 g·mm", U_B = "1000 g·mm" }
expect = { e_per = "0.020053523 mm", U_per = "2005.3523 g·mm", "U_(per,A)" = "1002.6761 g·mm", "U_(per,B)" = "1002.6761 g·mm" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Hand check: centre of mass near bearing A, the 30 % to 70 % limits"
source = "By hand: as before with the centre of mass 0.2 m from A and 0.8 m from B: L_B/L = 0.8 is held to 0.7 for plane A (1403.7466 g·mm) and L_A/L = 0.2 raised to 0.3 for plane B (601.60569 g·mm); 700 g·mm residual at B is over its share"
inputs = { G = "6.3 mm/s", m = "100 kg", n = "3000 rpm", L_A = "0.2 m", L_B = "0.8 m", U_A = "1200 g·mm", U_B = "700 g·mm" }
expect = { "U_(per,A)" = "1403.7466 g·mm", "U_(per,B)" = "601.60569 g·mm" }
tolerance = "1e-7"
checks = "fail"
+++

## Permissible residual unbalance, ISO 21940-11

How well a rigid rotor must be balanced. ISO 21940-11 (which replaced ISO 1940-1) expresses the tolerance as a balance quality grade G, the product of the permissible eccentricity of the centre of mass and the highest service angular speed, in mm/s. Its Table 1 gives grades for representative rotors; as an indication, G 6.3 is the usual grade for general machinery, fans, pumps and electric motor rotors, and G 2.5 for turbines, compressors and machine-tool drives, but use the table's entry for the rotor in hand.

The rotor must behave rigidly at its service speed (well below its first bending critical speed). The tolerance holds at the highest service speed; the residual unbalance is measured on a balancing machine or in situ.

### Rotor

```calc {description="Balance quality grade, ISO 21940-11 Table 1"}
G = 6.3 mm/s_
```

```calc {description="Rotor mass"}
m = 100 kg_
```

```calc {description="Highest service speed"}
n = 3000 rpm_
```

```calc
> Permissible specific unbalance (eccentricity of the centre of mass)
e_per = G / n = ?? mm
> Permissible residual unbalance
U_per = e_per m = ?? g·mm
```

### Allocation to the bearing planes

For a rotor whose centre of mass lies between its bearings, the standard shares U_per between the two bearing planes in inverse proportion to their distances from the centre of mass, each share kept between 30 % and 70 % of U_per. A rotor with large overhangs or correction planes far from the bearings needs the standard's fuller treatment.

```calc {description="Distance of the centre of mass from bearing A"}
L_A = 0.5 m_
```

```calc {description="Distance of the centre of mass from bearing B"}
L_B = 0.5 m_
```

```calc
> Permissible residual unbalance in bearing plane A
U_(per,A) = U_per min(max(L_B / (L_A + L_B), 0.3), 0.7) = ?? g·mm
> Permissible residual unbalance in bearing plane B
U_(per,B) = U_per min(max(L_A / (L_A + L_B), 0.3), 0.7) = ?? g·mm
```

### Check

```calc {description="Residual unbalance measured, referred to bearing plane A"}
U_A = 900 g·mm_
```

```calc {description="Residual unbalance measured, referred to bearing plane B"}
U_B = 1000 g·mm_
```

```calc
@test U_A ≤ U_(per,A)
```

```calc
@test U_B ≤ U_(per,B)
```
