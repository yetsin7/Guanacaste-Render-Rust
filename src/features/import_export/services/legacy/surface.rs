//! The physical surface categories of Guanacaste 3D (its `SurfaceKind`) and the PBR material each one gets here.
//! The numeric values are the byte stored per triangle in the detailed mesh families of the file.

use crate::features::materials::models::{Material as Pbr, Procedural, ProceduralKind, TextureRef};

/// What a triangle is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Surface {
    Wall,
    Floor,
    Glass,
    Ground,
    DoorLeaf,
    OpeningFrame,
    Structure,
    Foliage,
    Bark,
    Earth,
    Roof,
    Leaf,
    InteriorWall,
    Metal,
    LightEmitter,
    Upholstery,
    Bedding,
    Ceramic,
    LightOff,
    SpotEmitter,
    WallBrick,
    WallBlock,
    WallConcrete,
    WallStone,
    WallWood,
    WallMetal,
    Mirror,
    PipeCold,
    PipeHot,
    PipeDrain,
    PipeStorm,
    SolarCell,
    FireRed,
    Water,
    CarPaint,
    CarPaintSilver,
    Rubber,
    Asphalt,
    TintedGlass,
    Ocean,
    Rock,
}

use Surface::*;

/// Every surface in the order of its stored byte.
const ALL: [Surface; 41] = [
    Wall,
    Floor,
    Glass,
    Ground,
    DoorLeaf,
    OpeningFrame,
    Structure,
    Foliage,
    Bark,
    Earth,
    Roof,
    Leaf,
    InteriorWall,
    Metal,
    LightEmitter,
    Upholstery,
    Bedding,
    Ceramic,
    LightOff,
    SpotEmitter,
    WallBrick,
    WallBlock,
    WallConcrete,
    WallStone,
    WallWood,
    WallMetal,
    Mirror,
    PipeCold,
    PipeHot,
    PipeDrain,
    PipeStorm,
    SolarCell,
    FireRed,
    Water,
    CarPaint,
    CarPaintSilver,
    Rubber,
    Asphalt,
    TintedGlass,
    Ocean,
    Rock,
];

/// The procedural pattern of a surface: kind, size in meters of one tile and the UV rotation in degrees.
type Pattern = (ProceduralKind, f32, f32);

impl Surface {
    /// The surface stored as `byte` in a family mesh.
    pub fn from_byte(byte: u8) -> Option<Self> {
        ALL.get(byte as usize).copied()
    }

    /// The numeric value Guanacaste 3D stores for this surface (used by overrides, which name a category).
    pub fn from_category(category: i64) -> Option<Self> {
        u8::try_from(category).ok().and_then(Self::from_byte)
    }

    /// The procedural pattern that textures this surface, if any (the original draws these as procedural patterns).
    pub fn pattern(self) -> Option<Pattern> {
        match self {
            Floor => Some((ProceduralKind::Marble, 2.0, 0.0)),
            Ground => Some((ProceduralKind::Grass, 2.5, 0.0)),
            DoorLeaf => Some((ProceduralKind::Wood, 1.0, 0.0)),
            WallBrick => Some((ProceduralKind::Brick, 1.0, 0.0)),
            WallBlock => Some((ProceduralKind::Concrete, 2.4, 0.0)),
            WallConcrete | Structure => Some((ProceduralKind::Concrete, 3.0, 0.0)),
            WallStone | Rock => Some((ProceduralKind::Concrete, 1.0, 0.0)),
            WallWood => Some((ProceduralKind::Wood, 1.2, 90.0)),
            Asphalt => Some((ProceduralKind::Noise, 1.5, 0.0)),
            Roof => Some((ProceduralKind::Brick, 1.0, 0.0)),
            Earth => Some((ProceduralKind::Concrete, 1.5, 0.0)),
            _ => None,
        }
    }

    /// Whether triangles of this surface need texture coordinates.
    pub fn textured(self) -> bool {
        self.pattern().is_some()
    }

