+++
cruncha-template = 1
id = "structural.tensile.modesAtTension"
version = "1.0.0"
title = "Natural frequencies of a tensioned rod, bar or cable, pinned ends"
summary = "The first six transverse natural frequencies of a rod, bar or cable at a known tension, with its bending stiffness (pinned ends) and as a taut string, and the share of the fundamental's stiffness due to bending."
category = ["Structural engineering", "Analysis"]
tags = ["frequency", "natural frequency", "vibration", "modes", "taut string", "tie rod", "cable", "stay", "tension", "Den Hartog"]
from = "lacewing:structural.tensileFrequency"
license = "GPL-3.0-or-later"
inputs = ["ρ", "A", "E", "I", "L", "T"]
outputs = ["μ", "f", "f_s", "f_1", "f_2", "f_3", "f_(s,1)", "s_b"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "3rd (McGraw-Hill, 1947)"
locator = "§31, Strings and organ pipes, pp. 170 to 175: the equation of the string (Eq. 93) and its natural frequencies ωn = (nπ/l)√(T/μ1) from Eqs. 94 to 97 (Lacewing Lab cites the same section of the 4th edition)"

[[tests]]
name = "Lacewing Lab's opening rod at 72.04 kN: 7.5 Hz in mode 1"
source = "Lacewing Lab tensile_test (pinned, with tension: exactly the string × √(1 + n²π²EI/(TL²))), on the page's 25 mm A36 rod, 9.144 m: by hand f1 = 7.5 Hz at the tension the tension-from-frequency template gives, string 7.4766 Hz; modes 2 to 6 by hand. A beam-element eigenvalue check (200 elements) gives 7.5000, 15.1398 and 23.0550 Hz for the first three"
inputs = { "ρ" = "7850 kg/m^3", A = "490.87385 mm^2", E = "199.95 GPa", I = "19174.760 mm^4", L = "9.144 m", T = "72.040176 kN" }
expect = { "μ" = "3.8533597 kg/m", f_1 = "7.5 Hz", "f_(s,1)" = "7.4765526 Hz", f_2 = "15.139813 Hz", f_3 = "23.055013 Hz", s_b = "0.0062428683" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Pinned beam with no tension: (n²π/2L²)√(EI/μ)"
source = "Lacewing Lab tensile_test (pinned beam, no tension): L 2 m, μ 3 kg/m, EI 500 N·m² (here A = 1000 mm² at ρ = 3000 kg/m³, E 200 GPa, I = 2500 mm⁴): f2 = 4π/8 × √(500/3) = 20.278893 Hz, f1 a quarter of it"
inputs = { "ρ" = "3000 kg/m^3", A = "1000 mm^2", E = "200 GPa", I = "2500 mm^4", L = "2 m", T = "0 N" }
expect = { "μ" = "3 kg/m", f_1 = "5.0697233 Hz", f_2 = "20.278893 Hz", f_3 = "45.62751 Hz", s_b = "1" }
tolerance = "1e-6"
checks = "pass"
+++

## Natural frequencies at a tension (pinned ends)

The transverse natural frequencies of a straight, uniform member stretched between two supports at a known tension T. As a taut string, mode n vibrates at (n/2L)√(T/μ) (Den Hartog §31). A rod or bar also resists bending: the equation of motion EI w'''' − T w'' + μ ẅ = 0 with pinned ends gives exactly f_n = (n/2L)√(T/μ)·√(1 + n²π²EI/(TL²)), that is (n/2L)√((T + n²π²EI/L²)/μ): the string with the bending term added to the tension. With no tension it is the pinned beam's (n²π/2L²)√(EI/μ). Bending raises the higher modes most, so the overtones of a rod are not whole multiples of the fundamental.

Assumptions: linear elastic, uniform, vibrating in a plane, ends that hold position without restraining rotation; real ends lie between pinned and clamped. A stranded cable has negligible bending stiffness: give it I = 0 and a tension above zero.

### Member

For a solid round rod of diameter d, A = πd²/4 and I = πd⁴/64; for a bar w × t, A = wt and I = wt³/12 about the weak axis.

```calc {description="Density of the material"}
ρ = 7850 kg/m³_
```

```calc {description="Cross-section (metallic) area"}
A = 490.874 mm²_
```

```calc {description="Modulus of elasticity"}
E = 199.95 GPa_
```

```calc {description="Second moment of area about the axis it vibrates about (0 for a cable)"}
I = 19174.76 mm⁴_
```

```calc {description="Free (vibrating) length between supports"}
L = 9.144 m_
```

```calc {description="Axial tension"}
T = 72 kN_
```

### Frequencies

```calc
> Mass per length
μ = ρ A = ?? kg/m
> Mode numbers
n = [1; 2; 3; 4; 5; 6] = !
> Frequencies with the bending stiffness, pinned ends
f = n / (2 L) ∘ sqrt((T + n^2 π^2 E I / L^2) / μ) = ?? Hz
> Frequencies as a taut string
f_s = n / (2 L) sqrt(T / μ) = ?? Hz
```

The first three modes with the bending stiffness, and the string's fundamental:

```calc
> Mode 1, the fundamental
f_1 = f[1] = ?? Hz
> Mode 2
f_2 = f[2] = ?? Hz
> Mode 3
f_3 = f[3] = ?? Hz
> Fundamental as a taut string
f_(s,1) = f_s[1] = ?? Hz
```

s_b is the share of mode 1's stiffness that comes from bending: zero for a string, one for a beam with no tension. In terms of the stiffness parameter ξ = L√(T/EI) it is π²/(ξ² + π²); ξ above about 50 means the member is nearly a string.

```calc {description="Share of the fundamental's stiffness from bending"}
s_b = π^2 E I / (T L^2 + π^2 E I) = ??
```

### Checks

```calc
@test T ≥ 0 'N' and T L^2 + π^2 E I > 0 'N·m^2'
```
