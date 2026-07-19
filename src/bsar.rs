//! BSAR geometry and resolutions functions.

use bevy::math::DVec3;

use crate::{
    constants::TO_Y_UP_F64,
    entities::{AntennaBeamFootprintState, AntennaBeamState},
    scene::{RxCarrierState, TxCarrierState}
};

/// Speed of light in vacuum constant `c` \[m.s<sup>-1</sup>\] from [`CODATA`] database on [`NIST`] website.
///
/// [`CODATA`]: https://codata.org/
/// [`NIST`]: https://pml.nist.gov/cuu/Constants/
pub const SPEED_OF_LIGHT_IN_VACUUM: f64 = 299792458.0; // m/s
/// Boltzmann constant `k` \[J.K<sup>-1</sup>\] from [`CODATA`] database on [`NIST`] website.
///
/// [`CODATA`]: https://codata.org/
/// [`NIST`]: https://pml.nist.gov/cuu/Constants/
pub const BOLTZMANN_CONSTANT: f64 = 1.380649e-23; // J/K
/// The width of squared normalized cardinal sine function at half height.
/// 
/// This constant is twice the positive solution of sinc²(x) = 1/2.
const SINC_WIDTH_AT_HALF_POWER: f64 = 0.885892941378904715150369091935531;
/// The squared value of [`SINC_WIDTH_AT_HALF_POWER`].
const SINC_WIDTH_AT_HALF_POWER_SQUARED: f64 = 0.784806303584967506070224247343716;

/// Returns `num / den` if `den` is strictly positive, `NaN` otherwise.
///
/// All callers pass denominators built from norms or products of non-negative
/// values, so `den <= 0.0` (or `NaN`) only happens for degenerate geometries
/// (e.g. zero carrier velocity). Returning `NaN` matches the invalid-state
/// convention of [`BsarInfos::default`] instead of silently producing `inf`.
#[inline]
fn div_or_nan(num: f64, den: f64) -> f64 {
    if den > 0.0 { num / den } else { f64::NAN }
}

pub struct BsarInfos {
    /// The bistatic range extrema over the footprint in meters.
    pub range_min_m: f64,
    pub range_max_m: f64,
    pub range_center_m: f64,
    /// The Transmitter-Receiver direct range in meters.
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
    /// The (effective) integration time in seconds.
    pub integration_time_s: f64,
    /// The processed Doppler bandwidth in Hz.
    pub processed_doppler_bandwidth_hz: f64,
    /// The PRF bounds in Hz (not computed yet).
    pub prf_min_hz: f64,
    pub prf_max_hz: f64,
    /// The Noise-Equivalent Sigma Zero (linear scale).
    pub nesz: f64,
    /// Ground-projected bistatic bisector vector and its time derivative
    /// (`z = 0`), reused to plot the Generalized Ambiguity Function.
    pub betag: DVec3,
    pub dbetag: DVec3,
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
            betag: DVec3::splat(f64::NAN),
            dbetag: DVec3::splat(f64::NAN),
        }
    }
}

