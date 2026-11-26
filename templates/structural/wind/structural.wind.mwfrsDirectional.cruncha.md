+++
cruncha-template = 1
id = "structural.wind.mwfrsDirectional"
version = "1.1.0"
title = "Wind on buildings, MWFRS directional procedure, ASCE 7-10"
summary = "Design wind pressures on the walls and roof of an enclosed or partially enclosed rigid building (Eq. 27.4-1), each with positive and negative internal pressure, and the net windward plus leeward pressure the frames carry, from qz and qh (structural.wind.velocityPressure)."
category = ["Structural engineering", "Wind"]
tags = ["wind", "ASCE 7", "ASCE 7-10", "MWFRS", "directional procedure", "Chapter 27", "Eq. 27.4-1", "Cp", "GCpi", "windward", "leeward", "side wall", "roof", "building"]
from = "lacewing:structural.windBuilding"
license = "GPL-3.0-or-later"
inputs = ["q_h", "q_z", "G", "GC_pi", "C_(p,ww)", "C_(p,lw)", "C_(p,sw)", "C_(p,r1)", "C_(p,r2)"]
outputs = ["p_(ww,1)", "p_(ww,2)", "p_(lw,1)", "p_(lw,2)", "p_(sw,1)", "p_(sw,2)", "p_(r1,1)", "p_(r1,2)", "p_(r2,1)", "p_(r2,2)", "p_net"]

[[references]]
work = "ASCE/SEI 7-10 Minimum Design Loads for Buildings and Other Structures"
edition = "2010"
locator = "Sec. 27.4.1 Eq. 27.4-1; Fig. 27.4-1 (Cp); Table 26.11-1 (GCpi); Eq. 27.3-1 (qz, qh); Sec. 26.9 (G); Sec. 27.1.5 (minimum loads)"

[[references]]
work = "Wind Design Manual Based on the 2018 IBC and ASCE/SEI 7-16 (ICC)"
locator = "Design Example 5 (tornado storm shelter), Table 5-3 MWFRS wind loads"

[[tests]]
name = "Wind Design Manual: tornado shelter, 250 mph, exposure C, h 12 ft, partially enclosed"
source = "Wind Design Manual Based on the 2018 IBC and ASCE/SEI 7-16, Design Example 5, Table 5-3 (verified in the PDF; same Eq. 27.4-1, Cp and GCpi in ASCE 7-16, Ke = 1, Kd = 1.0 by ICC 500): q 136.0 psf at h = z = 12 ft (velocityPressure gives 135.82 psf, 0.13 % less); windward 17.7 / 167.3, leeward −132.6 / 17.0, side −155.7 / −6.1, roof 0 to h −178.8 / −29.2, roof h to 2h −132.6 / 17.0 psf"
inputs = { q_h = "136.0 psf", q_z = "136.0 psf", GC_pi = "0.55" }
expect = { "p_(ww,1)" = "17.7 psf", "p_(ww,2)" = "167.3 psf", "p_(lw,1)" = "-132.6 psf", "p_(lw,2)" = "17.0 psf", "p_(sw,1)" = "-155.7 psf", "p_(sw,2)" = "-6.1 psf", "p_(r1,1)" = "-178.8 psf", "p_(r1,2)" = "-29.2 psf", "p_(r2,1)" = "-132.6 psf", "p_(r2,2)" = "17.0 psf" }
tolerance = "0.5%"

[[tests]]
name = "100 × 60 ft enclosed building, h = 30 ft, 115 mph, exposure C"
source = "By hand (Lacewing Lab's opening case, flat-roof zones): qh (velocityPressure: exposure C, h 30 ft) = 0.00256 × 0.98225 × 0.85 × 115² = 28.267 psf; qz at 15 ft (Kz 0.84888) = 24.429 psf; windward 24.429 × 0.85 × 0.8 ∓ 28.267 × 0.18 = 11.524 / 21.700 psf; leeward 28.267 × 0.85 × (−0.5) ∓ 5.088 = −17.101 / −6.925 psf; net at h 28.267 × 0.85 × 1.3 = 31.235 psf"
inputs = { q_h = "28.26687 psf", q_z = "24.42885 psf" }
expect = { "p_(ww,1)" = "11.5236 psf", "p_(ww,2)" = "21.6997 psf", "p_(lw,1)" = "-17.1015 psf", "p_(lw,2)" = "-6.9254 psf", "p_(sw,1)" = "-21.9068 psf", "p_(r1,1)" = "-26.7122 psf", p_net = "31.2349 psf" }
tolerance = "1e-4"

[[tests]]
name = "60 ft building in exposure B at 130 mph, partially enclosed"
source = "By hand: qh = 31.406 psf and qz at 40 ft = 27.971 psf (velocityPressure's worked example: Kh 0.85403, Kz 0.76061); windward 27.971 × 0.85 × 0.8 − 31.406 × 0.55 = 19.020 − 17.274 = 1.747 psf; leeward −0.5: −13.348 ∓ 17.274 = −30.621 / 3.926 psf"
inputs = { q_h = "31.40645 psf", q_z = "27.97094 psf", GC_pi = "0.55" }
expect = { "p_(ww,1)" = "1.7467 psf", "p_(lw,1)" = "-30.6215 psf", "p_(lw,2)" = "3.9258 psf" }
tolerance = "1e-4"
+++

## Wind on buildings: MWFRS, directional procedure (ASCE 7-10)

