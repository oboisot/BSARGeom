use bevy::math::{DAffine3, DMat3, DVec3};

use crate::coordinates::{CartesianECEFPoint, GeographicPoint};

/// WGS84 constants
const WGS84_EQUATORIAL_RADIUS_M: f64 = 6378137.0;
const WGS84_FIRST_FLATTENING: f64 = 1.0 / 298.257223563;
const WGS84_POLAR_RADIUS_M: f64 = (1.0 - WGS84_FIRST_FLATTENING) * WGS84_EQUATORIAL_RADIUS_M;
const WGS84_SQUARED_ECCENTRICITY: f64 = WGS84_FIRST_FLATTENING * (2.0 - WGS84_FIRST_FLATTENING);

/// An Ellipsoid of revolution.
/// 
/// An Ellipsoid of revolution, also known as a [`Spheroid`], is used to model
/// the [`Earth`] shape (or any other celestial body).
/// 
/// This struct implements simple conversion methods between [`CartesianECEFPoint`] and
/// [`GeographicPoint`] coordinates.
///
/// [`Spheroid`]: https://en.wikipedia.org/wiki/Spheroid
/// [`Earth`]: https://en.wikipedia.org/wiki/Earth_ellipsoid
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ellipsoid {
    a: f64,  // equatorial radius in meters
    b: f64,  // polar radius in meters
    f: f64,  // first flattening parameter
    e2: f64, // squared eccentricity parameter
}

impl Default for Ellipsoid {
    fn default() -> Self {
        Self::WGS84
    }
}

impl Ellipsoid {
    /// [WGS84](https://en.wikipedia.org/wiki/World_Geodetic_System#WGS_84) Ellipsoid
    pub const WGS84: Ellipsoid = Ellipsoid {
        a: WGS84_EQUATORIAL_RADIUS_M,
        b: WGS84_POLAR_RADIUS_M,
        f: WGS84_FIRST_FLATTENING,
        e2: WGS84_SQUARED_ECCENTRICITY,
    };

    /// Creates a new Ellipsoid from its equatorial radius in meters and first flattening parameters.
    pub fn new(equatorial_radius_m: f64, first_flattening: f64) -> Self {
        Self {
            a: equatorial_radius_m,
            b: (1.0 - first_flattening) * equatorial_radius_m,
            f: first_flattening,
            e2: first_flattening * (2.0 - first_flattening),
        }
    }

    /// Computes the first flattening parameter of the Ellipsoid from equatorial and polar
    /// radii in meters.
    /// 
    /// Note: this is a convenient function for Ellipsoid defined with polar radius rather than flattening parameter.
    /// 
    /// Example:
    /// ```rust
    /// // Initialize a Clarke 1880 Ellipsoid with definition given by (https://epsg.org/ellipsoid_7034/Clarke-1880.html):
    /// let semi_major_axis = 20926202.0 * 0.3047972654; 
    /// let semi_minor_axis = 20854895.0 * 0.3047972654;
    /// let first_flattening = Ellipsoid::first_flattening_from_radii(semi_major_axis, semi_minor_axis);
    /// let Clarke1880 = Ellipsoid::new(semi_major_axis, first_flattening);
    /// ```
    #[inline]
    pub fn first_flattening_from_radii(equatorial_radius_m: f64, polar_radius_m: f64) -> f64 {
        (equatorial_radius_m - polar_radius_m) / equatorial_radius_m
    }

    /// Gets the equatorial radius (i.e. semi-major axis) in meters of this Ellipsoid.
    #[inline]
    pub const fn equatorial_radius_m(&self) -> f64 {
        self.a
    }
    /// Gets the polar radius (i.e. semi-minor axis) in meters of this Ellipsoid.
    #[inline]
    pub const fn polar_radius_m(&self) -> f64 {
        self.b
    }
    /// Gets the first flattening of this Ellipsoid.
    #[inline]
    pub const fn first_flattening(&self) -> f64 {
        self.f
    }
    /// Gets the eccentricity of this Ellipsoid.
    #[inline]
    pub fn eccentricity(&self) -> f64 {
        self.e2.sqrt()
    }
    /// Gets the eccentricity squared of this Ellipsoid.
    #[inline]
    pub const fn eccentricity_squared(&self) -> f64 {
        self.e2
    }    

