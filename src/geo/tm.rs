//! Transverse Mercator projection using Krüger series to 6th order in n,
//! following C. F. F. Karney, "Transverse Mercator with an accuracy of a few
//! nanometers", J. Geodesy 85(8), 475-485 (2011).
//!
//! Structure adapted from Chris Veness' geodesy library (utm.js, MIT licence,
//! https://github.com/chrisveness/geodesy) and GeographicLib (MIT licence).
//! Accuracy is a few nanometres within ~3,900 km of the central meridian.

use super::ellipsoid::Ellipsoid;

/// Result of a forward projection, before false easting/northing.
#[derive(Debug, Clone, Copy)]
pub struct Projected {
    /// Easting relative to the central meridian (m).
    pub x: f64,
    /// Northing relative to the equator (m).
    pub y: f64,
    /// Meridian convergence (degrees).
    pub convergence: f64,
    /// Point scale factor.
    pub scale: f64,
}

struct Series {
    e: f64,
    /// Rectifying radius A.
    big_a: f64,
    alpha: [f64; 7],
    beta: [f64; 7],
}

fn series(ell: Ellipsoid) -> Series {
    let f = ell.f;
    let e = (f * (2.0 - f)).sqrt();
    let n = f / (2.0 - f);
    let (n2, n3, n4, n5, n6) = (n * n, n.powi(3), n.powi(4), n.powi(5), n.powi(6));

    let big_a = ell.a / (1.0 + n) * (1.0 + n2 / 4.0 + n4 / 64.0 + n6 / 256.0);

    let alpha = [
        0.0,
        n / 2.0 - 2.0 / 3.0 * n2 + 5.0 / 16.0 * n3 + 41.0 / 180.0 * n4 - 127.0 / 288.0 * n5
            + 7891.0 / 37800.0 * n6,
        13.0 / 48.0 * n2 - 3.0 / 5.0 * n3 + 557.0 / 1440.0 * n4 + 281.0 / 630.0 * n5
            - 1983433.0 / 1935360.0 * n6,
        61.0 / 240.0 * n3 - 103.0 / 140.0 * n4 + 15061.0 / 26880.0 * n5
            + 167603.0 / 181440.0 * n6,
        49561.0 / 161280.0 * n4 - 179.0 / 168.0 * n5 + 6601661.0 / 7257600.0 * n6,
        34729.0 / 80640.0 * n5 - 3418889.0 / 1995840.0 * n6,
        212378941.0 / 319334400.0 * n6,
    ];

    let beta = [
        0.0,
        n / 2.0 - 2.0 / 3.0 * n2 + 37.0 / 96.0 * n3 - 1.0 / 360.0 * n4 - 81.0 / 512.0 * n5
            + 96199.0 / 604800.0 * n6,
        n2 / 48.0 + n3 / 15.0 - 437.0 / 1440.0 * n4 + 46.0 / 105.0 * n5
            - 1118711.0 / 3870720.0 * n6,
        17.0 / 480.0 * n3 - 37.0 / 840.0 * n4 - 209.0 / 4480.0 * n5 + 5569.0 / 90720.0 * n6,
        4397.0 / 161280.0 * n4 - 11.0 / 504.0 * n5 - 830251.0 / 7257600.0 * n6,
        4583.0 / 161280.0 * n5 - 108847.0 / 3991680.0 * n6,
        20648693.0 / 638668800.0 * n6,
    ];

    Series { e, big_a, alpha, beta }
}

/// Conformal latitude tangent τ' from geodetic latitude tangent τ.
fn tau_prime(tau: f64, e: f64) -> f64 {
    let sigma = (e * (e * tau / (1.0 + tau * tau).sqrt()).atanh()).sinh();
    tau * (1.0 + sigma * sigma).sqrt() - sigma * (1.0 + tau * tau).sqrt()
}

/// Forward projection. `lat`, `lon`, `lon0` in degrees; `k0` central scale.
pub fn forward(ell: Ellipsoid, k0: f64, lon0: f64, lat: f64, lon: f64) -> Projected {
    let s = series(ell);
    let e = s.e;

    let phi = lat.to_radians();
    let lam = (lon - lon0).to_radians();

    let (sin_lam, cos_lam) = lam.sin_cos();
    let tau = phi.tan();
    let tau_p = tau_prime(tau, e);

    let xi_p = tau_p.atan2(cos_lam);
    let eta_p = (sin_lam / (tau_p * tau_p + cos_lam * cos_lam).sqrt()).asinh();

    let mut xi = xi_p;
    let mut eta = eta_p;
    let mut p_p = 1.0;
    let mut q_p = 0.0;
    for j in 1..=6 {
        let jj = 2.0 * j as f64;
        let (s2, c2) = (jj * xi_p).sin_cos();
        let (sh, ch) = ((jj * eta_p).sinh(), (jj * eta_p).cosh());
        xi += s.alpha[j] * s2 * ch;
        eta += s.alpha[j] * c2 * sh;
        p_p += jj * s.alpha[j] * c2 * ch;
        q_p += jj * s.alpha[j] * s2 * sh;
    }

    let x = k0 * s.big_a * eta;
    let y = k0 * s.big_a * xi;

    // Meridian convergence.
    let gamma_p = (tau_p / (1.0 + tau_p * tau_p).sqrt() * lam.tan()).atan();
    let gamma_pp = q_p.atan2(p_p);
    let gamma = gamma_p + gamma_pp;

    // Scale factor.
    let sin_phi = phi.sin();
    let k_p = (1.0 - e * e * sin_phi * sin_phi).sqrt() * (1.0 + tau * tau).sqrt()
        / (tau_p * tau_p + cos_lam * cos_lam).sqrt();
    let k_pp = s.big_a / ell.a * (p_p * p_p + q_p * q_p).sqrt();
    let k = k0 * k_p * k_pp;

    Projected { x, y, convergence: gamma.to_degrees(), scale: k }
}

/// Inverse projection. `x`, `y` relative to the central meridian / equator
/// (false easting/northing already removed). Returns (lat, lon) in degrees.
pub fn inverse(ell: Ellipsoid, k0: f64, lon0: f64, x: f64, y: f64) -> (f64, f64) {
    let s = series(ell);
    let e = s.e;

    let eta = x / (k0 * s.big_a);
    let xi = y / (k0 * s.big_a);

    let mut xi_p = xi;
    let mut eta_p = eta;
    for j in 1..=6 {
        let jj = 2.0 * j as f64;
        xi_p -= s.beta[j] * (jj * xi).sin() * (jj * eta).cosh();
        eta_p -= s.beta[j] * (jj * xi).cos() * (jj * eta).sinh();
    }

    let sinh_eta_p = eta_p.sinh();
    let (sin_xi_p, cos_xi_p) = xi_p.sin_cos();
    let tau_p = sin_xi_p / (sinh_eta_p * sinh_eta_p + cos_xi_p * cos_xi_p).sqrt();

    // Newton-Raphson for τ from τ'.
    let e2 = e * e;
    let mut tau_i = tau_p;
    for _ in 0..20 {
        let tau_i_p = tau_prime(tau_i, e);
        let d = (tau_p - tau_i_p) / (1.0 + tau_i_p * tau_i_p).sqrt()
            * (1.0 + (1.0 - e2) * tau_i * tau_i)
            / ((1.0 - e2) * (1.0 + tau_i * tau_i).sqrt());
        tau_i += d;
        if d.abs() <= 1e-12 {
            break;
        }
    }

    let lat = tau_i.atan().to_degrees();
    let lon = lon0 + sinh_eta_p.atan2(cos_xi_p).to_degrees();
    (lat, lon)
}