Design wind pressures on the main wind force resisting system of an enclosed or partially enclosed **rigid** building of any height, by Eq. 27.4-1, p = q G C_p − q_i (GC_pi). The windward wall takes q_z at its height z; every other surface takes q_h at the mean roof height h. Internal pressure acts on all inside surfaces at once, both outward (+GC_pi, pressures numbered 1) and inward (−GC_pi, numbered 2); design each surface for the worse.

The velocity pressures q_h and q_z are inputs: compute them with the velocity pressure template (structural.wind.velocityPressure) and bind them to its q_h and q_z. Inputs that ASCE 7 gives only in a figure or a table are **inputs here**, with the figure cited: the external pressure coefficients C_p (Figure 27.4-1) and the internal pressure coefficient GC_pi (Table 26.11-1). G can come from the gust-effect factor template (Eq. 26.9-6).

Limits:
- Rigid buildings only (n_1 ≥ 1 Hz). A flexible building needs G_f (Sec. 26.9.5) in place of G.
- q_i is taken as q_h throughout. Sec. 27.4.1 lets a partially enclosed building take q_z at the highest opening for positive internal pressure; q_h is conservative where that opening is lower.
- The load cases of Figure 27.4-8 (partial and torsional loading) and the minimum loads of Sec. 27.1.5 (16 psf on walls, 8 psf on the roof's vertical projection) are separate checks.
- ASCE 7-10. ASCE 7-16 kept this procedure but changed the maps and added the ground elevation factor K_e; check the edition that applies.

### Velocity pressures

```calc {description="Velocity pressure at the mean roof height h, Eq. 27.3-1, from structural.wind.velocityPressure"}
q_h = 28.27 psf_
```

```calc {description="Velocity pressure at the height z on the windward wall, Eq. 27.3-1, from structural.wind.velocityPressure"}
q_z = 24.43 psf_
```

### Building

G = 0.85 is permitted for a rigid building (Sec. 26.9.1); Eq. 26.9-6 often gives a little less.

```calc {description="Gust-effect factor, Sec. 26.9 (from structural.wind.gustRigid)"}
G = 0.85_
```

Section 26.11.1.1 reduces GC_pi for a partially enclosed single large volume.

```calc {description="Internal pressure coefficient (magnitude) for the enclosure class, ASCE 7-10 Table 26.11-1 (0.18 is for an enclosed building)"}
GC_pi = 0.18_
```

### External pressure coefficients (Figure 27.4-1)

Walls take C_p by surface (windward, leeward by L/B, side). Roofs take q_h: for wind normal to the ridge with θ ≥ 10°, the windward and leeward slopes take the figure's values by θ and h/L (the windward slope has two, a suction and a pressure: check both). For θ < 10° and wind parallel to the ridge, the roof takes zones by distance from the windward edge. Enter the two roof surfaces or zones you need. The figure permits linear interpolation in L/B, h/L and θ between values of the same sign, and a reduction of its largest roof suction by area.

```calc {description="Windward wall Cp, ASCE 7-10 Fig. 27.4-1 (with qz)"}
C_(p,ww) = 0.8_
```

```calc {description="Leeward wall Cp by L/B, ASCE 7-10 Fig. 27.4-1 (−0.5 is for L/B ≤ 1)"}
C_(p,lw) = -0.5_
```

```calc {description="Side wall Cp, ASCE 7-10 Fig. 27.4-1"}
C_(p,sw) = -0.7_
```

```calc {description="Roof Cp, first surface or zone (windward slope, or the zone from the windward edge), ASCE 7-10 Fig. 27.4-1 (−0.9 is for 0 to h, θ < 10°, h/L ≤ 0.5)"}
C_(p,r1) = -0.9_
```

```calc {description="Roof Cp, second surface or zone (leeward slope, or the next zone), ASCE 7-10 Fig. 27.4-1 (−0.5 is for h to 2h, θ < 10°, h/L ≤ 0.5)"}
C_(p,r2) = -0.5_
```

### Design pressures, Eq. 27.4-1

Positive pressures act toward the surface, negative away from it. Pressures numbered 1 include +GC_pi, 2 include −GC_pi.

```calc
> Windward wall at z, with +GCpi
p_(ww,1) = q_z G C_(p,ww) - q_h GC_pi = ?? psf
> Windward wall at z, with −GCpi
p_(ww,2) = q_z G C_(p,ww) + q_h GC_pi = ?? psf
```

```calc
> Leeward wall, with +GCpi
p_(lw,1) = q_h G C_(p,lw) - q_h GC_pi = ?? psf
> Leeward wall, with −GCpi
p_(lw,2) = q_h G C_(p,lw) + q_h GC_pi = ?? psf
```

```calc
> Side walls, with +GCpi
p_(sw,1) = q_h G C_(p,sw) - q_h GC_pi = ?? psf
> Side walls, with −GCpi
p_(sw,2) = q_h G C_(p,sw) + q_h GC_pi = ?? psf
```

```calc
> Roof, first surface or zone, with +GCpi
p_(r1,1) = q_h G C_(p,r1) - q_h GC_pi = ?? psf
> Roof, first surface or zone, with −GCpi
p_(r1,2) = q_h G C_(p,r1) + q_h GC_pi = ?? psf
```

```calc
> Roof, second surface or zone, with +GCpi
p_(r2,1) = q_h G C_(p,r2) - q_h GC_pi = ?? psf
> Roof, second surface or zone, with −GCpi
p_(r2,2) = q_h G C_(p,r2) + q_h GC_pi = ?? psf
```

The internal pressures cancel across the building, so the frames resisting wind in this direction carry the windward less the leeward pressure. At roof level:

```calc {description="Net windward plus leeward wall pressure at h"}
p_net = q_h G (C_(p,ww) - C_(p,lw)) = ?? psf
```
