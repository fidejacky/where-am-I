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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
}

impl Place {
    pub fn new(
        city: Option<String>,
        region: Option<String>,
        country: Option<String>,
    ) -> Option<Self> {
        let clean = |s: Option<String>| s.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
        let (city, region, country) = (clean(city), clean(region), clean(country));
        if city.is_none() && region.is_none() && country.is_none() {
            return None;
        }
        Some(Self {
            city,
            region,
            country,
        })
    }
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<&str> = [&self.city, &self.region, &self.country]
            .into_iter()
            .flatten()
            .map(String::as_str)
            .collect();
        f.write_str(&parts.join(", "))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub accuracy_m: Option<f64>,
    pub place: Option<Place>,
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
            place: None,
            source,
        })
    }

    pub fn with_place(mut self, place: Option<Place>) -> Self {
        self.place = place;
        self
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}, {:.4}", self.lat, self.lon)?;
        if let Some(place) = &self.place {
            write!(f, " — {place}")?;
        }
        f.write_str(" (")?;
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

    fn place(city: Option<&str>, region: Option<&str>, country: Option<&str>) -> Option<Place> {
        Place::new(
            city.map(String::from),
            region.map(String::from),
            country.map(String::from),
        )
    }

    #[test]
    fn displays_the_place_between_coordinates_and_details() {
        let loc = Location::new(51.5074, -0.1278, None, Source::Ip)
            .unwrap()
            .with_place(place(Some("London"), Some("England"), Some("GB")));
        assert_eq!(
            loc.to_string(),
            "51.5074, -0.1278 — London, England, GB (city-level, source: ip)"
        );
    }

    #[test]
    fn displays_only_the_place_parts_that_exist() {
        let loc = Location::new(1.0, 2.0, None, Source::Ip)
            .unwrap()
            .with_place(place(None, Some("England"), Some("GB")));
        assert_eq!(
            loc.to_string(),
            "1.0000, 2.0000 — England, GB (city-level, source: ip)"
        );
    }

    #[test]
    fn place_drops_blank_parts_and_is_none_when_nothing_is_left() {
        let trimmed = place(Some(" London "), Some(""), Some("  ")).unwrap();
        assert_eq!(trimmed.city.as_deref(), Some("London"));
        assert_eq!(trimmed.region, None);
        assert_eq!(trimmed.country, None);
        assert_eq!(place(Some(""), Some(" "), None), None);
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
