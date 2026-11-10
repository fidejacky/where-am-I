use std::time::Duration;

use serde::Deserialize;

use crate::{Error, Location, Place, Provider, Source};

const ENDPOINT: &str = "https://ipinfo.io/json";

#[derive(Debug, Clone)]
pub struct IpInfo {
    timeout: Duration,
}

impl Default for IpInfo {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
        }
    }
}

impl IpInfo {
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Provider for IpInfo {
    fn name(&self) -> &'static str {
        "ipinfo"
    }

    fn locate(&self) -> Result<Location, Error> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(self.timeout))
            .build()
            .into();

        let mut response = agent
            .get(ENDPOINT)
            .header("Accept", "application/json")
            .call()
            .map_err(|e| match e {
                ureq::Error::StatusCode(429) => Error::Network(
                    "ipinfo.io rate limit reached (about 1,000 lookups a day); try again later"
                        .into(),
                ),
                other => Error::Network(other.to_string()),
            })?;

        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|e| Error::Network(e.to_string()))?;

        parse(&body)
    }
}

#[derive(Deserialize)]
struct Response {
    loc: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
}

fn parse(body: &str) -> Result<Location, Error> {
    let response: Response = serde_json::from_str(body)
        .map_err(|e| Error::BadResponse(format!("not valid JSON: {e}")))?;

    let loc = response.loc.ok_or_else(|| {
        Error::BadResponse(
            "no location in response (is this a private or reserved address?)".into(),
        )
    })?;

    let (lat, lon) = loc
        .split_once(',')
        .ok_or_else(|| Error::BadResponse(format!("expected \"lat,lon\", got {loc:?}")))?;
    let lat: f64 = lat
        .trim()
        .parse()
        .map_err(|_| Error::BadResponse(format!("bad latitude in {loc:?}")))?;
    let lon: f64 = lon
        .trim()
        .parse()
        .map_err(|_| Error::BadResponse(format!("bad longitude in {loc:?}")))?;

    let place = Place::new(response.city, response.region, response.country);
    Ok(Location::new(lat, lon, None, Source::Ip)?.with_place(place))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real response shape from ipinfo.io for 8.8.8.8 (Google public DNS).
    const SAMPLE: &str = r#"{
        "ip": "8.8.8.8",
        "hostname": "dns.google",
        "city": "Mountain View",
        "region": "California",
        "country": "US",
        "loc": "37.4056,-122.0775",
        "org": "AS15169 Google LLC",
        "postal": "94043",
        "timezone": "America/Los_Angeles",
        "readme": "https://ipinfo.io/missingauth",
        "anycast": true
    }"#;

    #[test]
    fn parses_a_real_response() {
        let loc = parse(SAMPLE).unwrap();
        assert_eq!(loc.lat, 37.4056);
        assert_eq!(loc.lon, -122.0775);
        assert_eq!(loc.accuracy_m, None);
        assert_eq!(loc.source, Source::Ip);
        assert_eq!(
            loc.place,
            Place::new(
                Some("Mountain View".into()),
                Some("California".into()),
                Some("US".into())
            )
        );
    }

    #[test]
    fn missing_or_blank_place_fields_leave_the_place_empty() {
        assert_eq!(parse(r#"{"loc": "1.0,2.0"}"#).unwrap().place, None);
        let blank = r#"{"loc": "1.0,2.0", "city": "", "region": " ", "country": ""}"#;
        assert_eq!(parse(blank).unwrap().place, None);
    }

    #[test]
    fn keeps_a_partial_place() {
        let loc = parse(r#"{"loc": "1.0,2.0", "country": "SG"}"#).unwrap();
        assert_eq!(loc.place, Place::new(None, None, Some("SG".into())));
    }

    #[test]
    fn tolerates_spaces_around_the_comma() {
        let loc = parse(r#"{"loc": "-33.8688 , 151.2093"}"#).unwrap();
        assert_eq!((loc.lat, loc.lon), (-33.8688, 151.2093));
    }

    #[test]
    fn bogon_response_without_loc_is_an_error() {
        let err = parse(r#"{"ip": "192.168.1.1", "bogon": true}"#).unwrap_err();
        assert!(matches!(err, Error::BadResponse(_)));
    }

    #[test]
    fn malformed_loc_is_an_error() {
        for body in [
            r#"{"loc": "37.4056"}"#,
            r#"{"loc": "north,west"}"#,
            r#"{"loc": ""}"#,
            r#"{"loc": "91.0,0.0"}"#,
            r#"{"loc": "0.0,181.0"}"#,
        ] {
            assert!(
                matches!(parse(body), Err(Error::BadResponse(_))),
                "should reject {body}"
            );
        }
    }

    #[test]
    fn non_json_is_an_error() {
        assert!(matches!(
            parse("<html>nope</html>"),
            Err(Error::BadResponse(_))
        ));
    }
}
