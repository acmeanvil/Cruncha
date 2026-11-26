+++
cruncha-template = 1
id = "structural.wind.solidSign"
version = "1.1.0"
title = "Wind on a solid freestanding wall or sign, ASCE 7-10"
summary = "Design wind force on a solid freestanding wall or sign (Eq. 29.4-1), with the reduction for small openings and where the resultant acts in Cases A and B."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "sign", "billboard", "freestanding wall", "fence", "Chapter 29", "Eq. 29.4-1", "Cf", "force coefficient", "Case B", "eccentricity"]
from = "lacewing:structural.windOther"
license = "GPL-3.0-or-later"
inputs = ["q_h", "G", "h", "s", "B", "ε", "C_f"]
outputs = ["A_s", "R_ε", "F_sign", "F_(sign,d)", "e_B", "y_F"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 29.4.1 Eq. 29.4-1; Fig. 29.4-1 (Cf, Cases A, B and C, notes 2, 3 and 5); Sec. 29.8 (minimum); Eq. 29.3-1 (qh)"

[[tests]]
name = "Sign at ground level, 10 × 10 ft, 115 mph, exposure C"
source = "ASCE 7-10 Fig. 29.4-1 (verified in the PDF): Cf 1.45 at s/h = 1, B/s = 1, as Lacewing Lab's solidSignCf test. By hand: qh from velocityPressure at h = 10 ft (Kh held at its 15 ft value, 0.84888) = 24.429 psf; F = 24.429 × 0.85 × 1.45 × 100 = 3010.9 lb; resultant 0.05h above the centre: y = 5.5 ft; Case B offset 0.2 × 10 = 2 ft"
inputs = { q_h = "24.42885 psf", h = "10 ft", s = "10 ft", B = "10 ft", "ε" = "1", C_f = "1.45" }
expect = { F_sign = "3010.85 lbf", e_B = "2 ft", y_F = "5.5 ft" }
tolerance = "1e-4"
checks = "pass"

[[tests]]
name = "Billboard 10 × 30 ft with its top at 30 ft"
source = "Lacewing Lab's opening case. Cf by Fig. 29.4-1 between s/h 0.3 and 0.5 at B/s = 3 (1.80 and 1.70 at both B/s = 2 and 4; verified in the PDF), linear in s/h: 1.7833. By hand: qh from velocityPressure at h = 30 ft = 28.267 psf; F = 28.267 × 0.85 × 1.7833 × 300 = 12 854 lb; Case B offset 6 ft; resultant at 25 ft"
inputs = { q_h = "28.26687 psf", h = "30 ft", s = "10 ft", B = "30 ft", "ε" = "1", C_f = "1.78333" }
expect = { F_sign = "12854 lbf", e_B = "6 ft", y_F = "25 ft" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Sign with 20 % openings: Cf reduced by 1 − (1 − ε)^1.5"
source = "Fig. 29.4-1 note 2; Lacewing Lab asce7_wind_test (1.45 × (1 − 0.2^1.5) at ε 0.8). By hand: R = 1 − 0.2^1.5 = 0.91056; F = 3010.85 × 0.91056 = 2741.6 lb"
inputs = { q_h = "24.42885 psf", h = "10 ft", s = "10 ft", B = "10 ft", "ε" = "0.8", C_f = "1.45" }
expect = { "R_ε" = "0.910557", F_sign = "2741.6 lbf" }
tolerance = "1e-4"
checks = "pass"

[[tests]]
name = "Low wind speed: the 16 psf minimum governs"
source = "By hand: 60 mph, exposure C, sign at ground 10 × 10 ft: qh from velocityPressure = 0.00256 × 0.84888 × 0.85 × 3600 = 6.650 psf; F = 6.650 × 0.85 × 1.45 × 100 = 819.6 lb, less than 16 × 100 = 1600 lb (Sec. 29.8)"
inputs = { q_h = "6.649819 psf", h = "10 ft", s = "10 ft", B = "10 ft", "ε" = "1", C_f = "1.45" }
expect = { F_sign = "819.6 lbf", "F_(sign,d)" = "1600 lbf" }
tolerance = "1e-4"
checks = "pass"

[[tests]]
name = "30 % or more open: not a solid sign"
source = "Fig. 29.4-1 note 2 and Fig. 29.5-2 note 1: openings of 30 % or more make an open sign; the ε check fails at 0.6"
inputs = { "ε" = "0.6" }
expect = { "R_ε" = "0.747018" }
tolerance = "1e-5"
checks = "fail"
+++

## Wind on a solid freestanding wall or sign (ASCE 7-10)

The design wind force on a solid freestanding wall or solid freestanding sign, F = q_h G C_f A_s (Eq. 29.4-1), with q_h at the top of the sign (h) and A_s its gross area. It is the total horizontal force at strength level (LRFD 1.0W, ASD 0.6W).

The net force coefficient C_f is given only as a **table** in Figure 29.4-1, by the clearance ratio s/h and the aspect ratio B/s (linear interpolation permitted, note 5). It is an input: the template computes both ratios for you to read it. A sign with openings of less than 30 % of its gross area is still solid, and its C_f may be reduced by 1 − (1 − ε)^1.5 (note 2, a formula, computed here). With 30 % or more open it is an open sign: use the other-structures template with Figure 29.5-2.

Figure 29.4-1 note 3 requires two load cases for every sign, and a third for long ones:
- **Case A:** the resultant acts through the geometric centre of the face.
- **Case B:** the same force, offset 0.2B horizontally toward the windward edge. Check the supports and foundation for the torsion.
- **Case C (B/s ≥ 2):** the force is distributed along the sign by region, with C_f from the figure's Case C table. Not computed here.

For a sign reaching the ground (s/h = 1), the resultants act 0.05h above the geometric centre.

### Velocity pressure and gust

The velocity pressure q_h is an input: compute it with the velocity pressure template (structural.wind.velocityPressure) with h at the top of the sign and K_d for solid signs, and bind it to its q_h.

```calc {description="Velocity pressure at the top of the sign (h), Eq. 29.3-1, from structural.wind.velocityPressure"}
q_h = 28.27 psf_
```

```calc {description="Gust-effect factor, Sec. 26.9 (0.85 for a rigid structure; from structural.wind.gustRigid)"}
G = 0.85_
```

### Sign

```calc {description="Height of the top of the sign or wall above ground"}
h = 30 ft_
```

```calc {description="Vertical dimension of the sign"}
s = 10 ft_
```

```calc {description="Horizontal dimension of the sign"}
B = 30 ft_
```

```calc {description="Solidity ratio: solid area over gross area (1 for no openings)"}
ε = 1_
```

```calc
> Clearance ratio, for Fig. 29.4-1
r_sh = s / h = ??
> Aspect ratio, for Fig. 29.4-1
r_Bs = B / s = ??
```

```calc {description="Net force coefficient, Fig. 29.4-1 Cases A and B, at s/h and B/s"}
C_f = 1.78_
```

```calc
@test s ≤ h
```

```calc
@test 0.7 < ε ≤ 1
```

### Force

```calc {description="Gross area of the sign"}
A_s = s B = ?? ft²
```

```calc {description="Reduction of Cf for openings, Fig. 29.4-1 note 2"}
R_ε = 1 - (1 - ε)^1.5 = ??
```

```calc {description="Design wind force on the sign, Eq. 29.4-1"}
F_sign = q_h G C_f R_ε A_s = ?? lbf
```

```calc {description="Design force with the 16 psf minimum on the area, Sec. 29.8"}
F_(sign,d) = max(F_sign, 16 'psf' A_s) = ?? lbf
```

### Where it acts

```calc {description="Case B: horizontal offset of the resultant from the centre, toward the windward edge"}
e_B = 0.2 B = ?? ft
```

```calc {description="Height of the resultant above ground (0.05h above the centre where s = h)"}
y_F = h - s / 2 + {0.05 h if s ≥ h; 0 'ft' otherwise} = ?? ft
```
