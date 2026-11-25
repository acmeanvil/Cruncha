+++
cruncha-template = 1
id = "mechanical.hertzContact.surfaceFatigue"
version = "1.0.0"
title = "Contact stress: surface fatigue and bearing stress limits"
summary = "From a maximum Hertz contact pressure: Shigley's surface endurance strength at the cycles required and the factors of safety on stress and on load, the cycles to pitting on the strength-life line, a known rolling life scaled to this stress, and the stress against ISO 76's static contact stress and ISO 281's fatigue stress limit."
category = ["Mechanical engineering", "Bearings"]
tags = ["Hertz", "contact stress", "surface fatigue", "pitting", "surface endurance strength", "Buckingham", "rolling contact fatigue", "ISO 76", "ISO 281", "static load rating", "Shigley"]
from = "lacewing:mechanical.hertzContact"
license = "GPL-3.0-or-later"
inputs = ["p_max", "contact", "S_C8", "b_C", "N", "p_ref", "L_ref", "p_0", "p_u"]
outputs = ["S_C", "n_σ", "n_F", "N_p", "L", "r_0", "s_0", "r_u"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 6-16 Surface Fatigue Strength: Eqs. 6-63, 6-67, 6-68 and the factors of safety on load and stress (pp. 335-337); Eqs. 3-69 and 3-74 (pmax against load); Eq. 11-1 (load-life)"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 14.1, rolling life against Hertz stress (p. 693) and the 1/2 in SAE 52100 ball data (p. 694)"

[[references]]
work = "ISO 76 Rolling bearings — Static load ratings; ISO 281 Rolling bearings — Dynamic load ratings and rating life"
edition = "ISO 76:2006, ISO 281:2007"
locator = "Contact stresses at the basic static load rating (4600, 4200, 4000 MPa); fatigue stress limit about 1.5 GPa (as SKF Rolling bearings 17000/1 EN p. 104 and JTEKT Basic Bearing Knowledge 5-2-4, 5-5-1 state them)"

[[tests]]
name = "Shigley Sec. 6-16: induction-hardened steel, 239 kpsi at 10⁸ cycles is 271 kpsi at 10⁷"
source = "Lacewing Lab hertz_contact_test, after Shigley 10th p. 337: the induction-hardened steel-on-steel slope b = log(271/239)/log(10⁷/10⁸) = −0.0545714 (the book prints −0.055) takes (SC)10⁸ = 239 kpsi to 271 kpsi at 10⁷ cycles; applied to the engine test's line contact (two 50 mm steel cylinders, 10 mm, 2 kN: pmax = 761.08740 MPa), n on stress 2.4550127 and on load its square 6.0270873"
inputs = { p_max = "761.08740 MPa", contact = "2", S_C8 = "239000 psi", b_C = "-0.0545714", N = "1e7" }
expect = { S_C = "271000 psi", "n_σ" = "2.4550127", n_F = "6.0270873" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Hand check: Eq. 6-68 at 300 HB, the line contact scaled from 10⁹ cycles at half the stress, roller bearing limits"
source = "By hand: (SC)10⁸ = 2.76 × 300 − 70 = 758 MPa at 10⁸ cycles (b = −0.056): n = 758/761.0874 = 0.99594343, under 1; cycles to pitting 10⁸(761.0874/758)^(1/−0.056) = 9.2998579e7; L = 10⁹ × 2^(−20/3) = 9.8431332e6; pmax/4000 MPa = 0.19027185, s0 = (4000/761.0874)² = 27.621733; pmax/1.5 GPa = 0.50739160"
inputs = { p_max = "761.08740 MPa", contact = "2", S_C8 = "758 MPa", b_C = "-0.056", N = "1e8", p_ref = "380.54370 MPa", L_ref = "1e9", p_0 = "4000 MPa", p_u = "1.5 GPa" }
expect = { S_C = "758 MPa", "n_σ" = "0.99594343", N_p = "9.2998579e7", L = "9.8431332e6", r_0 = "0.19027185", s_0 = "27.621733", r_u = "0.50739160" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Roark p. 694: 1/2 in SAE 52100 balls, 17.5 × 10⁶ cycles at 174 000 psi, scaled to 200 000 psi"
source = "By hand from Roark 7th p. 694's reference life with the ninth-power point-contact scaling: L = 17.5e6 × (174/200)^9 = 4.9970227e6 cycles"
inputs = { p_max = "200000 psi", contact = "1", S_C8 = "1700 MPa", N = "1e7", p_ref = "174000 psi", L_ref = "17.5e6" }
expect = { L = "4.9970227e6" }
tolerance = "1e-6"
checks = "pass"
+++

## Contact stress: surface fatigue and bearing limits

Checks on the maximum Hertz contact pressure p_max from one of the contact templates (spheres, cylinders, elliptical). Repeated rolling contact fails by pitting: fatigue cracks start just below the surface and pieces of it break away. The life estimates here compare p_max only: they assume a lubricated, smooth, rolling contact without the added stresses of sliding friction, and they scatter widely, so treat them as estimates and confirm from tests of the pair.

```calc {description="Maximum Hertz contact pressure, from a contact template"}
p_max = 761.0874 MPa_
```

```calc {description="Contact type: 1 point (ball, ellipse), 2 line (roller, gear tooth)"}
contact = 2_
```

### Surface endurance strength

Buckingham's surface endurance strength S_C is the contact stress a pair of rolling cylinders survives for N cycles; Shigley fits it as S_C = a N^b (Eq. 6-67). For steel without test data, (S_C)10⁸ ≈ 2.76 H_B − 70 MPa with H_B the Brinell hardness of the softer surface (Eq. 6-68; 758 MPa at 300 HB). AGMA uses b = −0.056 from 10⁴ to 10¹⁰ cycles. Shigley states the factor of safety on stress, S_C/p_max, and on load, whose square (line contact, p_max ∝ √F) or cube (point contact, p_max ∝ F^⅓) it is: say which one a design factor means.

```calc {description="Surface endurance strength at 10⁸ cycles, from tests or Shigley Eq. 6-68"}
S_C8 = 758 MPa_
```

```calc {description="Slope b of the strength-life line S_C = a·N^b"}
b_C = -0.056_
```

```calc {description="Stress cycles required (each pass of a rolling element is one)"}
N = 1e8_
```

```calc
> Surface endurance strength at N cycles, Eq. 6-67
S_C = S_C8 (N / 1e8)^b_C = ?? MPa
> Factor of safety on stress
n_σ = S_C / p_max = ??
> Factor of safety on load
n_F = {n_σ^3 if contact == 1; n_σ^2 if contact == 2} = ??
> Cycles to pitting at p_max on the strength-life line
N_p = 1e8 (p_max / S_C8)^(1 / b_C) = ??
```

### Rolling life scaled from a known one

A rolling bearing's life falls as the cube of the load for balls and the 10/3 power for rollers (Shigley Eq. 11-1), and p_max grows as F^⅓ at a point and F^½ on a line, so the life of a known contact scales as (p_ref/p_max)^9 at a point and (p_ref/p_max)^(20/3) on a line. Roark (p. 693) reports the ninth power for balls from tests. The reference is a life measured or rated at a known p_max, such as Roark's 17.5 × 10⁶ cycles to 10 % failures for ½ in SAE 52100 balls at 174 000 psi.

```calc {description="Maximum contact stress of the reference life"}
p_ref = 380.5437 MPa_
```

```calc {description="Reference life at p_ref, in cycles"}
L_ref = 1e9_
```

```calc {description="Reference life scaled to p_max, in cycles"}
L = L_ref (p_ref / p_max)^({9 if contact == 1; 20 / 3 if contact == 2}) = ??
```

### Bearing stress limits

A rolling bearing's basic static load rating C_0 is the load that raises the contact stress at the most heavily loaded element to ISO 76's value p_0: 4600 MPa for self-aligning ball bearings, 4200 MPa for other ball bearings, 4000 MPa for roller bearings, at which the permanent deformation is about 0.0001 of the element's diameter. The load margin the stress ratio means, (p_0/p_max)³ at a point or squared on a line, compares with the static safety factor s_0 = C_0/P_0. ISO 281:2007's fatigue load limit corresponds to a contact stress p_u of about 1.5 GPa for good bearing steel: below it, with a full lubricant film and clean oil, rolling fatigue is not expected.

```calc {description="ISO 76 contact stress at the static load rating (4600, 4200 or 4000 MPa)"}
p_0 = 4000 MPa_
```

```calc {description="Fatigue stress limit, about 1.5 GPa for bearing steel (ISO 281:2007)"}
p_u = 1.5 GPa_
```

```calc
> Stress against the ISO 76 static stress
r_0 = p_max / p_0 = ??
> Static load margin equivalent to it
s_0 = {(p_0 / p_max)^3 if contact == 1; (p_0 / p_max)^2 if contact == 2} = ??
> Stress against the fatigue stress limit
r_u = p_max / p_u = ??
```

### Checks

```calc
@test n_σ ≥ 1
```

```calc
@test p_max ≤ p_0
```