    /// Transforms a [`GeographicPoint`] to a [`CartesianECEFPoint`] using this Ellipsoid.
    #[inline]
    pub fn to_cartesian_ecef_point(&self, gp: &GeographicPoint) -> CartesianECEFPoint {
        let (slat, clat) = gp.lat_rad().sin_cos();
        let (slon, clon) = gp.lon_rad().sin_cos();
        // Computes the prime vertical curvature radius "nu"
        let nu: f64 = self.a / (1.0 - self.e2 * slat * slat).sqrt();
        let nuhcosphi: f64 = (nu + gp.height_m()) * clat;
        CartesianECEFPoint::new(
            nuhcosphi * clon,
            nuhcosphi * slon,
            ((1.0 - self.e2) * nu + gp.height_m()) * slat,
        )
    }

    /// Transforms a [`CartesianECEFPoint`] to a [`GeographicPoint`] using this Ellipsoid.
    /// 
    /// The transformation is based on the Vermeille algorithm[^note] with a final
    /// transform height accuracy of order of nanometers.
    /// 
    /// <div class="warning">The Vermeille algorithm is partially implemented here:
    /// there is no check on the sign of the "evolute" parameter, assuming that the
    /// height is not "too deep" towards the center of the Earth. The limit is given by
    /// 
    /// `polar_radius - a*e²/sqrt(1-e²)` which is approximately -6314 km for the WGS84 ellipsoid.
    /// </div>
    /// 
    /// [^note]: Vermeille, H., *Direct transformation from geocentric coordinates to geodetic coordinates*.
    /// Journal of Geodesy 76, 451–454 (2002). <https://doi.org/10.1007/s00190-002-0273-6>
    #[inline]
    pub fn to_geographic_point(&self, cp: &CartesianECEFPoint) -> GeographicPoint {
        let _inv_a2: f64 = 1.0 / (self.a * self.a);
        let _e4: f64 = self.e2 * self.e2;
        let dist: f64 = cp.x.hypot(cp.y); // Distance from the Ellipsoid center in the equatorial plane
        let p: f64 = dist * dist * _inv_a2;
        let q: f64 = (1.0 - self.e2) * cp.z * cp.z * _inv_a2;
        let r: f64 = (p + q - _e4) / 6.0;
        let _r2: f64 = r * r;
        let mut _cbrt: f64 =
            ((8.0 * _r2 * r + _e4 * p * q).sqrt() + self.e2 * (p * q).sqrt()).cbrt();
        _cbrt *= _cbrt; // ^(2/3)
        let u: f64 = r + 0.5 * _cbrt + 2.0 * _r2 / _cbrt;
        let v: f64 = (u * u + _e4 * q).sqrt();
        let _uv: f64 = u + v;
        let w: f64 = 0.5 * self.e2 * (_uv - q) / v;
        let k: f64 = _uv / (w + (w * w + _uv).sqrt());
        let d: f64 = k * dist / (k + self.e2);
        let _hypotdz: f64 = d.hypot(cp.z);

        GeographicPoint::from_radians(
            cp.y.atan2(cp.x),
            2.0 * (cp.z / (d + _hypotdz)).atan(),
            (k + self.e2 - 1.0) * _hypotdz / k,
        )
    }

    /// Computes the **first** point intersected by the given line with this Ellipsoid surface.
    /// 
    /// The line is defined by a [`CartesianECEFPoint`] `pos` and a direction vector `axis`.
    /// <div class="warning">The `axis` should be a normalized vector.</div>
    #[inline]
    pub fn line_intersection(
        &self,
        pos: &CartesianECEFPoint,
        axis: &DVec3,
    ) -> CartesianECEFPoint {
        let a2: f64 = (self.polar_radius_m() / self.equatorial_radius_m()).powi(2);
        let inv_denom = 1.0 / (a2 * (axis.x * axis.x + axis.y * axis.y) + axis.z * axis.z);
        let b = (a2 * (pos.x * axis.x + pos.y * axis.y) + pos.z * axis.z) * inv_denom;
        let c = (a2 * (pos.x * pos.x + pos.y * pos.y) + pos.z * pos.z
            - self.polar_radius_m() * self.polar_radius_m())
            * inv_denom;
        let mut delta = b * b - c;
        let t = if delta >= 0.0 {
            delta = delta.sqrt();
            let tp = -b + delta;
            let tm = -b - delta;
            if tp.abs() <= tm.abs() {
                tp
            } else {
                tm
            }
        } else {
            0.0
        };
        pos + t * axis
    }
}

