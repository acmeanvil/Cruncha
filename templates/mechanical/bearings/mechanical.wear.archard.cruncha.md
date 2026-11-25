+++
cruncha-template = 1
id = "mechanical.wear.archard"
version = "1.0.0"
title = "Sliding wear by Archard's equation (Juvinall)"
summary = "The wear of the softer of two sliding surfaces by the wear equation W = K·F·S/H: the volume worn over a sliding distance, the depth over the apparent contact area with the scatter band of the wear coefficient, the contact pressure and pv product, and the running time to an allowed wear depth."
category = ["Mechanical engineering", "Bearings"]
tags = ["wear", "Archard", "wear coefficient", "adhesive wear", "abrasive wear", "sliding", "pv", "wear depth", "wear life", "tribology", "Rabinowicz", "Juvinall"]
from = "lacewing:mechanical.wear"
license = "GPL-3.0-or-later"
inputs = ["K", "H_n", "F", "v", "t", "A", "h_allow"]
outputs = ["H", "S", "W", "h", "h_low", "h_high", "p", "pv", "t_life"]

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Secs. 9.9 to 9.12: Eqs. 9.1 and 9.1a (wear equation), footnote 2 (hardness number as a pressure), Fig. 9.12 (wear coefficients, after Rabinowicz), Sample Problem 9.2"

[[tests]]
name = "Juvinall Sample Problem 9.2: the copper pin's wear"
source = "Lacewing Lab wear_test, after Juvinall 5th Sample Problem 9.2: copper pin, 80 HV, 20 N on a steel disk, 16 mm track radius at 80 rev/min for 2 h (sliding speed 0.13404 m/s, 965 m); with the test's K = 1.10 × 10⁻⁴ it loses 2.7 mm³ and a 4 mm pin wears 0.21 mm"
inputs = { K = "1.10e-4", H_n = "80", F = "20 N", v = "0.13404129 m/s", t = "2 h", A = "12.566371 mm²", h_allow = "0.5 mm" }
expect = { W = "2.7 mm³", h = "0.21 mm" }
tolerance = "3%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 9.2: the steel disk's track"
source = "Lacewing Lab wear_test, after Juvinall 5th Sample Problem 9.2: the steel disk (210 HB, K = 6.94 × 10⁻⁵) wears its 32 mm diameter, 4 mm wide track (402.12386 mm²) by 0.0016 mm in the 2 h test"
inputs = { K = "6.94e-5", H_n = "210", F = "20 N", v = "0.13404129 m/s", t = "2 h", A = "402.12386 mm²", h_allow = "0.5 mm" }
expect = { h = "0.0016 mm" }
tolerance = "1.5%"
checks = "pass"

[[tests]]
name = "Hand check: Sample Problem 9.2's pin with K = 1.10 × 10⁻⁴ exactly"
source = "By hand: H = 80 × 9.80665 = 784.532 MPa, S = 0.13404129 × 7200 = 965.09729 m, W = 1.1e-4 × 20 × 965.09729/784.532e6 = 2.7063446 mm³, h = W/(π 4²/4) = 0.21536406 mm (×/÷ 4: 0.053841 to 0.86146 mm), p = 1.5915494 MPa, pv = 0.21333333 MPa·m/s, life to 0.5 mm = 0.5e-3 × 12.566371e-6 × 784.532e6/(1.1e-4 × 20 × 0.13404129) = 4.6433002 h"
inputs = { K = "1.10e-4", H_n = "80", F = "20 N", v = "0.13404129 m/s", t = "2 h", A = "12.566371 mm²", h_allow = "0.5 mm" }
expect = { H = "784.532 MPa", S = "965.09729 m", W = "2.7063446 mm³", h = "0.21536406 mm", h_low = "0.053841015 mm", h_high = "0.86145624 mm", p = "1.5915494 MPa", pv = "0.21333333 MPa·m/s", t_life = "4.6433002 h" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The pin run for 6 hours wears past 0.5 mm"
source = "By hand: three times the 2 h depth, 3 × 0.21536406 = 0.64609219 mm, over the 0.5 mm allowed"
inputs = { K = "1.10e-4", H_n = "80", F = "20 N", v = "0.13404129 m/s", t = "6 h", A = "12.566371 mm²", h_allow = "0.5 mm" }
expect = { h = "0.64609219 mm" }
tolerance = "1e-6"
checks = "fail"
+++

## Sliding wear by Archard's equation

Two surfaces sliding on each other wear by adhesion, abrasion or fretting. The wear equation (Archard's law, Juvinall Eq. 9.1a) makes the volume W worn from the **softer** surface proportional to the normal force F and the sliding distance S and inversely proportional to that surface's hardness H: W = K F S/H, independent of the apparent contact area. Spread evenly over an area A, the depth grows at the rate (K/H) p v (Eq. 9.1). The wear coefficient K is dimensionless and the uncertain part: values run from about 10⁻¹ to 10⁻⁸ and scatter by about a factor of 4 either way, so the depth is given with that band. Shigley's boundary-lubricated bushing method (w = K P V t, Sec. 12-15) is the same law with a dimensional wear factor.

### Wear coefficient and hardness

K for the pair is best found from a test of the same materials under the same lubrication and temperature (the pin-on-disk template). Without one, Juvinall's Fig. 9.12 (after Rabinowicz) gives bands of K by the kind of wear and, for adhesive wear, by how compatible the two metals are (Fig. 9.11) and how well they are lubricated: unlubricated identical metals near the top, well-lubricated incompatible metals and nonmetals near the bottom, two-body abrasion highest of all. The hardness is the softer surface's Brinell, Vickers or Knoop number, kgf/mm², used as a pressure (Juvinall footnote 2, p. 389).

```calc {description="Wear coefficient of the softer surface (test, or Juvinall Fig. 9.12)"}
K = 1.10e-4_
```

```calc {description="Hardness number of the softer surface, HB or HV (kgf/mm²)"}
H_n = 80_
```

```calc {description="Hardness as a pressure"}
H = H_n × 1 'kgf/mm²' = ?? MPa
```

### Sliding

```calc {description="Normal force pressing the surfaces together"}
F = 20 N_
```

```calc {description="Sliding speed"}
v = 0.13404129 m/s_
```

```calc {description="Running time"}
t = 2 h_
```

```calc {description="Apparent contact area over which the wear is spread"}
A = 12.566371 mm²_
```

```calc {description="Wear depth that ends the part's life"}
h_allow = 0.5 mm_
```

```calc
> Sliding distance
S = v t = ?? m
> Volume worn from the softer surface, Eq. 9.1a
W = K F S / H = ?? mm³
> Wear depth, spread evenly over A
h = W / A = ?? mm
> Lower end of the depth band, K ÷ 4
h_low = h / 4 = ?? mm
> Upper end of the depth band, K × 4
h_high = 4 h = ?? mm
> Apparent contact pressure
p = F / A = ?? MPa
> Pressure-velocity product
pv = p v = ?? MPa·m/s
> Running time to wear through the allowed depth, Eq. 9.1
t_life = h_allow H / (K p v) = ?? h
```

### Checks

```calc
@test h ≤ h_allow
```
