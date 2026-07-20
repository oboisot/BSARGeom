//! Saving a generated image from both the desktop and the web build.
//!
//! Native builds ask for a destination with an in-app "save as" dialog; the web
//! build hands the bytes straight to the browser as a download, since a wasm
//! page has no filesystem to browse. Both are driven through [`SaveRequest`],
//! so the calling UI has a single code path:
//!
//! ```text
//! if clicked            { state.request = Some(SaveRequest::new(name, bytes)); }
//! if let Some(r) = ...  { if let Some(status) = r.update(ctx) { /* finished */ } }
//! ```

use bevy_egui::egui;

/// A save operation in flight. [`SaveRequest::update`] returns `Some(status)`
/// once it resolves (saved, cancelled or failed), and the caller drops it.
pub struct SaveRequest {
    #[cfg(not(target_arch = "wasm32"))]
    dialog: egui_file_dialog::FileDialog,
    #[cfg(not(target_arch = "wasm32"))]
    bytes: Vec<u8>,
    /// Web builds resolve immediately: the outcome is produced in `new`.
    #[cfg(target_arch = "wasm32")]
    status: Option<String>,
}

impl SaveRequest {
    /// Starts saving `bytes` under a suggested `file_name` (a PNG image).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(file_name: &str, bytes: Vec<u8>) -> Self {
        let mut dialog = egui_file_dialog::FileDialog::new()
            .add_file_filter_extensions("PNG image", vec!["png"])
            .default_file_filter("PNG image")
            .default_file_name(file_name);
        dialog.save_file();
        Self { dialog, bytes }
    }

    /// Draws the dialog and reports the outcome once the user is done.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn update(&mut self, ctx: &egui::Context) -> Option<String> {
        use egui_file_dialog::DialogState;

        self.dialog.update(ctx);
        match self.dialog.state() {
            DialogState::Open => None,
            DialogState::Picked(path) => Some(match std::fs::write(path, &self.bytes) {
                Ok(()) => format!("Saved to {}", path.display()),
                Err(error) => format!("Save failed: {error}"),
            }),
            // Closed/cancelled without picking anything, or a multi-selection
            // that a save dialog never produces
            _ => Some("Save cancelled".to_string()),
        }
    }

    /// Web build: wrap the bytes in a Blob and click a synthetic download link.
    #[cfg(target_arch = "wasm32")]
    pub fn new(file_name: &str, bytes: Vec<u8>) -> Self {
        Self {
            status: Some(
                download_in_browser(file_name, &bytes)
                    .unwrap_or_else(|error| format!("Save failed: {error}")),
            ),
        }
    }

    /// Web build: the download was already triggered, so resolve at once.
    #[cfg(target_arch = "wasm32")]
    pub fn update(&mut self, _ctx: &egui::Context) -> Option<String> {
        self.status.take()
    }
}

#[cfg(target_arch = "wasm32")]
fn download_in_browser(file_name: &str, bytes: &[u8]) -> Result<String, String> {
    use wasm_bindgen::JsCast as _;

    let to_error = |value: wasm_bindgen::JsValue| {
        value
            .as_string()
            .unwrap_or_else(|| "browser refused the download".to_string())
    };

    let array = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&array);
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("image/png");
    let blob =
        web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(to_error)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(to_error)?;

    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "no document".to_string())?;
    let anchor = document
        .create_element("a")
        .map_err(to_error)?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "could not create the download link".to_string())?;
    anchor.set_href(&url);
    anchor.set_download(file_name);
    anchor.click();
    // The browser keeps the blob alive until the download completes
    let _ = web_sys::Url::revoke_object_url(&url);

    Ok(format!("Downloaded {file_name}"))
}
