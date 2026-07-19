mod app;
pub use app::{AppPlugin, SidePanelRects};

mod carrier_ui;
pub use carrier_ui::{carrier_ui, heading_with_reset};

mod gaf;
pub use gaf::{show_gaf_window, GafState};

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

    /// The camera focus system follows the menu selection (camera tracks the
    /// Tx carrier) and the one-shot "reset view" request restores the initial
    /// viewpoint targets.
    #[test]
    fn camera_focus_follows_menu_selection_and_view_reset() {
        use bevy_panorbit_camera::PanOrbitCamera;

        use crate::entities::Carrier;
        use crate::scene::Tx;
        use super::CameraFocus;

        let mut app = test_app();
        app.add_systems(Update, crate::camera::update_camera_focus);
        let camera = app.world_mut().spawn(PanOrbitCamera::default()).id();
        app.update(); // Startup: spawns the scene

        // Default is Free: the camera focus must be left untouched so the user
        // keeps orbit/pan/zoom control.
        assert_eq!(
            app.world().resource::<MenuWidget>().camera_focus,
            CameraFocus::Free
        );
        let user_panned_focus = Vec3::new(123.0, 45.0, -67.0);
        app.world_mut()
            .get_mut::<PanOrbitCamera>(camera)
            .unwrap()
            .target_focus = user_panned_focus;
        app.update();
        assert_eq!(
            app.world().get::<PanOrbitCamera>(camera).unwrap().target_focus,
            user_panned_focus,
            "Free focus must not override a user pan"
        );

        // Focus the Tx carrier: the target must follow its (non-origin) position
        app.world_mut().resource_mut::<MenuWidget>().camera_focus = CameraFocus::Tx;
        app.update();
        let tx_translation = {
            let mut tx_carrier_q = app
                .world_mut()
                .query_filtered::<&Transform, (With<Tx>, With<Carrier>)>();
            tx_carrier_q.single(app.world()).unwrap().translation
        };
        assert!(tx_translation.length() > 0.0);
        let pan_orbit_camera = app.world().get::<PanOrbitCamera>(camera).unwrap();
        assert_eq!(pan_orbit_camera.target_focus, tx_translation);

        // Reset view: free camera, origin focus and initial orientation/zoom
        {
            let mut menu_widget = app.world_mut().resource_mut::<MenuWidget>();
            menu_widget.camera_focus = CameraFocus::Free;
            menu_widget.reset_view_requested = true;
        }
        app.update();
        let pan_orbit_camera = app.world().get::<PanOrbitCamera>(camera).unwrap();
        assert_eq!(pan_orbit_camera.target_focus, Vec3::ZERO);
        assert!(pan_orbit_camera.force_update);
        assert!(!app.world().resource::<MenuWidget>().reset_view_requested);
    }
}