/// A Local Cartesian reference frame on a given Ellipsoid of revolution.
/// 
/// This struct allows transformations from/to local ENU/NED[^note] from/to [`GeographicPoint`]
/// and/or [`CartesianECEFPoint`].
/// 
/// [^note]: ENU: **E**ast-**N**orth-**U**p, NED: **N**orth-**E**ast-**D**own.
/// See [Local Tangent Plane](https://en.wikipedia.org/wiki/Local_tangent_plane_coordinates) for more details.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalCartesian {
    origin: (GeographicPoint, CartesianECEFPoint),
    transform: DAffine3, // local NED to ECEF isometry, i.e. translation + rotation for 'Point' and only rotation for 'Vector'
    inverse_transform: DAffine3, // ECEF to local NED isometry
}


impl Default for LocalCartesian {
    /// Sets a default local Cartesian with WGS84 Ellipsoid with origin set at 
    /// the intersection of the Greenwich meridian and equator lines,
    /// i.e. at geographic coordinates (0°, 0°, 0m).
    fn default() -> Self {
        const GP: GeographicPoint = GeographicPoint::origin();
        let cp = Ellipsoid::WGS84.to_cartesian_ecef_point(&GP);
        let (transform, inverse_transform) =
            Self::set_ned_to_ecef_transform(&GP, &cp);
        Self {
            origin: (GP, cp),
            transform,
            inverse_transform
        }
    }
}

impl LocalCartesian {
    /// Creates a new Local Cartesian reference frame on the given Ellipsoid of revolution.
    /// 
    /// The origin of the frame is set at the intersection of the Greenwich meridian and equator lines,
    /// i.e. at geographic coordinates (0°, 0°, 0m).
    pub fn new() -> Self {
        const GP: GeographicPoint = GeographicPoint::origin();
        let cp = Ellipsoid::WGS84.to_cartesian_ecef_point(&GP);
        let (transform, inverse_transform) =
            Self::set_ned_to_ecef_transform(&GP, &cp);
        Self {
            origin: (GP, cp),
            transform,
            inverse_transform
        }
    }

    /// Creates a new Local Cartesian reference frame on the given Ellipsoid of revolution
    /// with its origin set at the given [`GeographicPoint`].
    #[inline]
    pub fn from_geographic_point(gp: &GeographicPoint) -> Self {
        let cp = Ellipsoid::WGS84.to_cartesian_ecef_point(gp);
        let (transform, inverse_transform) =
            Self::set_ned_to_ecef_transform(gp, &cp);
        Self {
            origin: (gp.clone(), cp),
            transform,
            inverse_transform
        }
    }

    /// Creates a new Local Cartesian reference frame on the given Ellipsoid of revolution
    /// with its origin set at the given [`CartesianECEFPoint`].
    #[inline]
    pub fn from_cartesian_ecef_point(cp: &CartesianECEFPoint) -> Self {
        let gp = Ellipsoid::WGS84.to_geographic_point(cp);
        let (transform, inverse_transform) =
            Self::set_ned_to_ecef_transform(&gp, cp);
        Self {
            origin: (gp, cp.clone()),
            transform,
            inverse_transform
        }
    }

    /// Sets the origin of the Local Cartesian reference frame from a [`GeographicPoint`].
    #[inline]
    pub fn set_origin_from_geographic_point(&mut self, gp: &GeographicPoint) -> &mut Self {
        let cp = Ellipsoid::WGS84.to_cartesian_ecef_point(gp);
        self.origin = (gp.clone(), cp);
        (self.transform, self.inverse_transform) =
            Self::set_ned_to_ecef_transform(gp, &cp);
        self
    }

