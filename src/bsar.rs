//! BSAR geometry and resolutions functions.

use bevy::math::DVec3;

use crate::scene::{TxCarrierState, RxCarrierState};

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

pub struct BsarInfos {
    ///
    pub range_min_m: Option<f64>,
    pub range_max_m: Option<f64>,
    pub range_center_m: Option<f64>,
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
    pub doppler_frequency_hz: Option<f64>,
    /// The Doppler rate in Hz/s.
    pub doppler_rate_hzps: Option<f64>,
    /// 
    pub integration_time_s: Option<f64>,
    ///
    pub processed_doppler_bandwidth_hz: Option<f64>,
    ///
    pub prf_min_hz: Option<f64>,
    pub prf_max_hz: Option<f64>,
    ///
    pub nesz: Option<f64>,
}

impl Default for BsarInfos {
    fn default() -> Self {
        Self {
            range_min_m: None,
            range_max_m: None,
            range_center_m: None,
            direct_range_m: None,
            bistatic_angle_deg: None,
            slant_range_resolution_m: None,
            slant_lateral_resolution_m: None,
            ground_range_resolution_m: None,
            ground_lateral_resolution_m: None,
            resolution_area_m2: None,
            doppler_frequency_hz: None,
            doppler_rate_hzps: None,
            integration_time_s: None,
            processed_doppler_bandwidth_hz: None,
            prf_min_hz: None,
            prf_max_hz: None,
            nesz: None,
        }
    }
}

impl BsarInfos {
    pub fn update_from_state(
        &mut self,
        tx_state: &TxCarrierState,
        rx_state: &RxCarrierState,
    ) {
        self.update(
            &(-tx_state.inner.position_m),
            &tx_state.inner.velocity_vector_mps,
            &(-rx_state.inner.position_m),
            &rx_state.inner.velocity_vector_mps,
            tx_state.center_frequency_ghz * 1e9, // Convert GHz to Hz
            tx_state.bandwidth_mhz * 1e6, // Convert MHz to Hz
            rx_state.integration_time_s,
            rx_state.squared_pixels, // If `true` the integration time is computed to have squared pixels ignoring input integration_time_s
            rx_state.pixel_resolution.is_ground()
        );
    }

    pub fn update(
        &mut self,
        txp: &DVec3,
        vtx: &DVec3,
        rxp: &DVec3,
        vrx: &DVec3,
        center_frequency_hz: f64,
        bandwidth_hz: f64,
        integration_time_s: f64,
        squared_pixels: bool, // If `true` the integration time is computed to have squared pixels ignoring input integration_time_s
        ground_resolution: bool, // If `true` the integration time is computed for ground resolution, otherwise for slant resolution
    ) {
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
                let integration_time_s = if squared_pixels {
                    if ground_resolution {
                        bandwidth_hz / center_frequency_hz * betag_norm / dbetag_norm
                    } else {
                        bandwidth_hz / center_frequency_hz * beta_norm / dbeta_norm
                    }
                } else {
                    integration_time_s
                };
                // Slant ranges
                self.range_min_m = None; // TODO
                self.range_max_m = None; // TODO
                self.range_center_m = Some(txp_norm + rxp_norm);
                // Direct range
                self.direct_range_m = Some((txp - rxp).length());
                // Bistatic angle
                let arg = 0.5 * beta_norm;
                self.bistatic_angle_deg = if arg > 1.0 { // Check range outside 1            
                    Some(180.0)
                } else {
                    Some((2.0 * arg.acos()).to_degrees())
                };
                // Resolution parameters
                self.slant_range_resolution_m = Some(
                    SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM / (bandwidth_hz * beta_norm)
                );
                self.slant_lateral_resolution_m = Some(
                    SINC_WIDTH_AT_HALF_POWER * lem / (integration_time_s * dbeta_norm)
                );
                self.ground_range_resolution_m = Some(
                    SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM / (bandwidth_hz * betag_norm)
                );
                self.ground_lateral_resolution_m = Some(
                    SINC_WIDTH_AT_HALF_POWER * lem / (integration_time_s * dbetag_norm)
                );
                self.resolution_area_m2 = Some(
                    SINC_WIDTH_AT_HALF_POWER_SQUARED * SPEED_OF_LIGHT_IN_VACUUM * lem /
                        (bandwidth_hz * integration_time_s * betag.cross(dbetag).length())
                );
                // Doppler frequency
                self.doppler_frequency_hz = Some(
                    (vtx.dot(utxp) + vrx.dot(urxp)) / lem
                );
                // Doppler rate
                let singamma_tx = vtx.normalize_or_zero().dot(utxp); // sin(gamma_tx) = vtx.normalize().dot(utxp)
                let singamma_rx = vrx.normalize_or_zero().dot(urxp);
                let doppler_rate_hzps = -(
                    vtx.length_squared() * (1.0 - singamma_tx * singamma_tx) / txp_norm + // cos²(x) = 1 - sin²(x)
                    vrx.length_squared() * (1.0 - singamma_rx * singamma_rx) / rxp_norm
                ) / lem;
                self.processed_doppler_bandwidth_hz = Some(
                    integration_time_s * doppler_rate_hzps.abs()
                );
                self.doppler_rate_hzps = Some(doppler_rate_hzps);
                self.integration_time_s = Some(integration_time_s);

                // Self {
                //     range_min_m,
                //     range_max_m,
                //     range_center_m,
                //     direct_range_m,
                //     bistatic_angle_deg,
                //     slant_range_resolution_m,
                //     slant_lateral_resolution_m,
                //     ground_range_resolution_m,
                //     ground_lateral_resolution_m,
                //     resolution_area_m2,
                //     doppler_frequency_hz,
                //     doppler_rate_hzps: Some(doppler_rate_hzps),
                //     integration_time_s: Some(integration_time_s),
                //     processed_doppler_bandwidth_hz,
                // }
        //     } else { // rxp is a zero vector
        //         Self::default()
        //     }
        // } else { // txp is a zero vector
        //     Self::default()
        // }
            }
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
