+++
cruncha-template = 1
id = "mechanical.fatigue.enduranceLimit"
version = "1.0.1"
title = "Fatigue: endurance limit with Marin factors (Shigley)"
summary = "Endurance limit of a steel part: the rotating-beam limit S′e (estimated from Sut or measured) times the Marin factors for surface, size, temperature, reliability and miscellaneous effects."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["fatigue", "endurance limit", "Marin factors", "surface factor", "size factor", "reliability", "Shigley", "stress-life", "high-cycle"]
from = "lacewing:mechanical.fatigue"
license = "GPL-3.0-or-later"
inputs = ["S_ut", "S_(e,test)", "a_s", "b_s", "d_e", "T", "z_a", "k_f"]
outputs = ["S_e′", "k_a", "k_b", "k_d", "k_e", "S_e"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 6-7 to 6-9: Eq. 6-8 (S′e), Eq. 6-18 (Marin equation), Eq. 6-19 and Table 6-2 (surface), Eqs. 6-20 to 6-25 and Table 6-3 (size), Eq. 6-27 (temperature), Eq. 6-29 and Table 6-5 (reliability); Examples 6-3, 6-4, 6-5"

[[tests]]
name = "Shigley Ex. 6-3 and 6-4: machined 520 MPa steel, rotating 32 mm shaft"
source = "Lacewing Lab stress_test (fatigue, Shigley ch. 6 examples): ka = 0.860 for machined Sut 520 MPa (Ex. 6-3); kb = 0.858 at d = 32 mm rotating (Ex. 6-4). Se by hand: 260 × 0.8599 × 0.8577 = 191.7 MPa"
inputs = { S_ut = "520 MPa", "S_(e,test)" = "0 MPa", a_s = "4.51", b_s = "-0.265", d_e = "32 mm", T = "70 °F", z_a = "0", k_f = "1" }
expect = { "S_e′" = "260 MPa", k_a = "0.860", k_b = "0.858", k_d = "1", k_e = "1", S_e = "191.7 MPa" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 6-4: the same 32 mm round, not rotating"
source = "Lacewing Lab stress_test: kb = 0.954 for a non-rotating round, de = 0.370 d = 11.84 mm (Eq. 6-24)"
inputs = { S_ut = "520 MPa", a_s = "4.51", b_s = "-0.265", d_e = "11.84 mm" }
expect = { k_b = "0.954" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 6-5(a): measured S′e = 39.0 kpsi at 450 °F"
source = "Lacewing Lab stress_test: kd = 1.007 at 450 °F (Eq. 6-27); Shigley Ex. 6-5(a) gives (Se)450° = 1.007 × 39.0 = 39.3 kpsi. Polished (a = 1, b = 0) and de = 7.62 mm (kb = 1) leave only kd"
inputs = { S_ut = "70 ksi", "S_(e,test)" = "39.0 ksi", a_s = "1", b_s = "0", d_e = "7.62 mm", T = "450 °F" }
expect = { "S_e′" = "39.0 ksi", k_d = "1.007", S_e = "39.27 ksi" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Eq. 6-8 cap and Table 6-5: Sut 1600 MPa, polished, 99 % reliability"
source = "Lacewing Lab stress_test: S′e = 700 MPa above Sut = 1400 MPa (Eq. 6-8); ke = 0.814 at 99 % (Table 6-5, za = 2.326). Se by hand: 700 × 0.81392 = 569.7 MPa"
inputs = { S_ut = "1600 MPa", a_s = "1", b_s = "0", d_e = "7.62 mm", z_a = "2.326" }
expect = { "S_e′" = "700 MPa", k_e = "0.814", S_e = "569.7 MPa" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "A 300 mm shaft is beyond Eq. 6-20's range"
source = "Shigley Eq. 6-20 covers 2.79 to 254 mm (Lacewing Lab extrapolates beyond it and says so). By hand: kb = 1.51 × 300^-0.157 = 0.6167"
inputs = { d_e = "300 mm" }
expect = { k_b = "0.6167" }
tolerance = "0.1%"
checks = "fail"
+++

## Endurance limit with Marin factors

The endurance limit S_e of a machine part, from the endurance limit S′e of the polished rotating-beam test specimen, by the Marin equation (Shigley Eq. 6-18): S_e = k_a k_b k_c k_d k_e k_f S′e. It is the stress amplitude a steel part takes indefinitely (beyond about 10⁶ cycles) in fully reversed loading.

The estimate S′e = 0.5 Sut (700 MPa above Sut = 1400 MPa, Eq. 6-8) is fitted to wrought irons and steels. Aluminum alloys have no endurance limit, and cast irons and other metals have their own values: enter a measured endurance limit (or a fatigue strength at a stated life) for them.

The load factor k_c is taken as 1. The fluctuating-stress template applies the load type instead: it divides the alternating axial stress by 0.85 and carries torsion through the von Mises stress (Shigley Sec. 6-14). Notch effects are not here either: the fatigue stress-concentration factors multiply the nominal stresses, not S_e.

### Material

```calc {description="Tensile ultimate strength"}
S_ut = 690 MPa_
```

```calc {description="Measured rotating-beam endurance limit; 0 to estimate it from Sut (steels, Eq. 6-8)"}
S_(e,test) = 0 MPa_
```

```calc {description="Rotating-beam endurance limit: measured, or Eq. 6-8"}
S_e′ = { S_(e,test) if S_(e,test) > 0 'MPa'; 0.5 S_ut if S_ut ≤ 1400 'MPa'; 700 'MPa' otherwise } = ?? MPa
```

### Surface factor

k_a = a Sut^b, with Sut in MPa (Eq. 6-19). The factor a and exponent b for ground, machined or cold-drawn, hot-rolled and as-forged surfaces are in Shigley Table 6-2 (use the MPa column of a). For a polished surface, like the test specimen, a = 1 and b = 0.

```calc {description="Surface factor a, Shigley Table 6-2 (Sut in MPa); 1 for polished"}
a_s = 4.51_
```

```calc {description="Surface factor exponent b, Shigley Table 6-2; 0 for polished"}
b_s = -0.265_
```

```calc {description="Surface factor, Eq. 6-19"}
k_a = a_s (S_ut / (1 'MPa'))^b_s = ??
```

### Size factor

Eq. 6-20 takes the diameter of a rotating round in bending or torsion. For other sections use the equivalent diameter of Eqs. 6-24 and 6-25 and Table 6-3: 0.370 d for a round that doesn't rotate, 0.808 √(h b) for a rectangle in bending. Axial loading has no size effect (k_b = 1): enter d_e = 7.62 mm, where Eq. 6-20 gives 1.

```calc {description="Diameter, or equivalent diameter, at the critical section"}
d_e = 32 mm_
```

```calc {description="Size factor, Eq. 6-20"}
k_b = { (d_e / (7.62 'mm'))^(-0.107) if d_e ≤ 51 'mm'; 1.51 (d_e / (1 'mm'))^(-0.157) otherwise } = ??
```

### Temperature, reliability and other effects

At or below room temperature k_d = 1 (check brittle fracture first when colder). Above it, Eq. 6-27 is a fit for steels from 70 to 1000 °F, applied to a room-temperature S′e.

```calc {description="Operating temperature"}
T = 70 °F_
```

```calc {description="Operating temperature as a number of °F, for Eq. 6-27"}
T_F = T / (1 '°R') - 459.67 = ??
```

```calc {description="Temperature factor, Eq. 6-27"}
k_d = { 1 if T_F ≤ 70; 0.975 + 0.432e-3 T_F - 0.115e-5 T_F^2 + 0.104e-8 T_F^3 - 0.595e-12 T_F^4 otherwise } = ??
```

The reliability factor assumes an 8 % standard deviation of the endurance limit (Eq. 6-29). z_a is the standard normal variate of the reliability: 0 at 50 %, 1.288 at 90 %, 1.645 at 95 %, 2.326 at 99 %, 3.091 at 99.9 % (Shigley Table 6-5).

```calc {description="Standard normal variate of the reliability (0 for 50 %), Table 6-5"}
z_a = 0_
```

```calc {description="Reliability factor, Eq. 6-29"}
k_e = 1 - 0.08 z_a = ??
```

The miscellaneous factor covers plating, corrosion, fretting, residual stresses and the like; 1 when none applies. It is not the notch factor K_f.

```calc {description="Miscellaneous-effects factor (1 when none applies)"}
k_f = 1_
```

### Endurance limit of the part

```calc {description="Endurance limit of the part, Eq. 6-18 (k_c = 1)"}
S_e = k_a k_b k_d k_e k_f S_e′ = ?? MPa
```

### Range of the fits

```calc
@test 2.79 'mm' ≤ d_e ≤ 254 'mm'
```

```calc
@test T_F ≤ 1000
```
