#[command_collector::collect_all(pub)]
pub mod inner {
    /// 挨拶しましょう！
    #[command(auto)]
    pub fn greet(name: &str) -> String {
        tracing::info!("挨拶しました。");
        format!("Hello, {}! You've been greeted from Rust!", name)
    }
}
