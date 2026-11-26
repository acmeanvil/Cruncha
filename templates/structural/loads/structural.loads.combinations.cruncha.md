+++
cruncha-template = 1
id = "structural.loads.combinations"
version = "1.0.0"
title = "Load combinations, ASCE 7-10 (LRFD and ASD)"
summary = "The basic strength (LRFD) and allowable stress (ASD) load combinations of ASCE 7-10 for dead, live, roof live, snow, rain, wind and earthquake effects: the largest and least value of each combination, and the governing largest and least of each method."
category = ["Structural engineering", "Loads"]
tags = ["ASCE 7", "ASCE 7-10", "load combinations", "LRFD", "ASD", "strength design", "allowable stress design", "dead", "live", "snow", "wind", "earthquake", "uplift", "overturning"]
from = "lacewing:structural.loadCombinations"
license = "GPL-3.0-or-later"
inputs = ["D", "L", "L_r", "S", "R", "W", "E", "f_L"]
outputs = ["U_1", "U_(2,max)", "U_(3,max)", "U_(4,max)", "U_(5,max)", "U_(6,max)", "U_(7,max)", "U_(2,min)", "U_(3,min)", "U_(4,min)", "U_(5,min)", "U_(6,min)", "U_(7,min)", "U_LRFD", "U_(LRFD,min)", "U_(a1)", "U_(a2,max)", "U_(a3,max)", "U_(a4,max)", "U_(a5,max)", "U_(a6a,max)", "U_(a6b,max)", "U_(a7,max)", "U_(a8,max)", "U_(a2,min)", "U_(a3,min)", "U_(a4,min)", "U_(a5,min)", "U_(a6a,min)", "U_(a6b,min)", "U_(a7,min)", "U_(a8,min)", "U_ASD", "U_(ASD,min)"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 2.3.2 (strength design combinations 1 to 7, exceptions 1 and 2), Sec. 2.4.1 (allowable stress design combinations 1 to 8); Commentary C2.3.2, C2.4.1"

[[tests]]
name = "D 10, L 5, Lr 1, S 3, W 4 (no rain or earthquake)"
source = "Lacewing Lab asce7_loads_test, by hand from Sec. 2.3.2 and 2.4.1 (checked against the ASCE 7-10 PDF); the largest values of every combination also agree with the owner's own spreadsheet, Calcs/LRFD and ASD Load combination calculator.xlsx (ASCE 7-10): LRFD 2 = 12 + 8 + 1.5 = 21.5; 3 = 12 + 4.8 + 5 = 21.8 (L beats 0.5W); 4 = 12 + 4 + 5 + 1.5 = 22.5; 6 = 0.9D ± W = 13 and 5; ASD 4 = 10 + 3.75 + 2.25 = 16; 6a = 17.8; 7 = 6 + 2.4 = 8.4. Governing and least values by hand from the same numbers."
inputs = { D = "10 kips", L = "5 kips", L_r = "1 kips", S = "3 kips", R = "0 kips", W = "4 kips", E = "0 kips", f_L = "1" }
expect = { U_1 = "14 kips", "U_(2,max)" = "21.5 kips", "U_(3,max)" = "21.8 kips", "U_(4,max)" = "22.5 kips", "U_(6,max)" = "13 kips", "U_(6,min)" = "5 kips", U_LRFD = "22.5 kips", "U_(LRFD,min)" = "5 kips", "U_(a4,max)" = "16 kips", "U_(a6a,max)" = "17.8 kips", "U_(a7,max)" = "8.4 kips", "U_(a7,min)" = "3.6 kips", U_ASD = "17.8 kips", "U_(ASD,min)" = "3.6 kips" }
tolerance = "1e-9"

[[tests]]
name = "The same loads with the 0.5L exception"
source = "Lacewing Lab asce7_loads_test: with 0.5L in combination 4, 12 + 4 + 2.5 + 1.5 = 20.0; by hand, combination 3 becomes 12 + 4.8 + max(2.5, 2) = 19.3 and 5 becomes 12 + 2.5 + 0.6 = 15.1"
inputs = { D = "10 kips", L = "5 kips", L_r = "1 kips", S = "3 kips", R = "0 kips", W = "4 kips", E = "0 kips", f_L = "0.5" }
expect = { "U_(4,max)" = "20 kips", "U_(3,max)" = "19.3 kips", "U_(5,max)" = "15.1 kips", "U_(2,max)" = "21.5 kips", U_LRFD = "21.5 kips" }
tolerance = "1e-9"

[[tests]]
name = "Uplift: a light roof with wind reversing it"
source = "Lacewing Lab asce7_loads_test (the owner's spreadsheet takes W one way only, so it has no uplift case to compare): D 2, W 10: ASD 7 least 0.6 × 2 − 0.6 × 10 = −4.8, ASD 8 (no earthquake) 1.2; by hand, LRFD 6 least 0.9 × 2 − 10 = −8.2"
inputs = { D = "2 kips", L = "0 kips", L_r = "0 kips", S = "0 kips", R = "0 kips", W = "10 kips", E = "0 kips", f_L = "1" }
expect = { "U_(a7,min)" = "-4.8 kips", "U_(a8,max)" = "1.2 kips", "U_(ASD,min)" = "-4.8 kips", "U_(LRFD,min)" = "-8.2 kips" }
tolerance = "1e-9"

[[tests]]
name = "Lacewing's opening values: D 100, L 60, Lr 20, S 30, W 40"
source = "Lacewing Lab structural_screens_test, and the same largest values from the owner's spreadsheet Calcs/LRFD and ASD Load combination calculator.xlsx: LRFD 2 = 120 + 96 + 15 = 231, ASD 2 = 160; by hand, LRFD 4 = 120 + 40 + 60 + 15 = 235 governs, and ASD 6a = 100 + 45 + 18 + 22.5 = 185.5 governs"
inputs = { D = "100 kips", L = "60 kips", L_r = "20 kips", S = "30 kips", R = "0 kips", W = "40 kips", E = "0 kips", f_L = "1" }
expect = { "U_(2,max)" = "231 kips", "U_(a2,max)" = "160 kips", U_LRFD = "235 kips", U_ASD = "185.5 kips" }
tolerance = "1e-9"

[[tests]]
name = "Earthquake: D 100, L 50, S 20, E 30"
source = "By hand, Sec. 2.3.2 and 2.4.1; the largest values agree with the owner's spreadsheet Calcs/LRFD and ASD Load combination calculator.xlsx (it has no least values): LRFD 5 = 120 + 30 + 50 + 4 = 204, least 120 − 30 = 90; LRFD 7 least 90 − 30 = 60; ASD 6b = 100 + 37.5 + 15.75 + 15 = 168.25; ASD 8 least 60 − 21 = 39"
inputs = { D = "100 kips", L = "50 kips", L_r = "0 kips", S = "20 kips", R = "0 kips", W = "0 kips", E = "30 kips", f_L = "1" }
expect = { "U_(5,max)" = "204 kips", "U_(5,min)" = "90 kips", "U_(7,min)" = "60 kips", "U_(a6b,max)" = "168.25 kips", "U_(a8,min)" = "39 kips", "U_(LRFD,min)" = "60 kips", "U_(ASD,min)" = "39 kips" }
tolerance = "1e-9"
+++

## Load combinations, ASCE 7-10

The basic combinations of ASCE 7-10 for strength design (LRFD, Sec. 2.3.2) and for allowable stress design (ASD, Sec. 2.4.1), applied to the effects of the nominal loads at one point: a force, a moment or a stress. Each combination gives its largest and its least value. Use the largest for strength checks (against ϕR_n in LRFD, R_n/Ω in ASD) and the least for uplift, overturning and stress reversal: a negative least value means the net effect reverses.

The standard requires the cases where one or more loads are not acting to be investigated, so every variable load may be absent. A term written "(L_r or S or R)" takes whichever one of the three, or none, makes the value largest (and again least). Wind and earthquake act in either direction, so they enter as +W and −W; they never act together.

Not included: fluid loads F, lateral earth pressure H, flood, atmospheric ice and self-straining loads, and the special seismic combinations with overstrength (Sec. 12.4.3). The ASD exceptions for nonbuilding foundations and special reinforced masonry shear walls are not applied.

### Load effects

Type each nominal load effect in the same unit. The results are shown in kips; for moments or stresses, change the inputs and the display units together.

```calc {description="Dead load effect"}
D = 100 kips_
```

```calc {description="Live load effect (reduced where Chapter 4 allows)"}
L = 60 kips_
```

```calc {description="Roof live load effect"}
L_r = 20 kips_
```

```calc {description="Snow load effect (balanced, pf or ps, as a companion load: exception 2)"}
S = 30 kips_
```

```calc {description="Rain load effect"}
R = 0 kips_
```

```calc {description="Wind load effect, strength level (ASCE 7-10 wind speeds)"}
W = 40 kips_
```

```calc {description="Earthquake load effect E of Sec. 12.4, strength level"}
E = 0 kips_
```

Exception 1 of Sec. 2.3.2 lets the factor on L in combinations 3, 4 and 5 be 0.5 where the uniform live load L_o of Table 4-1 is 100 psf or less, except in garages and places of public assembly. Type 1 otherwise.

```calc {description="Factor on L in LRFD combinations 3, 4 and 5 (1, or 0.5 by exception 1)"}
f_L = 1_
```

### Strength design (LRFD), Sec. 2.3.2

`0 D` below stands for a load that is absent (zero, in the loads' unit).

```calc {description="LRFD 1: 1.4D"}
U_1 = 1.4 D = ?? kips
```

```calc
> LRFD 2: 1.2D + 1.6L + 0.5(Lr or S or R), largest
U_(2,max) = 1.2 D + max(0 D, 1.6 L) + max(0 D, 0.5 L_r, 0.5 S, 0.5 R) = ?? kips
> LRFD 2, least
U_(2,min) = 1.2 D + min(0 D, 1.6 L) + min(0 D, 0.5 L_r, 0.5 S, 0.5 R) = ?? kips
```

```calc
> LRFD 3: 1.2D + 1.6(Lr or S or R) + (L or 0.5W), largest
U_(3,max) = 1.2 D + max(0 D, 1.6 L_r, 1.6 S, 1.6 R) + max(0 D, f_L L, 0.5 abs(W)) = ?? kips
> LRFD 3, least
U_(3,min) = 1.2 D + min(0 D, 1.6 L_r, 1.6 S, 1.6 R) + min(0 D, f_L L, -0.5 abs(W)) = ?? kips
```

```calc
> LRFD 4: 1.2D + 1.0W + L + 0.5(Lr or S or R), largest
U_(4,max) = 1.2 D + abs(W) + max(0 D, f_L L) + max(0 D, 0.5 L_r, 0.5 S, 0.5 R) = ?? kips
> LRFD 4, least
U_(4,min) = 1.2 D - abs(W) + min(0 D, f_L L) + min(0 D, 0.5 L_r, 0.5 S, 0.5 R) = ?? kips
```

```calc
> LRFD 5: 1.2D + 1.0E + L + 0.2S, largest
U_(5,max) = 1.2 D + abs(E) + max(0 D, f_L L) + max(0 D, 0.2 S) = ?? kips
> LRFD 5, least
U_(5,min) = 1.2 D - abs(E) + min(0 D, f_L L) + min(0 D, 0.2 S) = ?? kips
```

```calc
> LRFD 6: 0.9D + 1.0W, largest
U_(6,max) = 0.9 D + abs(W) = ?? kips
> LRFD 6, least
U_(6,min) = 0.9 D - abs(W) = ?? kips
```

```calc
> LRFD 7: 0.9D + 1.0E, largest
U_(7,max) = 0.9 D + abs(E) = ?? kips
> LRFD 7, least
U_(7,min) = 0.9 D - abs(E) = ?? kips
```

```calc {description="LRFD governing (largest) load effect"}
U_LRFD = max(U_1, U_(2,max), U_(3,max), U_(4,max), U_(5,max), U_(6,max), U_(7,max)) = ?? kips
```

```calc {description="LRFD least load effect (uplift, overturning, reversal)"}
U_(LRFD,min) = min(U_1, U_(2,min), U_(3,min), U_(4,min), U_(5,min), U_(6,min), U_(7,min)) = ?? kips
```

### Allowable stress design (ASD), Sec. 2.4.1

```calc {description="ASD 1: D"}
U_(a1) = D = ?? kips
```

```calc
> ASD 2: D + L, largest
U_(a2,max) = D + max(0 D, L) = ?? kips
> ASD 2, least
U_(a2,min) = D + min(0 D, L) = ?? kips
```

```calc
> ASD 3: D + (Lr or S or R), largest
U_(a3,max) = D + max(0 D, L_r, S, R) = ?? kips
> ASD 3, least
U_(a3,min) = D + min(0 D, L_r, S, R) = ?? kips
```

```calc
> ASD 4: D + 0.75L + 0.75(Lr or S or R), largest
U_(a4,max) = D + max(0 D, 0.75 L) + max(0 D, 0.75 L_r, 0.75 S, 0.75 R) = ?? kips
> ASD 4, least
U_(a4,min) = D + min(0 D, 0.75 L) + min(0 D, 0.75 L_r, 0.75 S, 0.75 R) = ?? kips
```

```calc
> ASD 5: D + (0.6W or 0.7E), largest
U_(a5,max) = D + max(0 D, 0.6 abs(W), 0.7 abs(E)) = ?? kips
> ASD 5, least
U_(a5,min) = D + min(0 D, -0.6 abs(W), -0.7 abs(E)) = ?? kips
```

```calc
> ASD 6a: D + 0.75L + 0.75(0.6W) + 0.75(Lr or S or R), largest
U_(a6a,max) = D + max(0 D, 0.75 L) + 0.45 abs(W) + max(0 D, 0.75 L_r, 0.75 S, 0.75 R) = ?? kips
> ASD 6a, least
U_(a6a,min) = D + min(0 D, 0.75 L) - 0.45 abs(W) + min(0 D, 0.75 L_r, 0.75 S, 0.75 R) = ?? kips
```

```calc
> ASD 6b: D + 0.75L + 0.75(0.7E) + 0.75S, largest
U_(a6b,max) = D + max(0 D, 0.75 L) + 0.525 abs(E) + max(0 D, 0.75 S) = ?? kips
> ASD 6b, least
U_(a6b,min) = D + min(0 D, 0.75 L) - 0.525 abs(E) + min(0 D, 0.75 S) = ?? kips
```

```calc
> ASD 7: 0.6D + 0.6W, largest
U_(a7,max) = 0.6 D + 0.6 abs(W) = ?? kips
> ASD 7, least
U_(a7,min) = 0.6 D - 0.6 abs(W) = ?? kips
```

```calc
> ASD 8: 0.6D + 0.7E, largest
U_(a8,max) = 0.6 D + 0.7 abs(E) = ?? kips
> ASD 8, least
U_(a8,min) = 0.6 D - 0.7 abs(E) = ?? kips
```

```calc {description="ASD governing (largest) load effect"}
U_ASD = max(U_(a1), U_(a2,max), U_(a3,max), U_(a4,max), U_(a5,max), U_(a6a,max), U_(a6b,max), U_(a7,max), U_(a8,max)) = ?? kips
```

```calc {description="ASD least load effect (uplift, overturning, reversal)"}
U_(ASD,min) = min(U_(a1), U_(a2,min), U_(a3,min), U_(a4,min), U_(a5,min), U_(a6a,min), U_(a6b,min), U_(a7,min), U_(a8,min)) = ?? kips
```
