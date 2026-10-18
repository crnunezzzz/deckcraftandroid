#![forbid(unsafe_code)]
use crate::platform::Platform;
use deckcraft_ui_egui::{Inbox, InboxEvent, InboxFile, Services, SlideApp};
use egui::{Context, Ui};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub struct Editor {
    platform: Platform,
    app: SlideApp,
    inbox: Inbox,
    export_path: PathBuf,
}
impl Editor {
    pub const NAME: &'static str = "DeckCraft";
    pub fn new(dir: &Path, platform: Platform, _ctx: &Context) -> Self {
        let open = platform.clone();
        let save = platform.clone();
        let inbox: Inbox = Arc::new(Mutex::new(vec![]));
        let html_clipboard = platform.clone();
        let copy_clipboard = platform.clone();
        let picture_clipboard = platform.clone();
        let services = Services {
            clipboard_html: Some(Box::new(move || html_clipboard.clipboard("html"))),
            clipboard_set_html: Some(Box::new(move |html, text| copy_clipboard.copy_html(html, text).is_ok())),
            clipboard_image: Some(Box::new(move || picture_clipboard.clipboard_image())),
            audio_out: Some(Box::new(crate::audio::CpalOut::default())),
            open_async: Some(Box::new(move |purpose, request| {
                let _ = open.exchange(json!({"op":"open","ticket":request,"purpose":purpose}));
            })),
            pick_save: Some(Box::new(move |name| {
                save.exchange(json!({"op":"local_save","name":name,"export_after_write":true})).ok()?.get("path")?.as_str().map(str::to_owned)
            })),
            read: Some(Box::new(|path| std::fs::read(path).map_err(|e| e.to_string()))),
            write: Some(Box::new(|path, bytes| std::fs::write(path, bytes).map_err(|e| e.to_string()))),
            inbox: Some(inbox.clone()),
            ..Default::default()
        };
        let mut session = deckcraft_engine::Session::with_new();
        session.recovery_dir = Some(dir.join("recovery"));
        let recovered = dir.join("session.deckcraft");
        if recovered.is_file() {
            let _ = session.execute("file.open", &json!({"path":recovered}));
            if let Some(d) = session.active_mut() {
                d.path = None;
            }
        }
        let mut app = SlideApp::new(session, services);
        if let Ok(bytes) = std::fs::read(dir.join("editor.json"))
            && let Ok(prefs) = serde_json::from_slice::<deckcraft_ui_egui::UiState>(&bytes)
        {
            app.ui = prefs.sanitized();
        }
        Self { app, inbox, platform, export_path: dir.join("export.pptx") }
    }
    pub fn logic(&mut self, ctx: &Context) {
        self.app.logic(ctx);
    }
    pub fn raw_input(&mut self, raw: &mut egui::RawInput) {
        self.app.raw_input_hook(raw);
    }
    pub fn ui(&mut self, ui: &mut Ui) {
        self.app.ui(ui);
    }
    pub fn run(&mut self, id: &str, params: Value) -> Result<Value, String> {
        self.app.run(id, params)
    }
    pub fn agent_save(&mut self) -> Result<Value, String> {
        let path = if let Some(path) = self.app.session.active().and_then(|d| d.path.clone()) {
            path
        } else {
            self.platform.exchange(json!({"op":"local_save","name":"presentation.deckcraft"}))?["path"]
                .as_str()
                .ok_or("No se pudo crear el archivo local")?
                .to_owned()
        };
        self.app.run("file.save", json!({"path":path}))?;
        if self.app.session.active().is_none_or(|doc| doc.is_dirty()) || !Path::new(&path).is_file() {
            return Err("El guardado necesita completarse en el editor".into());
        }
        Ok(json!({"saved":true,"scope":"local_copy","name":Path::new(&path).file_name().and_then(|n| n.to_str())}))
    }
    pub fn catalog(&self) -> Value {
        serde_json::to_value(self.app.session.commands()).unwrap_or(json!([]))
    }
    pub fn context(&mut self) -> Value {
        self.app.session.execute("document.inspect", &json!({})).unwrap_or_else(|e| json!({"error":e.to_string()}))
    }
    pub fn stamp(&self) -> String {
        self.app.session.active().map(|d| format!("{}:{}:{:?}", d.uid, d.revision, d.selection)).unwrap_or_default()
    }
    pub fn file_event(&mut self, event: &Value) {
        if event["external"] == true
            && let Some(path) = event["path"].as_str()
        {
            if let Err(error) = self.app.run("file.open", json!({"path":path})) {
                self.app.set_status(error);
            }
            return;
        }
        let id = event["ticket"].as_u64().unwrap_or(0);
        let result = event["path"]
            .as_str()
            .and_then(|p| std::fs::read(p).ok())
            .map(|bytes| InboxEvent::File(InboxFile { id, name: event["name"].as_str().unwrap_or("presentation.pptx").into(), bytes }))
            .unwrap_or(InboxEvent::Cancelled { id });
        self.inbox.lock().unwrap_or_else(|e| e.into_inner()).push(result);
    }
    pub fn persist(&self, dir: &Path) {
        if let Ok(bytes) = serde_json::to_vec(&self.app.ui) {
            let _ = std::fs::write(dir.join("editor.json"), bytes);
        }
    }
    pub fn recover(&mut self, dir: &Path) {
        let _ = self.app.session.execute("file.export", &json!({"path":dir.join("session.deckcraft"),"format":"deckcraft"}));
    }
    pub fn export(&mut self, platform: &Platform) -> Result<(), String> {
        self.app.run("file.export", json!({"path":self.export_path,"format":"pptx"}))?;
        platform.exchange(json!({"op":"export","path":self.export_path,"name":"presentation.pptx","ticket":9000}))?;
        Ok(())
    }
}
