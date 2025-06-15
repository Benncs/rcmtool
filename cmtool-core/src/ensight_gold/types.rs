use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementsType {
    Point,
    GPoint,
    Bar2,
    GBar2,
    Bar3,
    GBar3,
    Tria3,
    GTria3,
    Tria6,
    GTria6,
    Quad4,
    GQuad4,
    Quad8,
    GQuad8,
    Tetra4,
    GTetra4,
    Tetra10,
    GTetra10,
    Pyramid5,
    GPyramid5,
    Pyramid13,
    GPyramid13,
    Penta6,
    GPenta6,
    Penta15,
    GPenta15,
    Hexa8,
    GHexa8,
    Hexa20,
    GHexa20,
    Nsided,
    GNsided,
    Nfaced,
    GNfaced,
}

impl FromStr for ElementsType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ElementsType::*;

        match s.trim().to_lowercase().as_str() {
            "point" => Ok(Point),
            "g_point" => Ok(GPoint),
            "bar2" => Ok(Bar2),
            "g_bar2" => Ok(GBar2),
            "bar3" => Ok(Bar3),
            "g_bar3" => Ok(GBar3),
            "tria3" => Ok(Tria3),
            "g_tria3" => Ok(GTria3),
            "tria6" => Ok(Tria6),
            "g_tria6" => Ok(GTria6),
            "quad4" => Ok(Quad4),
            "g_quad4" => Ok(GQuad4),
            "quad8" => Ok(Quad8),
            "g_quad8" => Ok(GQuad8),
            "tetra4" => Ok(Tetra4),
            "g_tetra4" => Ok(GTetra4),
            "tetra10" => Ok(Tetra10),
            "g_tetra10" => Ok(GTetra10),
            "pyramid5" => Ok(Pyramid5),
            "g_pyramid5" => Ok(GPyramid5),
            "pyramid13" => Ok(Pyramid13),
            "g_pyramid13" => Ok(GPyramid13),
            "penta6" => Ok(Penta6),
            "g_penta6" => Ok(GPenta6),
            "penta15" => Ok(Penta15),
            "g_penta15" => Ok(GPenta15),
            "hexa8" => Ok(Hexa8),
            "g_hexa8" => Ok(GHexa8),
            "hexa20" => Ok(Hexa20),
            "g_hexa20" => Ok(GHexa20),
            "nsided" => Ok(Nsided),
            "g_nsided" => Ok(GNsided),
            "nfaced" => Ok(Nfaced),
            "g_nfaced" => Ok(GNfaced),
            _ => Err(()),
        }
    }
}

impl ElementsType {
    pub fn number_of_nodes(&self) -> i32 {
        use ElementsType::*;

        match self {
            Point | GPoint => 1,
            Bar2 | GBar2 => 2,
            Bar3 | GBar3 => 3,
            Tria3 | GTria3 => 3,
            Tria6 | GTria6 => 6,
            Quad4 | GQuad4 => 4,
            Quad8 | GQuad8 => 8,
            Tetra4 | GTetra4 => 4,
            Tetra10 | GTetra10 => 10,
            Pyramid5 | GPyramid5 => 5,
            Pyramid13 | GPyramid13 => 13,
            Penta6 | GPenta6 => 6,
            Penta15 | GPenta15 => 15,
            Hexa8 | GHexa8 => 8,
            Hexa20 | GHexa20 => 20,
            Nsided | GNsided => panic!("Nsided has variable node count"),
            Nfaced | GNfaced => -1,
        }
    }

    pub fn number_vertex(&self) -> i32 {
        use ElementsType::*;

        match self {
            Point | GPoint         => 1,
            Bar2 | GBar2           => 2,
            Bar3 | GBar3           => 3,
            Tria3 | GTria3         => 3,
            Tria6 | GTria6         => 6,
            Quad4 | GQuad4         => 4,
            Quad8 | GQuad8         => 8,
            Tetra4 | GTetra4       => 4,
            Tetra10 | GTetra10     => 10,
            Pyramid5 | GPyramid5   => 5,
            Pyramid13 | GPyramid13 => 13,
            Penta6 | GPenta6       => 6,
            Penta15 | GPenta15     => 15,
            Hexa8 | GHexa8         => 8,
            Hexa20 | GHexa20       => 20,
            Nsided | GNsided       => -1,  
            Nfaced | GNfaced       => -1,  
        }
    }
}
