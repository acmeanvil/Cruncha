+++
cruncha-template = 1
id = "mechanical.timingBelts.drive"
version = "1.0.0"
title = "Timing belt drive: geometry and power rating (trapezoidal tooth)"
summary = "A synchronous (timing) belt of a trapezoidal-tooth section (XL, L, H, XH, XXH, ANSI/RMA IP-24) on two toothed pulleys: pitch diameters, wrap, belt length in whole teeth, teeth in mesh, belt speed, design power, the rated power of the section's widest belt from Machinery's Handbook's formula, and the rated power of the width chosen with the width and teeth-in-mesh factors."
category = ["Mechanical engineering", "Belts and chains"]
tags = ["belt", "timing belt", "synchronous belt", "toothed belt", "cog belt", "XL", "L", "H", "XH", "XXH", "pitch diameter", "teeth in mesh", "belt length", "power rating", "Machinery's Handbook", "Shigley"]
from = "lacewing:mechanical.timingBelts"
license = "GPL-3.0-or-later"
inputs = ["p", "N_1", "N_2", "N_min", "C", "n", "H_nom", "K_s", "a_r", "b_r", "b_w", "K_w", "K_z"]
outputs = ["d", "D", "φ", "L", "N_L", "L_p", "N_m", "V", "H_d", "H_r", "H_a", "n_r"]

[[references]]
work = "Machinery's Handbook (Oberg, Jones, Horton, Ryffel)"
edition = "26th"
locator = "Flexible Belts and Sheaves, synchronous belts (ANSI/RMA IP-24): rating formula and selection, pp. 2415-2417; Tables 34 (pitch lengths), 36 (pulley diameters), 38 (teeth-in-mesh factor), 40 (width factors), 42 (service factors)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 17-1, Eqs. 17-1, 17-2 (wrap and length); Sec. 17-4, Table 17-18 (timing belt pitches)"

