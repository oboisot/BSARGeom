use bevy::prelude::*;
use bevy_egui::egui;

const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(200, 200, 200);

const TX_MENU_OPEN_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-tx-open-48.png");
const TX_MENU_CLOSE_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-tx-close-48.png");
const RX_MENU_OPEN_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-rx-open-48.png");
const RX_MENU_CLOSE_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-rx-close-48.png");
const MENU_BIST_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-bist-48.png");
const MENU_MONO_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-mono-48.png");
const MENU_ORIGIN_CAMERA_FOCUS: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-origin-camera-focus-48.png");
const MENU_ORIGIN_CAMERA_FOCUS_ACTIVE: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-origin-camera-focus-active-48.png");
const MENU_TX_CAMERA_FOCUS: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-tx-camera-focus-48.png");
const MENU_TX_CAMERA_FOCUS_ACTIVE: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-tx-camera-focus-active-48.png");
const MENU_RX_CAMERA_FOCUS: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-rx-camera-focus-48.png");
const MENU_RX_CAMERA_FOCUS_ACTIVE: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-rx-camera-focus-active-48.png");
const MENU_GAF: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-gaf-48.png");

pub(crate) const RESET_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-reset-48.png");
pub(crate) const HELP_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/help-48.png");
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const SAVE_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/save-48.png");
#[cfg(target_arch = "wasm32")]
pub(crate) const SAVE_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/download-48.png");

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuWidget>();
    }
}

/// Which point the orbit camera keeps in focus.
///
/// [`CameraFocus::Free`] is the default and leaves the camera entirely to the
/// user (orbit / pan / zoom, the behaviour before focus tracking existed); the
/// other variants pin the focus point and therefore disable panning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraFocus {
    #[default]
    Free,
    Ground,
    Tx,
    Rx,
}

#[derive(Resource)]
#[derive(Default)]
pub struct MenuWidget {
    pub is_tx_panel_opened: bool,
    pub is_rx_panel_opened: bool,
    pub is_monostatic: bool,
    pub was_monostatic: bool,
    pub force_rx_system_update: bool,
    pub camera_focus: CameraFocus,
    /// One-shot request consumed by the camera system: restore the initial view.
    pub reset_view_requested: bool,
    pub is_gaf_opened: bool,
}


impl MenuWidget {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        ui.style_mut().spacing.button_padding = egui::vec2(0.0, 0.0); // No padding for buttons in Menu
        ui.style_mut().spacing.item_spacing = egui::vec2(50.0, 2.5); // Set spacing between items in Menu
        // ui.style_mut().visuals.button_frame = false; // Set background color for buttons

        let tx_menu_icon = if self.is_tx_panel_opened { TX_MENU_CLOSE_ICON } else { TX_MENU_OPEN_ICON };
        let rx_menu_icon = if self.is_rx_panel_opened { RX_MENU_CLOSE_ICON } else { RX_MENU_OPEN_ICON };
        let menu_bist_mono_icon = if self.is_monostatic { MENU_MONO_ICON } else { MENU_BIST_ICON };
        let (
            menu_origin_camera_focus_icon,
            menu_tx_camera_focus_icon,
            menu_rx_camera_focus_icon
        ) = match self.camera_focus {
            CameraFocus::Free => (MENU_ORIGIN_CAMERA_FOCUS, MENU_TX_CAMERA_FOCUS, MENU_RX_CAMERA_FOCUS),
            CameraFocus::Ground => (MENU_ORIGIN_CAMERA_FOCUS_ACTIVE, MENU_TX_CAMERA_FOCUS, MENU_RX_CAMERA_FOCUS),
            CameraFocus::Tx => (MENU_ORIGIN_CAMERA_FOCUS, MENU_TX_CAMERA_FOCUS_ACTIVE, MENU_RX_CAMERA_FOCUS),
            CameraFocus::Rx => (MENU_ORIGIN_CAMERA_FOCUS, MENU_TX_CAMERA_FOCUS, MENU_RX_CAMERA_FOCUS_ACTIVE),
        };

