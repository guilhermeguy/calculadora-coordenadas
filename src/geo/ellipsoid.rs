//! Reference ellipsoids and datums.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipsoid {
    /// Semi-major axis (m).
    pub a: f64,
    /// Flattening.
    pub f: f64,
}

impl Ellipsoid {
    pub const WGS84: Ellipsoid = Ellipsoid { a: 6_378_137.0, f: 1.0 / 298.257_223_563 };
    pub const GRS80: Ellipsoid = Ellipsoid { a: 6_378_137.0, f: 1.0 / 298.257_222_101 };
    /// GRS67 as adopted by SAD69 (flattening rounded to 1/298.25).
    pub const SAD69: Ellipsoid = Ellipsoid { a: 6_378_160.0, f: 1.0 / 298.25 };

    /// First eccentricity squared.
    pub fn e2(&self) -> f64 {
        self.f * (2.0 - self.f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Datum {
    Sirgas2000,
    Wgs84,
    Sad69,
}

impl Datum {
    pub const ALL: [Datum; 3] = [Datum::Sirgas2000, Datum::Wgs84, Datum::Sad69];

    pub fn ellipsoid(self) -> Ellipsoid {
        match self {
            Datum::Sirgas2000 => Ellipsoid::GRS80,
            Datum::Wgs84 => Ellipsoid::WGS84,
            Datum::Sad69 => Ellipsoid::SAD69,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Datum::Sirgas2000 => "SIRGAS 2000",
            Datum::Wgs84 => "WGS84",
            Datum::Sad69 => "SAD69",
        }
    }

    /// Geocentric translation (m) that takes ECEF coordinates in this datum to
    /// SIRGAS 2000. WGS84 (G1150+) and SIRGAS 2000 agree at the cm level.
    pub fn to_sirgas_shift(self) -> [f64; 3] {
        match self {
            Datum::Sirgas2000 | Datum::Wgs84 => [0.0, 0.0, 0.0],
            // IBGE, R.PR 1/2005: SAD69 -> SIRGAS 2000.
            Datum::Sad69 => [-67.35, 3.88, -38.22],
        }
    }
}
