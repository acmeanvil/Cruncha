+++
cruncha-template = 1
id = "mechanical.strainGauges.quarterBridge"
version = "1.0.0"
title = "Quarter bridge with lead wires and thermal output"
summary = "The strain under one active gauge from its quarter bridge output, solved exactly with two- or three-wire lead resistance, the gauge's thermal output between the zero-balance and test temperatures, and the gauge factor's change with temperature; the leads' offset and loss of sensitivity, and the uniaxial stress."
category = ["Mechanical engineering", "Strain measurement"]
tags = ["strain gauge", "quarter bridge", "Wheatstone bridge", "lead wires", "three-wire", "two-wire", "thermal output", "apparent strain", "self-temperature-compensated", "S-T-C", "gauge factor", "temperature", "TT-612", "TN-504", "TN-507"]
from = "lacewing:mechanical.strainGauges"
license = "GPL-3.0-or-later"
inputs = ["R_g", "F_0", "w", "R_L", "V_ex", "V_o", "V_(o,0)", "T", "T_0", "A_1", "A_2", "A_3", "A_4", "α_S", "α_ref", "β_F", "E"]
outputs = ["F_T", "ε_TO", "ε_off", "s_L", "ε", "σ", "P_g"]

[[references]]
work = "Micro-Measurements (Vishay Precision Group) Tech Tip TT-612, The Three-Wire Quarter-Bridge Circuit"
edition = "Rev. 14-Jul-2015"
locator = "Two-wire and three-wire quarter bridges: lead resistance offset and desensitization (Fig. 3)"

[[references]]
work = "Micro-Measurements (Vishay Precision Group) Tech Note TN-504-1, Strain Gage Thermal Output and Gage Factor Variation with Temperature"
edition = "Rev. 14-Aug-2014"
locator = "Eqs. 4 and 5 (thermal output polynomial), Eq. 8 (a part other than the S-T-C reference material), Eq. 10 (gauge factor against temperature), Eq. 11 and Sec. 4 (worked correction); Table 1 (S-T-C reference materials)"

[[references]]
work = "Micro-Measurements (Vishay Precision Group) Tech Note TN-507-1, Errors Due to Wheatstone Bridge Nonlinearity"
edition = "Rev. 19-Nov-2010"
locator = "Table 1, case 1 (quarter bridge output and nonlinearity)"

[[tests]]
name = "TT-612: 120 Ω gauge, two-wire, 0.85 Ω per lead"
source = "Micro-Measurements TT-612, as in Lacewing Lab strain_gauge_test: 20 ft of AWG 26 pair (1.7 Ω) in the arm of a two-wire 120 Ω quarter bridge gives an offset of about 7000 µε at F = 2 and makes it about 1.4 % less sensitive (by hand: offset 4(121.7/241.7 − ½)/2 = 7033.5 µε, sensitivity 4·120·120/241.7² = 0.98598)"
inputs = { R_g = "120 ohm", F_0 = "2.0", w = "2", R_L = "0.85 ohm", V_o = "0 mV", "V_(o,0)" = "0 mV" }
expect = { "ε_off" = "7000 µm/m", s_L = "0.986", "ε" = "0 µm/m" }
tolerance = "0.6%"
checks = "pass"

