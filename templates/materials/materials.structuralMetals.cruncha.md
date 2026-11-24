+++
cruncha-template = 1
id = "materials.structuralMetals"
version = "1.0.0"
title = "Material properties: common structural steels and 6061-T6"
summary = "Minimum yield and tensile strength, modulus, Poisson's ratio and density of a structural metal, looked up by grade from a small table."
category = ["Materials", "Metals"]
tags = ["material properties", "A36", "A992", "A572", "6061-T6", "aluminum", "steel", "yield strength", "tensile strength", "lookup", "table"]
license = "GPL-3.0-or-later"
inputs = ["grade"]
outputs = ["F_y", "F_u", "E", "ν", "G", "ρ"]

[[references]]
work = "ASTM A36/A36M, Standard Specification for Carbon Structural Steel"
locator = "Table 3 (tensile requirements, plates, shapes and bars)"

[[references]]
work = "ASTM A992/A992M, Standard Specification for Structural Steel Shapes"
locator = "Table 2 (tensile requirements)"

[[references]]
work = "ASTM A572/A572M, Standard Specification for High-Strength Low-Alloy Columbium-Vanadium Structural Steel"
locator = "Table 3 (tensile requirements, Grade 50)"

[[references]]
work = "ANSI/AISC 360-22 Specification for Structural Steel Buildings"
edition = "2022"
locator = "Symbols (E = 29 000 ksi, G = 11 200 ksi); Sec. A3.1"

[[references]]
work = "Aluminum Association, Aluminum Design Manual"
edition = "2020"
locator = "Specification Tables A.3.3 and A.3.4 (6061-T6 extrusions; E, ν, density in Sec. A.3.1)"

[[tests]]
name = "A992"
source = "ASTM A992 Table 2 minimums; AISC 360 E and G"
inputs = { grade = '"A992"' }
expect = { F_y = "50 ksi", F_u = "65 ksi", E = "29000 ksi", G = "11154 ksi" }
tolerance = "0.01%"

[[tests]]
name = "A36"
source = "ASTM A36 Table 3 minimums"
inputs = { grade = '"A36"' }
expect = { F_y = "36 ksi", F_u = "58 ksi", "ρ" = "490 lb/ft^3" }
tolerance = "0.1%"

[[tests]]
name = "6061-T6"
source = "Aluminum Design Manual, 6061-T6 extrusions: Ftu 38 ksi, Fty 35 ksi, E 10 100 ksi, ν 0.33"
inputs = { grade = '"6061-T6"' }
expect = { F_y = "35 ksi", F_u = "38 ksi", E = "10100 ksi", G = "3797 ksi", "ρ" = "0.098 lb/in^3" }
tolerance = "0.02%"
+++

## Material properties: common structural metals

Specified minimum properties of a few common structural metals, for design. Actual mill test values are higher; never design to them. Thicker plates and other product forms can have lower minimums than these (shapes and plates up to the specification's first thickness limit): check the specification for the product you use.

### Grade

Type the grade exactly as the table's first column gives it.

```calc {description="Material grade, from the table below"}
grade = "A992"_
```

### Table

```calc {description="Structural metals: minimum strengths, elastic constants, density"}
metals = ``#grade	F_y	F_u	E	ν	ρ
	ksi	ksi	ksi		lb/in³
A36	36	58	29000	0.3	0.2836
A992	50	65	29000	0.3	0.2836
A572-50	50	65	29000	0.3	0.2836
6061-T6	35	38	10100	0.33	0.098`` = !
```

### Properties

```calc {description="Specified minimum yield stress"}
F_y = metals[grade, "F_y"] = ?? ksi
```

```calc {description="Specified minimum tensile strength"}
F_u = metals[grade, "F_u"] = ?? ksi
```

```calc {description="Modulus of elasticity"}
E = metals[grade, "E"] = ?? ksi
```

```calc {description="Poisson's ratio"}
ν = metals[grade, "ν"] = ??
```

```calc {description="Density"}
ρ = metals[grade, "ρ"] = ?? lb/ft³
```

For isotropic metals the shear modulus follows from E and ν. AISC 360 rounds it to 11 200 ksi for steel.

```calc {description="Shear modulus, E / 2(1 + ν)"}
G = E / (2 (1 + ν)) = ?? ksi
```
