pub mod api;

use serde::Serialize;
use specta_typescript::Typescript;
use tauri::{Emitter, Wry};
use tauri_specta::ErrorHandlingMode;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    tauri_specta::Builder::<Wry>::new()
        .commands(api::commands())
        .types(&specta::collect())
        .error_handling(ErrorHandlingMode::Throw)
        .export(Typescript::default(), "../src/gen/bindings.ts")
        .expect("バインディングの出力に失敗しました。");

    tauri::Builder::<Wry>::new()
        .setup(|app| {
            tracing_subscriber::registry()
                .with(TauriLogLayer {
                    app_handle: app.handle().clone(),
                })
                .init();
            Ok(())
        })
        .invoke_handler(api::invoke_handler())
        .run(tauri::generate_context!())
        .expect("アプリケーションの実行中にエラーが発生しました。");
}

struct TauriLogLayer {
    app_handle: tauri::AppHandle,
}
impl<S: Subscriber> Layer<S> for TauriLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);

        let level = event.metadata().level().to_string().to_lowercase();
        let message = visitor.0;

        self.app_handle.emit("rust-log", (level, message)).ok();
    }
}
struct StringVisitor(String);
impl Visit for StringVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
#[derive(Serialize, specta::Type)]
pub struct AppError(String);
impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self(value.to_string())
    }
}
