+++
cruncha-template = 1
id = "dynamics.singleDof.forced"
version = "1.0.0"
title = "Single degree of freedom, damped free and forced vibration"
summary = "A mass on a spring with viscous damping driven by a harmonic force: natural frequency, static deflection, critical damping, damped frequency and logarithmic decrement; at the driving frequency the magnification x0/xst, phase lag, amplitude, transmissibility and force to the foundation; the resonant peak. Checks the amplitude and the transmissibility against limits, and plots both response curves."
category = ["Mechanical engineering", "Vibrations"]
tags = ["vibration", "single degree of freedom", "SDOF", "natural frequency", "damping ratio", "critical damping", "logarithmic decrement", "magnification", "dynamic amplification", "transmissibility", "vibration isolation", "machine mounts", "resonance", "Den Hartog"]
from = "lacewing:dynamics.singleDof"
license = "GPL-3.0-or-later"
inputs = ["m", "k", "ζ", "P_0", "ω", "x_allow", "TR_max"]
outputs = ["ω_n", "f_n", "δ_st", "c_c", "c", "ω_d", "δ", "r", "β", "φ", "TR", "x_st", "x_0", "F_T", "r_peak", "β_peak"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "Ch. II, §11 to §15 and §19: Eqs. 12 (natural frequency), 22 (critical damping), 24b (damped frequency), 25 (logarithmic decrement), 32a and 32b (forced amplitude and phase), 35 and 36 (transmitted force, transmissibility); Fig. 52"

[[references]]
work = "Rao, Mechanical Vibrations"
edition = "6th (2017)"
locator = "Secs. 2.6 (free vibration with viscous damping), 3.4 (forced response of a damped system), 9.10 (vibration isolation)"

[[tests]]
name = "Lacewing Lab hand check: m = 10 kg, k = 10 kN/m, c = 20 N·s/m, 100 N at 20 rad/s"
source = "Lacewing Lab vibration_test (Eqs. 32a, 36 by hand): ωn = √1000 = 31.622777 rad/s, cc = 2√(10 × 10⁴) = 632.45553 N·s/m, ζ = 20/632.45553 = 0.031622777, r = 0.63245553, x0/xst = 1/√((1 − 0.4)² + 0.04²) = 1.6629753, x0 = 1.6629753 × 100/10⁴ m = 16.629753 mm, δ = 2πζ/√(1 − ζ²) = 0.19879119; T = √(1 + 0.04²) β = 1.6643051"
inputs = { m = "10 kg", k = "10000 N/m", "ζ" = "0.0316227766", P_0 = "100 N", "ω" = "20 rad/s", x_allow = "20 mm", TR_max = "2" }
expect = { "ω_n" = "31.622777 rad/s", c_c = "632.45553 N·s/m", c = "20 N·s/m", r = "0.63245553", "β" = "1.6629753", x_0 = "16.629753 mm", "δ" = "0.19879119", TR = "1.6643051" }
tolerance = "1e-6"
checks = "pass"
figures = 1

[[tests]]
name = "Lacewing Lab default: 50 kg machine on 200 kN/m mounts, 5 % damping, 100 N at 1500 rpm"
source = "Lacewing Lab vibration_screens_test: ωn = 63.25 rad/s. By hand: fn = 10.065842 Hz, δst = 9.80665/4000 = 2.4516625 mm, cc = 6324.5553 N·s/m, r = 157.07963/63.245553 = 2.4836470, β = 1/√((1 − 6.1684)² + 0.24836²) = 0.19325, T = √(1 + 0.24836²) β = 0.19912, x0 = 96.6 μm, force to the foundation 19.9 N; the mounts isolate"
inputs = { m = "50 kg", k = "200 kN/m", "ζ" = "0.05", P_0 = "100 N", "ω" = "1500 rpm", x_allow = "0.2 mm", TR_max = "0.25" }
expect = { "ω_n" = "63.25 rad/s", f_n = "10.065842 Hz", "δ_st" = "2.4516625 mm", c_c = "6324.5553 N·s/m", r = "2.4836470", "β" = "0.19325", TR = "0.19912", x_0 = "0.0966 mm", F_T = "19.91 N" }
tolerance = "0.05%"
checks = "pass"
figures = 1

[[tests]]
name = "Hand check: resonance and the peak, 10 % damping"
source = "By hand: m = 1 kg, k = 1 N/m, ζ = 0.1, driven at ωn: β = 1/(2ζ) = 5, φ = 90°, T = √(1 + 0.2²) × 5 = 5.0990195; the peak is at r = √(1 − 2 × 0.01) = 0.98994949 with β = 1/(2 × 0.1 √0.99) = 5.0251891 (Lacewing Lab vibration_test checks the peak the same way). Ten times the static deflection allowed is exceeded, so the checks fail"
inputs = { m = "1 kg", k = "1 N/m", "ζ" = "0.1", P_0 = "1 N", "ω" = "1 rad/s", x_allow = "1 m", TR_max = "1" }
expect = { "β" = "5", "φ" = "90°", TR = "5.0990195", r_peak = "0.98994949", "β_peak" = "5.0251891", "ω_d" = "0.99498744 rad/s" }
tolerance = "1e-7"
checks = "fail"
figures = 1

[[tests]]
name = "Hand check: transmissibility is 1 at r = √2 for any damping"
source = "By hand, Den Hartog Fig. 52: at ω/ωn = √2, (1 − r²)² = 1 and T = √(1 + 8ζ²)/√(1 + 8ζ²) = 1; with ζ = 0.3, β = 1/√(1 + 0.72) = 0.7624928517"
inputs = { m = "1 kg", k = "1 N/m", "ζ" = "0.3", P_0 = "1 N", "ω" = "1.41421356237 rad/s", x_allow = "1 m", TR_max = "1.01" }
expect = { TR = "1", "β" = "0.7624928517" }
tolerance = "1e-9"
checks = "pass"
figures = 1
+++

## Single degree of freedom, damped free and forced vibration

A rigid mass on a linear spring with viscous damping, driven by a harmonic force P_0 sin ωt along the spring. Use it to check a machine on its mounts, to choose isolators, or to see how close a structure's one dominant mode is to its excitation. After Den Hartog, Mechanical Vibrations, ch. II. Only one direction of motion is modelled; a machine on mounts that can rock has more modes than this one.

### System

```calc {description="Vibrating mass: the machine and everything that moves with it"}
m = 50 kg_
```

```calc {description="Total stiffness of the spring or mounts in the direction of vibration (mounts in parallel add)"}
k = 200 kN/m_
```

```calc {description="Damping ratio c/cc: a few per cent for steel structures and bolted machinery, about 5 to 10 % for rubber mounts"}
ζ = 0.05_
```

In Cruncha a circular frequency (rad/s, rpm) and a frequency in Hz share a dimension, so the cycles per second are always worked out as ω/(2π) here; don't convert a result in Hz to rpm directly.

```calc
> Undamped natural frequency, Eq. 12
ω_n = sqrt(k / m) = ?? rad/s
> The same in cycles per second
f_n = ω_n / (2 π) = ?? Hz
> Static deflection under the mass's own weight (a vertical spring)
δ_st = 9.80665 'm/s²' / ω_n^2 = ?? mm
```

```calc
> Critical damping, Eq. 22
c_c = 2 sqrt(k m) = ?? N·s/m
> Damping coefficient
c = ζ c_c = ?? N·s/m
```

Free vibration oscillates only below critical damping. For ζ ≥ 1 the damped frequency is given as 0 and the decrement is meaningless.

```calc
> Damped natural frequency, Eq. 24b (0 if overdamped)
ω_d = {ω_n sqrt(1 - ζ^2) if ζ < 1; 0 ω_n otherwise} = ?? rad/s
> Logarithmic decrement, ln(x_n / x_(n+1)), Eq. 25
δ = 2 π ζ / sqrt(1 - ζ^2) = ??
```

### Forced response

The force amplitude for a rotating unbalance is m_u e ω² at the speed.

```calc {description="Amplitude of the harmonic force on the mass"}
P_0 = 100 N_
```

```calc {description="Driving (circular) frequency: for a rotating unbalance, the running speed"}
ω = 1500 rpm_
```

```calc
> Frequency ratio
r = ω / ω_n = ??
> Magnification x_0 / x_st, Eq. 32a
β = 1 / sqrt((1 - r^2)^2 + (2 ζ r)^2) = ??
> Phase by which the displacement lags the force, Eq. 32b
φ = atan2(1 - r^2, 2 ζ r) = ?? °
> Transmissibility, force to the foundation over force applied, Eq. 36
TR = sqrt(1 + (2 ζ r)^2) β = ??
```

```calc
> Static deflection under P_0
x_st = P_0 / k = ?? mm
> Amplitude of the mass, Eq. 32a
x_0 = β x_st = ?? mm
> Amplitude of the force to the foundation, Eqs. 35 and 36
F_T = TR P_0 = ?? N
```

The amplitude peaks a little below ω_n, at r = √(1 − 2ζ²), where the magnification is 1/(2ζ√(1 − ζ²)), about 1/(2ζ) for light damping. Above ζ = 1/√2 there is no peak and the largest magnification is 1, at r = 0.

```calc
> Frequency ratio of the resonant peak
r_peak = {sqrt(1 - 2 ζ^2) if ζ < 1 / sqrt(2); 0 otherwise} = ??
> Resonant magnification
β_peak = {1 / (2 ζ sqrt(1 - ζ^2)) if ζ < 1 / sqrt(2); 1 otherwise} = ??
```

### Response curves

The magnification and the transmissibility against ω/ω_n for this damping, with the working point marked. Isolation (T < 1) needs ω/ω_n above √2, and there more damping makes it worse (Den Hartog Fig. 52); damping helps only while passing through resonance on run-up.

```python {description="Magnification and transmissibility against the frequency ratio"}
import numpy as np
import plotly.graph_objects as go
from cruncha import show

_z = float(ζ)
_r0 = float(r)
_rs = np.linspace(0.0, max(3.0, 1.2 * _r0), 600)
_amp = 1 / np.sqrt((1 - _rs**2) ** 2 + (2 * _z * _rs) ** 2)
_tr = np.sqrt(1 + (2 * _z * _rs) ** 2) * _amp
_amp0 = 1 / np.sqrt((1 - _r0**2) ** 2 + (2 * _z * _r0) ** 2)
_tr0 = np.sqrt(1 + (2 * _z * _r0) ** 2) * _amp0

_fig = go.Figure()
_fig.add_trace(go.Scatter(x=_rs, y=_amp, name="x0/xst (Eq. 32a)"))
_fig.add_trace(go.Scatter(x=_rs, y=_tr, name="T (Eq. 36)"))
_fig.add_trace(go.Scatter(x=[_r0, _r0], y=[_amp0, _tr0], mode="markers", name="working point"))
_fig.update_layout(title=f"Response, ζ = {_z:.3g}", xaxis_title="ω/ωn", yaxis_title="ratio", yaxis_type="log")
show(_fig)
```

### Checks

The amplitude of the mass against what the machine or its connections allow, and the transmissibility against the isolation wanted (set TR_max above the peak magnification where isolation isn't the aim).

```calc {description="Largest acceptable amplitude of the mass"}
x_allow = 0.2 mm_
```

```calc {description="Largest acceptable transmissibility"}
TR_max = 0.25_
```

```calc
@test x_0 ≤ x_allow
```

```calc
@test TR ≤ TR_max
```
