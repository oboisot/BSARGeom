//! BSAR geometry and resolutions functions.

use bevy::math::DVec3;

/// Speed of light in vacuum constant `c` \[m.s<sup>-1</sup>\] from [`CODATA`] database on [`NIST`] website.
///
/// [`CODATA`]: https://codata.org/
/// [`NIST`]: https://pml.nist.gov/cuu/Constants/
const SPEED_OF_LIGHT_IN_VACUUM: f64 = 299792458.0; // m/s
/// The width of squared normalized cardinal sine function at half height.
/// 
/// This constant is twice the positive solution of sinc²(x) = 1/2.
const SINC_WIDTH_AT_HALF_POWER: f64 = 0.885892941378904715150369091935531;
/// The squared value of [`SINC_WIDTH_AT_HALF_POWER`].
const SINC_WIDTH_AT_HALF_POWER_SQUARED: f64 = 0.784806303584967506070224247343716;

pub struct BSARinfos {
    ///
    pub slant_range_min_m: Option<f64>,
    pub slant_range_max_m: Option<f64>,
    pub slant_range_center_m: Option<f64>,
    ///
    pub direct_range_m: Option<f64>,
    /// The bistatic angle in degrees.
    pub bistatic_angle_deg: Option<f64>,
    /// Resolution parameters.
    pub slant_range_resolution_m: Option<f64>,
    pub slant_lateral_resolution_m: Option<f64>,
    pub ground_range_resolution_m: Option<f64>,
    pub ground_lateral_resolution_m: Option<f64>,
    pub resolution_area_m2: Option<f64>,
    /// The Doppler frequency in Hz.
    pub doppler_frequency: Option<f64>,
    /// The Doppler rate in Hz/s.
    pub doppler_rate: Option<f64>,
    /// 
    pub integration_time_s: Option<f64>,
    ///
    pub processed_doppler_bandwidth_hz: Option<f64>,
}


impl BSARinfos {
    pub fn from_config(
        txp: &DVec3,
        vtx: &DVec3,
        rxp: &DVec3,
        vrx: &DVec3,
        center_frequency_hz: f64,
        bandwidth_hz: f64,
        integration_time_s: Option<f64>, // If None, the integration time is computed to have squared pixels
        ground_resolution: bool, // If `true` and integration_time_s is `None`, the ground resolution is computed, otherwise the slant resolution is computed
    ) -> Self {
        let mut txp_norm = txp.length_squared();
        if txp_norm > 0.0 {        
            let mut rxp_norm = rxp.length_squared();
            if rxp_norm > 0.0 {
                txp_norm = txp_norm.sqrt();
                rxp_norm = rxp_norm.sqrt();
                let utxp = txp / txp_norm; // Normalized txp            
                let urxp = rxp / rxp_norm; // Normalized rxp
                // Bisector vector and its first temporal derivative
                let beta = utxp + urxp;
                let dbeta = -((vtx - vtx.dot(utxp) * utxp) / txp_norm +
                                (vrx - vrx.dot(urxp) * urxp) / rxp_norm);
                let betag = DVec3::new(beta.x, beta.y, 0.0); // Projected bisector vector to ground plane
                let dbetag = DVec3::new(dbeta.x, dbeta.y, 0.0); // Projected bisector vector to ground plane
                let beta_norm = beta.length();
                let dbeta_norm = dbeta.length();
                let betag_norm = betag.length();
                let dbetag_norm = dbetag.length();
                // Integration time
                let lem = SPEED_OF_LIGHT_IN_VACUUM / center_frequency_hz; // wavelength in m
                let integration_time_s = if let Some(integration_time_s) = integration_time_s {
                    integration_time_s
                } else {
                    if ground_resolution {
                        bandwidth_hz / center_frequency_hz * betag_norm / dbetag_norm
                    } else {
                        bandwidth_hz / center_frequency_hz * beta_norm / dbeta_norm
                    }
                };
                // Slant ranges
                let slant_range_min_m = None; // TODO
                let slant_range_max_m = None; // TODO
                let slant_range_center_m = Some(txp_norm + rxp_norm);
                // Direct range
                let direct_range_m = Some((txp - rxp).length());
                // Bistatic angle
                let arg = 0.5 * beta_norm;
                let bistatic_angle_deg = if arg > 1.0 { // Check range outside 1            
                    Some(180.0)
                } else {
                    Some((2.0 * arg.acos()).to_degrees())
                };
                // Resolution parameters
                let slant_range_resolution_m = Some(
                    SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM / (bandwidth_hz * beta_norm)
                );
                let slant_lateral_resolution_m = Some(
                    SINC_WIDTH_AT_HALF_POWER * lem / (integration_time_s * dbeta_norm)
                );
                let ground_range_resolution_m = Some(
                    SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM / (bandwidth_hz * betag_norm)
                );
                let ground_lateral_resolution_m = Some(
                    SINC_WIDTH_AT_HALF_POWER * lem / (integration_time_s * dbetag_norm)
                );
                let resolution_area_m2 = Some(
                    SINC_WIDTH_AT_HALF_POWER_SQUARED * SPEED_OF_LIGHT_IN_VACUUM * lem /
                        (bandwidth_hz * integration_time_s * betag.cross(dbetag).length())
                );
                // Doppler frequency
                let doppler_frequency = Some(
                    (vtx.dot(utxp) + vrx.dot(urxp)) / lem
                );
                // Doppler rate
                let singamma_tx = vtx.normalize_or_zero().dot(utxp); // sin(gamma_tx) = vtx.normalize().dot(utxp)
                let singamma_rx = vrx.normalize_or_zero().dot(urxp);
                let doppler_rate = -(
                    vtx.length_squared() * (1.0 - singamma_tx * singamma_tx) / txp_norm + // cos²(x) = 1 - sin²(x)
                    vrx.length_squared() * (1.0 - singamma_rx * singamma_rx) / rxp_norm
                ) / lem;
                let processed_doppler_bandwidth_hz = Some(
                    integration_time_s * doppler_rate.abs()
                );

                Self {
                    slant_range_min_m,
                    slant_range_max_m,
                    slant_range_center_m,
                    direct_range_m,
                    bistatic_angle_deg,
                    slant_range_resolution_m,
                    slant_lateral_resolution_m,
                    ground_range_resolution_m,
                    ground_lateral_resolution_m,
                    resolution_area_m2,
                    doppler_frequency,
                    doppler_rate: Some(doppler_rate),
                    integration_time_s: Some(integration_time_s),
                    processed_doppler_bandwidth_hz,
                }
            } else { // rxp is a zero vector
                Self::default_none()
            }
        } else { // txp is a zero vector
            Self::default_none()
        }
    }

