use eframe::egui::{self, Color32, Margin, Rounding, Stroke};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub is_dark: bool,
    pub bg_primary: Color32,
    pub bg_surface: Color32,
    pub bg_surface_hover: Color32,
    pub bg_card: Color32,
    pub bg_input: Color32,
    pub primary: Color32,
    pub primary_hover: Color32,
    pub emerald: Color32,
    pub emerald_bg: Color32,
    pub red: Color32,
    pub red_bg: Color32,
    pub amber: Color32,
    pub amber_bg: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            bg_primary: Color32::from_rgb(13, 17, 23), // Slate 950 / GitHub Dark
            bg_surface: Color32::from_rgb(22, 27, 34), // Elevated card
            bg_surface_hover: Color32::from_rgb(33, 38, 45), // Hover state
            bg_card: Color32::from_rgb(22, 27, 34),
            bg_input: Color32::from_rgb(13, 17, 23),
            primary: Color32::from_rgb(99, 102, 241), // Indigo 500
            primary_hover: Color32::from_rgb(129, 140, 248), // Indigo 400
            emerald: Color32::from_rgb(52, 211, 153), // Emerald 400
            emerald_bg: Color32::from_rgb(10, 38, 28), // Dark emerald tint
            red: Color32::from_rgb(248, 113, 113),    // Red 400
            red_bg: Color32::from_rgb(48, 16, 16),    // Dark red tint
            amber: Color32::from_rgb(251, 191, 36),   // Amber 400
            amber_bg: Color32::from_rgb(48, 34, 10),  // Dark amber tint
            text_primary: Color32::from_rgb(243, 244, 246), // Gray 100
            text_secondary: Color32::from_rgb(156, 163, 175), // Gray 400
            text_muted: Color32::from_rgb(107, 114, 128), // Gray 500
            border: Color32::from_rgb(48, 54, 61),    // Border
            border_subtle: Color32::from_rgb(33, 38, 45),
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            bg_primary: Color32::from_rgb(248, 250, 252), // Slate 50
            bg_surface: Color32::from_rgb(255, 255, 255), // Pure White Card
            bg_surface_hover: Color32::from_rgb(241, 245, 249), // Slate 100
            bg_card: Color32::from_rgb(255, 255, 255),
            bg_input: Color32::from_rgb(241, 245, 249),
            primary: Color32::from_rgb(79, 70, 229), // Indigo 600
            primary_hover: Color32::from_rgb(67, 56, 202), // Indigo 700
            emerald: Color32::from_rgb(5, 150, 105), // Emerald 600
            emerald_bg: Color32::from_rgb(236, 253, 245), // Soft mint
            red: Color32::from_rgb(220, 38, 38),     // Red 600
            red_bg: Color32::from_rgb(254, 242, 242), // Soft red
            amber: Color32::from_rgb(217, 119, 6),   // Amber 600
            amber_bg: Color32::from_rgb(254, 243, 199), // Soft amber
            text_primary: Color32::from_rgb(15, 23, 42), // Slate 900
            text_secondary: Color32::from_rgb(71, 85, 105), // Slate 600
            text_muted: Color32::from_rgb(148, 163, 184), // Slate 400
            border: Color32::from_rgb(226, 232, 240), // Slate 200
            border_subtle: Color32::from_rgb(241, 245, 249),
        }
    }
}

pub fn detect_system_is_dark() -> bool {
    match dark_light::detect() {
        Ok(dark_light::Mode::Light) => false,
        _ => true,
    }
}

pub fn get_system_theme() -> Theme {
    if detect_system_is_dark() {
        Theme::dark()
    } else {
        Theme::light()
    }
}

pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let mut visuals = if theme.is_dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.dark_mode = theme.is_dark;
    visuals.override_text_color = Some(theme.text_primary);
    visuals.panel_fill = theme.bg_primary;
    visuals.window_fill = theme.bg_surface;
    visuals.faint_bg_color = theme.bg_surface;
    visuals.extreme_bg_color = theme.bg_input;

    visuals.widgets.noninteractive.bg_fill = theme.bg_surface;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, theme.border);
    visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, theme.text_primary);

    visuals.widgets.inactive.bg_fill = theme.bg_surface;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, theme.border);
    visuals.widgets.inactive.rounding = Rounding::same(6.0);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, theme.text_primary);

    visuals.widgets.hovered.bg_fill = theme.bg_surface_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, theme.primary);
    visuals.widgets.hovered.rounding = Rounding::same(6.0);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, theme.text_primary);

    visuals.widgets.active.bg_fill = theme.primary;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, theme.primary_hover);
    visuals.widgets.active.rounding = Rounding::same(6.0);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

    visuals.widgets.open.bg_fill = theme.bg_surface;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, theme.primary);
    visuals.widgets.open.rounding = Rounding::same(6.0);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, theme.text_primary);

    visuals.window_stroke = Stroke::new(1.0_f32, theme.border);
    visuals.menu_rounding = Rounding::same(6.0);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;

    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.window_margin = Margin::same(14.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);

    ctx.set_style(style);
}