[[tests]]
name = "TT-612: the same leads, three-wire"
source = "Micro-Measurements TT-612, as in Lacewing Lab strain_gauge_test: with one lead in each arm either side of the output corner the bridge stays balanced. By hand: sensitivity 4·120·120.85/241.7² = 0.992966"
inputs = { R_g = "120 ohm", F_0 = "2.0", w = "3", R_L = "0.85 ohm" }
expect = { "ε_off" = "0 µm/m", s_L = "0.9929664874" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "TN-504 Sec. 4: −1850 µε indicated at −45 °C"
source = "Micro-Measurements TN-504-1 Sec. 4, Eq. 11, as in Lacewing Lab strain_gauge_test: indicated −1850 µε (indicator at 2.0), thermal output −200 µε, gauge factor 2.07 at +24 °C, 0.6 % higher at −45 °C: ε ≈ (−1850 + 200)·2.0/2.08 = −1587 µε. The exact bridge with F(T) = 2.08242 gives −1581.41 µε, 0.21 % less, the quarter bridge's nonlinearity that Eq. 11 neglects. A linear thermal output curve A1 = 200/69 µε/°C gives the −200 µε between +24 and −45 °C; the indicated strain is the output 2.0 × (−1850 µε)/4 × 5 V = −4.625 mV"
inputs = { F_0 = "2.07", w = "3", R_L = "0 ohm", V_ex = "5 V", V_o = "-4.625 mV", "V_(o,0)" = "0 mV", T = "-45 °C", T_0 = "24 °C", A_1 = "2.898550725", "β_F" = "-0.8695652174" }
expect = { F_T = "2.0824", "ε" = "-1587 µm/m" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "Hand check: TN-504 Sec. 4 exactly"
source = "By hand: F(T) = 2.07 × (1 + (−0.869565/100)(−69/100)) = 2.08242; εT/O = (−200 µε) × 2.0/2.08242 = −192.0842 µε; r = −0.925 mV/V, a = 350 × 0.499075/0.500925 = 348.70753 Ω; ε = (a/350 − 1)/2.08242 − εT/O = −1581.41376 µε; σ = 200 GPa × ε = −316.28275 MPa"
inputs = { F_0 = "2.07", w = "3", R_L = "0 ohm", V_ex = "5 V", V_o = "-4.625 mV", "V_(o,0)" = "0 mV", T = "-45 °C", T_0 = "24 °C", A_1 = "2.898550725", "β_F" = "-0.8695652174", E = "200 GPa" }
expect = { F_T = "2.08242", "ε_TO" = "-192.0842097 µm/m", "ε" = "-1581.413759 µm/m", "σ" = "-316.2827518 MPa" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Hand check: a 2024 aluminium part with steel-compensated gauges at 120 °C, three-wire"
source = "By hand: 350 Ω, F 2.0, gauges with no thermal output on their reference material (A = 0) but αS − αref = (23 − 12.1) µ/°C over 96 °C: εT/O = 1046.4 µε (TN-504 Eq. 8); leads 0.5 Ω; reading 7.5 mV at 5 V (1.5 mV/V): a = 350.5 × 0.5015/0.4985 = 352.60933 Ω, less its 0.5 Ω lead 352.10933 Ω; ε = (352.10933/350 − 1)/2 − 1046.4 µε = 3013.33 − 1046.4 = 1966.926 µε; σ = 73 GPa × ε = 143.586 MPa"
inputs = { w = "3", R_L = "0.5 ohm", V_ex = "5 V", V_o = "7.5 mV", T = "120 °C", T_0 = "24 °C", "α_S" = "23e-6 K^-1", "α_ref" = "12.1e-6 K^-1", E = "73 GPa" }
expect = { "ε_TO" = "1046.4 µm/m", "ε" = "1966.92569136 µm/m", "σ" = "143.585575469 MPa", "ε_off" = "1043.816584 µm/m" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Four-wire is not a quarter bridge wiring"
source = "TT-612 treats two- and three-wire quarter bridges only"
inputs = { w = "4" }
expect = { F_T = "2.0" }
tolerance = "1e-9"
checks = "fail"
+++

## Quarter bridge with lead wires and thermal output

One active gauge in a quarter bridge, read through lead wires, possibly away from the temperature at which the bridge was zeroed. The bridge is solved exactly from its four arm resistances, so nonlinearity (TN-507), lead resistance (TT-612) and the gauge's thermal output (TN-504) all come in as they do in the real circuit, and a reading is turned back into strain without linearizing.

The active gauge is arm a, with the output between it and the completion resistor b; c and d are the other half bridge, all of the gauge's resistance and at the instrument. Two-wire wiring puts both leads in the gauge's arm: an offset, and less sensitivity. Three-wire wiring puts one lead in the gauge's arm and one in the adjacent arm b, across the output corner, so their effects largely cancel; the third lead carries no current. Completion resistors are taken equal to the gauge resistance and free of temperature change.

For a half or full bridge without leads or temperature effects, use the strain gauge bridge template.

### Gauge and wiring

```calc {description="Gauge resistance (120, 350 or 1000 Ω are usual)"}
R_g = 350 ohm_
```

```calc {description="Gauge factor at +24 °C, from the gauge package"}
F_0 = 2.0_
```

```calc {description="Wiring: 2 for two-wire, 3 for three-wire"}
w = 3_
```

```calc
@test w == 2 or w == 3
```

```calc {description="Resistance of one lead wire, instrument to gauge"}
R_L = 0 ohm_
```

```calc
> Leads in the gauge's arm
n_a = { 2 if w == 2; 1 otherwise } = ??
> Leads in the adjacent arm
n_b = { 0 if w == 2; 1 otherwise } = ??
```

### Temperature

The thermal output of a self-temperature-compensated gauge is a polynomial in temperature on the data sheet of its lot: εT/O = A₀ + A₁T + A₂T² + A₃T³ + A₄T⁴, in µε with T in °C, measured at a gauge factor of 2.0 on the S-T-C number's reference material (TN-504 Table 1: 1018 steel for S-T-C 06, 2024-T4 aluminium for 13, and so on). Only its change between the zero-balance and test temperatures matters, so A₀ cancels and is not asked for. On another material, the curve rotates by (αS − αref)·ΔT (Eq. 8). Leave the coefficients zero and αS equal to αref, with T equal to T₀, to ignore temperature.

```calc {description="Temperature of the gauge when the reading is taken"}
T = 24 °C_
```

```calc {description="Temperature at which the bridge was zeroed"}
T_0 = 24 °C_
```

```calc {description="Thermal output coefficient A₁, µε/°C, from the gauge lot's data sheet"}
A_1 = 0_
```

```calc {description="Thermal output coefficient A₂, µε/°C²"}
A_2 = 0_
```

```calc {description="Thermal output coefficient A₃, µε/°C³"}
A_3 = 0_
```

```calc {description="Thermal output coefficient A₄, µε/°C⁴"}
A_4 = 0_
```

```calc {description="Expansion coefficient of the part"}
α_S = 12.1e-6 K^-1_
```

```calc {description="Expansion coefficient of the S-T-C reference material (12.1 µ/°C: 1018 steel)"}
α_ref = 12.1e-6 K^-1_
```

```calc {description="Gauge factor change, % per 100 °C, from the gauge package"}
β_F = 0_
```

```calc
> Test temperature, °C as a number
t = (T - 0 '°C') / (1 'K') = ??
> Zero-balance temperature, °C as a number
t_0 = (T_0 - 0 '°C') / (1 'K') = ??
```

```calc {description="Gauge factor at the test temperature, TN-504 Eq. 10"}
F_T = F_0 (1 + β_F / 100 × (t - 24) / 100) = ??
```

```calc {description="Gauge factor at the zero-balance temperature, TN-504 Eq. 10"}
F_T0 = F_0 (1 + β_F / 100 × (t_0 - 24) / 100) = ??
```

```calc {description="Thermal output of the gauge, T₀ to T, at F(T) on this part, TN-504 Eqs. 5 and 8"}
ε_TO = (A_1 (t - t_0) + A_2 (t^2 - t_0^2) + A_3 (t^3 - t_0^3) + A_4 (t^4 - t_0^4)) × 10^-6 × 2.0 / F_T + (α_S - α_ref) (T - T_0) = ?? µm/m
```

### The bridge at zero strain

What the leads and the thermal output do by themselves: the output with no strain, as the strain an indicator set to F₀ would show, and the bridge's sensitivity to strain against an ideal quarter bridge.

```calc
> Gauge arm at zero strain, with its leads
R_(a,0) = R_g (1 + F_T ε_TO) + n_a R_L = ?? ohm
> Adjacent arm, the completion resistor with its lead
R_b = R_g + n_b R_L = ?? ohm
> Apparent strain from leads and thermal output, indicator at F₀
ε_off = 4 / F_0 × (R_(a,0) / (R_(a,0) + R_b) - 1 / 2) = ?? µm/m
> Sensitivity at zero strain, relative to the bridge without leads
s_L = 4 R_g R_b / (R_(a,0) + R_b)^2 = ??
```

### Reading

An indicator showing εi at a gauge factor setting Fᵢ has an output Eo/E = Fᵢ·εi/4: enter V_o = Fᵢ·εi·V_ex/4. The zero reading is the output of the unloaded part at the zero-balance temperature; 0 when the bridge was nulled there.

```calc {description="Bridge excitation voltage"}
V_ex = 5 V_
```

```calc {description="Bridge output under load, at the test temperature"}
V_o = 1.75 mV_
```

```calc {description="Bridge output with the part unloaded, at the zero-balance temperature"}
V_(o,0) = 0 mV_
```

The gauge arm follows from the output: a/(a + b) − ½ = Eo/E, so a = b·q/(1 − q) with q = Eo/E + ½. Taking off the leads and solving R_g(1 + F(ε + εT/O)) for ε gives the strain; the zero reading is solved the same way at T₀, where the thermal output is zero by definition.

```calc
> Gauge arm under load
R_a = R_b (V_o / V_ex + 1 / 2) / (1 / 2 - V_o / V_ex) = ?? ohm
> Gauge arm unloaded, at T₀
R_(a,z) = R_b (V_(o,0) / V_ex + 1 / 2) / (1 / 2 - V_(o,0) / V_ex) = ?? ohm
> Strain under load, thermal output removed
ε_raw = ((R_a - n_a R_L) / R_g - 1) / F_T - ε_TO = ?? µm/m
> Strain of the zero reading
ε_z = ((R_(a,z) - n_a R_L) / R_g - 1) / F_T0 = ?? µm/m
```

```calc {description="Mechanical strain along the gauge"}
ε = ε_raw - ε_z = ?? µm/m
```

### Stress and gauge power

```calc {description="Young's modulus of the part"}
E = 200 GPa_
```

```calc {description="Uniaxial stress along the gauge (an axial member, or a beam's extreme fibre)"}
σ = E ε = ?? MPa
```

The heat in the gauge rises with the square of the excitation. Whether a grid of this size on this part can shed it is a matter for the gauge maker's guidance (Micro-Measurements TN-502).

```calc {description="Power dissipated in the gauge, balanced bridge"}
P_g = V_ex^2 / (4 R_g) = ?? mW
```
