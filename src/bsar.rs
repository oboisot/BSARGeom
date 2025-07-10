//! BSAR geometry and resolutions functions.

use bevy::math::DVec3;

use crate::{
    constants::TO_Y_UP_F64,
    entities::AntennaBeamFootprintState,
    scene::{RxCarrierState, TxCarrierState}
};

/// Speed of light in vacuum constant `c` \[m.s<sup>-1</sup>\] from [`CODATA`] database on [`NIST`] website.
///
/// [`CODATA`]: https://codata.org/
/// [`NIST`]: https://pml.nist.gov/cuu/Constants/
pub const SPEED_OF_LIGHT_IN_VACUUM: f64 = 299792458.0; // m/s
/// The width of squared normalized cardinal sine function at half height.
/// 
/// This constant is twice the positive solution of sinc²(x) = 1/2.
const SINC_WIDTH_AT_HALF_POWER: f64 = 0.885892941378904715150369091935531;
/// The squared value of [`SINC_WIDTH_AT_HALF_POWER`].
const SINC_WIDTH_AT_HALF_POWER_SQUARED: f64 = 0.784806303584967506070224247343716;

pub struct BsarInfos {
    ///
    pub range_min_m: f64,
    pub range_max_m: f64,
    pub range_center_m: f64,
    ///
    pub direct_range_m: f64,
    /// The bistatic angle in degrees.
    pub bistatic_angle_deg: f64,
    /// Resolution parameters.
    pub slant_range_resolution_m: f64,
    pub slant_lateral_resolution_m: f64,
    pub ground_range_resolution_m: f64,
    pub ground_lateral_resolution_m: f64,
    pub resolution_area_m2: f64,
    /// The Doppler frequency in Hz.
    pub doppler_frequency_hz: f64,
    /// The Doppler rate in Hz/s.
    pub doppler_rate_hzps: f64,
    /// 
    pub integration_time_s: f64,
    ///
    pub processed_doppler_bandwidth_hz: f64,
    ///
    pub prf_min_hz: f64,
    pub prf_max_hz: f64,
    ///
    pub nesz: f64,
}

impl Default for BsarInfos {
    fn default() -> Self {
        Self {
            range_min_m: f64::NAN,
            range_max_m: f64::NAN,
            range_center_m: f64::NAN,
            direct_range_m: f64::NAN,
            bistatic_angle_deg: f64::NAN,
            slant_range_resolution_m: f64::NAN,
            slant_lateral_resolution_m: f64::NAN,
            ground_range_resolution_m: f64::NAN,
            ground_lateral_resolution_m: f64::NAN,
            resolution_area_m2: f64::NAN,
            doppler_frequency_hz: f64::NAN,
            doppler_rate_hzps: f64::NAN,
            integration_time_s: f64::NAN,
            processed_doppler_bandwidth_hz: f64::NAN,
            prf_min_hz: f64::NAN,
            prf_max_hz: f64::NAN,
            nesz: f64::NAN,
        }
    }
}

