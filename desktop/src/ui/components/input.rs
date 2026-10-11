use eframe::egui::{self, Align, Margin};

/// TextEdit singleline with vertically centered text, padding, and text clipping.
pub fn singleline_input<'a>(text: &'a mut String) -> egui::TextEdit<'a> {
    egui::TextEdit::singleline(text)
        .vertical_align(Align::Center)
        .margin(Margin::symmetric(10.0, 6.0))
        .clip_text(true)
}
