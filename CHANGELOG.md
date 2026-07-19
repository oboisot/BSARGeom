# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-07-19

### Added

- Camera focus toggle in the menu (Ground / Tx / Rx) that follows the selected
  carrier as its parameters move it, plus a "reset view" button restoring the
  initial orientation and zoom. Focus defaults to "free" (unconstrained orbit /
  pan / zoom); clicking the active focus button releases the camera back to it.
- Per-section "reset to defaults" buttons on the Transmitter and Receiver
  panels (Carrier, Antenna orientation, Beamwidth, System).
- Value labels on the ground iso-Range / iso-Doppler texture, with the unit
  chosen per family for readability (m/km, Hz/kHz) and decluttering. Labels are
  rotated to follow their contour and carry a ground-colored halo that
  interrupts the line underneath, like plotly's inline contour labels.
- Normalized Generalized Ambiguity Function (GAF) plot, opened from a menu
  button, showing the point-target response with its −3, −6, −10, −13 and
  −20 dB resolution contours (cross-validated bit-exact against the BSARConf
  reference), framed by `egui_plot` with metric Easting/Northing axes, grid,
  zoom and pan. The iso-dB contours are drawn as vector plot lines over the
  intensity heatmap, so they stay crisp at any zoom and can be toggled from the
  plot legend. The window is freely draggable and resizable, the plot following
  the window size.

### Changed

- Linux builds now enable both the `x11` and `wayland` Bevy backends: winit
  selects Wayland when a compositor is available and falls back to X11, so the
  window is natively scaled on Wayland sessions instead of going through
  XWayland.
- Iso-Doppler contours are drawn thinner than the iso-Range ones so the two
  families stay distinguishable where they cross.
- The iso-range/iso-Doppler contours are rasterized by a small anti-aliased
  polyline rasterizer (`src/raster.rs`) instead of `plotters`, which drew them
  with an integer-coordinate Bresenham algorithm and no anti-aliasing. The lines
  are now smooth and sub-pixel accurate, and the `plotters` dependency is gone.
- Contours for all levels are extracted in a single pass over the grid
  (`contour::march_levels`) rather than one full grid scan per level. Together
  with the rasterizer change the ground texture rebuild went from ~82 ms to
  ~51 ms per update, despite now being anti-aliased.
- Contour/label text is now rasterized from an embedded DejaVu Sans font via
  `ab_glyph`, replacing the `plotters` default font backend (which cannot draw
  text on the web/wasm target); the native fontconfig build dependency is gone.

### Fixed

- Contour value labels were placed on the vertically mirrored contour: the
  chart's reversed y-range already puts grid row 0 at the top, so the label
  rasterizer must not flip it again. Iso-Doppler labels consequently showed the
  sign of the opposite contour (positive values on dashed lines and vice versa).


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

[unreleased]: https://github.com/oboisot/BSARGeom/compare/1.1.0...HEAD
[1.1.0]: https://github.com/oboisot/BSARGeom/compare/1.0.0...1.1.0
[1.0.0]: https://github.com/oboisot/BSARGeom/releases/tag/1.0.0
