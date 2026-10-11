use eframe::egui;

/// Loads and registers a PNG image buffer as an egui TextureHandle.
pub fn load_png_texture(
    ctx: &egui::Context,
    name: &str,
    png_bytes: &[u8],
) -> Option<egui::TextureHandle> {
    if let Ok(img) = image::load_from_memory(png_bytes) {
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            rgba.as_flat_samples().as_slice(),
        );
        Some(ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR))
    } else {
        None
    }
}
