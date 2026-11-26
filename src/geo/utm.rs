//! Universal Transverse Mercator grid on top of `tm`.

use super::ellipsoid::Ellipsoid;
use super::tm;

pub const K0: f64 = 0.9996;
pub const FALSE_EASTING: f64 = 500_000.0;
pub const FALSE_NORTHING_SOUTH: f64 = 10_000_000.0;

const BANDS: &[u8] = b"CDEFGHJKLMNPQRSTUVWXX"; // X is repeated for 80..84°N

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Utm {
    pub zone: u8,
    pub north: bool,
    /// Latitude band letter (C..X), or None if outside -80..84°.
    pub band: Option<char>,
    pub easting: f64,
    pub northing: f64,
    /// Meridian convergence (degrees).
    pub convergence: f64,
    /// Point scale factor.
    pub scale: f64,
}

/// Central meridian of a zone (degrees).
pub fn central_meridian(zone: u8) -> f64 {
    zone as f64 * 6.0 - 183.0
}

pub fn lat_band(lat: f64) -> Option<char> {
    if !(-80.0..=84.0).contains(&lat) {
        return None;
    }
    let idx = ((lat + 80.0) / 8.0).floor() as usize;
    Some(BANDS[idx.min(BANDS.len() - 1)] as char)
}

/// Standard zone for a point, including the Norway and Svalbard exceptions.
pub fn standard_zone(lat: f64, lon: f64) -> u8 {
    let mut zone = (((lon + 180.0) / 6.0).floor() as i32 + 1).clamp(1, 60) as u8;
    match lat_band(lat) {
        Some('V') if zone == 31 && lon >= 3.0 => zone = 32,
        Some('X') => match zone {
            32 => zone = if lon < 9.0 { 31 } else { 33 },
            34 => zone = if lon < 21.0 { 33 } else { 35 },
            36 => zone = if lon < 33.0 { 35 } else { 37 },
            _ => {}
        },
        _ => {}
    }
    zone
}

/// Geodetic (degrees) -> UTM. `forced_zone` overrides the standard zone.
pub fn from_geodetic(
    ell: Ellipsoid,
    lat: f64,
    lon: f64,
    forced_zone: Option<u8>,
) -> Result<Utm, String> {
    if !(-80.0..=84.0).contains(&lat) {
        return Err(format!(
            "Latitude {lat:.6}° fora do intervalo UTM (-80° a 84°)."
        ));
    }
    let zone = match forced_zone {
        Some(z) if (1..=60).contains(&z) => z,
        Some(z) => return Err(format!("Fuso {z} inválido (1 a 60).")),
        None => standard_zone(lat, lon),
    };
    let lon0 = central_meridian(zone);
    // Normalise longitude difference into [-180, 180) so zone 1/60 wrap works.
    let dlon = (lon - lon0 + 540.0).rem_euclid(360.0) - 180.0;
    if dlon.abs() > 30.0 {
        return Err(format!(
            "Longitude {lon:.6}° está longe demais do meridiano central do fuso {zone}."
        ));
    }
    let p = tm::forward(ell, K0, 0.0, lat, dlon);
    let north = lat >= 0.0;
    let northing = if north { p.y } else { p.y + FALSE_NORTHING_SOUTH };
    Ok(Utm {
        zone,
        north,
        band: lat_band(lat),
        easting: p.x + FALSE_EASTING,
        northing,
        convergence: p.convergence,
        scale: p.scale,
    })
}

/// UTM -> geodetic (degrees).
pub fn to_geodetic(
    ell: Ellipsoid,
    zone: u8,
    north: bool,
    easting: f64,
    northing: f64,
) -> Result<(f64, f64), String> {
    if !(1..=60).contains(&zone) {
        return Err(format!("Fuso {zone} inválido (1 a 60)."));
    }
    if !(0.0..=1_000_000.0).contains(&easting) {
        return Err("Easting (E) deve estar entre 0 e 1 000 000 m.".into());
    }
    if !(0.0..=10_000_000.0).contains(&northing) {
        return Err("Northing (N) deve estar entre 0 e 10 000 000 m.".into());
    }
    let x = easting - FALSE_EASTING;
    let y = if north { northing } else { northing - FALSE_NORTHING_SOUTH };
    let (lat, lon) = tm::inverse(ell, K0, central_meridian(zone), x, y);
    let lon = (lon + 540.0).rem_euclid(360.0) - 180.0;
    Ok((lat, lon))
}
