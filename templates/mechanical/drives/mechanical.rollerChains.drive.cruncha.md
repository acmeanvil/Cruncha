+++
cruncha-template = 1
id = "mechanical.rollerChains.drive"
version = "1.0.0"
title = "Roller chain drive: power rating and geometry (Shigley)"
summary = "An ANSI roller chain drive by Shigley's rating: the link-plate and roller-bushing limited powers of the ACA equations (which set whether the drive runs before or after the extreme and so the tooth factor K1), the allowable power from the tabulated rating with the tooth and strand factors against the design power, chain length in an even number of pitches and the centre distance it gives, sprocket pitch diameters, chain speed, chordal speed variation and rise, chain tension per strand against the strength, and the galling speed limit."
category = ["Mechanical engineering", "Belts and chains"]
tags = ["chain", "roller chain", "ANSI chain", "sprocket", "chain drive", "power rating", "tooth correction", "multiple strand", "chain length", "pitches", "chordal speed variation", "chordal rise", "galling", "Shigley"]
from = "lacewing:mechanical.rollerChains"
license = "GPL-3.0-or-later"
inputs = ["p", "F_u", "H_nom", "n_1", "N_1", "N_2", "C", "K_s", "n_d", "H_tab", "N_s", "K_2", "K_r", "a_p"]
outputs = ["H_1", "H_2", "K_1", "H_a", "H_d", "n_fs", "L_(p,exact)", "N_L", "C_a", "D_1", "D_2", "V", "δ_V", "Δr", "F", "n_u", "n_(1,gall)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 17-5: Eqs. 17-29 to 17-31 (pitch diameter, speed, chordal variation), 17-32, 17-33 (ACA link-plate and roller-bushing powers), 17-34 to 17-36 (length and centre distance), 17-37, 17-38 (allowable and design power), the galling limit of p. 906; Tables 17-19 (chain dimensions and strength), 17-20 (rated power and lubrication), 17-22 (K1), 17-23 (K2); Example 17-5"

[[references]]
work = "Fundamentals of Machine Component Design (Juvinall, Marshek)"
edition = "5th"
locator = "Sec. 19.5, Eq. 19.4 (chordal rise)"

[[tests]]
name = "Shigley Ex. 17-5: 90 hp at 300 rev/min, 17 and 34 teeth, three strands of no. 140"
source = "Lacewing Lab roller_chains_test, after Shigley 10th Example 17-5: Ks = 1.3, nd = 1.5, Hd = 175.5 hp; no. 140 (p = 1.75 in) Htab = 72.4 hp (Table 17-20), K2 = 2.5, Ha = 181 hp; pre-extreme (K1 = 1); C = 25p gives Lp/p = 75.79 → 76 pitches and C = 25.104p = 43.93 in"
inputs = { p = "1.75 in", F_u = "38000 lbf", H_nom = "90 hp", n_1 = "300 rpm", N_1 = "17", N_2 = "34", C = "43.75 in", K_s = "1.3", n_d = "1.5", H_tab = "72.4 hp", N_s = "3", K_2 = "2.5", K_r = "17", a_p = "0.004" }
expect = { H_d = "175.5 hp", K_1 = "1", H_a = "181 hp", "L_(p,exact)" = "75.79", N_L = "76", C_a = "43.93 in" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Shigley p. 905: no. 40 chain, 17 teeth, 1000 rev/min: H1 = 5.48 hp, H2 = 21.64 hp"
source = "Lacewing Lab roller_chains_test, after Shigley 10th p. 905 (Eqs. 17-32, 17-33 for no. 40 at 1000 rev/min): H1 = 5.48 hp, H2 = 21.64 hp, so Table 17-20's 5.48 hp is link-plate limited; here 5 hp nominal on one strand"
inputs = { p = "0.5 in", F_u = "3130 lbf", H_nom = "5 hp", n_1 = "1000 rpm", N_1 = "17", N_2 = "34", C = "20 in", K_s = "1", n_d = "1", H_tab = "5.48 hp", N_s = "1", K_2 = "1", K_r = "17", a_p = "0.004" }
expect = { H_1 = "5.48 hp", H_2 = "21.64 hp", K_1 = "1", H_a = "5.48 hp" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 17-5 exactly"
source = "By hand: H1 = 0.004 × 17^1.08 × 300^0.9 × 1.75^(3 − 0.1225) = 72.392894 hp, H2 = 1000 × 17 × (17/300)^1.5 × 1.75^0.8 = 358.81522 hp; Lp/p = 50 + 25.5 + 289/(100π²) = 75.792818; C/p = 25.104199, C = 43.932348 in; D1 = 1.75/sin(π/17) = 9.5238345 in, D2 = 18.966415 in; V = 17 × 1.75 × 300/12 = 743.75 ft/min; ΔV/V = 0.017124202; rise 0.08108069 in; F = 175.5 × 33 000/743.75/3 = 2595.6303 lbf, Fu/F = 14.63999; galling limit 1000[82.5/(7.95^1.75 × 1.0278^17 × 1.323^2.5956303)]^(1/(1.59 log 1.75 + 1.873)) = 834.81673 rev/min; nfs = 181/117 = 1.5470085"
inputs = { p = "1.75 in", F_u = "38000 lbf", H_nom = "90 hp", n_1 = "300 rpm", N_1 = "17", N_2 = "34", C = "43.75 in", K_s = "1.3", n_d = "1.5", H_tab = "72.4 hp", N_s = "3", K_2 = "2.5", K_r = "17", a_p = "0.004" }
expect = { H_1 = "72.392894 hp", H_2 = "358.81522 hp", "L_(p,exact)" = "75.792818", C_a = "43.932348 in", D_1 = "9.5238345 in", D_2 = "18.966415 in", V = "743.75 ft/min", "δ_V" = "0.017124202", "Δr" = "0.08108069 in", F = "2595.6303 lbf", n_u = "14.63999", "n_(1,gall)" = "834.81673 rpm", n_fs = "1.5470085" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: no. 60 at 2500 rev/min, 19 teeth, after the extreme: too little chain"
source = "By hand: H1 = 0.004 × 17^1.08 × 2500^0.9 × 0.75^2.9475 = 41.76706 hp, H2 = 1000 × 17 × (17/2500)^1.5 × 0.75^0.8 = 7.5728839 hp, so roller-bushing limited and K1 = (19/17)^1.5 = 1.1815634; with Table 17-20's 7.57 hp, Ha = 8.9444347 hp under Hd = 10 hp; Lp/p = 108.72861 → 110, C = 30.478118 in"
inputs = { p = "0.75 in", F_u = "7030 lbf", H_nom = "10 hp", n_1 = "2500 rpm", N_1 = "19", N_2 = "38", C = "30 in", K_s = "1", n_d = "1", H_tab = "7.57 hp", N_s = "1", K_2 = "1", K_r = "17", a_p = "0.004" }
expect = { H_1 = "41.76706 hp", H_2 = "7.5728839 hp", K_1 = "1.1815634", H_a = "8.9444347 hp", N_L = "110", C_a = "30.478118 in" }
tolerance = "1e-6"
checks = "fail"
+++

## Roller chain drive

A roller chain on two sprockets, the small one driving, rated by Shigley's method (Sec. 17-5). Table 17-20's power ratings are for a single strand on a 17-tooth sprocket, 100 pitches long, for 15 000 h. They come from the American Chain Association's two equations: at lower speeds the link plates' fatigue limits the chain (Eq. 17-32), at higher speeds the impact of rollers and bushings (Eq. 17-33). Which one governs, the drive being before or after the "extreme", sets the tooth correction K_1 (Table 17-22): (N_1/17)^1.08 before, (N_1/17)^1.5 after. The allowable power K_1 K_2 H_tab must cover the design power.

Read the rating H_tab off Table 17-20 for the chain number and the small sprocket's speed, with the lubrication it calls for there (type A manual or drip, B bath or disk, C oil stream). The pitch and the ultimate strength are in Table 17-19, the strand factor K_2 in Table 17-23. Shigley recommends at least 17 teeth on the driver (19 or 21 for smoother running), at most 120 on the driven sprocket, speed ratios up to about 6, and centre distances of 30 to 50 pitches.

### Chain

```calc {description="Chain pitch, Shigley Table 17-19"}
p = 1.75 in_
```

```calc {description="Ultimate tensile strength of one strand, Shigley Table 17-19"}
F_u = 38000 lbf_
```

```calc {description="Rated power of one strand on a 17-tooth sprocket, Shigley Table 17-20"}
H_tab = 72.4 hp_
```

```calc {description="Number of strands"}
N_s = 3_
```

```calc {description="Multiple-strand factor, Shigley Table 17-23"}
K_2 = 2.5_
```

```calc {description="Roller-bushing constant Kr of Eq. 17-33: 29 for nos. 25 and 35, 3.4 for no. 41, 17 for nos. 40 to 240"}
K_r = 17_
```

```calc {description="Link-plate constant of Eq. 17-32: 0.004, or 0.0022 for no. 41 lightweight chain"}
a_p = 0.004_
```

### Drive

```calc {description="Nominal power transmitted"}
H_nom = 90 hp_
```

```calc {description="Speed of the small (driving) sprocket"}
n_1 = 300 rpm_
```

```calc {description="Teeth on the small sprocket"}
N_1 = 17_
```

```calc {description="Teeth on the large sprocket"}
N_2 = 34_
```

```calc {description="Centre distance wanted"}
C = 43.75 in_
```

```calc {description="Service factor"}
K_s = 1.3_
```

```calc {description="Design factor"}
n_d = 1.5_
```

### Power rating

```calc
> Pitch in inches
p_in = p / (1 'in') = ??
> Speed in rev/min
n_rpm = n_1 / (1 'rpm') = ??
> Link-plate limited power, 17 teeth, Eq. 17-32
H_1 = a_p 17^1.08 n_rpm^0.9 p_in^(3 - 0.07 p_in) × 1 'hp' = ?? hp
> Roller-bushing limited power, 17 teeth, Eq. 17-33
H_2 = 1000 K_r (17 / n_rpm)^1.5 p_in^0.8 × 1 'hp' = ?? hp
> Tooth correction factor, Table 17-22
K_1 = {(N_1 / 17)^1.08 if H_2 ≥ H_1; (N_1 / 17)^1.5 otherwise} = ??
> Allowable power, Eq. 17-37
H_a = K_1 K_2 H_tab = ?? hp
> Design power, Eq. 17-38
H_d = H_nom K_s n_d = ?? hp
> Factor of safety
n_fs = H_a / (H_nom K_s) = ??
```

### Length and centre distance

The chain length is Eq. 17-34 rounded up to an even number of pitches (an odd number needs an offset link); the centre distance for that chain is Eqs. 17-35 and 17-36.

```calc
> Chain length in pitches for C, Eq. 17-34
L_(p,exact) = 2 C / p + (N_1 + N_2) / 2 + (N_2 - N_1)^2 / (4 π^2 C / p) = ??
> Pitches, rounded up to an even number
N_L = 2 ceil(L_(p,exact) / 2) = ??
> Eq. 17-36
A_c = (N_1 + N_2) / 2 - N_L = ??
> Centre distance for that chain, Eq. 17-35
C_a = p / 4 (-A_c + sqrt(A_c^2 - 8 ((N_2 - N_1) / (2 π))^2)) = ?? in
```

### Sprockets, speed and tension

The speed n_1 is an angular speed, so the chain speed N p n (n in revolutions) is N p n_1/(2π) here. The tension is the design power over the chain speed, shared by the strands; chains seldom fail in tension, but the margin against Table 17-19's strength is shown. The galling limit is the speed above which pins and bushings gall at this tension (p. 906).

```calc
> Small sprocket pitch diameter, Eq. 17-29
D_1 = p / sin(π / N_1) = ?? in
> Large sprocket pitch diameter, Eq. 17-29
D_2 = p / sin(π / N_2) = ?? in
> Chain speed, Eq. 17-30
V = N_1 p n_1 / (2 π) = ?? ft/min
> Chordal speed variation ΔV/V, Eq. 17-31
δ_V = π / N_1 (1 / sin(π / N_1) - 1 / tan(π / N_1)) = ??
> Chordal rise of the chain line, Juvinall Eq. 19.4
Δr = D_1 / 2 (1 - cos(π / N_1)) = ?? in
> Tension per strand
F = H_d / (V N_s) = ?? lbf
> Strength over tension
n_u = F_u / F = ??
> Galling speed limit, p. 906
n_(1,gall) = 1000 (82.5 / (7.95^p_in 1.0278^N_1 1.323^(F / (1000 'lbf'))))^(1 / (1.59 log10(p_in) + 1.873)) × 1 'rpm' = ?? rpm
```

### Checks

The chain carries the design power, runs below its galling speed, and the sprockets are within Shigley's recommended tooth counts.

```calc
@test H_a ≥ H_d
```

```calc
@test n_1 ≤ n_(1,gall)
```

```calc
@test N_1 ≥ 17
```

```calc
@test N_2 ≤ 120
```
