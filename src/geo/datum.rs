//! Datum transformation by geocentric translation (3 parameters).

use super::ellipsoid::{Datum, Ellipsoid};

fn to_ecef(ell: Ellipsoid, lat: f64, lon: f64, h: f64) -> [f64; 3] {
    let e2 = ell.e2();
    let (sp, cp) = lat.to_radians().sin_cos();
    let (sl, cl) = lon.to_radians().sin_cos();
    let n = ell.a / (1.0 - e2 * sp * sp).sqrt();
    [(n + h) * cp * cl, (n + h) * cp * sl, (n * (1.0 - e2) + h) * sp]
}

fn from_ecef(ell: Ellipsoid, xyz: [f64; 3]) -> (f64, f64, f64) {
    let [x, y, z] = xyz;
    let e2 = ell.e2();
    let p = (x * x + y * y).sqrt();
    let lon = y.atan2(x);
    let mut lat = z.atan2(p * (1.0 - e2));
    let mut h = 0.0;
    for _ in 0..10 {
        let s = lat.sin();
        let n = ell.a / (1.0 - e2 * s * s).sqrt();
        h = p / lat.cos() - n;
        let next = z.atan2(p * (1.0 - e2 * n / (n + h)));
        let done = (next - lat).abs() < 1e-14;
        lat = next;
        if done {
            break;
        }
    }
    (lat.to_degrees(), lon.to_degrees(), h)
}

/// Transforms geodetic coordinates (degrees, ellipsoidal height assumed 0)
/// from one datum to another, via SIRGAS 2000.
pub fn transform(lat: f64, lon: f64, from: Datum, to: Datum) -> (f64, f64) {
    let s_from = from.to_sirgas_shift();
    let s_to = to.to_sirgas_shift();
    // Same reference frame (e.g. WGS84 <-> SIRGAS 2000): nothing to do.
    if from == to || s_from == s_to {
        return (lat, lon);
    }
    let mut xyz = to_ecef(from.ellipsoid(), lat, lon, 0.0);
    for i in 0..3 {
        xyz[i] += s_from[i] - s_to[i];
    }
    let (lat2, lon2, _h) = from_ecef(to.ellipsoid(), xyz);
    (lat2, lon2)
}