    /// Sets the origin of the Local Cartesian reference frame from a [`CartesianECEFPoint`].
    #[inline]
    pub fn set_origin_from_cartesian_ecef_point(&mut self, cp: &CartesianECEFPoint) -> &mut Self {
        let gp = Ellipsoid::WGS84.to_geographic_point(cp);
        self.origin = (gp.clone(), cp.clone());
        (self.transform, self.inverse_transform) =
            Self::set_ned_to_ecef_transform(&gp, cp);
        self
    }

    /// Gets the origin of the Local Cartesian reference frame as a [`GeographicPoint`].
    #[inline]
    pub const fn origin_as_geographic_point(&self) -> &GeographicPoint {
        &self.origin.0
    }

    /// Gets the origin of the Local Cartesian reference frame as a [`CartesianECEFPoint`].
    #[inline]
    pub const fn origin_as_cartesian_ecef_point(&self) -> &CartesianECEFPoint {
        &self.origin.1
    }

    /******************************/
    /* NED <-> GeoPoint transform */
    /******************************/
    /// Transforms a point from Local Cartesian NED coordinates to geocentric Cartesian ECEF coordinates.
    #[inline]
    pub fn transform_from_ned_point_to_cartesian_ecef_point(
        &self,
        point: &DVec3,
    ) -> CartesianECEFPoint {
        self.transform.transform_point3(*point)
    }

    /// Transforms a point from Local Cartesian NED coordinates to geographic coordinates.
    #[inline]
    pub fn transform_from_ned_point_to_geographic_point(
        &self,
        point: &DVec3
    ) -> GeographicPoint {
        Ellipsoid::WGS84
            .to_geographic_point(
                &self.transform_from_ned_point_to_cartesian_ecef_point(point)
            )
    }

    /// Transforms a point from geocentric cartesian ECEF coordinates to Local Cartesian NED coordinates.
    #[inline]
    pub fn transform_from_cartesian_ecef_point_to_ned_point(
        &self,
        cp: &CartesianECEFPoint,
    ) -> DVec3 {
        self.inverse_transform.transform_point3(*cp)
    }

    /// Transforms a point from geographic coordinates to Local Cartesian NED coordinates.
    #[inline]
    pub fn transform_from_geographic_point_to_ned_point(
        &self,
        gp: &GeographicPoint
    ) -> DVec3 {
        self.transform_from_cartesian_ecef_point_to_ned_point(
            &Ellipsoid::WGS84.to_cartesian_ecef_point(gp),
        )
    }

    /******************************/
    /* ENU <-> GeoPoint transform */
    /******************************/
    /// Transforms a point from Local Cartesian ENU coordinates to geocentric Cartesian ECEF coordinates.
    #[inline]
    pub fn transform_from_enu_point_to_cartesian_ecef_point(
        &self,
        point: &DVec3,
    ) -> CartesianECEFPoint {
        self.transform_from_ned_point_to_cartesian_ecef_point(
            &self.transform_point_from_enu_to_ned(point),
        )
    }

    /// Transforms a point from Local Cartesian ENU coordinates to geographic coordinates.
    #[inline]
    pub fn transform_from_enu_point_to_geographic_point(
        &self,
        point: &DVec3
    ) -> GeographicPoint {
        Ellipsoid::WGS84.to_geographic_point(
            &self.transform_from_enu_point_to_cartesian_ecef_point(point)
        )
    }

    /// Transforms a point from geocentric cartesian ECEF coordinates to Local Cartesian ENU coordinates.
    #[inline]
    pub fn transform_from_cartesian_ecef_point_to_enu_point(
        &self,
        cp: &CartesianECEFPoint,
    ) -> DVec3 {
        self.transform_point_from_ned_to_enu(
            &self.transform_from_cartesian_ecef_point_to_ned_point(cp),
        )
    }

    /// Transforms a point from geographic coordinates to Local Cartesian ENU coordinates.
    #[inline]
    pub fn transform_from_geographic_point_to_enu_point(
        &self,
        gp: &GeographicPoint
    ) -> DVec3 {
        self.transform_point_from_ned_to_enu(
            &self.transform_from_geographic_point_to_ned_point(gp)
        )
    }

    /************************************************/
    /* ENU/NED <-> ECEF vector transform */
    /************************************************/
    /// Transforms a vector from Local Cartesian NED coordinates to geocentric Cartesian ECEF coordinates.
    #[inline]
    pub fn transform_vector_from_ned_to_ecef(&self, vec: &DVec3) -> DVec3 {
        self.transform.transform_vector3(*vec) // Applies only rotation to vector [no translation]
    }