    /// Name, linear base color, roughness, metalness and index of refraction of the surface.
    fn spec(self) -> (&'static str, [f32; 3], f32, f32, f32) {
        match self {
            Wall => ("Pared", [0.80, 0.78, 0.73], 0.92, 0.0, 1.5),
            InteriorWall => ("Pared interior", [0.80, 0.78, 0.73], 0.92, 0.0, 1.5),
            Floor => ("Piso", [0.56, 0.55, 0.52], 0.38, 0.0, 1.5),
            Water => ("Agua", [0.015, 0.05, 0.06], 0.01, 0.0, 1.33),
            Ocean => ("Océano", [0.010, 0.125, 0.150], 0.02, 0.0, 1.33),
            CarPaint => ("Pintura de auto", [0.018, 0.024, 0.045], 0.08, 0.3, 1.5),
            CarPaintSilver => ("Pintura plateada", [0.72, 0.73, 0.74], 0.16, 0.85, 1.5),
            TintedGlass => ("Vidrio polarizado", [0.006, 0.008, 0.010], 0.03, 0.0, 1.5),
            Rubber => ("Caucho", [0.025, 0.025, 0.025], 0.9, 0.0, 1.5),
            Asphalt => ("Asfalto", [0.11, 0.11, 0.11], 0.95, 0.0, 1.5),
            Glass => ("Vidrio", [0.86, 0.92, 0.94], 0.02, 0.0, 1.5),
            Ground => ("Terreno", [0.22, 0.28, 0.13], 0.97, 0.0, 1.5),
            DoorLeaf => ("Madera", [0.60, 0.39, 0.22], 0.5, 0.0, 1.5),
            OpeningFrame => ("Marco", [0.25, 0.27, 0.29], 0.45, 0.35, 1.5),
            Foliage => ("Follaje", [0.085, 0.17, 0.045], 0.62, 0.0, 1.5),
            Bark => ("Corteza", [0.20, 0.16, 0.12], 0.95, 0.0, 1.5),
            LightOff => ("Difusor apagado", [0.9, 0.88, 0.84], 0.6, 0.0, 1.5),
            WallBrick => ("Ladrillo", [0.52, 0.24, 0.15], 0.9, 0.0, 1.5),
            WallBlock => ("Bloque de concreto", [0.50, 0.50, 0.48], 0.92, 0.0, 1.5),
            WallConcrete => ("Concreto visto", [0.52, 0.51, 0.49], 0.85, 0.0, 1.5),
            WallStone => ("Piedra natural", [0.55, 0.50, 0.43], 0.92, 0.0, 1.5),
            WallWood => ("Revestimiento de madera", [0.50, 0.34, 0.20], 0.7, 0.0, 1.5),
            PipeCold => ("Tubería agua fría", [0.16, 0.36, 0.72], 0.35, 0.0, 1.5),
            PipeHot => ("Tubería agua caliente", [0.72, 0.16, 0.12], 0.35, 0.0, 1.5),
            SolarCell => ("Celda solar", [0.02, 0.04, 0.10], 0.06, 0.2, 1.5),
            FireRed => ("Esmalte rojo incendio", [0.70, 0.05, 0.04], 0.25, 0.0, 1.5),
            PipeStorm => ("PVC pluvial", [0.30, 0.55, 0.32], 0.4, 0.0, 1.5),
            PipeDrain => ("PVC sanitario", [0.78, 0.76, 0.70], 0.4, 0.0, 1.5),
            Mirror => ("Espejo", [0.92, 0.93, 0.93], 0.02, 1.0, 1.5),
            WallMetal => ("Panel metálico", [0.70, 0.72, 0.74], 0.4, 0.7, 1.5),
            SpotEmitter | LightEmitter => ("Difusor encendido", [0.9, 0.88, 0.84], 0.6, 0.0, 1.5),
            Upholstery => ("Tapiz", [0.30, 0.28, 0.25], 0.95, 0.0, 1.5),
            Bedding => ("Lino", [0.80, 0.79, 0.76], 0.92, 0.0, 1.5),
            Ceramic => ("Porcelana", [0.86, 0.86, 0.84], 0.07, 0.0, 1.5),
            Metal => ("Acero cepillado", [0.62, 0.62, 0.63], 0.32, 1.0, 1.5),
            Leaf => ("Hoja", [0.10, 0.20, 0.045], 0.45, 0.0, 1.5),
            Roof => ("Teja", [0.50, 0.22, 0.12], 0.72, 0.0, 1.5),
            Earth => ("Tierra", [0.30, 0.22, 0.15], 0.97, 0.0, 1.5),
            Rock => ("Roca", [0.34, 0.32, 0.29], 0.93, 0.0, 1.5),
            Structure => ("Estructura", [0.45, 0.42, 0.38], 0.85, 0.0, 1.5),
        }
    }

    /// The display name of the material of this surface.
    pub fn name(self) -> &'static str {
        self.spec().0
    }

    /// The PBR material of this surface: its color (sRGB), roughness, metalness, glass transparency and procedural
    /// texture. Patterned surfaces take the texture's own color, so their base color is a neutral tint.
    pub fn material(self) -> Pbr {
        let (name, linear, roughness, metallic, ior) = self.spec();
        let mut material = Pbr::new(name);
        let encode = |c: f32| crate::features::materials::logic::color::linear_to_srgb(c);
        material.base_color = [encode(linear[0]), encode(linear[1]), encode(linear[2]), 1.0];
        material.roughness = roughness;
        material.metallic = metallic;
        material.ior = ior;
        if matches!(self, Glass) {
            material.transparency = 0.9;
        }
        if matches!(self, Water | Ocean) {
            material.transparency = 0.0;
        }
        if let Some((kind, tile, rotation)) = self.pattern() {
            let mut procedural = Procedural::new(kind);
            procedural.seed = 7;
            material.maps.albedo = Some(TextureRef::procedural(procedural));
            material.uv.scale = [1.0 / tile, 1.0 / tile];
            material.uv.rotation = rotation;
            material.base_color = tint(self);
        }
        material
    }
}

/// The base color factor that multiplies the procedural texture of `surface`.
fn tint(surface: Surface) -> [f32; 4] {
    match surface {
        Asphalt => [0.62, 0.62, 0.64, 1.0],
        Ground => [0.95, 1.0, 0.9, 1.0],
        Floor => [0.97, 0.96, 0.94, 1.0],
        DoorLeaf | WallWood => [1.0, 0.92, 0.82, 1.0],
        WallConcrete | Structure => [1.0, 1.0, 1.0, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_bytes_map_to_the_original_order() {
        assert_eq!(Surface::from_byte(8), Some(Bark));
        assert_eq!(Surface::from_byte(11), Some(Leaf));
        assert_eq!(Surface::from_byte(34), Some(CarPaint));
        assert_eq!(Surface::from_byte(40), Some(Rock));
        assert_eq!(Surface::from_byte(41), None);
    }

    #[test]
    fn glass_is_see_through_and_paint_is_glossy() {
        assert!(Glass.material().transparency > 0.5);
        assert!(CarPaint.material().roughness < 0.2);
        assert!(WallBrick.material().maps.albedo.is_some());
    }
}
