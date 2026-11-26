+++
cruncha-template = 1
id = "structural.tensile.tensionFromFrequencyClamped"
version = "1.0.0"
title = "Tension from a measured frequency: rod or bar with clamped ends"
summary = "The tension in a rod or bar clamped at both ends from a measured natural frequency and its mode number, solving the frequency equation of a tensioned beam with fixed ends, with its first six modes at that tension and the pinned-end and taut-string tensions for comparison."
category = ["Structural engineering", "Analysis"]
tags = ["tension", "frequency", "vibration", "clamped ends", "fixed ends", "tie rod", "tension measurement", "taut string", "Python"]
from = "lacewing:structural.tensileFrequency"
license = "GPL-3.0-or-later"
inputs = ["ρ", "A", "E", "I", "σ_u", "L", "f", "n"]
outputs = ["μ", "T_c", "T_p", "T_s", "f_c", "σ", "r_u"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "3rd (McGraw-Hill, 1947)"
locator = "§31, pp. 170 to 175 (the taut string); §33, Bending vibrations of uniform beams, p. 185 (the clamped beam, the T = 0 limit)"

[[references]]
work = "Lacewing Lab, tensile elements (engineering_core tensile.dart)"
edition = "the owner's app (GPL-3.0-or-later)"
locator = "The clamped-clamped frequency equation of a tensioned beam, 1 − cosh(aL)cos(bL) + (a² − b²)/(2ab)·sinh(aL)sin(bL) = 0, derived there"

[[tests]]
name = "Lacewing Lab's opening rod, clamped: 25 mm A36, 9.144 m, 7.5 Hz in mode 1"
source = "Lacewing Lab's own algorithm (tensile.dart, scan and bisection) run by hand in Python: Tc = 64.510606 kN, against 72.040 kN pinned and 72.493 kN as a string. Checked with a beam-element eigenvalue solution (200 Hermite elements with geometric stiffness, clamped ends): 7.5000 Hz in mode 1 at that tension. Not from a book"
inputs = { "ρ" = "7850 kg/m^3", A = "490.87385 mm^2", E = "199.95 GPa", I = "19174.760 mm^4", "σ_u" = "399.9 MPa", L = "9.144 m", f = "7.5 Hz", n = "1" }
expect = { "μ" = "3.8533597 kg/m", T_c = "64.510606 kN", T_p = "72.040176 kN", T_s = "72.492739 kN", "σ" = "131.42011 MPa" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Short rod in mode 2: 3 m at 40 Hz"
source = "As above, by Lacewing Lab's algorithm: Tc = 12.457554 kN (pinned 38.6706 kN); the beam-element check gives 40.0000 Hz in mode 2 at that tension"
inputs = { "ρ" = "7850 kg/m^3", A = "490.87385 mm^2", E = "199.95 GPa", I = "19174.760 mm^4", "σ_u" = "399.9 MPa", L = "3 m", f = "40 Hz", n = "2" }
expect = { T_c = "12.457554 kN", T_p = "38.670604 kN" }
tolerance = "1e-5"
checks = "pass"
+++

## Tension from a measured frequency (clamped ends)

The axial tension in a rod or bar whose ends are clamped (built in), from one of its measured natural frequencies. Clamping stiffens the member against bending, so a given frequency means less tension than with pinned ends: for a stocky or short member the difference is large, and for a slender, tight one (ξ = L√(T/EI) large) both approach the taut string.

The equation of motion EI w'''' − T w'' + μ ẅ = 0 with zero deflection and slope at both ends gives the frequency equation 1 − cosh(aL)cos(bL) + (a² − b²)/(2ab)·sinh(aL)sin(bL) = 0, where a² − b² = T/EI and a²b² = μω²/EI. It reduces to the taut string as EI → 0 and to the clamped beam, (22.373/2πL²)√(EI/μ), as T → 0. It has no closed-form solution, so the Python cell below finds the frequencies by scanning for roots, and the tension by bisection, as Lacewing Lab does. This equation was derived for Lacewing Lab and checked here against a finite-element eigenvalue solution, not against a published table.

Assumptions: a straight, uniform, linear elastic member, ends fully fixed in position and rotation. Real end fixity lies between pinned and clamped; the pinned-end template gives the other bound. Not for stranded cables, which act as strings.

### Member

For a solid round rod of diameter d, A = πd²/4 and I = πd⁴/64; for a bar w × t, A = wt and I = wt³/12 about the weak axis.

```calc {description="Density of the material"}
ρ = 7850 kg/m³_
```

```calc {description="Cross-section area"}
A = 490.874 mm²_
```

```calc {description="Modulus of elasticity"}
E = 199.95 GPa_
```

```calc {description="Second moment of area about the axis it vibrates about"}
I = 19174.76 mm⁴_
```

```calc {description="Ultimate tensile strength, for the stress ratio"}
σ_u = 399.9 MPa_
```

### Measurement

```calc {description="Free (vibrating) length between the clamps"}
L = 9.144 m_
```

```calc {description="Measured natural frequency"}
f = 7.5 Hz_
```

```calc {description="Mode number of the measured frequency (half-waves)"}
n = 1_
```

### Pinned and string bounds

```calc
> Mass per length
μ = ρ A = ?? kg/m
> Tension as a taut string
T_s = 4 μ L^2 f^2 / n^2 = ?? kN
> Tension with pinned ends
T_p = T_s - n^2 π^2 E I / L^2 = ?? kN
```

### Clamped ends

```python {description="Tension for the measured frequency with clamped ends, and the first six modes at it"}
import math
from cruncha import Q_

_mu = μ.to("kg/m").magnitude
_ei = (E * I).to("N*m**2").magnitude
_l = L.to("m").magnitude
_f = f.to("Hz").magnitude
_n = int(n)

def _fpin(k, t, ei):
    return k * k * math.pi / (2 * _l * _l) * math.sqrt(ei / _mu) * math.sqrt(1 + t * _l * _l / (k * k * math.pi**2 * ei))

def _g(w, t):
    s, q = t / _ei, _mu * w * w / _ei
    a = math.sqrt(s / 2 + math.sqrt(s * s / 4 + q))
    b = math.sqrt(-s / 2 + math.sqrt(s * s / 4 + q))
    al, bl = a * _l, b * _l
    sech = 0.0 if al > 700 else 2 / (math.exp(al) + math.exp(-al))
    return sech - math.cos(bl) + (a * a - b * b) / (2 * a * b) * math.tanh(al) * math.sin(bl)

def _fclamped(k, t):
    # The k-th clamped mode lies above the k-th pinned one and below pinned mode k + 1 at 4EI.
    lo0 = 2 * math.pi * _fpin(k, t, _ei) * 0.999
    hi = 2 * math.pi * _fpin(k + 1, t, 4 * _ei)
    w0, g0 = lo0, _g(lo0, t)
    for i in range(1, 4001):
        w1 = lo0 + (hi - lo0) * i / 4000
        g1 = _g(w1, t)
        if (g0 > 0) != (g1 > 0):
            lo, up, glo = w0, w1, g0
            for _ in range(100):
                m = (lo + up) / 2
                gm = _g(m, t)
                if (gm > 0) == (glo > 0):
                    lo, glo = m, gm
                else:
                    up = m
            return (lo + up) / 2 / (2 * math.pi)
        w0, g0 = w1, g1
    raise ValueError("clamped frequency not found")

_f0 = _fclamped(_n, 0.0)
if _f <= _f0:
    raise ValueError(f"{_f} Hz is below the untensioned clamped frequency {_f0:.3f} Hz of mode {_n}")
_lo, _hi = 0.0, 4 * _mu * _l * _l * _f * _f / (_n * _n)
while _fclamped(_n, _hi) < _f:
    _hi *= 2
for _ in range(100):
    _m = (_lo + _hi) / 2
    if _fclamped(_n, _m) < _f:
        _lo = _m
    else:
        _hi = _m
T_c = Q_((_lo + _hi) / 2, "N")
f_c = Q_([_fclamped(k, T_c.magnitude) for k in range(1, 7)], "Hz")
```

```calc
> Axial stress at the clamped-end tension
σ = T_c / A = ?? MPa
> Share of the ultimate strength
r_u = σ / σ_u = ??
> Clamped-end tension as a share of the pinned-end one
T_c / T_p = ??
```

f_c lists the first six clamped-end frequencies at T_c; the n-th is the measured one.

### Checks

```calc
@test r_u < 1
```
