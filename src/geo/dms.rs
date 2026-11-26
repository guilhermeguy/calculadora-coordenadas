//! Parsing and formatting of decimal degrees (DD) and degrees-minutes-seconds
//! (GMS / DMS). Accepts both '.' and ',' as decimal separator.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Lat,
    Lon,
}

impl Axis {
    fn name(self) -> &'static str {
        match self {
            Axis::Lat => "Latitude",
            Axis::Lon => "Longitude",
        }
    }

    fn max(self) -> f64 {
        match self {
            Axis::Lat => 90.0,
            Axis::Lon => 180.0,
        }
    }

    /// Hemisphere letter -> is negative. Portuguese L (leste) / O (oeste) accepted.
    fn hemisphere(self, c: char) -> Option<bool> {
        match (self, c) {
            (Axis::Lat, 'N') => Some(false),
            (Axis::Lat, 'S') => Some(true),
            (Axis::Lon, 'E' | 'L') => Some(false),
            (Axis::Lon, 'W' | 'O') => Some(true),
            _ => None,
        }
    }

    pub fn hemisphere_letter(self, negative: bool) -> char {
        match (self, negative) {
            (Axis::Lat, false) => 'N',
            (Axis::Lat, true) => 'S',
            (Axis::Lon, false) => 'E',
            (Axis::Lon, true) => 'W',
        }
    }
}

/// Parses a plain number. Spaces are ignored; ',' is a decimal separator,
/// unless both ',' and '.' appear (then '.' groups thousands, e.g. 7.394.000,5).
pub fn parse_number(s: &str, what: &str) -> Result<f64, String> {
    let mut t: String = s.chars().filter(|c| !c.is_whitespace() && *c != '_').collect();
    if t.is_empty() {
        return Err(format!("{what}: campo vazio."));
    }
    if t.contains(',') {
        t = t.replace('.', "").replace(',', ".");
    } else if t.matches('.').count() > 1 {
        t = t.replace('.', "");
    }
    t.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| format!("{what}: \"{}\" não é um número válido.", s.trim()))
}

/// Parses an angle written as DD, DM or DMS, with optional sign or hemisphere
/// letter. Returns (absolute value in degrees, explicit sign if any).
fn parse_angle_parts(s: &str, axis: Axis) -> Result<(f64, Option<bool>), String> {
    let what = axis.name();
    let up = s.trim().to_uppercase();
    if up.is_empty() {
        return Err(format!("{what}: campo vazio."));
    }

    let mut explicit: Option<bool> = None;
    let mut cleaned = String::with_capacity(up.len());
    for (i, c) in up.chars().enumerate() {
        if c.is_ascii_alphabetic() {
            let neg = axis.hemisphere(c).ok_or_else(|| {
                format!("{what}: letra '{c}' não reconhecida como hemisfério.")
            })?;
            if explicit.is_some_and(|e| e != neg) {
                return Err(format!("{what}: hemisférios conflitantes."));
            }
            explicit = Some(neg);
            cleaned.push(' ');
        } else if c == '-' || c == '+' {
            if !cleaned.trim().is_empty() && i > 0 {
                return Err(format!("{what}: sinal '{c}' fora de posição."));
            }
            let neg = c == '-';
            if explicit.is_some_and(|e| e != neg) {
                return Err(format!("{what}: sinal e hemisfério conflitantes."));
            }
            explicit = Some(neg);
            cleaned.push(' ');
        } else if c.is_ascii_digit() || c == '.' || c == ',' {
            cleaned.push(c);
        } else if c.is_whitespace() || "°º˚'\"′″’”´`:;".contains(c) {
            // Degree / minute / second marks separate components.
            cleaned.push(' ');
        } else {
            return Err(format!("{what}: caractere '{c}' não reconhecido."));
        }
    }

    let parts: Vec<f64> = cleaned
        .split_whitespace()
        .map(|p| parse_number(p, what))
        .collect::<Result<_, _>>()?;

    let value = match parts.as_slice() {
        [d] => *d,
        [d, m] => dms_to_deg(*d, *m, 0.0, what)?,
        [d, m, sec] => {
            if m.fract() != 0.0 {
                return Err(format!("{what}: minutos devem ser inteiros quando há segundos."));
            }
            dms_to_deg(*d, *m, *sec, what)?
        }
        [] => return Err(format!("{what}: nenhum número encontrado.")),
        _ => return Err(format!("{what}: componentes demais (use G M S).")),
    };
    if value > axis.max() {
        return Err(format!("{what}: {value}° excede ±{}°.", axis.max()));
    }
    Ok((value, explicit))
}

fn dms_to_deg(d: f64, m: f64, s: f64, what: &str) -> Result<f64, String> {
    if d.fract() != 0.0 {
        return Err(format!("{what}: graus devem ser inteiros quando há minutos."));
    }
    if !(0.0..60.0).contains(&m) {
        return Err(format!("{what}: minutos devem estar entre 0 e 59."));
    }
    if !(0.0..60.0).contains(&s) {
        return Err(format!("{what}: segundos devem estar entre 0 e 59,999."));
    }
    Ok(d + m / 60.0 + s / 3600.0)
}

/// Parses a full angle string (DD or DMS). Positive unless a '-' or S/W/O
/// letter is present.
pub fn parse_angle(s: &str, axis: Axis) -> Result<f64, String> {
    let (v, neg) = parse_angle_parts(s, axis)?;
    Ok(if neg == Some(true) { -v } else { v })
}

