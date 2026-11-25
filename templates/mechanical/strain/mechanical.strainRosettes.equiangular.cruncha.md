+++
cruncha-template = 1
id = "mechanical.strainRosettes.equiangular"
version = "1.0.0"
title = "Equiangular (delta) strain rosette (0°, 120°, 240°)"
summary = "Principal strains and their direction from the three readings of an equiangular (delta) rosette, the strains on the gauge axes, and the plane stresses at the surface behind them by Hooke's law: principal stresses, maximum shear, von Mises, and the strain normal to the surface."
category = ["Mechanical engineering", "Strain measurement"]
tags = ["strain gauge", "rosette", "delta rosette", "equiangular rosette", "0-120-240", "0-60-120", "principal strain", "Mohr's circle", "plane stress", "Hooke's law", "von Mises", "experimental stress analysis"]
from = "lacewing:mechanical.strainRosettes"
license = "GPL-3.0-or-later"
inputs = ["ε_a", "ε_b", "ε_c", "E", "ν"]
outputs = ["ε_x", "ε_y", "γ_xy", "ε_1", "ε_2", "θ_1", "γ_max", "σ_1", "σ_2", "ε_3", "τ_max", "σ′", "σ_x", "σ_y", "τ_xy"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 5.2 to 5.5 (pp. 195-203): strain transformation and Mohr's strain circle (Fig. 5.4); equiangular rosette, Eqs. 5.1 and 5.2, Sample Problem 5.1; Hooke's law for plane stress, Eq. 5.5, and its worked example on aluminium (p. 203)"

[[tests]]
name = "Juvinall Sample Problem 5.1: −750, 400, 1850 µε"
source = "Juvinall & Marshek 5th, Sample Problem 5.1 (pp. 198-199), as in Lacewing Lab rosette_test: ε1 = 0.0020, ε2 = −0.0010 (Juvinall rounds 0.0005 ± 0.0015; Eq. 5.1 gives ± 0.001504), ε1 107° clockwise of gauge a, the same axis as 73° counterclockwise"
inputs = { "ε_a" = "-750 µm/m", "ε_b" = "400 µm/m", "ε_c" = "1850 µm/m" }
expect = { "ε_1" = "2004 µm/m", "ε_2" = "-1004 µm/m", "θ_1" = "73 °" }
tolerance = "0.2%"

[[tests]]
name = "Juvinall §5.5: Sample Problem 5.1 on aluminium, E 71 GPa, ν 0.35"
source = "Juvinall & Marshek 5th, §5.5 (p. 203), as in Lacewing Lab rosette_test: σ1 = 134, σ2 = −24 MPa (−24.5 unrounded), ε3 = −0.0005 (−0.000538 unrounded); σ = Eε instead would give 142 and −71 MPa; σ3 = 0 lies between σ1 and σ2, so the largest shear is in plane"
inputs = { "ε_a" = "-750 µm/m", "ε_b" = "400 µm/m", "ε_c" = "1850 µm/m", E = "71 GPa", "ν" = "0.35" }
expect = { "σ_1" = "134 MPa", "σ_2" = "-24.5 MPa", "ε_3" = "-538 µm/m" }
tolerance = "0.5%"

[[tests]]
name = "Hand check: Sample Problem 5.1 on aluminium exactly"
source = "By hand: centre (−750 + 400 + 1850)/3 = 500 µε; radius √((−1500 − 2250)²/9 + (−1450)²/3) = √(1562500 + 700833.3) = 1504.43788 µε (Eq. 5.1); εy = (800 + 3700 + 750)/3 = 1750 µε; γxy = 2(1850 − 400)/√3 = 1674.31578 µε; θ1 = ½ atan2(γxy, εx − εy) = ½ atan2(1674.316, −2500) = 73.09437°; σ1,2 = 71 GPa/0.8775 × (ε1,2 + 0.35 ε2,1) = 133.73767, −24.50690 MPa; τmax = (σ1 − σ2)/2 = 79.12229 MPa; σ′ = 147.52576 MPa; σx = −11.12536, σy = 120.35613, τxy = 26.2963 GPa × γxy = 44.02830 MPa"
inputs = { "ε_a" = "-750 µm/m", "ε_b" = "400 µm/m", "ε_c" = "1850 µm/m", E = "71 GPa", "ν" = "0.35" }
expect = { "ε_x" = "-750 µm/m", "ε_y" = "1750 µm/m", "γ_xy" = "1674.315781 µm/m", "ε_1" = "2004.43788 µm/m", "ε_2" = "-1004.43788 µm/m", "γ_max" = "3008.875759 µm/m", "θ_1" = "73.09436948 °", "σ_1" = "133.7376731 MPa", "σ_2" = "-24.50690386 MPa", "τ_max" = "79.12228848 MPa", "σ′" = "147.5257599 MPa", "ε_3" = "-538.4615385 µm/m", "σ_x" = "-11.12535613 MPa", "σ_y" = "120.3561254 MPa", "τ_xy" = "44.02830386 MPa" }
tolerance = "1e-8"
+++

## Equiangular (delta) strain rosette (0°, 120°, 240°)

Three gauges at 120° to one another fix the plane strain state at a point on a free surface, whatever the directions of the principal strains. This template finds the principal strains and their direction, then the stresses behind them for a linear elastic, isotropic material.

Number the gauges a, b, c counterclockwise: b at 120° from a, c at 240°. A 0°, 60°, 120° rosette is the same thing: a gauge at 60° reads what one at 240° would, since 60° and 240° are one direction; it is gauge c, and the 120° gauge is gauge b. Strains are tension positive. The readings are taken as true strains along each grid: corrected for bridge nonlinearity and temperature (the strain gauge bridge templates do that for one gauge) and for transverse sensitivity, which this template does not correct.

### Readings

```calc {description="Strain read by gauge a, the reference direction (0°)"}
ε_a = -750 µm/m_
```

```calc {description="Strain read by gauge b, 120° counterclockwise from a"}
ε_b = 400 µm/m_
```

```calc {description="Strain read by gauge c, 240° counterclockwise from a"}
ε_c = 1850 µm/m_
```

### Material

```calc {description="Young's modulus of the part"}
E = 71 GPa_
```

```calc {description="Poisson's ratio of the part (below 0.5)"}
ν = 0.35_
```

### Strains on the gauge axes

Axis x runs along gauge a and y at 90° counterclockwise from it. The strain transformation ε(θ) = (εx + εy)/2 + (εx − εy)/2·cos 2θ + γxy/2·sin 2θ at 0°, 120° and 240° gives three equations for εx, εy and γxy.

```calc
> Normal strain along gauge a
ε_x = ε_a = ?? µm/m
> Normal strain at 90° to gauge a
ε_y = (2 ε_b + 2 ε_c - ε_a) / 3 = ?? µm/m
> Engineering shear strain on the x and y axes
γ_xy = 2 (ε_c - ε_b) / sqrt(3) = ?? µm/m
```

### Principal strains

The centre and radius of Mohr's strain circle are Juvinall's Eq. 5.1. The direction of ε₁ is θ₁, counterclockwise from gauge a when positive and clockwise when negative, between −90° and 90°; ε₂ lies 90° from it. (Juvinall's Eq. 5.2 gives the same axes as tan 2α, α clockwise, and leaves which axis is ε₁ to a rule; the two-argument arctangent names it directly.)

```calc
> Centre of Mohr's strain circle, Eq. 5.1
ε_m = (ε_a + ε_b + ε_c) / 3 = ?? µm/m
> Its radius, Eq. 5.1
R_ε = sqrt((2 ε_a - ε_b - ε_c)^2 / 9 + (ε_b - ε_c)^2 / 3) = ?? µm/m
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
> Normal stress at 90° to gauge a
σ_y = E / (1 - ν^2) (ε_y + ν ε_x) = ?? MPa
> Shear stress on the gauge axes, τ = G γ with G = E / 2(1 + ν)
τ_xy = E / (2 (1 + ν)) γ_xy = ?? MPa
```

For a factor of safety against yield, take σ₁ and σ₂ (with σ₃ = 0) to the plane stress static failure template.
