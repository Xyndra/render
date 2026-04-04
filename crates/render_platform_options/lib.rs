#[derive(PartialEq, Clone, Default)]
pub enum RenderMode {
    HighPerformance,
    #[default]
    LowPower,
}

#[derive(Clone)]
pub struct WindowOptions {
    pub title: String,
    pub default_width: u32,
    pub default_height: u32,
    pub render_mode: RenderMode,
    pub clear_color: (u8, u8, u8),
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "Render Window".to_string(),
            default_width: 800,
            default_height: 600,
            render_mode: RenderMode::LowPower,
            clear_color: (200, 255, 200),
        }
    }
}
