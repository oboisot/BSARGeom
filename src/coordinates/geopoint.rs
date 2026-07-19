use bevy::math::DVec3;

/// A Geographic Point on an Ellipsoid of revolution.
/// 
/// A geographic (or geodetic) point is defined by its three coordinates:
/// longitude, latitude and height above the ellipsoid.  
/// See: [Geographic coordinate system](https://en.wikipedia.org/wiki/Geographic_coordinate_system)
/// for more details.
#[derive(Debug, Clone, PartialEq)]
pub struct GeographicPoint {
    coords: [f64; 3],
}

impl Default for GeographicPoint {
    /// Sets a default [`GeographicPoint`] at the origin (0°, 0°, 0m), i.e. at the intersection of
    /// the Greenwich meridian and the equator on the surface of the Ellipsoid.
    fn default() -> Self {
        Self {
            coords: [0.0, 0.0, 0.0],
        }
    }
}

impl GeographicPoint {
    /// Creates a new [`GeographicPoint`] from longitude, latitude in radians and height in meters.
    #[inline]
    pub const fn from_radians(lon_rad: f64, lat_rad: f64, height_m: f64) -> Self {
        Self {
            coords: [lon_rad, lat_rad, height_m],
        }
    }

    /// Creates a new [`GeographicPoint`] from longitude, latitude in degrees and height in meters.
    #[inline]
    pub fn from_degrees(lon_deg: f64, lat_deg: f64, height_m: f64) -> Self {
        Self {
            coords: [lon_deg.to_radians(), lat_deg.to_radians(), height_m],
        }
    }

    /// Creates a new [`GeographicPoint`] at the origin (0°, 0°, 0m), i.e. at the intersection of
    /// the Greenwich meridian and the equator on the surface of the Ellipsoid.
    #[inline]
    pub const fn origin() -> Self {
        Self {
            coords: [0.0, 0.0, 0.0],
        }
    }

    /// Gets the underlying coordinates as a 3-tuple: (lon_rad, lat_rad, height_m)
    #[inline]
    pub const fn coordinates(&self) -> (f64, f64, f64) {
        (self.coords[0], self.coords[1], self.coords[2])
    }

    /// Gets the underlying longitude in decimal degrees
    #[inline]
    pub fn lon_deg(&self) -> f64 {
        self.coords[0].to_degrees()
    }

    /// Gets the underlying longitude in radians
    #[inline]
    pub const fn lon_rad(&self) -> f64 {
        self.coords[0]
    }

    /// Gets the underlying longitude in Degrees Minutes Seconds (DMS) as a 3-tuple: (d, m ,s)
    #[inline]
    pub fn lon_dms(&self) -> (f64, f64, f64) {
        Self::dd_to_dms(self.coords[0].to_degrees())
    }

    /// Gets the underlying latitude in decimal degrees
    #[inline]
    pub fn lat_deg(&self) -> f64 {
        self.coords[1].to_degrees()
    }

    /// Gets the underlying latitude in radians
    #[inline]
    pub const fn lat_rad(&self) -> f64 {
        self.coords[1]
    }

    /// Gets the underlying latitude in Degrees Minutes Seconds (DMS) as a 3-tuple: (d, m ,s)
    #[inline]
    pub fn lat_dms(&self) -> (f64, f64, f64) {
        Self::dd_to_dms(self.coords[1].to_degrees())
    }

    /// Gets the underlying height in meters
    #[inline]
    pub const fn height_m(&self) -> f64 {
        self.coords[2]
    }

    /// Gets the underlying array of coordinates: [lon_rad, lat_rad, height_m]
    #[inline]
    pub const fn as_array(&self) -> [f64; 3] {
        self.coords
    }

    /// Converts decimal degrees to Degrees Minutes Seconds (DMS) and 
    /// return the values as a 3-tuple: (d, m, s)
    #[inline]
    pub fn dd_to_dms(dd: f64) -> (f64, f64, f64) {
        let d: f64 = dd.trunc();
        let _m: f64 = (dd - d).abs() * 60.0;
        let m: f64 = _m.trunc();
        let s: f64 = (_m - m) * 60.0;
        (d, m, s)
    }

