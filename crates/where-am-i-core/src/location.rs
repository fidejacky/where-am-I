use std::fmt;

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Source {
    Ip,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Ip => "ip",
        }
    }

    fn coarse_label(self) -> &'static str {
        match self {
            Source::Ip => "city-level",
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub accuracy_m: Option<f64>,
    pub source: Source,
}

impl Location {
    pub fn new(lat: f64, lon: f64, accuracy_m: Option<f64>, source: Source) -> Result<Self, Error> {
        if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) {
            return Err(Error::BadResponse(format!(
                "latitude {lat} is out of range"
            )));
        }
        if !lon.is_finite() || !(-180.0..=180.0).contains(&lon) {
            return Err(Error::BadResponse(format!(
                "longitude {lon} is out of range"
            )));
        }
        if let Some(m) = accuracy_m
            && (!m.is_finite() || m < 0.0)
        {
            return Err(Error::BadResponse(format!("accuracy {m} m is invalid")));
        }
        Ok(Self {
            lat,
            lon,
            accuracy_m,
            source,
        })
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}, {:.4} (", self.lat, self.lon)?;
        match self.accuracy_m {
            Some(m) if m >= 1000.0 => write!(f, "±{:.0} km", m / 1000.0)?,
            Some(m) => write!(f, "±{m:.0} m")?,
            None => f.write_str(self.source.coarse_label())?,
        }
        write!(f, ", source: {})", self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_coarse_label_when_accuracy_unknown() {
        let loc = Location::new(51.5074, -0.1278, None, Source::Ip).unwrap();
        assert_eq!(loc.to_string(), "51.5074, -0.1278 (city-level, source: ip)");
    }

    #[test]
    fn displays_metres_and_kilometres() {
        let near = Location::new(1.0, 2.0, Some(3.0), Source::Ip).unwrap();
        assert_eq!(near.to_string(), "1.0000, 2.0000 (±3 m, source: ip)");
        let far = Location::new(1.0, 2.0, Some(5000.0), Source::Ip).unwrap();
        assert_eq!(far.to_string(), "1.0000, 2.0000 (±5 km, source: ip)");
    }

    #[test]
    fn accepts_the_extremes() {
        assert!(Location::new(90.0, 180.0, None, Source::Ip).is_ok());
        assert!(Location::new(-90.0, -180.0, None, Source::Ip).is_ok());
    }

    #[test]
    fn rejects_impossible_coordinates() {
        assert!(Location::new(90.1, 0.0, None, Source::Ip).is_err());
        assert!(Location::new(0.0, -180.1, None, Source::Ip).is_err());
        assert!(Location::new(f64::NAN, 0.0, None, Source::Ip).is_err());
        assert!(Location::new(0.0, f64::INFINITY, None, Source::Ip).is_err());
        assert!(Location::new(0.0, 0.0, Some(-1.0), Source::Ip).is_err());
        assert!(Location::new(0.0, 0.0, Some(f64::NAN), Source::Ip).is_err());
    }
}
