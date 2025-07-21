use std::str::FromStr;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeElementTypes {
    Tetra4,
    Tetra10,
    GTetra4,
    Pyramid5,
    GPyramid5,
    GTetra10,
    Penta6,
    GPenta6,
    Penta15,
    GPenta15,
    Hexa8,
    GHexa8,
    Hexa20,
    GHexa20,
    Pyramid13,
    GPyramid13,
}

//"tetra4", "tetra10", "pyramid5", "pyramid13", "penta6", "penta15", "hexa8", "hexa20"

impl VolumeElementTypes {
    pub const NUMBER_OF_TYPES:usize = 8;//G and non G type count for 1
   

    pub fn to_index(&self) -> usize {
        match self {
            VolumeElementTypes::Tetra4 | VolumeElementTypes::GTetra4 => 0,
            VolumeElementTypes::Tetra10 | VolumeElementTypes::GTetra10 => 1,
            VolumeElementTypes::Pyramid5 | VolumeElementTypes::GPyramid5 => 2,
            VolumeElementTypes::Penta6 | VolumeElementTypes::GPenta6 => 3,
            VolumeElementTypes::Penta15 | VolumeElementTypes::GPenta15 => 4,
            VolumeElementTypes::Hexa8 | VolumeElementTypes::GHexa8 => 5,
            VolumeElementTypes::Hexa20 | VolumeElementTypes::GHexa20 => 6,
            VolumeElementTypes::GPyramid13 | VolumeElementTypes::Pyramid13 => 7,
        }
    }
}

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
    Nsided,
    GNsided,
    Nfaced,
    GNfaced,
    VolumeElementType(VolumeElementTypes),
}

impl Default for ElementsType {
    fn default() -> Self {
        Self::Point
    }
}

impl FromStr for ElementsType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ElementsType::*;
        use VolumeElementTypes::*;

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
            "tetra4" => Ok(VolumeElementType(Tetra4)),
            "g_tetra4" => Ok(VolumeElementType(GTetra4)),
            "tetra10" => Ok(VolumeElementType(Tetra10)),
            "g_tetra10" => Ok(VolumeElementType(GTetra10)),
            "pyramid5" => Ok(VolumeElementType(Pyramid5)),
            "g_pyramid5" => Ok(VolumeElementType(GPyramid5)),
            "pyramid13" => Ok(VolumeElementType(Pyramid13)),
            "g_pyramid13" => Ok(VolumeElementType(GPyramid13)),
            "penta6" => Ok(VolumeElementType(Penta6)),
            "g_penta6" => Ok(VolumeElementType(GPenta6)),
            "penta15" => Ok(VolumeElementType(Penta15)),
            "g_penta15" => Ok(VolumeElementType(GPenta15)),
            "hexa8" => Ok(VolumeElementType(Hexa8)),
            "g_hexa8" => Ok(VolumeElementType(GHexa8)),
            "hexa20" => Ok(VolumeElementType(Hexa20)),
            "g_hexa20" => Ok(VolumeElementType(GHexa20)),
            "nsided" => Ok(Nsided),
            "g_nsided" => Ok(GNsided),
            "nfaced" => Ok(Nfaced),
            "g_nfaced" => Ok(GNfaced),
            _ => Err(()),
        }
    }
}

impl ElementsType {
    // Consolidated method to get node count and vertex count, assuming they are the same.
    pub fn node_count(&self) -> i32 {
        use ElementsType::*;
        use VolumeElementTypes::*;

        match self {
            Point | GPoint => 1,
            Bar2 | GBar2 => 2,
            Bar3 | GBar3 => 3,
            Tria3 | GTria3 => 3,
            Tria6 | GTria6 => 6,
            Quad4 | GQuad4 => 4,
            Quad8 | GQuad8 => 8,

            Nsided | GNsided => panic!("Nsided has variable node count"),
            Nfaced | GNfaced => -1,
            VolumeElementType(Tetra4) | VolumeElementType(GTetra4) => 4,
            VolumeElementType(Tetra10) | VolumeElementType(GTetra10) => 10,
            VolumeElementType(Pyramid5) | VolumeElementType(GPyramid5) => 5,
            VolumeElementType(Penta6) | VolumeElementType(GPenta6) => 6,
            VolumeElementType(Penta15) | VolumeElementType(GPenta15) => 15,
            VolumeElementType(Hexa8) | VolumeElementType(GHexa8) => 8,
            VolumeElementType(Hexa20) | VolumeElementType(GHexa20) => 20,
            VolumeElementType(Pyramid13) | VolumeElementType(GPyramid13) => 13,
        }
    }
}