    /// Converts decimal degrees to Degrees Minutes Seconds (DMS)
    /// returning the values as a formated String.
    /// 
    /// Note: this function first rounds to the 12-th decimal before converting into DMS.
    #[inline]
    pub fn dd_to_dms_as_string(dd: f64) -> String {
        let (d, m, s) = Self::dd_to_dms((dd * 1e12).round() * 1e-12);
        format!("{}° {}m {}s", d, m, s)
    }

    /// Converts Degrees Minutes Seconds (DMS) to decimal degrees
    #[inline]
    pub fn dms_to_dd(d: f64, m: f64, s: f64) -> f64 {
        const FRAC_1_60: f64 = 1.0 / 60.0;
        // note: is_sign_negative() (not `< 0.0`) so that angles in (-1°, 0°),
        // whose degrees part is -0.0, keep their sign
        if d.is_sign_negative() {
            -(-d + (m + s * FRAC_1_60) * FRAC_1_60)
        } else {
            d + (m + s * FRAC_1_60) * FRAC_1_60
        }
    }
}

/// A Cartesian ECEF[^note] Point on an Ellipsoid of revolution.
/// 
/// This is simply a type alias of [`DVec3`] for clarity.
/// 
/// [^note]: ECEF: **E**arth-**C**entered, **E**arth-**F**ixed.
/// See [Earth-centered, Earth-fixed coordinate system](https://en.wikipedia.org/wiki/Earth-centered,_Earth-fixed_coordinate_system) for more details.
pub type CartesianECEFPoint = DVec3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dd_to_dms_reference_values() {
        assert_eq!(GeographicPoint::dd_to_dms(43.5), (43.0, 30.0, 0.0));
        assert_eq!(GeographicPoint::dd_to_dms(-7.5), (-7.0, 30.0, 0.0));
        let (d, m, s) = GeographicPoint::dd_to_dms(12.505);
        assert_eq!((d, m), (12.0, 30.0));
        assert!((s - 18.0).abs() < 1e-9);
    }

    #[test]
    fn dms_to_dd_reference_values() {
        assert_eq!(GeographicPoint::dms_to_dd(43.0, 30.0, 0.0), 43.5);
        assert_eq!(GeographicPoint::dms_to_dd(-7.0, 30.0, 0.0), -7.5);
        assert_eq!(GeographicPoint::dms_to_dd(0.0, 15.0, 0.0), 0.25);
    }

    #[test]
    fn dms_roundtrip_keeps_sign_between_minus_one_and_zero_degrees() {
        // Regression test: the degrees part of -0.25° is -0.0 and its sign used
        // to be dropped by `d < 0.0` in dms_to_dd
        let (d, m, s) = GeographicPoint::dd_to_dms(-0.25);
        assert!(d == 0.0 && d.is_sign_negative()); // -0.0
        assert_eq!((m, s), (15.0, 0.0));
        assert_eq!(GeographicPoint::dms_to_dd(d, m, s), -0.25);
    }

    #[test]
    fn dms_roundtrip_random_values() {
        for &dd in [12.3456789, -45.987654, 0.001, -0.999, 179.999999].iter() {
            let (d, m, s) = GeographicPoint::dd_to_dms(dd);
            assert!(
                (GeographicPoint::dms_to_dd(d, m, s) - dd).abs() < 1e-9,
                "roundtrip failed for {dd}"
            );
        }
    }

    #[test]
    fn geographic_point_accessors() {
        let gp = GeographicPoint::from_degrees(5.93, 43.12, 100.0);
        assert!((gp.lon_deg() - 5.93).abs() < 1e-12);
        assert!((gp.lat_deg() - 43.12).abs() < 1e-12);
        let (lon_rad, lat_rad, height_m) = gp.coordinates();
        assert!((lon_rad - 5.93f64.to_radians()).abs() < 1e-15);
        assert!((lat_rad - 43.12f64.to_radians()).abs() < 1e-15);
        assert_eq!(height_m, 100.0);
    }
}