/// Combines separate degree / minute / second fields. If minutes and seconds
/// are empty, the degree field may hold a full DD/DMS string (pasted).
/// `default_negative` comes from the hemisphere selector and is used unless
/// the text has an explicit sign/letter.
pub fn parse_dms_fields(
    deg: &str,
    min: &str,
    sec: &str,
    default_negative: bool,
    axis: Axis,
) -> Result<f64, String> {
    let what = axis.name();
    let (abs, explicit) = if min.trim().is_empty() && sec.trim().is_empty() {
        parse_angle_parts(deg, axis)?
    } else {
        let (d, explicit) = parse_angle_parts(deg, axis)?;
        let m = if min.trim().is_empty() { 0.0 } else { parse_number(min, what)? };
        let s = if sec.trim().is_empty() { 0.0 } else { parse_number(sec, what)? };
        if !sec.trim().is_empty() && m.fract() != 0.0 {
            return Err(format!("{what}: minutos devem ser inteiros quando há segundos."));
        }
        let v = dms_to_deg(d, m, s, what)?;
        if v > axis.max() {
            return Err(format!("{what}: excede ±{}°.", axis.max()));
        }
        (v, explicit)
    };
    let neg = explicit.unwrap_or(default_negative);
    Ok(if neg { -abs } else { abs })
}

/// Splits decimal degrees into (negative, degrees, minutes, seconds), with
/// seconds rounded to `sec_decimals` places and carries propagated.
pub fn to_dms(value: f64, sec_decimals: u32) -> (bool, u32, u32, f64) {
    let scale = 10u64.pow(sec_decimals);
    let units = (value.abs() * 3600.0 * scale as f64).round() as u64;
    let per_min = 60 * scale;
    let per_deg = 3600 * scale;
    let d = (units / per_deg) as u32;
    let m = ((units % per_deg) / per_min) as u32;
    let s = (units % per_min) as f64 / scale as f64;
    let negative = value < 0.0 && units > 0;
    (negative, d, m, s)
}

pub fn format_dms(value: f64, axis: Axis) -> String {
    let (neg, d, m, s) = to_dms(value, 3);
    format!("{d}°{m:02}'{s:06.3}\" {}", axis.hemisphere_letter(neg))
}

/// UTM metres as a whole number (rounded half away from zero), e.g. "333288".
pub fn format_metres(value: f64) -> String {
    format!("{}", value.round() as i64)
}

pub fn format_dd(value: f64) -> String {
    format!("{value:.7}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn numbers() {
        assert!(close(parse_number("-23,5505", "x").unwrap(), -23.5505));
        assert!(close(parse_number("7 394 000.25", "x").unwrap(), 7_394_000.25));
        assert!(close(parse_number("7.394.000,25", "x").unwrap(), 7_394_000.25));
        assert!(parse_number("abc", "x").is_err());
        assert!(parse_number("", "x").is_err());
    }

    #[test]
    fn angles() {
        let v = 23.0 + 32.0 / 60.0 + 51.2 / 3600.0;
        assert!(close(parse_angle("23°32'51.2\"S", Axis::Lat).unwrap(), -v));
        assert!(close(parse_angle("23 32 51,2 S", Axis::Lat).unwrap(), -v));
        assert!(close(parse_angle("-23 32 51.2", Axis::Lat).unwrap(), -v));
        assert!(close(parse_angle("S 23º 32′ 51,2″", Axis::Lat).unwrap(), -v));
        assert!(close(parse_angle("-23.5475", Axis::Lat).unwrap(), -23.5475));
        assert!(close(parse_angle("46 38.5 O", Axis::Lon).unwrap(), -(46.0 + 38.5 / 60.0)));
        assert!(close(parse_angle("46.5 L", Axis::Lon).unwrap(), 46.5));
        assert!(close(parse_angle("46.5W", Axis::Lon).unwrap(), -46.5));
        assert!(parse_angle("23 61 0", Axis::Lat).is_err());
        assert!(parse_angle("23 10 60", Axis::Lat).is_err());
        assert!(parse_angle("91", Axis::Lat).is_err());
        assert!(parse_angle("23 S", Axis::Lon).is_err());
        assert!(parse_angle("-23 N", Axis::Lat).is_err());
        assert!(parse_angle("23.5 10", Axis::Lat).is_err());
        assert!(parse_angle("/23,5505", Axis::Lat).is_err());
        assert!(parse_angle("23*30", Axis::Lat).is_err());
        assert!(close(parse_angle("23:30:00 S", Axis::Lat).unwrap(), -23.5));
    }

    #[test]
    fn dms_fields() {
        let v = 46.0 + 38.0 / 60.0 + 0.5 / 3600.0;
        assert!(close(parse_dms_fields("46", "38", "0,5", true, Axis::Lon).unwrap(), -v));
        assert!(close(parse_dms_fields("46", "38", "0.5", false, Axis::Lon).unwrap(), v));
        // pasted full string in degree field overrides the selector
        assert!(close(parse_dms_fields("46°38'0.5\"E", "", "", true, Axis::Lon).unwrap(), v));
        assert!(parse_dms_fields("46", "60", "", true, Axis::Lon).is_err());
    }

    #[test]
    fn formatting() {
        assert_eq!(to_dms(-23.5, 3), (true, 23, 30, 0.0));
        // 59.9999 s rounds up and carries into minutes and degrees
        let (_, d, m, s) = to_dms(10.0 + 59.0 / 60.0 + 59.9999 / 3600.0, 3);
        assert_eq!((d, m, s), (11, 0, 0.0));
        assert_eq!(format_dms(-23.547_555_6, Axis::Lat), "23°32'51.200\" S");
        assert_eq!(format_dms(0.0, Axis::Lon), "0°00'00.000\" E");
        assert_eq!(format_metres(333_287.915), "333288");
        assert_eq!(format_metres(7_394_588.319), "7394588");
        assert_eq!(format_metres(500_000.5), "500001");
        assert_eq!(format_metres(166_021.443), "166021");
    }
}
