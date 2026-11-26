+++
cruncha-template = 1
id = "structural.seismic.baseShear"
version = "1.0.0"
title = "Seismic base shear, ASCE 7-10 equivalent lateral force"
summary = "Seismic base shear by the equivalent lateral force procedure of ASCE 7-10: design spectral accelerations SDS and SD1, the seismic design category, the approximate period and the period used, the response coefficient Cs with its limits, and V = Cs W."
category = ["Structural engineering", "Loads"]
tags = ["ASCE 7", "ASCE 7-10", "seismic", "earthquake", "base shear", "equivalent lateral force", "ELF", "SDS", "SD1", "seismic design category", "SDC", "Cs", "period", "Ta", "response modification", "R"]
from = "lacewing:structural.seismic"
license = "GPL-3.0-or-later"
inputs = ["S_S", "S_1", "F_a", "F_v", "T_L", "k_E", "k_IV", "I_e", "R", "C_t", "x", "h_n", "C_u", "T_an", "W"]
outputs = ["S_MS", "S_M1", "S_DS", "S_D1", "n_SDC", "SDC", "T_a", "T", "C_(s,2)", "C_(s,max)", "C_(s,min)", "C_s", "V", "k"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 11.4.3, 11.4.4 (Eqs. 11.4-1 to 11.4-4, Tables 11.4-1, 11.4-2); Sec. 11.6 (Tables 11.6-1, 11.6-2); Table 1.5-2; Table 12.2-1; Sec. 12.8.1 (Eqs. 12.8-1 to 12.8-6); Sec. 12.8.2 (Eq. 12.8-7, Tables 12.8-1, 12.8-2); Sec. 12.8.3 (k)"

[[references]]
work = "2018 IBC SEAOC Structural/Seismic Design Manual, Vol. 1 (Code Application Examples; ASCE 7-16)"
edition = "2018"
locator = "Design Example 30 (seismic base shear), Example 31 (approximate period)"

[[tests]]
name = "Lacewing's opening building: 60 ft steel special moment frame, site D"
source = "Lacewing Lab help and asce7_loads_test (no published example; checked by hand against ASCE 7-10, the library copy, which is the same file as Lacewing's): SS 1.5, S1 0.6, site D (Fa 1.0, Fv 1.5, Tables 11.4-1 and 11.4-2, checked against the PDF), SDS 1.000, SD1 0.600, category D; Ta = 0.028 × 60^0.8 = 0.74076 s; Cs = SD1/(T R/Ie) = 0.6/(0.74076 × 8) = 0.10125 (Eq. 12.8-3); V = 0.10125 × 5000 = 506.2 kN (Lacewing: 'about 506 kN')"
inputs = { S_S = "1.5", S_1 = "0.6", F_a = "1.0", F_v = "1.5", T_L = "8 s", k_IV = "0", I_e = "1.0", R = "8", C_t = "0.028", x = "0.8", h_n = "60 ft", C_u = "1.4", T_an = "0 s", W = "5000 kN" }
expect = { S_DS = "1.0", S_D1 = "0.6", n_SDC = "4", T_a = "0.740763 s", T = "0.740763 s", C_s = "0.101247", V = "506.24 kN", k = "1.120382" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "The same building with a period from analysis, capped at Cu Ta"
source = "By hand, Sec. 12.8.2: T from analysis 2.0 s > Cu Ta = 1.4 × 0.74076 = 1.03707 s, so T = 1.03707 s; Eq. 12.8-3 gives 0.6/(1.03707 × 8) = 0.07232, Eq. 12.8-6 0.5 × 0.6/8 = 0.0375, Eq. 12.8-5 0.044 × 1.0 × 1.0 = 0.044; Cs = 0.07232, V = 361.6 kN"
inputs = { S_S = "1.5", S_1 = "0.6", F_a = "1.0", F_v = "1.5", T_L = "8 s", k_IV = "0", I_e = "1.0", R = "8", C_t = "0.028", x = "0.8", h_n = "60 ft", C_u = "1.4", T_an = "2.0 s", W = "5000 kN" }
expect = { T = "1.037068 s", "C_(s,max)" = "0.072319", "C_(s,min)" = "0.044", C_s = "0.072319", V = "361.60 kN" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Long period, low seismicity: the 0.01 floor governs"
source = "Lacewing Lab asce7_loads_test: SDS 0.2, SD1 0.1, R 8, Ie 1, T 5 s > TL 4 s: Eq. 12.8-4 gives 0.1 × 4/(25 × 8) = 0.002, Eq. 12.8-5 0.044 × 0.2 = 0.0088, so Cs = 0.01. Here SS 0.3 and S1 0.15 with Fa = Fv = 1 give SDS 0.2 and SD1 0.1; hn 500 ft makes Cu Ta = 1.7 × 4.04 s, above the 5 s analysis period; category B by hand"
inputs = { S_S = "0.3", S_1 = "0.15", F_a = "1.0", F_v = "1.0", T_L = "4 s", k_IV = "0", I_e = "1.0", R = "8", C_t = "0.028", x = "0.8", h_n = "500 ft", C_u = "1.7", T_an = "5 s", W = "10000 kN" }
expect = { S_DS = "0.2", S_D1 = "0.1", T = "5 s", "C_(s,max)" = "0.002", C_s = "0.01", V = "100 kN", n_SDC = "2", k = "2" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Near-fault site, S1 ≥ 0.6: Eq. 12.8-6 governs, category E"
source = "Lacewing Lab asce7_loads_test: SDS 1.0, SD1 0.9, S1 1.2, R 8, Ie 1, T 4 s = TL: Eq. 12.8-3 gives 0.9/(4 × 8) = 0.0281, Eq. 12.8-6 0.5 × 1.2/8 = 0.075 governs; S1 ≥ 0.75 puts risk category II in category E (Sec. 11.6). Fv = 1.125 is chosen to give SD1 = 0.9."
inputs = { S_S = "1.5", S_1 = "1.2", F_a = "1.0", F_v = "1.125", T_L = "4 s", k_IV = "0", I_e = "1.0", R = "8", C_t = "0.028", x = "0.8", h_n = "500 ft", C_u = "1.4", T_an = "4 s", W = "1000 kips" }
expect = { S_D1 = "0.9", "C_(s,min)" = "0.075", C_s = "0.075", V = "75 kips", n_SDC = "5" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Risk category IV at low SDS: category C"
source = "Lacewing Lab asce7_loads_test: SDS 0.2, SD1 0.05, S1 0.1, risk category IV → C (Table 11.6-1: 0.167 ≤ SDS < 0.33 is C for IV; Table 11.6-2: SD1 < 0.067 is A). SS 0.3, S1 0.075, Fa = Fv = 1; Ie 1.5 for risk category IV (Table 1.5-2). By hand, a 30 ft bearing-wall building (Ct 0.02, x 0.75): Ta = 0.02 × 30^0.75 = 0.25637 s; Eq. 12.8-2 gives 0.2/(5/1.5) = 0.06 but Eq. 12.8-3 caps it at 0.05/(0.25637 × 5/1.5) = 0.058509, so V = 0.058509 × 2000 = 117.02 kips"
inputs = { S_S = "0.3", S_1 = "0.075", F_a = "1.0", F_v = "1.0", T_L = "8 s", k_IV = "1", I_e = "1.5", R = "5", C_t = "0.02", x = "0.75", h_n = "30 ft", C_u = "1.7", T_an = "0 s", W = "2000 kips" }
expect = { n_SDC = "3", T_a = "0.256372 s", C_s = "0.058509", V = "117.02 kips", k = "1" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Site class E with SS ≥ 1.0: the caution check fails"
source = "By hand: site class E (Fa 0.9 at SS ≥ 1.0, Table 11.4-1, checked against the PDF), SS 1.2, S1 0.5 (Fv 2.4): SDS = 2/3 × 0.9 × 1.2 = 0.72, SD1 = 2/3 × 2.4 × 0.5 = 0.8; Sec. 11.4.7 may require a site response analysis, so the @test on the site flags it"
inputs = { S_S = "1.2", S_1 = "0.5", F_a = "0.9", F_v = "2.4", T_L = "8 s", k_IV = "0", I_e = "1.0", R = "8", C_t = "0.028", x = "0.8", h_n = "60 ft", C_u = "1.4", T_an = "0 s", W = "5000 kN", k_E = "1" }
expect = { S_DS = "0.72", S_D1 = "0.8" }
tolerance = "1e-9"
checks = "fail"
[[tests]]
name = "SEAOC Design Example 30: five-storey steel SMF, SDS 0.45, SD1 0.28"
source = "2018 IBC SEAOC Structural/Seismic Design Manual Vol. 1 (ASCE 7-16), Design Example 30, verified against the PDF: hn 60 ft, Ct 0.028, x 0.8, Ta = 0.74 s; Eq. 12.8-2 0.0562, Eq. 12.8-3 0.047, Eq. 12.8-5 0.0198; SDC D; W 1626 kips; S1 0.21 (its commentary). The example then uses 0.0562 (V = 91.4 kips) under ASCE 7-16's Sec. 11.4.8 exception 2, which ASCE 7-10 doesn't have; under ASCE 7-10 Eq. 12.8-3 caps Cs, so Cs = 0.04725 and V = 0.04725 × 1626 = 76.8 kips (by hand). SS 0.675 with Fa 1.0 and Fv 2.0 with S1 0.21 are chosen to give the example's SDS and SD1."
inputs = { S_S = "0.675", S_1 = "0.21", F_a = "1.0", F_v = "2.0", T_L = "8 s", k_IV = "0", I_e = "1.0", R = "8", C_t = "0.028", x = "0.8", h_n = "60 ft", C_u = "1.4", T_an = "0 s", W = "1626 kips" }
expect = { S_DS = "0.45", S_D1 = "0.28", n_SDC = "4", T_a = "0.74 s", "C_(s,2)" = "0.0562", "C_(s,max)" = "0.047", "C_(s,min)" = "0.0198", C_s = "0.04725", V = "76.83 kips" }
tolerance = "0.6%"
checks = "pass"
[[tests]]
name = "SEAOC Design Example 31: approximate period of a 33 ft concrete special moment frame"
source = "2018 IBC SEAOC Structural/Seismic Design Manual Vol. 1, Design Example 31 part 2, verified against the PDF: Ct 0.016, x 0.9, hn 33 ft, Ta = 0.016 × 33^0.9 = 0.37 s (Eq. 12.8-7, unchanged from ASCE 7-10; unrounded 0.3722 s, hence the 1% tolerance). The other inputs are the opening building's."
inputs = { S_S = "1.5", S_1 = "0.6", F_a = "1.0", F_v = "1.5", T_L = "8 s", k_IV = "0", I_e = "1.0", R = "8", C_t = "0.016", x = "0.9", h_n = "33 ft", C_u = "1.4", T_an = "0 s", W = "5000 kN" }
expect = { T_a = "0.37 s" }
tolerance = "1%"
checks = "pass"
+++

## Seismic base shear, ASCE 7-10

The seismic base shear of a building by the equivalent lateral force (ELF) procedure of ASCE 7-10, Chapters 11 and 12: the design spectral accelerations from the mapped values and the site, the seismic design category, the fundamental period, the seismic response coefficient C_s with its upper and lower limits, and V = C_s W. V is at strength level: the load combinations take 1.0E (LRFD) or 0.7E (ASD). The vertical distribution template spreads V over the height.

Check first that Table 12.6-1 permits the ELF procedure for the structure, and that Table 12.2-1 allows the system, at its height, in the design category. This follows ASCE 7-10. ASCE 7-16 keeps the Sec. 12.8 equations but changes the site coefficient tables, requires site-specific analysis in more cases (Sec. 11.4.8, whose exceptions can change which Cs equation governs) and adds a cap on S_S (Sec. 12.8.1.3). Not covered: site class F and site response analysis (Sec. 11.4.7, 21), the 1.5 cap on S_S of Sec. 12.8.1.3, soil-structure interaction, the alternative period formulas (Eqs. 12.8-8, 12.8-9), redundancy ρ and overstrength Ω₀, drift (C_d), and torsion.

### Site

The mapped risk-targeted MCE_R accelerations at 0.2 s and 1 s for site class B, in g, are from Figs. 22-1 and 22-2 (or the USGS data for the site), and the long-period transition period from Figs. 22-12 to 22-16. The site class (A to E) is from the soil profile by Chapter 20; use D where the soil isn't known well enough (Sec. 11.4.2). The site coefficients F_a and F_v are from Tables 11.4-1 and 11.4-2 by site class, interpolated in S_S and S_1.

```calc {description="Mapped MCE_R spectral acceleration at short periods, Fig. 22-1 (g)"}
S_S = 1.5_
```

```calc {description="Mapped MCE_R spectral acceleration at 1 s, Fig. 22-2 (g)"}
S_1 = 0.6_
```

```calc {description="Short-period site coefficient Fa, Table 11.4-1"}
F_a = 1.0_
```

```calc {description="Long-period site coefficient Fv, Table 11.4-2"}
F_v = 1.5_
```

```calc {description="Long-period transition period, Fig. 22-12"}
T_L = 8 s_
```

Site class E with S_S of 1.0 or more may need a site response analysis (Sec. 11.4.7): flag it here.

```calc {description="Site: 1 for site class E, 0 otherwise"}
k_E = 0_
```

### Building

The risk category (Table 1.5-1) sets the importance factor I_e of Table 1.5-2 (1.0 for risk categories I and II), and risk category IV uses the stricter column of the design category tables. The seismic force-resisting system sets R in Table 12.2-1, and C_t and x for the approximate period in Table 12.8-2 (C_t here is the coefficient for h_n in ft).

```calc {description="Risk category: 1 for IV, 0 for I to III"}
k_IV = 0_
```

```calc {description="Seismic importance factor Ie, Table 1.5-2"}
I_e = 1.0_
```

```calc {description="Response modification coefficient R, Table 12.2-1"}
R = 8_
```

```calc {description="Period coefficient Ct (hn in ft), Table 12.8-2"}
C_t = 0.028_
```

```calc {description="Period exponent x, Table 12.8-2"}
x = 0.8_
```

```calc {description="Structural height hn, Sec. 11.2"}
h_n = 60 ft_
```

C_u limits a period from analysis (Table 12.8-1 by S_D1: 1.4 for S_D1 ≥ 0.3, rising to 1.7 at 0.1 and below). Type the analysis period, or 0 s to use T_a.

```calc {description="Coefficient for upper limit on calculated period Cu, Table 12.8-1"}
C_u = 1.4_
```

```calc {description="Fundamental period from a substantiated analysis (0 s: use Ta)"}
T_an = 0 s_
```

```calc {description="Effective seismic weight W, Sec. 12.7.2"}
W = 5000 kN_
```

### Design spectral accelerations

```calc
> MCE_R spectral acceleration at short periods, Eq. 11.4-1
S_MS = F_a S_S = ??
> MCE_R spectral acceleration at 1 s, Eq. 11.4-2
S_M1 = F_v S_1 = ??
> Design spectral acceleration at short periods, Eq. 11.4-3
S_DS = 2/3 S_MS = ??
> Design spectral acceleration at 1 s, Eq. 11.4-4
S_D1 = 2/3 S_M1 = ??
```

### Seismic design category

Each table gives a category by risk category and one design acceleration; the more severe governs (Sec. 11.6). Where S_1 is 0.75 or more, the category is E (F for risk category IV) whatever the tables say. The categories are numbered A = 1 to F = 6.

```calc
> Category by S_DS, Table 11.6-1
n_DS = {1 if S_DS < 0.167; 2 + k_IV if S_DS < 0.33; 3 + k_IV if S_DS < 0.50; 4 otherwise} = ??
> Category by S_D1, Table 11.6-2
n_D1 = {1 if S_D1 < 0.067; 2 + k_IV if S_D1 < 0.133; 3 + k_IV if S_D1 < 0.20; 4 otherwise} = ??
> Seismic design category, A = 1 … F = 6
n_SDC = {5 + k_IV if S_1 ≥ 0.75; max(n_DS, n_D1) otherwise} = ??
```

```calc {description="Seismic design category"}
SDC = {"A" if n_SDC == 1; "B" if n_SDC == 2; "C" if n_SDC == 3; "D" if n_SDC == 4; "E" if n_SDC == 5; "F" otherwise} = ?
```

### Period

```calc {description="Approximate fundamental period, Eq. 12.8-7"}
T_a = C_t (h_n / (1 'ft'))^x × 1 's' = ?? s
```

```calc {description="Period used: Ta, or the analysis period not above Cu Ta (Sec. 12.8.2)"}
T = {T_a if T_an ≤ 0 's'; min(T_an, C_u T_a) otherwise} = ?? s
```

### Seismic response coefficient and base shear

```calc
> Cs, Eq. 12.8-2
C_(s,2) = S_DS / (R / I_e) = ??
> Upper limit, Eq. 12.8-3 (T ≤ T_L) or 12.8-4 (T > T_L)
C_(s,max) = {S_D1 × 1 's' / (T R / I_e) if T ≤ T_L; S_D1 T_L × 1 's' / (T^2 R / I_e) otherwise} = ??
> Lower limit, Eq. 12.8-5, and Eq. 12.8-6 where S_1 ≥ 0.6
C_(s,min) = max(0.044 S_DS I_e, 0.01, {0.5 S_1 / (R / I_e) if S_1 ≥ 0.6; 0 otherwise}) = ??
> Seismic response coefficient
C_s = max(min(C_(s,2), C_(s,max)), C_(s,min)) = ??
```

```calc {description="Seismic base shear, Eq. 12.8-1 (strength level)"}
V = C_s W = ?? kN
```

```calc {description="Distribution exponent k, Sec. 12.8.3"}
k = {1 if T ≤ 0.5 's'; 2 if T ≥ 2.5 's'; 1 + (T - 0.5 's') / (2 's') otherwise} = ??
```

### Checks

```calc
@test k_E == 0 or S_S < 1.0
```
