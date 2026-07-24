# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.0] - 2026-07-24

### Added

- `cargo xtask dist` packaging command that builds the release binary and writes
  platform installers plus plain archives to `target/dist`: on Windows a
  self-contained `.exe` installer (installs to `%LOCALAPPDATA%\BSARGeom` with
  Start-Menu and Desktop shortcuts) and a `.zip`; on Linux a `makeself`
  self-extracting `.run` installer (installs `bsargeom` to `~/.local/bin` or,
  with `sudo`, `/usr/local/bin`, plus a desktop-menu entry) and a `.tar.gz`; on
  macOS a `BSARGeom.app` bundle in a `.tar.gz`. The release workflow now produces
  these for every target. Modeled on the RustSAR packaging setup.
- Application icon (derived from the bistatic menu glyph on a rounded tile)
  applied to every build: embedded into `bsargeom.exe` so it shows in Explorer
  and the taskbar (via a build script), set on the native window at runtime
  through winit, carried by the macOS `.app` bundle, installed alongside the
  Linux desktop entry, and used as the web page favicon.

### Changed

- Renamed the antenna-orientation sliders on the Transmitter / Receiver panels
  for convention-correct naming: "Heading" is now "Bearing" and "Elevation" is
  now "Depression". The hover tooltips also name the rotation axis of each angle
  — the carrier's heading / elevation / bank as the yaw / pitch / roll axes of
  its NED frame, and the antenna's bearing / depression / bank as the azimuth /
  elevation / pointing axes of its NED frame.

## [1.2.0] - 2026-07-20

### Added

- "Save image" button on the Generalized Ambiguity Function window, exporting a
  standalone figure — the heatmap with its iso-dB contours, metric
  Easting/Northing axes with ticks, a title and a legend — as a PNG. The figure
  is rendered (not upscaled) at print resolution, 1626x1590 px tagged 300 dpi,
  i.e. about 5.4 x 5.3 inches. The desktop build opens a "save as" dialog
  (`egui-file-dialog`) and reports the written path; the web build hands the
  bytes to the browser as a download.
- "?" button next to the GAF window's save button, listing the plot navigation
  gestures (scroll to zoom, drag to pan, right-drag to zoom to a box,
  double-click to reset the view) on hover. The web build additionally warns
  that Ctrl+scroll zooms the browser page rather than the plot.
- Reset button on the TRANSMITTER / RECEIVER SETTINGS title row, restoring every
  setting of that element (carrier, antenna orientation, beamwidth and system)
  to its defaults in one click.
- `BSARGEOM_DEBUG_GAF=1` logs a line whenever the GAF plot's inputs or layout
  change between frames (and stays silent otherwise), to pin down flicker in the
  running application.

### Changed

- The GAF plot zooms on a plain mouse wheel scroll instead of Ctrl+scroll (which
  previously panned it). In a browser Ctrl+scroll is the page-zoom gesture and
  the canvas deliberately does not suppress it, so it used to scale the whole
  page along with the plot; a plain scroll also matches how the 3D scene zooms.
- The desktop "save as" dialog is centered on the window.

### Fixed

- The GAF plot could flicker: egui_plot derives its axis thickness from the
  previous frame, and with `data_aspect` that fed back into the bounds
  (bounds -> y tick labels -> axis width -> plot width -> bounds). For some
  geometries the loop had no fixed point and the y-bounds alternated between two
  values every frame. The y-axis width is now pinned, which breaks the loop.

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

[unreleased]: https://github.com/oboisot/BSARGeom/compare/1.3.0...HEAD
[1.3.0]: https://github.com/oboisot/BSARGeom/compare/1.2.0...1.3.0
[1.2.0]: https://github.com/oboisot/BSARGeom/compare/1.1.0...1.2.0
[1.1.0]: https://github.com/oboisot/BSARGeom/compare/1.0.0...1.1.0
[1.0.0]: https://github.com/oboisot/BSARGeom/releases/tag/1.0.0