    fn default_none() -> Self {
        Self {
            slant_range_min_m: None,
            slant_range_max_m: None,
            slant_range_center_m: None,
            direct_range_m: None,
            bistatic_angle_deg: None,
            slant_range_resolution_m: None,
            slant_lateral_resolution_m: None,
            ground_range_resolution_m: None,
            ground_lateral_resolution_m: None,
            resolution_area_m2: None,
            doppler_frequency: None,
            doppler_rate: None,
            integration_time_s: None,
            processed_doppler_bandwidth_hz: None,
        }
    }
}

/// Returns the bistatic angle formed by triangle Transmitter - ground point - Receiver in radians.
///
/// * `txp` is the Transmitter -> ground point vector in m, i.e., `TxP = OP - OTx` with `OP` the targeted ground point
/// * `rxp` is the Receiver -> ground point vector in m, i.e., `TxP = OP - OTx` with `OP` the targeted ground point
/// ```
#[inline(always)]
pub fn bistatic_angle_sg(
    txp: &DVec3,
    rxp: &DVec3
) -> f64 {
    let txp_norm = txp.length_squared();
    let rxp_norm = rxp.length_squared();
    if txp_norm > 0.0 && rxp_norm > 0.0 {
        let arg = 0.5 * (
            txp / txp_norm.sqrt() +
            rxp / rxp_norm.sqrt()
        ).length(); // = 0.5 * beta.length()
        if arg > 1.0 { // Check range outside 1            
            std::f64::consts::PI
        } else {
            2.0 * arg.acos()
        }
    } else { // There is no triangle
        0.0
    }
}

/// Returns the bistatic range from Transmitter -> ground point -> Receiver in m.
///
/// * `txp` is the Transmitter -> ground point vector in m, i.e., `TxP = OP - OTx` with `OP` the targeted ground point
/// * `rxp` is the Receiver -> ground point vector in m, i.e., `TxP = OP - OTx` with `OP` the targeted ground point
#[inline(always)]
pub fn bistatic_range_sg(
    txp: &DVec3,
    rxp: &DVec3
) -> f64 {
    txp.length() + rxp.length()
}

/// Returns the approximated Doppler frequency of the BSAR system relative to
/// ground point of interest in Hz.
#[inline(always)]
pub fn doppler_frequency_sg(
    lem: f64,
    txp: &DVec3,
    vtx: &DVec3,
    rxp: &DVec3,
    vrx: &DVec3,
) -> Option<f64> {
    let mut txp_norm = txp.length_squared();
    if txp_norm > 0.0 {        
        let mut rxp_norm = rxp.length_squared();
        if rxp_norm > 0.0 {
            txp_norm = txp_norm.sqrt();
            rxp_norm = rxp_norm.sqrt();
            let utxp = txp / txp_norm; // Normalized txp            
            let urxp = rxp / rxp_norm; // Normalized rxp
            Some((vtx.dot(utxp) + vrx.dot(urxp)) / lem)
        } else { // rxp is a zero vector
            None
        }
    } else { // txp is a zero vector
        None
    }
}