impl BsarInfos {
    pub fn update_from_state(
        &mut self,
        tx_state: &TxCarrierState,
        rx_state: &RxCarrierState,
        tx_antenna_beam_state: &AntennaBeamState,
        rx_antenna_beam_state: &AntennaBeamState,
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
        // NESZ (Noise-Equivalent Sigma Zero) from the bistatic radar equation:
        //
        //        (4π)³.R_tx².R_rx².k.T_rx.10^((L_tx + F_rx - G_tx - G_rx)/10)
        // NESZ = ------------------------------------------------------------
        //                    λ².P_peak.duty_cycle.T_int.A_res
        //
        // with duty_cycle = pulse_duration.PRF and A_res the resolution cell area.
        // Invalid geometries (T_int or A_res NaN) and zero duty cycle yield NaN.
        let lem = SPEED_OF_LIGHT_IN_VACUUM / (tx_state.center_frequency_ghz * 1e9); // wavelength in m
        let duty_cycle = tx_state.pulse_duration_us * 1e-6 * tx_state.prf_hz;
        self.nesz = div_or_nan(
            64.0 * std::f64::consts::PI.powi(3) *
                tx_state.inner.position_m.length_squared() * // = R_tx²
                rx_state.inner.position_m.length_squared() * // = R_rx²
                BOLTZMANN_CONSTANT * rx_state.noise_temperature_k *
                10f64.powf(0.1 * (
                    tx_state.loss_factor_db + rx_state.noise_factor_db -
                    tx_antenna_beam_state.one_way_gain_dbi - rx_antenna_beam_state.one_way_gain_dbi
                )),
            lem * lem * tx_state.peak_power_w * duty_cycle *
                self.integration_time_s * self.resolution_area_m2
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
                self.betag = betag;
                self.dbetag = dbetag;
                let beta_norm = beta.length();
                let dbeta_norm = dbeta.length();
                let betag_norm = betag.length();
                let dbetag_norm = dbetag.length();
                // Integration time
                let lem = SPEED_OF_LIGHT_IN_VACUUM / center_frequency_hz; // wavelength in m
                self.integration_time_s = if squared_pixels {
                    if ground_resolution {
                        bandwidth_hz / center_frequency_hz * div_or_nan(betag_norm, dbetag_norm)
                    } else {
                        bandwidth_hz / center_frequency_hz * div_or_nan(beta_norm, dbeta_norm)
                    }
                } else {
                    integration_time_s
                };
                // Slant ranges
                self.range_center_m = txp_norm + rxp_norm;
                (self.range_min_m,
                    self.range_max_m) = bsar_range_min_max(
                    txp, rxp,
                    tx_footprint,
                    rx_footprint
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
                // Resolution parameters (guarded: degenerate geometries yield NaN, not inf)
                self.slant_range_resolution_m =
                    div_or_nan(SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM, bandwidth_hz * beta_norm);
                self.slant_lateral_resolution_m =
                    div_or_nan(SINC_WIDTH_AT_HALF_POWER * lem, self.integration_time_s * dbeta_norm);
                self.ground_range_resolution_m =
                    div_or_nan(SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM, bandwidth_hz * betag_norm);
                self.ground_lateral_resolution_m =
                    div_or_nan(SINC_WIDTH_AT_HALF_POWER * lem, self.integration_time_s * dbetag_norm);
                self.resolution_area_m2 =
                    div_or_nan(SINC_WIDTH_AT_HALF_POWER_SQUARED * SPEED_OF_LIGHT_IN_VACUUM * lem,
                        bandwidth_hz * self.integration_time_s * betag.cross(dbetag).length());
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
            } else {
                // rxp is a zero vector: all fields are invalid (NaN)
                *self = Self::default();
            }
        } else {
            // txp is a zero vector: all fields are invalid (NaN)
            *self = Self::default();
        }
    }
}

/// Computes the BSAR system min and max ranges in meters
/// from Tx or Rx footprint. The used footprint for calculation
/// is heuristically determined by choosing the one with the
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