[[tests]]
name = "H belt, 24 and 48 grooves at 1750 rev/min, 16 in centres, 5 hp, Ks 1.7: the 1.5 in belt"
source = "Lacewing Lab's default timing belt page (Machinery's p. 2416 formula); by hand: d = 24 × 0.5/π = 3.8197186 in, D = 7.6394373 in, φ = 2.9022897 rad, L = 50.228245 in = 100.46 teeth → 100 teeth, 50 in; 11 teeth in mesh; V = 1750 ft/min; Hd = 8.5 hp; dr = 6.6845076, HP = dr(3.73 − 1.41e-3 dr²) = 24.512073 hp; Hd/HP = 0.347, so Table 40's 1.50 in width (Kw 0.45): Ha = 11.030433 hp, 1.2977 times Hd"
inputs = { p = "0.5 in", N_1 = "24", N_2 = "48", N_min = "14", C = "16 in", n = "1750 rpm", H_nom = "5 hp", K_s = "1.7", a_r = "3.73", b_r = "1.41e-3", b_w = "1.5 in", K_w = "0.45", K_z = "1" }
expect = { d = "3.8197186 in", D = "7.6394373 in", "φ" = "2.9022897", L = "50.228245 in", N_L = "100", L_p = "50 in", N_m = "11", V = "1750 ft/min", H_d = "8.5 hp", H_r = "24.512073 hp", H_a = "11.030433 hp", n_r = "1.2976980" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "H belt at half the 3 in rating: the 2 in belt, 1.26 times the design power"
source = "Lacewing Lab timing_belts_test: 24 and 48 grooves at 1750 rev/min, 0.5 m centres, design power 0.5 × the 3 in belt's rating (12.256036 hp): width factor 0.5 → 2.00 in (Kw 0.63), rated over design 0.63/0.5 = 1.26; Kz = 1"
inputs = { p = "0.5 in", N_1 = "24", N_2 = "48", N_min = "14", C = "0.5 m", n = "1750 rpm", H_nom = "12.256036 hp", K_s = "1", a_r = "3.73", b_r = "1.41e-3", b_w = "2 in", K_w = "0.63", K_z = "1" }
expect = { H_r = "24.512073 hp", N_m = "11", n_r = "1.26" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: XL belt, 10 and 30 grooves at 3000 rev/min, 6 in centres, four teeth in mesh"
source = "By hand: d = 10 × 0.2/π = 0.63661977 in, D = 1.9098593 in, φ = 2.9289859 rad, L = 16.067611 in → 80 teeth, 16 in; teeth in mesh floor(10 × 2.9289859/2π) = 4, so Kz = 0.6 (Table 38); V = 500 ft/min; dr = 1.9098593, HP = dr(0.0916 − 7.07e-5 dr²) = 0.17445059 hp at the 0.38 in width (Kw 1); Ha = 0.10467035 hp against Hd = 0.05 × 1.4 = 0.07 hp"
inputs = { p = "0.2 in", N_1 = "10", N_2 = "30", N_min = "10", C = "6 in", n = "3000 rpm", H_nom = "0.05 hp", K_s = "1.4", a_r = "0.0916", b_r = "7.07e-5", b_w = "0.38 in", K_w = "1", K_z = "0.6" }
expect = { d = "0.63661977 in", "φ" = "2.9289859", L = "16.067611 in", N_L = "80", L_p = "16 in", N_m = "4", V = "500 ft/min", H_r = "0.17445059 hp", H_a = "0.10467035 hp", H_d = "0.07 hp" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The XL drive with Kz left at 1: four teeth in mesh need Table 38's reduction"
source = "As the XL hand check with Kz = 1: under six teeth in mesh the full rating does not hold"
inputs = { p = "0.2 in", N_1 = "10", N_2 = "30", N_min = "10", C = "6 in", n = "3000 rpm", H_nom = "0.05 hp", K_s = "1.4", a_r = "0.0916", b_r = "7.07e-5", b_w = "0.38 in", K_w = "1", K_z = "1" }
expect = { N_m = "4", H_a = "0.17445059 hp" }
tolerance = "1e-6"
checks = "fail"
+++

## Timing belt drive

A synchronous (timing) belt with trapezoidal teeth on two toothed pulleys, the small one driving. Shigley (Sec. 17-4) and Juvinall (Sec. 19.4) describe these belts but give no ratings; the rating here is Machinery's Handbook's for the ANSI/RMA IP-24 sections (pp. 2415-2417). The rated power of a section's widest standard belt, with six or more teeth in mesh and up to 6500 ft/min, is HP = d_r (a − b d_r²) hp, with d_r the small pulley's pitch diameter in inches times its speed in thousands of rev/min. A narrower belt carries that times its width factor K_w (Table 40); fewer than six teeth in mesh reduce it by K_z (Table 38). Choose the narrowest standard width whose K_w is at least H_d/(HP K_z).

The constants a and b by section, for the widest width: XL (0.38 in) 0.0916, 7.07 × 10⁻⁵; L (1.00 in) 0.436, 3.01 × 10⁻⁴; H (3.00 in) 3.73, 1.41 × 10⁻³; XH (4.00 in) 7.21, 4.68 × 10⁻³; XXH (5.00 in) 11.4, 7.81 × 10⁻³. The pitches are XL 0.200, L 0.375, H 0.500, XH 0.875 and XXH 1.250 in (MXL, 0.080 in, is rated by torque and is not covered here). Machinery's recommends a service factor of at least 2.0 where the drive may be choked, and flanges on both pulleys when the centre distance is more than eight times the small pulley's diameter.

### Belt and pulleys

```calc {description="Belt pitch (circular pitch of the teeth), Shigley Table 17-18"}
p = 0.5 in_
```

```calc {description="Grooves on the small (faster) pulley"}
N_1 = 24_
```

```calc {description="Grooves on the large pulley"}
N_2 = 48_
```

```calc {description="Fewest grooves stocked for the section, Machinery's Table 36"}
N_min = 14_
```

```calc {description="Centre distance"}
C = 16 in_
```

### Load

```calc {description="Speed of the small (faster) pulley"}
n = 1750 rpm_
```

```calc {description="Nominal power transmitted"}
H_nom = 5 hp_
```

```calc {description="Service factor, Machinery's Table 42"}
K_s = 1.7_
```

### Rating

```calc {description="Rating constant a of the section, Machinery's p. 2416"}
a_r = 3.73_
```

```calc {description="Rating constant b of the section, Machinery's p. 2416"}
b_r = 1.41e-3_
```

```calc {description="Belt width chosen"}
b_w = 1.5 in_
```

```calc {description="Width factor of the width chosen, Machinery's Table 40"}
K_w = 0.45_
```

```calc {description="Teeth-in-mesh factor, Machinery's Table 38 (1 for six or more)"}
K_z = 1_
```

### Geometry

The pitch diameter of a pulley is its grooves times the pitch over π (Table 36). The belt length is Shigley's Eq. 17-2 on the pitch diameters, rounded to a whole number of teeth: order the stock belt of that pitch length (Table 34) and set the centre distance to suit. The speed n is an angular speed, so the belt speed π d n (n in revolutions) is n d / 2 here.

```calc
> Small pulley pitch diameter, Table 36
d = N_1 p / π = ?? in
> Large pulley pitch diameter
D = N_2 p / π = ?? in
> Wrap on the small pulley, Shigley Eq. 17-1
φ = π - 2 asin((D - d) / (2 C)) = ??
> Belt pitch length for this centre distance, Shigley Eq. 17-2
L = sqrt(4 C^2 - (D - d)^2) + (D (2 π - φ) + d φ) / 2 = ?? in
> Teeth on the belt, nearest whole number
N_L = round(L / p) = ??
> Pitch length of that belt
L_p = N_L p = ?? in
> Teeth in mesh on the small pulley
N_m = floor(N_1 φ / (2 π)) = ??
> Belt speed
V = n d / 2 = ?? ft/min
```

### Power

```calc
> Design power
H_d = H_nom K_s = ?? hp
> Pitch diameter (in) times speed (1000 rev/min)
d_r = (d / (1 'in')) (n / (1000 'rpm')) = ??
> Rated power of the widest belt, Machinery's p. 2416
H_r = d_r (a_r - b_r d_r^2) × 1 'hp' = ?? hp
> Rated power of the belt chosen
H_a = H_r K_z K_w = ?? hp
> Rated over design power
n_r = H_a / H_d = ??
```

### Checks

The belt chosen carries the design power; under six teeth in mesh the teeth-in-mesh factor must be reduced; the speed is within the ratings; the belt is no wider than the small pulley's diameter (side thrust, p. 2416); and the small pulley is a stocked size.

```calc
@test H_a ≥ H_d
```

```calc
@test N_m ≥ 6 or K_z < 1
```

```calc
@test V ≤ 6500 'ft/min'
```

```calc
@test b_w ≤ d
```

```calc
@test N_1 ≥ N_min
```
