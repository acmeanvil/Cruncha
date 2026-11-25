+++
cruncha-template = 1
id = "mechanical.interferenceFits.pressFit"
version = "1.0.0"
title = "Interference (press or shrink) fit: pressure, stresses, holding capacity and assembly temperature"
summary = "A hub pressed or shrunk on a solid or hollow shaft, each of its own material: the contact pressure at the least and greatest diametral interference, the tangential and radial stresses at the interface and the bore, their von Mises stresses and yield factors of safety, the torque and axial force the fit holds by friction, the force to press it on, the temperature to heat the hub or cool the shaft for assembly, and the interference left at an operating temperature."
category = ["Mechanical engineering", "Shafts"]
tags = ["interference fit", "press fit", "shrink fit", "force fit", "hub", "shaft", "contact pressure", "Lamé", "thick-walled cylinder", "torque capacity", "pressing force", "shrink temperature", "Shigley", "Machinery's Handbook"]
from = "lacewing:mechanical.interferenceFits"
license = "GPL-3.0-or-later"
inputs = ["d", "d_i", "d_o", "l", "δ_min", "δ_max", "E_i", "ν_i", "E_o", "ν_o", "S_(y,i)", "S_(y,o)", "f", "f_as", "T", "F_x", "n_d", "α_i", "α_o", "c", "ΔT_op"]
outputs = ["p_max", "p_min", "σ_(t,o)", "σ_(t,o,OD)", "σ_(t,i)", "σ_(t,i,bore)", "σ_o′", "σ_i′", "n_o", "n_i", "T_cap", "F_cap", "F_dem", "F_press", "ΔT_hub", "ΔT_shaft", "δ_op", "p_op", "T_op", "F_(cap,op)", "ΔT_rel"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-16, Eqs. 3-49, 3-56 to 3-59 (thick-walled cylinders, press and shrink fits); Sec. 7-8, Eqs. 7-39 to 7-49 (limits and fits: pressure, stresses, torque and axial capacity); Eq. 5-13 (von Mises, plane stress)"

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "\"Temperatures for Shrinkage Fits\" (p. 627) and \"Expansion Fits\" (p. 624); ISO 286 preferred fits (Table 13, interference ranges)"

[[tests]]
name = "Steel on steel, 50 mm shaft in a 100 mm hub 60 mm long, 20 to 50 µm interference"
source = "Lacewing Lab fits_couplings_test (Shigley 10th Eqs. 7-39, 7-40, 7-44 to 7-49): same material, so p = Eδ/(2d³)·(do² − d²)d²/do² = 77.625 MPa at 50 µm and 31.05 MPa at 20 µm; σt hub = p(do² + d²)/(do² − d²) = 129.375 MPa, σ′ hub = √(σt² + σt p + p²) = 181.125 MPa (n = 300/181.125 = 1.6563), the solid shaft at σt = σr = −p (n = 400/77.625 = 5.1530); f = 0.12: T = (π/2) f p l d² = 877.918 N·m, Fa = π f p l d = 35.1167 kN; press force at 50 µm 87.7918 kN; α 12e-6/K: heat the hub 50e-6/(12e-6 × 0.05) = 83.33 K"
inputs = { d = "50 mm", d_i = "0 mm", d_o = "100 mm", l = "60 mm", "δ_min" = "20 μm", "δ_max" = "50 μm", E_i = "207 GPa", "ν_i" = "0.3", E_o = "207 GPa", "ν_o" = "0.3", "S_(y,i)" = "400 MPa", "S_(y,o)" = "300 MPa", f = "0.12", f_as = "0.12", T = "0 N·m", F_x = "0 N", n_d = "1.5", "α_i" = "12e-6 1/K", "α_o" = "12e-6 1/K", c = "0 mm", "ΔT_op" = "0 ΔK" }
expect = { p_max = "77.625 MPa", p_min = "31.05 MPa", "σ_(t,o)" = "129.375 MPa", "σ_(t,o,OD)" = "51.75 MPa", "σ_(t,i)" = "-77.625 MPa", "σ_o′" = "181.125 MPa", "σ_i′" = "77.625 MPa", n_o = "1.6563147", n_i = "5.1529791", T_cap = "877.91807 N·m", F_cap = "35116.723 N", F_press = "87791.807 N", "ΔT_hub" = "83.333333 ΔK" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "ISO 50 H7/s6, steel, interference 0.018 to 0.059 mm: the hub's yield margin falls under 1.5"
source = "Lacewing Lab fits_couplings_test: Machinery's 26th Table 13 gives 50 H7/s6 an interference of 0.018 to 0.059 mm (Shigley Eqs. 7-42, 7-43). By hand with the parts above: p 91.5975 and 27.945 MPa, σ′ hub 213.7275 MPa, n hub 1.40366 under the design factor 1.5; T = 790.126 N·m against a required 600 N·m"
inputs = { d = "50 mm", d_i = "0 mm", d_o = "100 mm", l = "60 mm", "δ_min" = "18 μm", "δ_max" = "59 μm", E_i = "207 GPa", "ν_i" = "0.3", E_o = "207 GPa", "ν_o" = "0.3", "S_(y,i)" = "400 MPa", "S_(y,o)" = "300 MPa", f = "0.12", f_as = "0.12", T = "600 N·m", F_x = "0 N", n_d = "1.5", "α_i" = "12e-6 1/K", "α_o" = "12e-6 1/K", c = "0 mm", "ΔT_op" = "0 ΔK" }
expect = { p_max = "91.5975 MPa", p_min = "27.945 MPa", "σ_o′" = "213.7275 MPa", n_o = "1.4036565", T_cap = "790.12626 N·m" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Machinery's p. 627: a steel hub expanded 0.005 in per inch is heated 769 °F"
source = "Lacewing Lab fits_couplings_test, after Machinery's 26th \"Temperatures for Shrinkage Fits\": 1 in bore, greatest interference 0.001 in plus 0.004 in assembly clearance, α = 6.5e-6 per °F: ΔT = 0.005/(6.5e-6 × 1) = 769.2 °F"
inputs = { d = "1 in", d_i = "0 in", d_o = "2 in", l = "1 in", "δ_min" = "0.0005 in", "δ_max" = "0.001 in", E_i = "207 GPa", "ν_i" = "0.3", E_o = "207 GPa", "ν_o" = "0.3", "α_i" = "6.5e-6 1/Δ°F", "α_o" = "6.5e-6 1/Δ°F", c = "0.004 in", "ΔT_op" = "0 Δ°F" }
expect = { "ΔT_hub" = "769.23077 Δ°F", "ΔT_shaft" = "769.23077 Δ°F" }
tolerance = "1e-6"

[[tests]]
name = "Aluminium hub on a steel shaft loosens at 100 K above assembly"
source = "Lacewing Lab fits_couplings_test: 50 mm steel shaft (α 12e-6/K), aluminium hub (70 GPa, ν 0.33, α 23e-6/K), 20 to 50 µm: at 100 K the least interference changes by (12 − 23)e-6 × 100 × 0.05 = −55 µm, so it is lost (δ_op = 0, no torque); it is lost from 20/(11e-6 × 0.05) = 36.36 K"
inputs = { d = "50 mm", d_i = "0 mm", d_o = "100 mm", l = "50 mm", "δ_min" = "20 μm", "δ_max" = "50 μm", E_i = "207 GPa", "ν_i" = "0.3", E_o = "70 GPa", "ν_o" = "0.33", "S_(y,i)" = "400 MPa", "S_(y,o)" = "250 MPa", f = "0.15", f_as = "0.15", T = "100 N·m", F_x = "0 N", n_d = "1.5", "α_i" = "12e-6 1/K", "α_o" = "23e-6 1/K", c = "0 mm", "ΔT_op" = "100 ΔK" }
expect = { "δ_op" = "0 mm", T_op = "0 N·m", "ΔT_rel" = "36.363636 ΔK" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: hollow steel shaft in a cast-iron hub, torque and thrust together, slips when warm"
source = "By hand, Shigley Eqs. 3-49, 7-39 to 7-49: d 40, bore 15, hub OD 90, l 50 mm; shaft 207 GPa, ν 0.29, α 11.5e-6/K; hub 100 GPa, ν 0.26, α 17e-6/K; δ 25 to 45 µm: p = 49.92443 and 27.73580 MPa; at δmax σt hub 74.50262, at the hub OD 24.57818, shaft surface −66.26334, shaft bore −116.18777 MPa; σ′ hub 108.45547 MPa (n = 200/108.455 = 1.84407), shaft governed by its bore 116.18777 MPa (n = 4.30338); f 0.15: T = 522.8074 N·m, Fa = 26.14037 kN; 400 N·m with 5 kN thrust needs √(5000² + (2 × 400/0.04)²) = 20.61553 kN of friction; pressing with f 0.2: 62.73689 kN; with 10 µm clearance heat the hub 80.88235 K or cool the shaft 119.5652 K; 60 K warmer in service: δ = 25 − 5.5e-6 × 60 × 40e3 µm = 11.8 µm, p 13.09130 MPa, T 246.7651 N·m and π f p l d = 12.33826 kN, under the 20.6 kN needed: the warm fit slips; lost entirely at 113.636 K"
inputs = { d = "40 mm", d_i = "15 mm", d_o = "90 mm", l = "50 mm", "δ_min" = "25 μm", "δ_max" = "45 μm", E_i = "207 GPa", "ν_i" = "0.29", E_o = "100 GPa", "ν_o" = "0.26", "S_(y,i)" = "500 MPa", "S_(y,o)" = "200 MPa", f = "0.15", f_as = "0.2", T = "400 N·m", F_x = "5 kN", n_d = "1.5", "α_i" = "11.5e-6 1/K", "α_o" = "17e-6 1/K", c = "10 μm", "ΔT_op" = "60 ΔK" }
expect = { p_max = "49.924434 MPa", p_min = "27.735797 MPa", "σ_(t,o)" = "74.502617 MPa", "σ_(t,o,OD)" = "24.578183 MPa", "σ_(t,i)" = "-66.263339 MPa", "σ_(t,i,bore)" = "-116.18777 MPa", "σ_o′" = "108.45547 MPa", "σ_i′" = "116.18777 MPa", n_o = "1.8440747", n_i = "4.3033788", T_cap = "522.80745 N·m", F_cap = "26140.372 N", F_dem = "20615.528 N", F_press = "62736.894 N", "ΔT_hub" = "80.882353 ΔK", "ΔT_shaft" = "119.56522 ΔK", "δ_op" = "11.8 μm", p_op = "13.091296 MPa", T_op = "246.76512 N·m", "F_(cap,op)" = "12338.256 N", "ΔT_rel" = "113.63636 ΔK" }
tolerance = "1e-6"
checks = "fail"
+++

## Interference fit

A hub (outer member, subscript o) pressed or shrunk on a shaft (inner member, subscript i), after Shigley Secs. 3-16 and 7-8. The diametral interference δ sets a contact pressure p by the Lamé solution for thick-walled cylinders, each part with its own modulus and Poisson's ratio (Eq. 7-39). The pressure puts the hub bore in tangential tension and the shaft in compression; with the radial stress −p, the von Mises stress at the hub bore and in the shaft is checked against yield at the greatest interference. If either part yields, the full pressure is not reached. The fit holds torque and axial force by friction over the contact area, counted at the least interference (Eqs. 7-48, 7-49). Torque and thrust together share one friction force, so the demand is √(F² + (2T/d)²) against π f p l d.

The least and greatest interferences come from the tolerances (Shigley Eqs. 7-42, 7-43: shaft maximum less hole minimum, and so on), for example an ISO 286 preferred fit from Machinery's Table 13. The theory assumes a hub as long as the contact; the pressure peaks at the hub ends, which Shigley covers with a stress concentration factor seldom above 2. Friction coefficients of press fits commonly lie around 0.1 to 0.2 and depend strongly on finish and lubrication.

For a shrink fit, heat the hub, or cool the shaft, by enough to open the greatest interference plus an assembly clearance (Machinery's p. 627). A hub of greater expansion than the shaft (aluminium on steel) loosens as both warm up in service.

### Geometry and interference

```calc {description="Nominal diameter of the fit"}
d = 50 mm_
```

```calc {description="Bore of a hollow shaft (0 for solid)"}
d_i = 0 mm_
```

```calc {description="Outside diameter of the hub"}
d_o = 100 mm_
```

```calc {description="Length of the contact (hub length)"}
l = 60 mm_
```

```calc {description="Least diametral interference (≤ 0 for a transition fit)"}
δ_min = 20 μm_
```

```calc {description="Greatest diametral interference"}
δ_max = 50 μm_
```

### Materials

```calc {description="Young's modulus of the shaft"}
E_i = 207 GPa_
```

```calc {description="Poisson's ratio of the shaft"}
ν_i = 0.3_
```

```calc {description="Young's modulus of the hub"}
E_o = 207 GPa_
```

```calc {description="Poisson's ratio of the hub"}
ν_o = 0.3_
```

```calc {description="Tensile yield strength of the shaft"}
S_(y,i) = 400 MPa_
```

```calc {description="Tensile yield strength of the hub"}
S_(y,o) = 300 MPa_
```

```calc {description="Design factor against yield"}
n_d = 1.5_
```

### Contact pressure

```calc
function fitPressure(δ, d, d_i, d_o, E_i, ν_i, E_o, ν_o)
    return δ / (d ((d_o^2 + d^2) / (E_o (d_o^2 - d^2)) + ν_o / E_o + (d^2 + d_i^2) / (E_i (d^2 - d_i^2)) - ν_i / E_i))
end
```

```calc
> Contact pressure at the greatest interference, Eq. 7-39
p_max = fitPressure(δ_max, d, d_i, d_o, E_i, ν_i, E_o, ν_o) = ?? MPa
> Contact pressure at the least interference (none for a clearance)
p_min = fitPressure(max(δ_min, 0 'mm'), d, d_i, d_o, E_i, ν_i, E_o, ν_o) = ?? MPa
```

### Stresses at the greatest interference

```calc
> Tangential stress at the hub bore, Eq. 7-45 (its largest)
σ_(t,o) = p_max (d_o^2 + d^2) / (d_o^2 - d^2) = ?? MPa
> Tangential stress at the hub's outside surface, Eq. 3-49
σ_(t,o,OD) = 2 p_max d^2 / (d_o^2 - d^2) = ?? MPa
> Tangential stress at the shaft surface, Eq. 7-44
σ_(t,i) = -p_max (d^2 + d_i^2) / (d^2 - d_i^2) = ?? MPa
> Tangential stress at the shaft bore (the radial stress is zero there), Eq. 3-49
σ_(t,i,bore) = { -2 p_max d^2 / (d^2 - d_i^2) if d_i > 0 'mm'; -p_max otherwise } = ?? MPa
> Von Mises stress at the hub bore, σr = −p, Eq. 5-13
σ_o′ = sqrt(σ_(t,o)^2 + σ_(t,o) p_max + p_max^2) = ?? MPa
> Von Mises stress in the shaft: the larger of its surface and its bore
σ_i′ = max(sqrt(σ_(t,i)^2 + σ_(t,i) p_max + p_max^2), abs(σ_(t,i,bore))) = ?? MPa
> Yield factor of safety of the hub
n_o = S_(y,o) / σ_o′ = ??
> Yield factor of safety of the shaft
n_i = S_(y,i) / σ_i′ = ??
```

### Holding capacity and assembly

```calc {description="Coefficient of friction in service"}
f = 0.12_
```

```calc {description="Coefficient of friction when pressing on"}
f_as = 0.12_
```

```calc {description="Torque the fit must carry"}
T = 0 N·m_
```

```calc {description="Axial force the fit must carry"}
F_x = 0 N_
```

```calc
> Torque the fit holds at the least interference, Eq. 7-49
T_cap = π / 2 f p_min l d^2 = ?? N·m
> Axial force the fit holds at the least interference, Eq. 7-48
F_cap = π f p_min l d = ?? N
> Friction force needed for the torque and thrust together
F_dem = hypot(F_x, 2 T / d) = ?? N
> Force to press the hub on at the greatest interference, Eq. 7-48
F_press = π f_as p_max l d = ?? kN
```

### Shrink fit and operating temperature

```calc {description="Coefficient of thermal expansion of the shaft"}
α_i = 12e-6 1/K_
```

```calc {description="Coefficient of thermal expansion of the hub"}
α_o = 12e-6 1/K_
```

```calc {description="Diametral clearance wanted for assembly"}
c = 0 mm_
```

```calc {description="Operating temperature above assembly temperature, both parts"}
ΔT_op = 0 ΔK_
```

```calc
> Heat the hub this much above the shaft to slide it on, Machinery's p. 627
ΔT_hub = (δ_max + c) / (α_o d) = ?? ΔK
> Or cool the shaft this much below the hub
ΔT_shaft = (δ_max + c) / (α_i d) = ?? ΔK
> Least interference at the operating temperature (none once lost)
δ_op = max(δ_min + (α_i - α_o) ΔT_op d, 0 'mm') = ?? μm
> Contact pressure at the operating temperature
p_op = fitPressure(δ_op, d, d_i, d_o, E_i, ν_i, E_o, ν_o) = ?? MPa
> Torque held at the operating temperature
T_op = π / 2 f p_op l d^2 = ?? N·m
> Axial force held at the operating temperature
F_(cap,op) = π f p_op l d = ?? N
```

```calc {description="Temperature rise at which a hub of greater expansion loses the least interference (0 if it never does)"}
ΔT_rel = { δ_min / ((α_o - α_i) d) if α_o > α_i; 0 'ΔK' otherwise } = ?? ΔK
```

### Checks

```calc
@test n_o ≥ n_d
```

```calc
@test n_i ≥ n_d
```

```calc
@test F_dem ≤ F_cap
```

```calc
@test F_dem ≤ F_(cap,op)
```

```calc
@test δ_op > 0 'mm'
```
