pub enum RenderMode {
    HighPerformance,
    LowPower,
}

pub struct WindowOptions {
    pub title: String,
    pub default_width: u32,
    pub default_height: u32,
    pub render_mode: RenderMode,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Render Window".to_string(),
            default_width: 800,
            default_height: 600,
            render_mode: RenderMode::LowPower,
        }
    }
}
