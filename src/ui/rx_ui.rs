// use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn rx_ui(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::right("Receiver")
        .resizable(false)
        .default_width(250.0)
        .max_width(300.0)
        .show_separator_line(true)
        .show(ctx, |ui| {
            ui.separator();
            ui.vertical_centered(|ui| ui.label(
                egui::RichText::new("RECEIVER SETTINGS")
                    .size(15.0)
                    .strong()
            ));
            ui.separator();

            ui.separator();
            ui.vertical_centered(|ui| ui.label(
                egui::RichText::new("CARRIER").strong()
            ));
            ui.separator();
            
            ui.separator();
            ui.vertical_centered(|ui| ui.label(
                egui::RichText::new("ANTENNA").strong()
            ));
            ui.separator();

            ui.label("Orientation");

            ui.label("Beamwidth");
            ui.separator();
        });
}
