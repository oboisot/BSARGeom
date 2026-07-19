# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-07-19

First stable release.

### Added

- Interactive 3D visualization of bistatic SAR acquisition geometries, built on
  Bevy 0.19 with the WebGPU backend and an orbit camera.
- Transmitter and Receiver side panels covering the carrier state (height,
  velocity, heading/elevation/bank), the antenna orientation and half-power
  beamwidths, and the system parameters (center frequency, bandwidth, pulse
  duration, PRF, peak power, loss factor, antenna gains, noise
  temperature/factor, integration time with an optional squared-pixels mode).
- Monostatic mode mirroring the Transmitter configuration onto the Receiver.
- Live scene entities: antenna beam cones, ground footprints with their
  elevation/azimuth traces, velocity vectors, the iso-range ellipsoid and the
  iso-range/iso-Doppler ground plane drawn from marching-squares contours.
- BSAR information windows: slant/ground ranges, bistatic angle, slant and
  ground range/lateral resolutions, resolution cell area, Doppler frequency and
  rate, integration time, illumination time, and NESZ following the
  [BSARConf](https://github.com/oboisot/BSARConf) conventions (cross-validated
  bit-exact against the reference implementation).
- Geodesy module: WGS84 ellipsoid, Vermeille geodetic inverse, DMS/DD
  conversions, local ENU/NED frames.
- Web deployment through the Bevy CLI: WebGPU availability check on the landing
  page (with the list of known working browsers when unavailable) and a Wasm
  download progress bar on the loading screen.
- Regression test suite covering the BSAR formulas, footprint geometry, contour
  extraction, geodesy conversions, and the monostatic update pipeline.

[unreleased]: https://github.com/oboisot/BSARGeom/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/oboisot/BSARGeom/releases/tag/v1.0.0
