+++
cruncha-template = 1
id = "structural.snow.stepDrift"
version = "1.0.0"
title = "Snow drift at a roof step, ASCE 7-10 Sec. 7.7.1"
summary = "The snow drift on a lower roof next to a higher one: snow density, balanced depth and clear height, leeward and windward drift heights by Fig. 7-9, the governing drift height and width, the drift surcharge and the peak load at the step."
category = ["Structural engineering", "Loads"]
tags = ["ASCE 7", "ASCE 7-10", "snow", "drift", "roof step", "leeward", "windward", "lower roof", "surcharge", "snow density", "hd", "pd"]
from = "lacewing:structural.snow"
license = "GPL-3.0-or-later"
inputs = ["p_g", "p_s", "h", "l_(u,up)", "l_(u,low)"]
outputs = ["γ", "h_b", "h_c", "r_c", "h_(d,lee)", "h_(d,wind)", "h_d", "w_d", "p_d", "p_peak"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 7.7.1 (Eq. 7.7-1); Figs. 7-7, 7-8 and 7-9"

[[references]]
work = "ASCE/SEI 7-10 Commentary, Chapter C7"
edition = "2010"
locator = "C7 Example 3 (drift on a lower roof)"

[[tests]]
name = "ASCE 7-10 C7 Example 3: unheated low roof 10 ft below a 100 ft high roof"
source = "ASCE 7-10 Commentary C7, Example 3 (verified against the PDF): pg 40, low-roof pf 27 psf (26.88), γ = 19 pcf, hb = 1.4 ft, hc = 8.6 ft, hc/hb = 6.1; leeward hd = 3.8 ft (lu 100 ft), windward 3/4 × 4.8 = 3.6 ft (lower roof 170 ft); leeward governs, w = 4hd = 15.2 ft, pd = 3.8 × 19 = 72 psf. By hand with γ unrounded (19.2 pcf): hd = 0.43 × 100^(1/3) × 50^(1/4) − 1.5 = 3.807 ft, windward 3.625 ft, w = 15.23 ft, pd = 73.1 psf."
inputs = { p_g = "40 psf", p_s = "26.88 psf", h = "10 ft", "l_(u,up)" = "100 ft", "l_(u,low)" = "170 ft" }
expect = { "γ" = "19.2 lbf/ft^3", h_b = "1.4 ft", h_c = "8.6 ft", r_c = "6.14", "h_(d,lee)" = "3.807 ft", "h_(d,wind)" = "3.625 ft", h_d = "3.807 ft", w_d = "15.23 ft", p_d = "73.1 psf" }
tolerance = "0.1%"

[[tests]]
name = "Lacewing: the drift fills the step (hd capped at hc)"
source = "Lacewing Lab asce7_loads_test: pg 30, lu 100 ft, lower roof 50 ft, hc 2.5 ft, γ 17.9 pcf: Fig. 7-9 gives 3.52 ft > hc, so hd = hc = 2.5 ft, w = 4 × 3.52²/2.5 = 19.8 ft, pd = 2.5 × 17.9 = 44.75 psf. Here ps = 21 psf and h = 2.5 + 21/17.9 ft, so hc = 2.5 ft."
inputs = { p_g = "30 psf", p_s = "21 psf", h = "3.6731844 ft", "l_(u,up)" = "100 ft", "l_(u,low)" = "50 ft" }
expect = { "γ" = "17.9 lbf/ft^3", h_c = "2.5 ft", "h_(d,lee)" = "3.52 ft", h_d = "2.5 ft", w_d = "19.82 ft", p_d = "44.75 psf", p_peak = "65.75 psf" }
tolerance = "0.3%"

[[tests]]
name = "A short upper roof counts as 20 ft"
source = "By hand, Fig. 7-9 (lu not taken below 20 ft, as Lacewing does): pg 30, lu 5 ft → 20 ft, hd = 0.43 × 20^(1/3) × 40^(1/4) − 1.5 = 0.43 × 2.7144 × 2.5149 − 1.5 = 1.4354 ft; with ps 21 psf and h 10 ft, hc = 8.83 ft, so w = 4hd = 5.742 ft and pd = 1.4354 × 17.9 = 25.69 psf"
inputs = { p_g = "30 psf", p_s = "21 psf", h = "10 ft", "l_(u,up)" = "5 ft", "l_(u,low)" = "5 ft" }
expect = { "h_(d,lee)" = "1.4354 ft", h_d = "1.4354 ft", w_d = "5.742 ft", p_d = "25.69 psf" }
tolerance = "0.05%"

[[tests]]
name = "Heavy ground snow: density capped at 30 pcf, and a small step that the drift overtops"
source = "By hand: pg 150 psf, γ = 0.13 × 150 + 14 = 33.5 → 30 pcf (Eq. 7.7-1); ps 80 psf, hb = 2.667 ft, h 4 ft, hc = 1.333 ft; leeward hd = 0.43 × 200^(1/3) × 160^(1/4) − 1.5 = 0.43 × 5.848 × 3.557 − 1.5 = 7.444 ft > hc, so w = 4 × 7.444²/1.333 = 166.3 ft, capped at 8hc = 10.67 ft; pd = hc γ = 40 psf"
inputs = { p_g = "150 psf", p_s = "80 psf", h = "4 ft", "l_(u,up)" = "200 ft", "l_(u,low)" = "50 ft" }
expect = { "γ" = "30 lbf/ft^3", h_c = "1.33333 ft", "h_(d,lee)" = "7.444 ft", h_d = "1.33333 ft", w_d = "10.6667 ft", p_d = "40 psf" }
tolerance = "0.05%"
+++

## Snow drift at a roof step, ASCE 7-10

The drift that forms on a lower roof in the wind shadow of a higher part of the same building (Sec. 7.7.1). Snow blown off the upper roof makes a **leeward** drift; snow blown across the lower roof against the step makes a **windward** drift, taken as three quarters of the Fig. 7-9 height for the lower roof's length. The larger governs. The drift is a triangle of surcharge on top of the balanced load, peaking at the step and falling to zero over the drift width (Fig. 7-8).

Not covered: drifts at parapets and rooftop units (Sec. 7.8, three quarters of the Fig. 7-9 height), adjacent structures (Sec. 7.7.2), sliding snow from a sloped upper roof (Sec. 7.9), and truncation of a drift wider than the lower roof (the drift is cut off at the far edge, not reduced to zero there). Chapter 7's drift equation is written in ft and psf; the calcs convert at the edges.

### Roofs

The balanced load on the lower roof is its p_s (or p_f on a flat roof): the roof snow load template gives it.

```calc {description="Ground snow load, Fig. 7-1"}
p_g = 30 psf_
```

```calc {description="Balanced snow load on the lower roof"}
p_s = 21 psf_
```

```calc {description="Height of the step, lower roof to the edge of the upper roof (with any parapet)"}
h = 4 ft_
```

```calc {description="Length of the upper roof upwind of the step (leeward drift)"}
l_(u,up) = 100 ft_
```

```calc {description="Length of the lower roof upwind of the step (windward drift)"}
l_(u,low) = 50 ft_
```

### Balanced snow and clear height

```calc {description="Snow density, Eq. 7.7-1"}
γ = min(0.13 p_g / (1 'psf') + 14, 30) × 1 'lbf/ft^3' = ?? lbf/ft^3
```

```calc
> Depth of the balanced snow on the lower roof
h_b = p_s / γ = ?? ft
> Clear height from the balanced snow to the upper roof
h_c = max(h - h_b, 0 'ft') = ?? ft
> Clear height over balanced depth: below 0.2 no drift load is required
r_c = h_c / h_b = ??
```

### Drift height and width

Fig. 7-9's equation gives the drift height in ft from the upwind length in ft (not taken below 20 ft) and the ground snow load in psf.

```calc
function driftHeight(l, p)
    return max(0.43 max(l, 20)^(1/3) (p + 10)^(1/4) - 1.5, 0)
end
```

```calc
> Leeward drift height, from the upper roof, Fig. 7-9
h_(d,lee) = driftHeight(l_(u,up) / (1 'ft'), p_g / (1 'psf')) × 1 'ft' = ?? ft
> Windward drift height, three quarters of Fig. 7-9 for the lower roof
h_(d,wind) = 0.75 driftHeight(l_(u,low) / (1 'ft'), p_g / (1 'psf')) × 1 'ft' = ?? ft
```

When the larger height fits below the upper roof, the drift is that high and four times as wide. When it doesn't, the drift is cut to h_c and widened to keep its area, w = 4h_d²/h_c. Either way the width is at most 8h_c.

```calc
> Larger of the two drift heights
h_(d,0) = max(h_(d,lee), h_(d,wind)) = ?? ft
> Drift height, not above the clear height
h_d = min(h_(d,0), h_c) = ?? ft
> Drift width
w_d = {0 'ft' if h_c ≤ 0 'ft'; 4 h_(d,0) if h_(d,0) ≤ h_c; min(4 h_(d,0)^2 / h_c, 8 h_c) otherwise} = ?? ft
```

### Drift load

```calc {description="Maximum intensity of the drift surcharge at the step"}
p_d = h_d γ = ?? psf
```

```calc {description="Peak snow load at the step, balanced plus drift"}
p_peak = p_s + p_d = ?? psf
```
