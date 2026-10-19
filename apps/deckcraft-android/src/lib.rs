//! Android host. The document engine and egui editor remain the desktop implementations.
#![deny(unsafe_code, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented, clippy::unreachable)]

#[cfg(target_os = "android")]
mod adapter;
pub mod assistant;
pub mod attach;
#[cfg(target_os = "android")]
mod audio;
pub mod input;
#[cfg(target_os = "android")]
mod mobile;
#[cfg(target_os = "android")]
mod platform;
pub mod protocol;

/// The assistant's texts for this app.
pub const BRANDING: assistant::Branding = assistant::Branding {
    name: "DeckCraft",
    noun: "la presentación",
    welcome: "Pregunta sobre la presentación o pide cambios. El asistente la lee y la edita con las mismas órdenes que la línea de comandos.",
    suggestions: &[
        ("Resumen de la presentación", "Resume esta presentación diapositiva por diapositiva."),
        ("Crea una diapositiva de conclusiones", "Añade al final una diapositiva de conclusiones basada en el contenido de la presentación."),
        ("Revisa ortografía y estilo", "Revisa la ortografía y el estilo de los textos de todas las diapositivas y corrige los errores."),
        ("Sugiere mejoras de diseño", "Sugiere mejoras de diseño y de estructura para esta presentación."),
    ],
};

// android-activity requires this exact exported Rust ABI symbol. No unsafe operations occur
// here or in the platform bridge; JNI uses checked calls and JavaVM::singleton().
#[cfg(target_os = "android")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub fn android_main(app: android_activity::AndroidApp) {
    mobile::start(app);
}
