//! The AI assistant panel in the desktop app (`deckcraft-assistant`): the editor it drives and
//! the ribbon's Asistente tab. Kept in this file so `main.rs` only wires it in.
#![forbid(unsafe_code)]

use deckcraft_assistant::{BRANDING, agent::Editor, desktop::Desktop};
use deckcraft_ui_egui::SlideApp;
use serde_json::{Value, json};

/// The editor as the assistant drives it.
struct Driven<'a>(&'a mut SlideApp);

impl Editor for Driven<'_> {
    fn catalog(&self) -> Value {
        serde_json::to_value(self.0.session.commands()).unwrap_or(json!([]))
    }
    fn run(&mut self, id: &str, params: Value) -> Result<Value, String> {
        self.0.run(id, params)
    }
    fn save(&mut self) -> Result<Value, String> {
        let Some(path) = self.0.session.active().and_then(|d| d.path.clone()) else {
            return Err("The presentation has no file yet: ask the user to save it once with File › Save As".into());
        };
        self.0.run("file.save", json!({"path":path}))?;
        if self.0.session.active().is_none_or(|doc| doc.is_dirty()) {
            return Err("The save did not complete".into());
        }
        Ok(json!({"saved":true,"name":std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned())}))
    }
    fn context(&mut self) -> Value {
        self.0.session.execute("document.inspect", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}))
    }
    fn interface_scale(&self, _native: f32) -> f32 {
        self.0.ui.ui_scale
    }
    fn set_interface_scale(&mut self, _native: f32, scale: f32) {
        self.0.ui.ui_scale = scale.clamp(0.5, 3.0);
    }
}

pub struct AssistantHost(Desktop);

impl AssistantHost {
    /// `dir`: the settings folder (assistant.json lives there).
    pub fn new(dir: Option<std::path::PathBuf>) -> Self {
        let dir = dir.unwrap_or_else(|| std::env::temp_dir().join("DeckCraft"));
        Self(Desktop::new("DeckCraft", &BRANDING, dir))
    }

    /// Before the editor's frame logic.
    pub fn frame(&mut self, ctx: &egui::Context, app: &mut SlideApp) {
        self.0.frame(ctx, &mut Driven(app));
    }

    /// Before the editor's UI: the panel takes the right side of the window.
    pub fn panel(&mut self, ui: &mut egui::Ui, app: &mut SlideApp) {
        if let Some(host) = &mut app.services.host_toolbar {
            host.assistant_open = self.0.assistant.open;
            host.busy = self.0.busy();
        }
        self.0.panel(ui);
    }

    /// After the editor's UI: the ribbon's Asistente tab and the panel's actions.
    pub fn finish(&mut self, ctx: &egui::Context, app: &mut SlideApp) {
        match app.services.host_toolbar.as_mut().and_then(|h| h.action.take()) {
            Some("assistant") => self.0.toggle(),
            Some("settings") => self.0.open_settings(),
            _ => {}
        }
        self.0.finish(ctx, &mut Driven(app));
    }

    pub fn on_exit(&mut self) {
        self.0.on_exit();
    }
}
