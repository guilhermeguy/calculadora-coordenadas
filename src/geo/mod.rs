//! Coordinate conversion core (no GUI dependencies).

pub mod datum;
pub mod dms;
pub mod ellipsoid;
pub mod tm;
pub mod utm;

pub use ellipsoid::Datum;
pub use utm::Utm;

/// A parsed input coordinate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coord {
    /// Geodetic latitude / longitude in decimal degrees.
    Geo { lat: f64, lon: f64 },
    Utm { zone: u8, north: bool, easting: f64, northing: f64 },
}

/// Result of a conversion, expressed in the output datum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Converted {
    pub lat: f64,
    pub lon: f64,
    /// Present when UTM output was requested.
    pub utm: Option<Utm>,
}

/// Converts `input` (in datum `from`) to datum `to`. When `want_utm` is set,
/// the UTM coordinates are computed too, optionally in a forced zone.
pub fn convert(
    input: Coord,
    from: Datum,
    to: Datum,
    want_utm: bool,
    forced_zone: Option<u8>,
) -> Result<Converted, String> {
    let (lat, lon) = match input {
        Coord::Geo { lat, lon } => {
            if !(-90.0..=90.0).contains(&lat) {
                return Err("Latitude deve estar entre -90° e 90°.".into());
            }
            if !(-180.0..=180.0).contains(&lon) {
                return Err("Longitude deve estar entre -180° e 180°.".into());
            }
            (lat, lon)
        }
        Coord::Utm { zone, north, easting, northing } => {
            utm::to_geodetic(from.ellipsoid(), zone, north, easting, northing)?
        }
    };

    let (lat, lon) = datum::transform(lat, lon, from, to);

    let utm = if want_utm {
        Some(utm::from_geodetic(to.ellipsoid(), lat, lon, forced_zone)?)
    } else {
        None
    };

    Ok(Converted { lat, lon, utm })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ellipsoid::Ellipsoid;

    fn utm_of(lat: f64, lon: f64) -> Utm {
        utm::from_geodetic(Ellipsoid::WGS84, lat, lon, None).unwrap()
    }

    #[test]
    fn origin() {
        // Classic check value: (0, 0) -> 31N 166021.443 0.000
        let u = utm_of(0.0, 0.0);
        assert_eq!(u.zone, 31);
        assert!((u.easting - 166_021.443_1).abs() < 1e-3, "{}", u.easting);
        assert!(u.northing.abs() < 1e-6);
    }

    #[test]
    fn central_meridian_is_scaled_meridian_arc() {
        // Meridian distance to 45° on WGS84 = 4 984 944.378 m (GeographicLib).
        let u = utm_of(45.0, 9.0);
        assert_eq!(u.zone, 32);
        assert!((u.easting - 500_000.0).abs() < 1e-6);
        assert!((u.northing - 0.9996 * 4_984_944.378).abs() < 2e-3, "{}", u.northing);
        assert!((u.scale - 0.9996).abs() < 1e-12);
        assert!(u.convergence.abs() < 1e-12);
    }

    #[test]
    fn known_points() {
        // Reference values from PROJ 9.8 (pyproj), mm tolerance.
        let check = |u: Utm, e: f64, n: f64| {
            assert!((u.easting - e).abs() < 1e-3, "E {} vs {e}", u.easting);
            assert!((u.northing - n).abs() < 1e-3, "N {} vs {n}", u.northing);
        };
        let u = utm_of(48.8583, 2.2945); // Eiffel Tower
        assert_eq!((u.zone, u.north, u.band), (31, true, Some('U')));
        check(u, 448_251.898_291, 5_411_943.793_808);

        let u = utm_of(-33.857, 151.215); // Sydney Opera House
        assert_eq!((u.zone, u.north, u.band), (56, false, Some('H')));
        check(u, 334_873.198_834, 6_252_266.091_823);

        // São Paulo, SIRGAS 2000 (GRS80)
        let u = utm::from_geodetic(Ellipsoid::GRS80, -23.5505, -46.6333, None).unwrap();
        assert_eq!((u.zone, u.north, u.band), (23, false, Some('K')));
        check(u, 333_287.915_101, 7_394_588.318_637);

        // Brasília, SAD69 (GRS67 ellipsoid)
        let u = utm::from_geodetic(Ellipsoid::SAD69, -15.7939, -47.8828, None).unwrap();
        assert_eq!(u.zone, 23);
        check(u, 191_137.627_4, 8_251_739.875_2);
    }

    #[test]
    fn sad69_to_sirgas_matches_proj() {
        // PROJ "SAD69 to SIRGAS 2000 (1)" (EPSG), same IBGE parameters.
        let (lat, lon) = datum::transform(-15.7939, -47.8828, Datum::Sad69, Datum::Sirgas2000);
        assert!((lat - -15.794_347_722_7).abs() < 1e-9, "{lat}");
        assert!((lon - -47.883_241_992_6).abs() < 1e-9, "{lon}");
    }

    #[test]
    fn zone_exceptions() {
        assert_eq!(utm::standard_zone(60.0, 5.0), 32); // Norway
        assert_eq!(utm::standard_zone(60.0, 2.0), 31);
        assert_eq!(utm::standard_zone(78.0, 8.0), 31); // Svalbard
        assert_eq!(utm::standard_zone(78.0, 10.0), 33);
        assert_eq!(utm::standard_zone(78.0, 20.0), 33);
        assert_eq!(utm::standard_zone(78.0, 22.0), 35);
        assert_eq!(utm::standard_zone(78.0, 34.0), 37);
        assert_eq!(utm::standard_zone(0.0, 180.0), 60);
        assert_eq!(utm::standard_zone(0.0, -180.0), 1);
        // Brazil
        assert_eq!(utm::standard_zone(-23.55, -46.63), 23);
        assert_eq!(utm::standard_zone(-3.1, -61.0), 20);
        assert_eq!(utm::standard_zone(-3.1, -60.0), 21); // boundary belongs to the east zone
    }

    #[test]
    fn round_trip_grid() {
        let mut max_err: f64 = 0.0;
        for datum in Datum::ALL {
            let ell = datum.ellipsoid();
            let mut lat = -79.5;
            while lat < 84.0 {
                let mut lon = -179.7;
                while lon < 180.0 {
                    let u = utm::from_geodetic(ell, lat, lon, None).unwrap();
                    let (lat2, lon2) =
                        utm::to_geodetic(ell, u.zone, u.north, u.easting, u.northing).unwrap();
                    max_err = max_err.max((lat2 - lat).abs()).max((lon2 - lon).abs());
                    lon += 1.37;
                }
                lat += 0.93;
            }
        }
        assert!(max_err < 1e-9, "max round-trip error {max_err}°");
    }

    #[test]
    fn forced_zone_round_trip() {
        // A point in zone 23 computed in neighbouring zone 22 (common in Brazil).
        let ell = Ellipsoid::GRS80;
        let u = utm::from_geodetic(ell, -23.55, -46.63, Some(22)).unwrap();
        assert_eq!(u.zone, 22);
        let (lat, lon) = utm::to_geodetic(ell, 22, false, u.easting, u.northing).unwrap();
        assert!((lat + 23.55).abs() < 1e-9 && (lon + 46.63).abs() < 1e-9);
    }

    #[test]
    fn datum_round_trip_and_magnitude() {
        let (lat, lon) = (-15.7939, -47.8828); // Brasília
        let (l2, o2) = datum::transform(lat, lon, Datum::Sad69, Datum::Sirgas2000);
        let (l3, o3) = datum::transform(l2, o2, Datum::Sirgas2000, Datum::Sad69);
        assert!((l3 - lat).abs() < 1e-9 && (o3 - lon).abs() < 1e-9);
        // SAD69 -> SIRGAS shifts are tens of metres (~0.0003-0.0006°).
        let dlat_m = (l2 - lat) * 111_000.0;
        let dlon_m = (o2 - lon) * 111_000.0 * lat.to_radians().cos();
        let shift = (dlat_m * dlat_m + dlon_m * dlon_m).sqrt();
        assert!((20.0..100.0).contains(&shift), "shift {shift} m");
        // WGS84 <-> SIRGAS 2000 is identity here.
        let (l4, o4) = datum::transform(lat, lon, Datum::Wgs84, Datum::Sirgas2000);
        assert!((l4 - lat).abs() < 1e-12 && (o4 - lon).abs() < 1e-12);
    }

    #[test]
    fn convert_pipeline() {
        let out = convert(
            Coord::Geo { lat: 0.0, lon: 0.0 },
            Datum::Wgs84,
            Datum::Wgs84,
            true,
            None,
        )
        .unwrap();
        let u = out.utm.unwrap();
        assert!((u.easting - 166_021.443_1).abs() < 1e-3);

        let back = convert(
            Coord::Utm { zone: 31, north: true, easting: u.easting, northing: u.northing },
            Datum::Wgs84,
            Datum::Wgs84,
            false,
            None,
        )
        .unwrap();
        assert!(back.lat.abs() < 1e-9 && back.lon.abs() < 1e-9);

        assert!(convert(Coord::Geo { lat: 85.0, lon: 0.0 }, Datum::Wgs84, Datum::Wgs84, true, None)
            .is_err());
        assert!(convert(
            Coord::Utm { zone: 0, north: true, easting: 500_000.0, northing: 0.0 },
            Datum::Wgs84,
            Datum::Wgs84,
            false,
            None
        )
        .is_err());
    }

    /// Cross-check against a CSV produced by PROJ (pyproj):
    /// `ellipsoid,lat,lon,zone,easting,northing` per line.
    /// Run with: PROJ_CSV=path/to/proj_ref.csv cargo test -- --ignored
    #[test]
    #[ignore]
    fn proj_cross_check() {
        let path = std::env::var("PROJ_CSV").expect("set PROJ_CSV");
        let data = std::fs::read_to_string(path).unwrap();
        let (mut max_fwd, mut max_inv, mut count): (f64, f64, usize) = (0.0, 0.0, 0);
        for line in data.lines().filter(|l| !l.trim().is_empty()) {
            let c: Vec<&str> = line.split(',').collect();
            let ell = match c[0] {
                "wgs84" => Ellipsoid::WGS84,
                "grs80" => Ellipsoid::GRS80,
                "sad69" => Ellipsoid::SAD69,
                other => panic!("unknown ellipsoid {other}"),
            };
            let num = |i: usize| c[i].parse::<f64>().unwrap();
            let (lat, lon, zone, e, n) = (num(1), num(2), c[3].parse::<u8>().unwrap(), num(4), num(5));
            let u = utm::from_geodetic(ell, lat, lon, Some(zone)).unwrap();
            max_fwd = max_fwd.max((u.easting - e).abs()).max((u.northing - n).abs());
            let (lat2, lon2) = utm::to_geodetic(ell, zone, lat >= 0.0, e, n).unwrap();
            max_inv = max_inv.max((lat2 - lat).abs()).max((lon2 - lon).abs());
            count += 1;
        }
        println!("{count} points: max forward error {max_fwd:.2e} m, max inverse error {max_inv:.2e} deg");
        assert!(max_fwd < 1e-3, "forward error {max_fwd} m");
        assert!(max_inv < 1e-8, "inverse error {max_inv} deg");
    }
}