        ui.vertical_centered(|ui| {
            // Top buttons
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center)
                    .with_main_wrap(false),
                |ui| {
                    // TX / RX / Monostatic buttons
                    ui.add_space(1.0);
                    ui.separator();
                    ui.label(egui::RichText::new("Tx / Rx").size(10.0).color(TEXT_COLOR));
                    ui.separator();
                    // Transmitter settings panel button
                    let tx_panel_button = egui::Button::selectable(
                        self.is_tx_panel_opened,
                        tx_menu_icon
                    );
                    let hover_text = egui::RichText::new("Open/Close Transmitter settings panel")
                        .color(TEXT_COLOR)
                        .monospace();
                    if ui.add(tx_panel_button)
                        .on_hover_text(hover_text)
                        .clicked() {
                            self.is_tx_panel_opened = !self.is_tx_panel_opened;
                        };
                         
                    // Receiver settings panel button
                    let rx_panel_button = egui::Button::selectable(
                        self.is_rx_panel_opened,
                        rx_menu_icon
                    );
                    let hover_text = egui::RichText::new("Open/Close Receiver settings panel")
                        .color(TEXT_COLOR)
                        .monospace();
                    if ui.add(rx_panel_button)
                        .on_hover_text(hover_text)
                        .clicked() {
                            self.is_rx_panel_opened = !self.is_rx_panel_opened;
                        };
                    // Monostatic button
                    let monostatic_button = egui::Button::selectable(
                        self.is_monostatic,
                        menu_bist_mono_icon
                    );
                    let hover_text = egui::RichText::new("Switch between Bistatic/Monostatic mode")
                            .color(TEXT_COLOR)
                            .monospace();
                    if ui.add(monostatic_button)
                        .on_hover_text(hover_text)
                        .clicked() {
                            self.is_monostatic = !self.is_monostatic;
                        };

                    // CAMERA FOCUS
                    ui.separator();
                    ui.label(egui::RichText::new("Cam.").size(10.0).color(TEXT_COLOR));
                    ui.separator();                    
                    // Camera focus buttons. Clicking the active one releases the
                    // focus back to CameraFocus::Free (free orbit/pan/zoom).
                    for (focus, icon, hover) in [
                        (CameraFocus::Ground, menu_origin_camera_focus_icon, "Sets camera focus on ground origin\n(click again to free the camera)"),
                        (CameraFocus::Tx, menu_tx_camera_focus_icon, "Sets camera focus on Transmitter origin\n(click again to free the camera)"),
                        (CameraFocus::Rx, menu_rx_camera_focus_icon, "Sets camera focus on Receiver origin\n(click again to free the camera)"),
                    ] {
                        let hover_text = egui::RichText::new(hover)
                            .color(TEXT_COLOR)
                            .monospace();
                        if ui.add(egui::Button::selectable(
                                self.camera_focus == focus,
                                icon
                            ))
                            .on_hover_text(hover_text)
                            .clicked() {
                                self.camera_focus = if self.camera_focus == focus {
                                    CameraFocus::Free // Toggle off: restore free camera
                                } else {
                                    focus
                                };
                            };
                    }
                    // Reset view button
                    let hover_text = egui::RichText::new("Resets camera view (free camera, initial orientation and zoom)")
                        .color(TEXT_COLOR)
                        .monospace();
                    if ui.add(
                        egui::Button::new(RESET_ICON)
                            .frame_when_inactive(false)
                        )
                        .on_hover_text(hover_text)
                        .clicked() {
                            // Free, so the camera stays fully controllable after the reset
                            self.camera_focus = CameraFocus::Free;
                            self.reset_view_requested = true;
                        };
                    ui.add_space(1.0);
                    ui.separator();
                    ui.label(egui::RichText::new("Plots").size(10.0).color(TEXT_COLOR));
                    ui.separator();

                    // GAF plot toggle button
                    let hover_text = egui::RichText::new("Open/Close the Generalized Ambiguity Function plot")
                        .color(TEXT_COLOR)
                        .monospace();
                    if ui.add(egui::Button::selectable(
                            self.is_gaf_opened,
                            MENU_GAF
                        ))
                        .on_hover_text(hover_text)
                        .clicked() {
                            self.is_gaf_opened = !self.is_gaf_opened;
                        };
                    ui.add_space(1.0);
                    ui.separator();
                }
            );

            // // Bottom buttons
            // ui.with_layout(
            //     egui::Layout::bottom_up(egui::Align::Center),
            //     |ui| {
            //         ui.add_space(1.0);
            //         // Exit button
            //         if ui.add(
            //             egui::Button::image(exit_image)
            //                 .fill(egui::Color32::TRANSPARENT)
            //         )
            //         .on_hover_text("Exit application")
            //         .clicked() {
            //             ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close); // Close the current viewport
            //         };

            //         //
            //         global_theme_preference_switch(ui)
            //     }
            // );
        });
    }
}
