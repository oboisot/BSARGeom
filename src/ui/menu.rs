use bevy::prelude::*;
use bevy_egui::egui;

const TX_MENU_OPEN_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-tx-open-48.png");
const TX_MENU_CLOSE_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-tx-close-48.png");
const RX_MENU_OPEN_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-rx-open-48.png");
const RX_MENU_CLOSE_ICON: egui::ImageSource<'_> = egui::include_image!("../../assets/menu-rx-close-48.png");


pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuWidget>();
    }
}

#[derive(Resource)]
pub struct MenuWidget {
    pub is_tx_panel_opened: bool,
    pub is_rx_panel_opened: bool,
}

impl Default for MenuWidget {
    fn default() -> Self {
        Self {
            is_tx_panel_opened: false,
            is_rx_panel_opened: false,
        }
    }
}

impl MenuWidget {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().spacing.button_padding = egui::vec2(0.0, 0.0); // No padding for buttons in Menu
        ui.style_mut().spacing.item_spacing = egui::vec2(1.0, 1.0); // Set spacing between items in Menu

        let tx_menu_icon = if self.is_tx_panel_opened { TX_MENU_CLOSE_ICON } else { TX_MENU_OPEN_ICON };
        let rx_menu_icon = if self.is_rx_panel_opened { RX_MENU_CLOSE_ICON } else { RX_MENU_OPEN_ICON };

        ui.vertical_centered(|ui| {
            // Top buttons
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.add_space(1.0);
                    ui.separator();
                    // Transmitter settings panel button
                    let tx_panel_button = egui::Button::image(tx_menu_icon)
                        .selected(self.is_tx_panel_opened);
                    let hover_text = egui::RichText::new("Open/Close Transmitter settings panel")
                        .color(egui::Color32::from_rgb(200, 200, 200))
                        .monospace();
                    if ui.add(tx_panel_button)
                        .on_hover_text(hover_text)
                        .clicked() {
                            self.is_tx_panel_opened = !self.is_tx_panel_opened;
                        };
                         
                    // Receiver settings panel button
                    let rx_panel_button = egui::Button::image(rx_menu_icon)
                        .selected(self.is_rx_panel_opened);
                    let hover_text = egui::RichText::new("Open/Close Receiver settings panel")
                        .color(egui::Color32::from_rgb(200, 200, 200))
                        .monospace();
                    if ui.add(rx_panel_button)
                        .on_hover_text(hover_text)
                        .clicked() {
                            self.is_rx_panel_opened = !self.is_rx_panel_opened;
                        };
                    ui.separator();
                    ui.add_space(1.0);
                    
                    // ui.separator();
                    // // Camera focus buttons
                    // if ui.add(egui::Button::new("Ground"))
                    //      .on_hover_text("Sets camera focus on ground origin")
                    //      .clicked() { return; };
                    // if ui.add(egui::Button::new("Tx"))
                    //      .on_hover_text("Sets camera focus on Transmitter origin")
                    //      .clicked() { return; };
                    // if ui.add(egui::Button::new("Rx"))
                    //      .on_hover_text("Sets camera focus on Receiver origin")
                    //      .clicked() { return; };
                    //     // egui::Button::image(menu_image)
                    //     //     .fill(egui::Color32::TRANSPARENT)
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