    /// Transforms a vector from geocentric Cartesian ECEF coordinates to Local Cartesian NED coordinates.
    #[inline]
    pub fn transform_vector_from_ecef_to_ned(&self, vec: &DVec3) -> DVec3 {
        self.inverse_transform.transform_vector3(*vec) // Applies only rotation to vector [no translation]
    }

    /// Transforms a vector from Local Cartesian ENU coordinates to geocentric Cartesian ECEF coordinates.
    #[inline]
    pub fn transform_vector_from_enu_to_ecef(&self, vec: &DVec3) -> DVec3 {
        self.transform_vector_from_ned_to_ecef(
            &self.transform_vector_from_enu_to_ned(vec)
        )
    }

    /// Transforms a vector from geocentric Cartesian ECEF coordinates to Local Cartesian ENU coordinates.
    #[inline]
    pub fn transform_vector_from_ecef_to_enu(&self, vec: &DVec3) -> DVec3 {
        self.transform_vector_from_ned_to_enu(
            &self.transform_vector_from_ecef_to_ned(vec)
        )
    }

    /******************************/
    /* ENU <-> NED transform      */
    /******************************/
    /// Transforms a point from Local Cartesian NED coordinates to Local Cartesian ENU coordinates.
    #[inline]
    pub fn transform_point_from_ned_to_enu(&self, point: &DVec3) -> DVec3 {
        DVec3::new(point.y, point.x, -point.z)
    }

     /// Transforms a point from Local Cartesian ENU coordinates to Local Cartesian NED coordinates.
    #[inline]
    pub fn transform_point_from_enu_to_ned(&self, point: &DVec3) -> DVec3 {
        DVec3::new(point.y, point.x, -point.z)
    }
    /// Transforms a vector from Local Cartesian NED coordinates to Local Cartesian ENU coordinates.
    #[inline]
    pub fn transform_vector_from_ned_to_enu(&self, vec: &DVec3) -> DVec3 {
        DVec3::new(vec.y, vec.x, -vec.z)
    }
    /// Transforms a vector from Local Cartesian ENU coordinates to Local Cartesian NED coordinates.
    #[inline]
    pub fn transform_vector_from_enu_to_ned(&self, vec: &DVec3) -> DVec3 {
        DVec3::new(vec.y, vec.x, -vec.z)
    }    

    // #[inline]
    // fn set_ned_to_ecef_isometry(gp: &GeographicPoint, cp: &CartesianECEFPoint) -> RSIsometryMatrix {
    //     let (slon0, clon0) = gp.lon_rad().sin_cos();
    //     let (slat0, clat0) = gp.lat_rad().sin_cos();
    //     RSIsometryMatrix::from_parts(
    //         RSTranslation::from(*cp),
    //         RSRotation::from_matrix_unchecked(                
    //             RSMatrix::new( // NED -> ECEF rotation
    //             //  |    North   |,| East|, |     Down    |     vectors expressed in Cartesian ECEF frame
    //                 -clon0 * slat0, -slon0, -clon0 * clat0,
    //                 -slon0 * slat0,  clon0, -slon0 * clat0,
    //                          clat0,    0.0,         -slat0
    //             ),
    //         ),
    //     )
    // }

    #[inline]
    fn set_ned_to_ecef_transform(gp: &GeographicPoint, cp: &CartesianECEFPoint) -> (DAffine3, DAffine3) {
        let (slon0, clon0) = gp.lon_rad().sin_cos();
        let (slat0, clat0) = gp.lat_rad().sin_cos();
        let transform = DAffine3::from_mat3_translation(
            DMat3 { // NED -> ECEF rotation | vectors expressed in Cartesian ECEF frame
                x_axis: DVec3::new(-clon0 * slat0, -slon0 * slat0, clat0), // North
                y_axis: DVec3::new(-slon0, clon0, 0.0), // East
                z_axis: DVec3::new(-clon0 * clat0, -slon0 * clat0, -slat0) // Down
            },
            *cp
        );
        let inverse_transform = transform.inverse();
        (transform, inverse_transform)
    }
}
