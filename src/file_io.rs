use crate::canvas::Canvas;
use crate::project::Project;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::path::Path;

pub fn save_project(canvas: &Canvas, path: &Path) -> Result<(), String> {
    let project = Project::from_canvas(canvas);

    let data = serde_json::to_vec_pretty(&project)
        .map_err(|e| e.to_string())?;

    std::fs::write(path, data).map_err(|e| e.to_string())
}

pub fn load_project(path: &Path) -> Result<Canvas, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;

    let project: Project = serde_json::from_slice(&data)
        .map_err(|e| e.to_string())?;

    if project.version != 3 {
        return Err(format!(
            "Неподдерживаемая версия проекта: {}",
            project.version
        ));
    }

    Ok(project.into_canvas())
}

fn blend_pixel(dst: &mut Rgba<u8>, src: [u8; 4]) {
    let a = src[3] as f32 / 255.0;
    let da = dst[3] as f32 / 255.0;
    let out_a = a + da * (1.0 - a);

    if out_a <= 0.0 {
        *dst = Rgba([0, 0, 0, 0]);
        return;
    }

    for i in 0..3 {
        let value = (
            src[i] as f32 * a
            + dst[i] as f32 * da * (1.0 - a)
        ) / out_a;

        dst[i] = value.round().clamp(0.0, 255.0) as u8;
    }

    dst[3] = (out_a * 255.0).round() as u8;
}

fn stamp(image: &mut RgbaImage, x: f32, y: f32, size: f32, color: [u8; 4]) {
    let radius = (size / 2.0).max(0.5);
    let min_x = (x - radius).floor().max(0.0) as i32;
    let max_x = (x + radius).ceil().min(image.width() as f32 - 1.0) as i32;
    let min_y = (y - radius).floor().max(0.0) as i32;
    let max_y = (y + radius).ceil().min(image.height() as f32 - 1.0) as i32;

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let dx = px as f32 + 0.5 - x;
            let dy = py as f32 + 0.5 - y;

            if dx * dx + dy * dy <= radius * radius {
                blend_pixel(image.get_pixel_mut(px as u32, py as u32), color);
            }
        }
    }
}

fn draw_stroke(image: &mut RgbaImage, stroke: &crate::canvas::Stroke) {
    let color = stroke.color;
    let radius = (stroke.size / 2.0).max(0.5);

    if stroke.points.is_empty() {
        return;
    }

    if stroke.points.len() == 1 {
        let (x, y) = stroke.points[0];
        stamp(image, x, y, stroke.size, color);
        return;
    }

    for pair in stroke.points.windows(2) {
        let (x1, y1) = pair[0];
        let (x2, y2) = pair[1];

        let dx = x2 - x1;
        let dy = y2 - y1;
        let distance = (dx * dx + dy * dy).sqrt();
        let steps = (distance / radius.max(1.0)).ceil().max(1.0) as usize;

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            stamp(
                image,
                x1 + dx * t,
                y1 + dy * t,
                stroke.size,
                color,
            );
        }
    }
}

pub fn export_image(canvas: &Canvas, path: &Path) -> Result<(), String> {
    let width = canvas.width.clamp(1, 8192) as u32;
    let height = canvas.height.clamp(1, 8192) as u32;

    let mut image = RgbaImage::from_pixel(
        width,
        height,
        Rgba(canvas.background),
    );

    for layer in &canvas.layers {
        if !layer.visible {
            continue;
        }

        for stroke in &layer.strokes {
            draw_stroke(&mut image, stroke);
        }
    }

    let extension = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let format = match extension.as_str() {
        "png" => ImageFormat::Png,
        "bmp" => ImageFormat::Bmp,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        _ => return Err("Поддерживаются PNG, BMP и JPEG".into()),
    };

    let result = if format == ImageFormat::Jpeg {
        DynamicImage::ImageRgba8(image)
            .to_rgb8()
            .save_with_format(path, format)
    } else {
        DynamicImage::ImageRgba8(image)
            .save_with_format(path, format)
    };

    result.map_err(|e| e.to_string())
}
