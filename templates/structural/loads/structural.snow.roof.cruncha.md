+++
cruncha-template = 1
id = "structural.snow.roof"
version = "1.0.0"
title = "Roof snow load, ASCE 7-10 Chapter 7"
summary = "Flat roof snow load pf, the slope factor Cs and sloped roof (balanced) load ps, the minimum for low-slope roofs pm, and the rain-on-snow surcharge, by ASCE 7-10 Chapter 7."
category = ["Structural engineering", "Loads"]
tags = ["ASCE 7", "ASCE 7-10", "snow", "roof snow", "flat roof", "sloped roof", "balanced", "slope factor", "Cs", "Ce", "Ct", "Is", "pf", "ps", "pm", "rain-on-snow"]
from = "lacewing:structural.snow"
license = "GPL-3.0-or-later"
inputs = ["p_g", "C_e", "C_t", "I_s", "θ", "k_sl", "W_(eave)"]
outputs = ["p_f", "θ_0", "C_s", "p_s", "p_m", "p_ros", "p_(s,ros)"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 7.3 (Eq. 7.3-1), 7.3.4, 7.4 (Eq. 7.4-1), 7.4.1, 7.4.2, 7.10; Tables 1.5-2, 7-2, 7-3; Fig. 7-2"

[[references]]
work = "ASCE/SEI 7-10 Commentary, Chapter C7"
edition = "2010"
locator = "C7 Examples 1, 2 and 3"

[[tests]]
name = "ASCE 7-10 C7 Example 1: 8-on-12 gable, Hartford"
source = "ASCE 7-10 Commentary C7, Example 1 (verified against the PDF): pg 30, Ce 1.0, Ct 1.0, Is 1.0, pf = 21 psf; Cs = 0.91 (solid line, Fig. 7-2a); ps = 19 psf; pm does not apply (slope over 15°); no rain-on-snow (pg > 20). By hand Cs = (70 − 33.69)/40 = 0.908, ps = 19.07 psf."
inputs = { p_g = "30 psf", C_e = "1.0", C_t = "1.0", I_s = "1.0", "θ" = "33.690068 °", k_sl = "0", "W_(eave)" = "30 ft" }
expect = { p_f = "21 psf", C_s = "0.91", p_s = "19.07 psf", p_m = "0 psf", p_ros = "0 psf" }
tolerance = "0.3%"

[[tests]]
name = "ASCE 7-10 C7 Example 2: vaulted theater, at the eaves (41°)"
source = "ASCE 7-10 Commentary C7, Example 2 (verified against the PDF): pg 25, Ce 0.9, Ct 1.0, I 1.1, pf = 17 psf; at the eaves (41°) Cs = 0.72 from Fig. 7-2a. By hand pf = 0.7 × 0.9 × 1.1 × 25 = 17.33 psf, Cs = 29/40 = 0.725, ps = 12.56 psf (the example rounds to 17 × 0.72 = 12)."
inputs = { p_g = "25 psf", C_e = "0.9", C_t = "1.0", I_s = "1.1", "θ" = "41 °", k_sl = "0", "W_(eave)" = "40 ft" }
expect = { p_f = "17.325 psf", C_s = "0.72", p_s = "12.56 psf" }
tolerance = "0.8%"

[[tests]]
name = "ASCE 7-10 C7 Example 3: the unheated low roof"
source = "ASCE 7-10 Commentary C7, Example 3 (verified against the PDF): pg 40, Ce 1.0, Ct 1.2, Is 0.8: pf = 27 psf (0.7 × 1.0 × 1.2 × 0.8 × 40 = 26.88), pm = 20 × 0.8 = 16 psf"
inputs = { p_g = "40 psf", C_e = "1.0", C_t = "1.2", I_s = "0.8", "θ" = "0 °", k_sl = "0", "W_(eave)" = "50 ft" }
expect = { p_f = "26.88 psf", p_s = "26.88 psf", p_m = "16 psf", p_ros = "0 psf" }
tolerance = "1e-9"

[[tests]]
name = "Lacewing's opening values: pg 30, flat, exposure C partially exposed, heated, risk II"
source = "Lacewing Lab asce7_loads_test: pf = 0.7 × 1.0 × 1.0 × 1.0 × 30 = 21 psf; pm = 20 × Is = 20 psf (pg above 20 psf)"
inputs = { p_g = "30 psf", C_e = "1.0", C_t = "1.0", I_s = "1.0", "θ" = "0 °", k_sl = "0", "W_(eave)" = "50 ft" }
expect = { p_f = "21 psf", p_m = "20 psf" }
tolerance = "1e-9"

[[tests]]
name = "Light snow on a nearly flat, cold, sheltered roof: rain-on-snow applies"
source = "Lacewing Lab asce7_loads_test: pg 15, exposure B sheltered (Ce 1.2), unheated (Ct 1.2), risk IV (Is 1.2), W 100 ft, θ 1°: 1° < 100/50, so 5 psf rain-on-snow; by hand pf = 0.7 × 1.2 × 1.2 × 1.2 × 15 = 18.144 psf, pm = Is pg = 18 psf"
inputs = { p_g = "15 psf", C_e = "1.2", C_t = "1.2", I_s = "1.2", "θ" = "1 °", k_sl = "0", "W_(eave)" = "100 ft" }
expect = { p_ros = "5 psf", p_f = "18.144 psf", p_m = "18 psf", "p_(s,ros)" = "23.144 psf" }
tolerance = "1e-9"

[[tests]]
name = "Slope factor: warm slippery roof at 30°"
source = "Lacewing Lab asce7_loads_test: Fig. 7-2a dashed line, Cs = (70 − 30)/(70 − 5) = 40/65"
inputs = { p_g = "30 psf", C_e = "1.0", C_t = "1.0", I_s = "1.0", "θ" = "30 °", k_sl = "1", "W_(eave)" = "30 ft" }
expect = { C_s = "0.6153846154" }
tolerance = "1e-9"

[[tests]]
name = "Slope factor: cold (Ct 1.1) roof, other surface, at 50°"
source = "Lacewing Lab asce7_loads_test: Fig. 7-2b solid line, Cs = (70 − 50)/(70 − 37.5) = 20/32.5"
inputs = { p_g = "30 psf", C_e = "1.0", C_t = "1.1", I_s = "1.0", "θ" = "50 °", k_sl = "0", "W_(eave)" = "30 ft" }
expect = { C_s = "0.6153846154" }
tolerance = "1e-9"

[[tests]]
name = "Slope factor: Ct 1.2 at 45° and at 75°"
source = "Lacewing Lab asce7_loads_test: Fig. 7-2c solid line, Cs = 1 up to 45°; no snow above 70°"
inputs = { p_g = "30 psf", C_e = "1.0", C_t = "1.2", I_s = "1.0", "θ" = "75 °", k_sl = "0", "W_(eave)" = "30 ft" }
expect = { C_s = "0", p_s = "0 psf", "θ_0" = "45 °" }
tolerance = "1e-9"
+++

## Roof snow load, ASCE 7-10

The design snow loads on a roof by ASCE 7-10 Chapter 7: the flat roof load p_f from the ground snow load, its reduction for slope to the sloped roof (balanced) load p_s, the minimum p_m for low-slope roofs, and the rain-on-snow surcharge. Loads act on the horizontal projection of the roof.

Not covered: unbalanced loads on gable, hip and curved roofs, partial loading of continuous members, drifts (the roof step drift template has the drift at a step), sliding snow, ice dams at eaves, and ponding. Curved, folded plate, sawtooth and barrel vault roofs have their own slope rules (Sec. 7.4.3, 7.4.4).

### Site and roof

The ground snow load is from the map of Fig. 7-1 (Table 7-1 in Alaska), or a site-specific study where the map calls for one (the CS areas and sites above its elevation limits).

```calc {description="Ground snow load, Fig. 7-1"}
p_g = 30 psf_
```

The exposure factor depends on the terrain category and how exposed the roof is (fully, partially, sheltered): Table 7-2. The thermal factor is 1.0 for ordinary heated buildings and rises for colder roofs (Table 7-3). The importance factor is by risk category (Table 1.5-2): 1.0 for risk category II.

```calc {description="Exposure factor Ce, Table 7-2"}
C_e = 1.0_
```

```calc {description="Thermal factor Ct, Table 7-3"}
C_t = 1.0_
```

```calc {description="Snow importance factor Is, Table 1.5-2"}
I_s = 1.0_
```

```calc {description="Roof slope from horizontal"}
θ = 0°_
```

A slippery surface (metal, slate, glass, smooth membranes) counts only where it is unobstructed and there is room below the eaves for all the snow sliding off it; a warm roof also needs the insulation Sec. 7.4.1 asks for. Shingles and granular membranes are not slippery.

```calc {description="Surface: 1 unobstructed slippery, 0 other"}
k_sl = 0_
```

```calc {description="Horizontal distance from eave to ridge (rain-on-snow)"}
W_(eave) = 50 ft_
```

### Flat and sloped roof loads

```calc {description="Flat roof snow load, Eq. 7.3-1"}
p_f = 0.7 C_e C_t I_s p_g = ?? psf
```

The slope factor of Fig. 7-2 is 1 up to a corner slope, then falls linearly to 0 at 70°. The corner is 5° (slippery) or 30° (other) on a warm roof, Ct ≤ 1.0 (Fig. 7-2a); 10° or 37.5° at Ct = 1.1 (Fig. 7-2b); 15° or 45° on colder roofs (Fig. 7-2c).

```calc {description="Slope where Cs starts to fall, Fig. 7-2"}
θ_0 = {5° if C_t ≤ 1.0 and k_sl == 1; 30° if C_t ≤ 1.0; 10° if C_t < 1.15 and k_sl == 1; 37.5° if C_t < 1.15; 15° if k_sl == 1; 45° otherwise} = ?? °
```

```calc {description="Roof slope factor, Fig. 7-2"}
C_s = {1 if θ ≤ θ_0; 0 if θ ≥ 70°; (70° - θ) / (70° - θ_0) otherwise} = ??
```

```calc {description="Sloped roof (balanced) snow load, Eq. 7.4-1"}
p_s = C_s p_f = ?? psf
```

### Minimum load for low-slope roofs

Roofs sloped less than 15° (curved roofs with less than 10° from eaves to crown) take at least p_m, as a separate uniform load case that is not combined with drift, sliding, unbalanced or partial loads (Sec. 7.3.4). It is zero here for steeper roofs.

```calc {description="Minimum snow load for low-slope roofs, Sec. 7.3.4"}
p_m = {I_s p_g if θ < 15° and p_g ≤ 20 'psf'; 20 'psf' × I_s if θ < 15°; 0 'psf' otherwise} = ?? psf
```

### Rain-on-snow

Where the ground snow load is 20 psf or less (but not zero), roofs with a slope in degrees below W/50 (W in ft) carry 5 psf more on the balanced load case only (Sec. 7.10).

```calc {description="Rain-on-snow surcharge, Sec. 7.10"}
p_ros = {5 'psf' if p_g > 0 'psf' and p_g ≤ 20 'psf' and θ / (1 '°') < W_(eave) / (50 'ft'); 0 'psf' otherwise} = ?? psf
```

```calc {description="Balanced load with rain-on-snow"}
p_(s,ros) = p_s + p_ros = ?? psf
```
