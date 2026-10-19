//! Render the native Android ribbon without opening a desktop window.
use deckcraft_ui_egui::{Services, SlideApp, theme::Brightness};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "android-snapshot.png".into());
    let dark = args.any(|arg| arg == "--dark");
    let services = Services { host_toolbar: Some(Default::default()), ..Default::default() };
    let mut app = SlideApp::new(deckcraft_engine::Session::with_new(), services);
    app.ui.brightness = if dark { Brightness::Dark } else { Brightness::Light };
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1600.0, 900.0)).wgpu().build_ui_state(
        |ui, app: &mut SlideApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(6);
    h.render()?.save(out)?;
    Ok(())
}
