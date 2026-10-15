//! Paramètres de traitement reçus de l'interface, revérifiés côté Rust.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Params {
    /// Taille de l'image finale, en pixels.
    pub width: u32,
    pub height: u32,
    /// Écart voulu entre les deux yeux dans l'image finale, en pixels.
    pub eye_distance: f32,
    /// Position du milieu des yeux, en fraction de la largeur / hauteur.
    pub eye_x: f32,
    pub eye_y: f32,
    pub background: Background,
    pub light: LightParams,
    pub export: ExportParams,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Background {
    Color { color: String },
    Gradient { top: String, bottom: String },
    Image { path: String },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LightRegion {
    /// Mesure sur le visage seul (insensible aux vêtements).
    Face,
    /// Mesure sur tout le sujet détouré.
    Subject,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LightParams {
    pub enabled: bool,
    /// Luminance L* (Lab) moyenne visée, de 0 (noir) à 100 (blanc).
    pub target: f32,
    pub region: LightRegion,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportParams {
    pub avif: bool,
    pub avif_quality: u8,
    pub webp: bool,
    pub webp_quality: u8,
    /// PNG du sujet détouré sur fond transparent, en plus des formats web.
    pub png_transparent: bool,
}

impl Params {
    pub fn validate(&self) -> Result<(), String> {
        check_range("largeur", self.width as f32, 128.0, 4096.0)?;
        check_range("hauteur", self.height as f32, 128.0, 4096.0)?;
        check_range("écart des yeux", self.eye_distance, 16.0, self.width as f32 * 0.6)?;
        check_range("position horizontale des yeux", self.eye_x, 0.2, 0.8)?;
        check_range("position verticale des yeux", self.eye_y, 0.15, 0.75)?;
        check_range("luminosité cible", self.light.target, 10.0, 95.0)?;
        check_range("qualité AVIF", self.export.avif_quality as f32, 1.0, 100.0)?;
        check_range("qualité WebP", self.export.webp_quality as f32, 1.0, 100.0)?;
        let ex = &self.export;
        if !(ex.avif || ex.webp || ex.png_transparent) {
            return Err("Choisissez au moins un format d'export.".into());
        }
        match &self.background {
            Background::Color { color } => {
                parse_hex(color)?;
            }
            Background::Gradient { top, bottom } => {
                parse_hex(top)?;
                parse_hex(bottom)?;
            }
            Background::Image { path } => {
                if path.trim().is_empty() {
                    return Err("Choisissez l'image de fond.".into());
                }
            }
        }
        Ok(())
    }
}

/// Refuse aussi NaN (toute comparaison avec NaN est fausse).
fn check_range(name: &str, value: f32, min: f32, max: f32) -> Result<(), String> {
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(format!("Valeur invalide pour « {name} » : attendu entre {min} et {max}."))
    }
}

/// Lit une couleur `#rrggbb`.
pub fn parse_hex(s: &str) -> Result<[u8; 3], String> {
    let err = || format!("Couleur invalide : « {s} » (attendu #rrggbb).");
    let hex = s.strip_prefix('#').ok_or_else(err)?;
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(err());
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| err());
    Ok([byte(0)?, byte(2)?, byte(4)?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colors() {
        assert_eq!(parse_hex("#ff8000"), Ok([255, 128, 0]));
        assert!(parse_hex("ff8000").is_err());
        assert!(parse_hex("#ff80").is_err());
        assert!(parse_hex("#gg8000").is_err());
    }

    #[test]
    fn range_rejects_nan() {
        assert!(check_range("x", f32::NAN, 0.0, 1.0).is_err());
        assert!(check_range("x", 0.5, 0.0, 1.0).is_ok());
    }
}