impl BsarInfos {
    pub fn update_from_state(
        &mut self,
        tx_state: &TxCarrierState,
        rx_state: &RxCarrierState,
        tx_footprint: &AntennaBeamFootprintState,
        rx_footprint: &AntennaBeamFootprintState,
    ) {
        self.update(
            &(-tx_state.inner.position_m),
            &tx_state.inner.velocity_vector_mps,
            &(-rx_state.inner.position_m),
            &rx_state.inner.velocity_vector_mps,
            tx_footprint,
            rx_footprint,
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
        tx_footprint: &AntennaBeamFootprintState,
        rx_footprint: &AntennaBeamFootprintState,
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
                self.integration_time_s = if squared_pixels {
                    if ground_resolution {
                        bandwidth_hz / center_frequency_hz * betag_norm / dbetag_norm
                    } else {
                        bandwidth_hz / center_frequency_hz * beta_norm / dbeta_norm
                    }
                } else {
                    integration_time_s
                };
                // Slant ranges
                self.range_center_m = txp_norm + rxp_norm;
                (self.range_min_m,
                    self.range_max_m) = bsar_range_min_max(
                    txp, rxp,
                    &tx_footprint,
                    &rx_footprint
                );
                // Direct range
                self.direct_range_m = (txp - rxp).length();
                // Bistatic angle
                let arg = 0.5 * beta_norm;
                self.bistatic_angle_deg = if arg > 1.0 { // Check range outside 1            
                    180.0
                } else {
                    (2.0 * arg.acos()).to_degrees()
                };
                // Resolution parameters
                self.slant_range_resolution_m = 
                    SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM / (bandwidth_hz * beta_norm);
                self.slant_lateral_resolution_m = 
                    SINC_WIDTH_AT_HALF_POWER * lem / (self.integration_time_s * dbeta_norm);
                self.ground_range_resolution_m = 
                    SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM / (bandwidth_hz * betag_norm);
                self.ground_lateral_resolution_m =
                    SINC_WIDTH_AT_HALF_POWER * lem / (self.integration_time_s * dbetag_norm);
                self.resolution_area_m2 = 
                    SINC_WIDTH_AT_HALF_POWER_SQUARED * SPEED_OF_LIGHT_IN_VACUUM * lem /
                        (bandwidth_hz * self.integration_time_s * betag.cross(dbetag).length());
                // Doppler frequency
                self.doppler_frequency_hz = (vtx.dot(utxp) + vrx.dot(urxp)) / lem;
                // Doppler rate
                let singamma_tx = vtx.normalize_or_zero().dot(utxp); // sin(gamma_tx) = vtx.normalize().dot(utxp)
                let singamma_rx = vrx.normalize_or_zero().dot(urxp);
                self.doppler_rate_hzps = -(
                    vtx.length_squared() * (1.0 - singamma_tx * singamma_tx) / txp_norm + // cos²(x) = 1 - sin²(x)
                    vrx.length_squared() * (1.0 - singamma_rx * singamma_rx) / rxp_norm
                ) / lem;
                self.processed_doppler_bandwidth_hz = self.integration_time_s * self.doppler_rate_hzps.abs();
                // TODO NESZ
            }
        }
    }
}

/// Commputes the BSAR system min and max ranges in meters
/// from Tx or Rx footprint. The used footprint for calculation
/// is heuristically determined by chooseing the one with the
/// smallest `ground_range_swath_m`.
pub fn bsar_range_min_max(
    txp: &DVec3,
    rxp: &DVec3,
    tx_footprint: &AntennaBeamFootprintState,
    rx_footprint: &AntennaBeamFootprintState,
) -> (f64, f64) {
    // Transform to Y-up coordinate system for computation with antenna beam footprint
    let txp_yup = TO_Y_UP_F64 * *txp;
    let rxp_yup = TO_Y_UP_F64 * *rxp;    
    let mut min_range = f64::MAX;
    let mut max_range = 0.0;
    // Temporary variables
    let mut range: f64;
    if rx_footprint.ground_range_swath_m <= tx_footprint.ground_range_swath_m {
        // Use Rx footprint
        for p in rx_footprint.points.iter() {
            // Compute range to Tx footprint
            range = (txp_yup + p).length() + (rxp_yup + p).length();
            // Min range
            if range < min_range {
                min_range = range;
            }
            // Max range
            if range > max_range {
                max_range = range;
            }
        }
    } else {
        // Use Tx footprint
        for p in tx_footprint.points.iter() {
            // Compute range to Rx footprint
            range = (txp_yup + p).length() + (rxp_yup + p).length();
            // Min range
            if range < min_range {
                min_range = range;
            }
            // Max range
            if range > max_range {
                max_range = range;
            }
        }
    }

    (min_range, max_range)
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
) -> f64 {
    let mut txp_norm = txp.length_squared();
    if txp_norm > 0.0 {        
        let mut rxp_norm = rxp.length_squared();
        if rxp_norm > 0.0 {
            txp_norm = txp_norm.sqrt();
            rxp_norm = rxp_norm.sqrt();
            let utxp = txp / txp_norm; // Normalized txp            
            let urxp = rxp / rxp_norm; // Normalized rxp
            (vtx.dot(utxp) + vrx.dot(urxp)) / lem
        } else { // rxp is a zero vector
            f64::NAN
        }
    } else { // txp is a zero vector
        f64::NAN
    }
}