/// Normalized cardinal sine `sin(πx)/(πx)`, with `sinc(0) = 1`.
/// Matches BSARConf's `sinc` (used to plot the Generalized Ambiguity Function).
#[inline]
pub fn sinc(x: f64) -> f64 {
    let arg = std::f64::consts::PI * x;
    if x.abs() < 1e-6 { // Series expansion near 0 for double precision
        1.0 - arg * arg / 6.0
    } else {
        arg.sin() / arg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Relative comparison helper.
    fn assert_close(value: f64, expected: f64, rel_tol: f64) {
        assert!(
            (value - expected).abs() <= rel_tol * expected.abs().max(1e-300),
            "value = {value}, expected = {expected}"
        );
    }

    #[test]
    fn sinc_matches_reference_values() {
        // sinc(0) = 1 (via the near-zero series branch)
        assert_close(sinc(0.0), 1.0, 1e-15);
        assert!((sinc(1e-9) - 1.0).abs() < 1e-15);
        // Integer arguments are exact zeros of the normalized cardinal sine
        for n in 1..=5 {
            assert!(sinc(n as f64).abs() < 1e-12, "sinc({n}) should be ~0");
        }
        // Even symmetry
        assert_close(sinc(0.37), sinc(-0.37), 1e-15);
        // Half-power width: sinc²(x) = 1/2 at x = ±SINC_WIDTH_AT_HALF_POWER/2
        let half = SINC_WIDTH_AT_HALF_POWER / 2.0;
        assert_close(sinc(half) * sinc(half), 0.5, 1e-12);
    }

    /// Runs `update()` for a monostatic broadside geometry:
    /// carrier at range R with velocity orthogonal to the line of sight.
    fn monostatic_broadside(velocity: f64, tint: f64, squared_pixels: bool) -> BsarInfos {
        let mut infos = BsarInfos::default();
        let txp = DVec3::new(0.0, 10_000.0, 0.0); // carrier -> target vector, R = 10 km
        let vtx = DVec3::new(velocity, 0.0, 0.0); // broadside: v orthogonal to LOS
        infos.update(
            &txp, &vtx, &txp, &vtx,
            &AntennaBeamFootprintState::default(),
            &AntennaBeamFootprintState::default(),
            10.0e9,  // 10 GHz
            300.0e6, // 300 MHz
            tint,
            squared_pixels,
            true
        );
        infos
    }

    #[test]
    fn monostatic_broadside_sanity() {
        let r = 10_000.0;
        let (fc, bandwidth, tint, v) = (10.0e9, 300.0e6, 1.0, 100.0);
        let lem = SPEED_OF_LIGHT_IN_VACUUM / fc;
        let infos = monostatic_broadside(v, tint, false);
        // Monostatic: zero bistatic angle, ranges are twice the slant range
        assert_close(infos.bistatic_angle_deg, 0.0, 1e-12);
        assert_close(infos.range_center_m, 2.0 * r, 1e-12);
        // Footprint points default to the origin => min = max = 2R
        assert_close(infos.range_min_m, 2.0 * r, 1e-12);
        assert_close(infos.range_max_m, 2.0 * r, 1e-12);
        // |beta| = 2 => monostatic slant range resolution k.c/(2B)
        assert_close(
            infos.slant_range_resolution_m,
            SINC_WIDTH_AT_HALF_POWER * SPEED_OF_LIGHT_IN_VACUUM / (2.0 * bandwidth),
            1e-12
        );
        // |dbeta| = 2v/R => slant lateral resolution k.lem.R/(2v.Tint)
        assert_close(
            infos.slant_lateral_resolution_m,
            SINC_WIDTH_AT_HALF_POWER * lem * r / (2.0 * v * tint),
            1e-12
        );
        // Broadside: v is orthogonal to the LOS => zero Doppler frequency
        assert_close(infos.doppler_frequency_hz, 0.0, 1e-12);
        // Monostatic broadside Doppler rate: -2v^2/(lem.R)
        assert_close(infos.doppler_rate_hzps, -2.0 * v * v / (lem * r), 1e-12);
    }

    #[test]
    fn zero_velocity_yields_nan_not_inf() {
        // Regression test: divisions by |dbeta| = 0 used to produce silent inf
        let infos = monostatic_broadside(0.0, 1.0, false);
        assert!(infos.slant_lateral_resolution_m.is_nan());
        assert!(infos.ground_lateral_resolution_m.is_nan());
        assert!(infos.resolution_area_m2.is_nan());
        // Range resolution does not depend on velocity: still finite
        assert!(infos.slant_range_resolution_m.is_finite());
        // Zero velocity => zero Doppler (semantically correct, not NaN)
        assert_close(infos.doppler_frequency_hz, 0.0, 1e-12);

        // Squared pixels: the auto integration time is undefined too
        let infos = monostatic_broadside(0.0, 1.0, true);
        assert!(infos.integration_time_s.is_nan());
    }

    #[test]
    fn nadir_geometry_yields_nan_ground_range_resolution() {
        // Both carriers at zenith: beta is vertical => ground projection is zero
        let mut infos = BsarInfos::default();
        let txp = DVec3::new(0.0, 0.0, -3000.0); // carrier -> target, straight down
        let vtx = DVec3::new(100.0, 0.0, 0.0);
        infos.update(
            &txp, &vtx, &txp, &vtx,
            &AntennaBeamFootprintState::default(),
            &AntennaBeamFootprintState::default(),
            10.0e9, 300.0e6, 1.0, false, true
        );
        assert!(infos.ground_range_resolution_m.is_nan()); // |betag| = 0
        assert!(infos.slant_range_resolution_m.is_finite());
        assert!(infos.slant_lateral_resolution_m.is_finite()); // |dbeta| > 0
    }

    /// Builds the bistatic reference configuration used by the NESZ tests.
    fn nesz_reference_states() -> (TxCarrierState, RxCarrierState, AntennaBeamState, AntennaBeamState) {
        let mut tx_state = TxCarrierState::default();
        tx_state.inner.position_m = DVec3::new(0.0, -8000.0, 6000.0); // R_tx = 10 km
        tx_state.inner.velocity_vector_mps = DVec3::new(150.0, 0.0, 0.0);
        tx_state.center_frequency_ghz = 9.65;
        tx_state.bandwidth_mhz = 300.0;
        tx_state.pulse_duration_us = 10.0;
        tx_state.prf_hz = 2000.0; // duty cycle = 0.02
        tx_state.peak_power_w = 250.0;
        tx_state.loss_factor_db = 3.0;
        let mut rx_state = RxCarrierState::default();
        rx_state.inner.position_m = DVec3::new(3000.0, 0.0, 4000.0); // R_rx = 5 km
        rx_state.inner.velocity_vector_mps = DVec3::new(0.0, 100.0, 0.0);
        rx_state.noise_temperature_k = 290.0;
        rx_state.noise_factor_db = 5.0;
        rx_state.integration_time_s = 1.0;
        rx_state.squared_pixels = false;
        let tx_beam = AntennaBeamState {
            elevation_beam_width_deg: 20.0,
            azimuth_beam_width_deg: 20.0,
            one_way_gain_dbi: 20.0,
        };
        let rx_beam = AntennaBeamState {
            elevation_beam_width_deg: 16.0,
            azimuth_beam_width_deg: 16.0,
            one_way_gain_dbi: 16.0,
        };
        (tx_state, rx_state, tx_beam, rx_beam)
    }

    #[test]
    fn nesz_reference_value() {
        // Reference values computed independently with the BSARConf (JS
        // predecessor) compute_nesz convention for this exact geometry
        let (tx_state, rx_state, tx_beam, rx_beam) = nesz_reference_states();
        let mut infos = BsarInfos::default();
        infos.update_from_state(
            &tx_state, &rx_state, &tx_beam, &rx_beam,
            &AntennaBeamFootprintState::default(),
            &AntennaBeamFootprintState::default(),
        );
        assert_close(infos.resolution_area_m2, 1.0151823973118719, 1e-12);
        assert_close(infos.nesz, 6.426137576501484e-3, 1e-12); // = -21.92 dB
    }

    #[test]
    fn nesz_is_nan_for_zero_duty_cycle() {
        let (mut tx_state, rx_state, tx_beam, rx_beam) = nesz_reference_states();
        tx_state.pulse_duration_us = 0.0; // UI lower bound: no transmitted power
        let mut infos = BsarInfos::default();
        infos.update_from_state(
            &tx_state, &rx_state, &tx_beam, &rx_beam,
            &AntennaBeamFootprintState::default(),
            &AntennaBeamFootprintState::default(),
        );
        assert!(infos.nesz.is_nan());
        assert!(infos.resolution_area_m2.is_finite()); // Geometry itself is valid
    }

    #[test]
    fn zero_position_invalidates_all_fields() {
        let mut infos = monostatic_broadside(100.0, 1.0, false);
        assert!(infos.range_center_m.is_finite());
        // Degenerate call: carrier at the target position
        infos.update(
            &DVec3::ZERO, &DVec3::X, &DVec3::Y, &DVec3::X,
            &AntennaBeamFootprintState::default(),
            &AntennaBeamFootprintState::default(),
            10.0e9, 300.0e6, 1.0, false, true
        );
        assert!(infos.range_center_m.is_nan());
        assert!(infos.doppler_frequency_hz.is_nan());
        assert!(infos.nesz.is_nan());
    }
}

