+++
cruncha-template = 1
id = "mechanical.strainRosettes.rectangular"
version = "1.0.0"
title = "Rectangular strain rosette (0°, 45°, 90°)"
summary = "Principal strains and their direction from the three readings of a rectangular rosette, the strains on the gauge axes, and the plane stresses at the surface behind them by Hooke's law: principal stresses, maximum shear, von Mises, and the strain normal to the surface."
category = ["Mechanical engineering", "Strain measurement"]
tags = ["strain gauge", "rosette", "rectangular rosette", "0-45-90", "principal strain", "Mohr's circle", "plane stress", "Hooke's law", "von Mises", "experimental stress analysis"]
from = "lacewing:mechanical.strainRosettes"
license = "GPL-3.0-or-later"
inputs = ["ε_a", "ε_b", "ε_c", "E", "ν"]
outputs = ["ε_x", "ε_y", "γ_xy", "ε_1", "ε_2", "θ_1", "γ_max", "σ_1", "σ_2", "ε_3", "τ_max", "σ′", "σ_x", "σ_y", "τ_xy"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 5.2 to 5.5 (pp. 195-203): strain transformation and Mohr's strain circle (Fig. 5.4); rectangular rosette, Eqs. 5.3 and 5.4, Sample Problem 5.2; Hooke's law for plane stress, Eq. 5.5"

[[tests]]
name = "Juvinall Sample Problem 5.2: 2600, −200, 450 µε"
source = "Juvinall & Marshek 5th, Sample Problem 5.2 (pp. 200-201), as in Lacewing Lab rosette_test: ε1 = 3558, ε2 = −508 µm/m, ε1 29° clockwise of gauge a (readings relabelled counterclockwise as in Fig. 5.10b)"
inputs = { "ε_a" = "2600 µm/m", "ε_b" = "-200 µm/m", "ε_c" = "450 µm/m" }
expect = { "ε_1" = "3558 µm/m", "ε_2" = "-508 µm/m", "θ_1" = "-29 °" }
tolerance = "0.2%"

[[tests]]
name = "Hand check: Sample Problem 5.2 on steel, E 200 GPa, ν 0.3"
source = "By hand: centre (2600 + 450)/2 = 1525, radius √((2800² + 650²)/2) = 2032.5476 µε (Eq. 5.3); γxy = 2(−200) − 2600 − 450 = −3450 µε; θ1 = ½ atan2(γxy, εx − εy) = ½ atan2(−3450, 2150) = −29.03466°; σ1,2 = 200 GPa/0.91 × (ε1,2 + 0.3 ε2,1) = 748.414, 123.015 MPa; σ′ = 695.119 MPa; τmax = σ1/2 = 374.207 MPa (both principal stresses tensile); ε3 = −0.3/0.7 × 3050 = −1307.143 µε; σx = 601.099, σy = 270.330, τxy = 76.923 GPa × −3450 µε = −265.385 MPa"
inputs = { "ε_a" = "2600 µm/m", "ε_b" = "-200 µm/m", "ε_c" = "450 µm/m", E = "200 GPa", "ν" = "0.3" }
expect = { "ε_x" = "2600 µm/m", "ε_y" = "450 µm/m", "γ_xy" = "-3450 µm/m", "ε_1" = "3557.547662 µm/m", "ε_2" = "-507.5476624 µm/m", "γ_max" = "4065.095325 µm/m", "θ_1" = "-29.03465895 °", "σ_1" = "748.4139261 MPa", "σ_2" = "123.0146453 MPa", "σ′" = "695.1187913 MPa", "τ_max" = "374.206963 MPa", "ε_3" = "-1307.142857 µm/m", "σ_x" = "601.0989011 MPa", "σ_y" = "270.3296703 MPa", "τ_xy" = "-265.3846154 MPa" }
tolerance = "1e-8"

[[tests]]
name = "Hand check: principal stresses of the same sign, the largest shear out of plane"
source = "By hand: 1000, 800, 400 µε on steel (E 200 GPa, ν 0.3): ε1,2 = 700 ± √((200² + 400²)/2) = 1016.228, 383.772 µε; γxy = 1600 − 1400 = 200 µε, θ1 = ½ atan2(200, 600) = 9.21747°; σ1,2 = 219.780 GPa × (ε1,2 + 0.3 ε2,1) = 248.650, 151.350 MPa; with σ3 = 0, τmax = σ1/2 = 124.325 MPa"
inputs = { "ε_a" = "1000 µm/m", "ε_b" = "800 µm/m", "ε_c" = "400 µm/m", E = "200 GPa", "ν" = "0.3" }
expect = { "ε_1" = "1016.228 µm/m", "ε_2" = "383.772 µm/m", "θ_1" = "9.21747 °", "σ_1" = "248.650 MPa", "σ_2" = "151.350 MPa", "τ_max" = "124.325 MPa" }
tolerance = "0.01%"

[[tests]]
name = "Hand check: principal stresses of opposite sign, the largest shear in plane"
source = "By hand: 1000, −800, −400 µε on steel (E 200 GPa, ν 0.3): ε1,2 = 300 ± √((1800² + 400²)/2) = 1603.840, −1003.840 µε; γxy = −1600 − 1000 + 400 = −2200 µε, θ1 = ½ atan2(γxy, εx − εy) = ½ atan2(−2200, 1400) = −28.7644°; σ1,2 = 219.780 GPa × (ε1,2 + 0.3 ε2,1) = 286.305, −114.877 MPa; τmax = (σ1 − σ2)/2 = 200.591 MPa"
inputs = { "ε_a" = "1000 µm/m", "ε_b" = "-800 µm/m", "ε_c" = "-400 µm/m", E = "200 GPa", "ν" = "0.3" }
expect = { "ε_1" = "1603.840 µm/m", "ε_2" = "-1003.840 µm/m", "σ_1" = "286.305 MPa", "σ_2" = "-114.877 MPa", "τ_max" = "200.591 MPa", "θ_1" = "-28.7644 °" }
tolerance = "0.01%"

+++

## Rectangular strain rosette (0°, 45°, 90°)

Three gauges stacked or side by side at 0°, 45° and 90° fix the plane strain state at a point on a free surface, whatever the directions of the principal strains. This template finds the principal strains and their direction, then the stresses behind them for a linear elastic, isotropic material.

Number the gauges a, b, c counterclockwise: b at 45° from a, c at 90°. A rosette read the other way round is renumbered first (Juvinall does this in Fig. 5.10b). Strains are tension positive. The readings are taken as true strains along each grid: corrected for bridge nonlinearity and temperature (the strain gauge bridge templates do that for one gauge) and for transverse sensitivity, which this template does not correct.

### Readings

```calc {description="Strain read by gauge a, the reference direction (0°)"}
ε_a = 2600 µm/m_
```

```calc {description="Strain read by gauge b, 45° counterclockwise from a"}
ε_b = -200 µm/m_
```

```calc {description="Strain read by gauge c, 90° counterclockwise from a"}
ε_c = 450 µm/m_
```

### Material

```calc {description="Young's modulus of the part"}
E = 200 GPa_
```

```calc {description="Poisson's ratio of the part (below 0.5)"}
ν = 0.3_
```

### Strains on the gauge axes

Axis x runs along gauge a and y along gauge c. The 45° gauge gives the shear strain through the strain transformation ε(θ) = (εx + εy)/2 + (εx − εy)/2·cos 2θ + γxy/2·sin 2θ.

```calc
> Normal strain along gauge a
ε_x = ε_a = ?? µm/m
> Normal strain along gauge c
ε_y = ε_c = ?? µm/m
> Engineering shear strain on the x and y axes
γ_xy = 2 ε_b - ε_a - ε_c = ?? µm/m
```

### Principal strains

The centre and radius of Mohr's strain circle are Juvinall's Eq. 5.3. The direction of ε₁ is θ₁, counterclockwise from gauge a when positive and clockwise when negative, between −90° and 90°; ε₂ lies 90° from it. (Juvinall's Eq. 5.4 gives the same axes as tan 2α, α clockwise, and leaves which axis is ε₁ to a rule; the two-argument arctangent names it directly.)

```calc
> Centre of Mohr's strain circle, Eq. 5.3
ε_m = (ε_a + ε_c) / 2 = ?? µm/m
> Its radius, Eq. 5.3
R_ε = sqrt(((ε_a - ε_b)^2 + (ε_b - ε_c)^2) / 2) = ?? µm/m
> Maximum principal strain
ε_1 = ε_m + R_ε = ?? µm/m
> Minimum principal strain
ε_2 = ε_m - R_ε = ?? µm/m
> Maximum in-plane shear strain, on axes at 45° to the principal ones
γ_max = ε_1 - ε_2 = ?? µm/m
```

```calc {description="Angle from gauge a to the ε₁ axis, counterclockwise positive"}
θ_1 = atan2(ε_x - ε_y, γ_xy) / 2 = ?? °
```

### Stresses at the surface

At a free surface σ₃ = 0, and Hooke's law for plane stress (Eq. 5.5) gives the principal stresses. Taking σ = E·ε for each principal strain is wrong in a biaxial state: Juvinall's §5.5 shows how far off it is.

```calc
> Maximum principal stress, Eq. 5.5
σ_1 = E / (1 - ν^2) (ε_1 + ν ε_2) = ?? MPa
> Minimum principal stress, Eq. 5.5
σ_2 = E / (1 - ν^2) (ε_2 + ν ε_1) = ?? MPa
> Strain normal to the surface, which no gauge reads, Eq. 5.5
ε_3 = -ν / (1 - ν) (ε_1 + ε_2) = ?? µm/m
```

```calc {description="Maximum shear stress, the largest of the three Mohr circles with σ₃ = 0"}
τ_max = (max(σ_1, 0 'MPa') - min(σ_2, 0 'MPa')) / 2 = ?? MPa
```

```calc {description="Von Mises stress, plane stress"}
σ′ = sqrt(σ_1^2 - σ_1 σ_2 + σ_2^2) = ?? MPa
```

The stresses on the gauge axes are often what a drawing or a finite element model gives, for comparison.

```calc
> Normal stress along gauge a
σ_x = E / (1 - ν^2) (ε_x + ν ε_y) = ?? MPa
> Normal stress along gauge c
σ_y = E / (1 - ν^2) (ε_y + ν ε_x) = ?? MPa
> Shear stress on the gauge axes, τ = G γ with G = E / 2(1 + ν)
τ_xy = E / (2 (1 + ν)) γ_xy = ?? MPa
```

For a factor of safety against yield, take σ₁ and σ₂ (with σ₃ = 0) to the plane stress static failure template.
