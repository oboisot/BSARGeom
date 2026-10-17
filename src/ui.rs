mod app;
pub use app::{AppPlugin, SidePanelRects};

mod carrier_ui;
pub use carrier_ui::{carrier_ui, heading_with_reset};

mod gaf;
pub use gaf::{show_gaf_window, GafState};
#[cfg(test)]
pub(crate) use gaf::gaf_key;

mod menu;
pub use menu::{CameraFocus, MenuPlugin, MenuWidget};

mod infos;
pub use infos::{bsar_infos_ui, carrier_infos_ui};

mod tx_panel;
pub use tx_panel::{TxPanelPlugin, TxPanelWidget};

mod rx_panel;
pub use rx_panel::{RxPanelPlugin, RxPanelWidget};

#[cfg(test)]
mod tests {
    use bevy::asset::AssetPlugin;
    use bevy::prelude::*;

    use crate::entities::IsoRangeDopplerPlaneState;
    use crate::scene::{
        spawn_scene, BsarInfosState,
        RxAntennaBeamFootprintState, RxAntennaBeamState, RxAntennaState, RxCarrierState,
        TxAntennaBeamFootprintState, TxAntennaBeamState, TxAntennaState, TxCarrierState,
    };
    use super::{MenuWidget, RxPanelPlugin, RxPanelWidget, TxPanelPlugin, TxPanelWidget};

    /// Headless App running the real spawned scene graph and the real panel
    /// update systems (update_rx ordered before update_tx), without rendering.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.init_asset::<Image>();
        // Scene state resources, as ScenePlugin initializes them (without its
        // camera/world rendering plugins)
        app.init_resource::<TxCarrierState>();
        app.init_resource::<TxAntennaState>();
        app.init_resource::<TxAntennaBeamState>();
        app.init_resource::<TxAntennaBeamFootprintState>();
        app.init_resource::<RxCarrierState>();
        app.init_resource::<RxAntennaState>();
        app.init_resource::<RxAntennaBeamState>();
        app.init_resource::<RxAntennaBeamFootprintState>();
        app.init_resource::<BsarInfosState>();
        app.init_resource::<IsoRangeDopplerPlaneState>();
        app.init_resource::<MenuWidget>();
        app.add_plugins((TxPanelPlugin, RxPanelPlugin));
        app.add_systems(Startup, spawn_scene);
        app
    }

    /// Regression test: in monostatic mode, dragging the Tx velocity to zero
    /// must invalidate the BSAR infos (NaN) in the same frame. The Rx state
    /// mirrored during the egui pass still carries the stale (non-zero)
    /// derived velocity vector, which update_rx refreshes — so update_tx,
    /// which computes the infos from both carriers, must run after it.
    #[test]
    fn monostatic_zero_velocity_invalidates_infos_same_frame() {
        let mut app = test_app();
        {
            let mut menu = app.world_mut().resource_mut::<MenuWidget>();
            menu.is_monostatic = true;
            menu.was_monostatic = true;
        }
        app.update(); // Startup: spawns the scene and computes the initial infos

        // Baseline sanity: the default scene produces finite infos
        assert!(app.world().resource::<BsarInfosState>().inner.nesz.is_finite());

        // Simulate the egui pass for "Tx velocity dragged to 0" exactly as
        // TxPanelWidget::ui performs it in monostatic mode: scalar updated,
        // Rx mirrored from Tx (stale derived velocity vector included),
        // dirty flags mirrored.
        {
            let world = app.world_mut();
            let mut tx_carrier_state = world.resource_mut::<TxCarrierState>();
            tx_carrier_state.inner.velocity_mps = 0.0;
            let tx_inner = tx_carrier_state.inner.clone();
            // Premise of the regression: the mirrored derived vector is stale
            assert!(tx_inner.velocity_vector_mps.length() > 0.0);
            world.resource_mut::<RxCarrierState>().inner = tx_inner;
            world.resource_mut::<TxPanelWidget>().velocity_vector_needs_update = true;
            world.resource_mut::<RxPanelWidget>().velocity_vector_needs_update = true;
        }
        app.update();

        let infos = &app.world().resource::<BsarInfosState>().inner;
        assert!(
            infos.nesz.is_nan(),
            "nesz = {} — infos were computed from the stale Rx velocity vector",
            infos.nesz
        );
        // Flags must have been consumed by the update systems
        assert!(!app.world().resource::<TxPanelWidget>().velocity_vector_needs_update);
        assert!(!app.world().resource::<RxPanelWidget>().velocity_vector_needs_update);
    }

    /// Diagnostic: in monostatic mode the GAF inputs must be stable across
    /// frames. An oscillating key would rebuild the texture with different
    /// content every frame, i.e. visible flicker.
    #[test]
    fn gaf_key_is_stable_across_frames_in_monostatic() {
        let mut app = test_app();
        {
            let mut menu = app.world_mut().resource_mut::<MenuWidget>();
            menu.is_monostatic = true;
            menu.was_monostatic = true;
        }
        app.update(); // startup
        // Mirror Tx onto Rx as the egui pass does in monostatic mode
        {
            let world = app.world_mut();
            let tx_inner = world.resource::<TxCarrierState>().inner.clone();
            let tx_ant = world.resource::<TxAntennaState>().inner.clone();
            let tx_beam = world.resource::<TxAntennaBeamState>().inner.clone();
            world.resource_mut::<RxCarrierState>().inner = tx_inner;
            world.resource_mut::<RxAntennaState>().inner = tx_ant;
            world.resource_mut::<RxAntennaBeamState>().inner = tx_beam;
            world.resource_mut::<RxPanelWidget>().transform_needs_update = true;
            world.resource_mut::<RxPanelWidget>().velocity_vector_needs_update = true;
            world.resource_mut::<RxPanelWidget>().system_needs_update = true;
        }
        app.update();

        let key_of = |app: &App| {
            let infos = &app.world().resource::<BsarInfosState>().inner;
            let tx = app.world().resource::<TxCarrierState>();
            crate::ui::gaf_key(infos, tx.bandwidth_mhz * 1e6, tx.center_frequency_ghz * 1e9)
        };
        let first = key_of(&app);
        assert!(first.is_some(), "monostatic default scene must yield a GAF");
        for frame in 0..8 {
            app.update();
            let now = key_of(&app);
            assert_eq!(
                now, first,
                "GAF key changed on frame {frame}: {now:?} != {first:?}"
            );
        }
    }

}
