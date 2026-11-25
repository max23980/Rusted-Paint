mod canvas;
mod layer;
mod project;
mod file_io;
mod history;

use canvas::{Canvas, Tool};
use history::History;
use eframe::egui;
use std::path::PathBuf;

struct RustedPaint {
    canvas: Canvas,
    tool: Tool,
    color: egui::Color32,
    brush_size: f32,
    pan: egui::Vec2,
    drawing: bool,
    history: History,
    width_input: usize,
    height_input: usize,
    status: String,
}

impl Default for RustedPaint {
    fn default() -> Self {
        Self {
            canvas: Canvas::new(1000, 700),
            tool: Tool::Brush,
            color: egui::Color32::BLACK,
            brush_size: 4.0,
            pan: egui::Vec2::ZERO,
            drawing: false,
            history: History::new(),
            width_input: 1000,
            height_input: 700,
            status: "Готово".into(),
        }
    }
}

impl RustedPaint {
    fn save(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rusted Paint", &["rpaint"])
            .set_file_name("project.rpaint")
            .save_file()
        {
            match file_io::save_project(&self.canvas, &path) {
                Ok(()) => self.status = format!("Сохранено: {}", path.display()),
                Err(e) => self.status = format!("Ошибка: {e}"),
            }
        }
    }

    fn open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rusted Paint", &["rpaint"])
            .pick_file()
        {
            match file_io::load_project(&path) {
                Ok(canvas) => {
                    self.canvas = canvas;
                    self.width_input = self.canvas.width;
                    self.height_input = self.canvas.height;
                    self.pan = egui::Vec2::ZERO;
                    self.drawing = false;
                    self.history = History::new();
                    self.status = format!("Открыто: {}", path.display());
                }
                Err(e) => self.status = format!("Ошибка открытия: {e}"),
            }
        }
    }

    fn export(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .add_filter("BMP", &["bmp"])
            .add_filter("JPEG", &["jpg", "jpeg"])
            .set_file_name("image.png")
            .save_file()
        {
            match file_io::export_image(&self.canvas, &path) {
                Ok(()) => self.status = format!("Экспортировано: {}", path.display()),
                Err(e) => self.status = format!("Ошибка экспорта: {e}"),
            }
        }
    }

    fn undo(&mut self) {
        if let Some(canvas) = self.history.undo(&self.canvas) {
            self.canvas = canvas;
        }
    }

    fn redo(&mut self) {
        if let Some(canvas) = self.history.redo(&self.canvas) {
            self.canvas = canvas;
        }
    }
}

impl eframe::App for RustedPaint {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let undo_pressed = ctx.input(|i| {
            i.modifiers.command && i.key_pressed(egui::Key::Z)
        });

        let redo_pressed = ctx.input(|i| {
            i.modifiers.command && i.key_pressed(egui::Key::Y)
        });

        if undo_pressed {
            self.undo();
        }
        if redo_pressed {
            self.redo();
        }

        // ===== Панель инструментов =====
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("Rusted Paint 0.3");
                ui.separator();

                if ui.button("Сохранить").clicked() {
                    self.save();
                }
                if ui.button("Открыть").clicked() {
                    self.open();
                }
                if ui.button("Экспорт").clicked() {
                    self.export();
                }

                ui.separator();

                if ui.selectable_label(self.tool == Tool::Brush, "Кисть").clicked() {
                    self.tool = Tool::Brush;
                }
                if ui.selectable_label(self.tool == Tool::Eraser, "Ластик").clicked() {
                    self.tool = Tool::Eraser;
                }

                ui.label("Толщина");
                ui.add(egui::Slider::new(&mut self.brush_size, 1.0..=40.0));

                ui.color_edit_button_srgba(&mut self.color);

                for c in [
                    egui::Color32::BLACK,
                    egui::Color32::WHITE,
                    egui::Color32::RED,
                    egui::Color32::GREEN,
                    egui::Color32::BLUE,
                    egui::Color32::YELLOW,
                    egui::Color32::from_rgb(255, 128, 0),
                    egui::Color32::from_rgb(128, 0, 255),
                ] {
                    if ui.add(
                        egui::Button::new(" ")
                            .fill(c)
                            .min_size(egui::vec2(22.0, 22.0))
                    ).clicked() {
                        self.color = c;
                    }
                }

                if ui.button("Очистить").clicked() {
                    self.history.save(&self.canvas);
                    self.canvas.clear();
                }

                if ui.button("↶").on_hover_text("Отменить Ctrl+Z").clicked() {
                    self.undo();
                }
                if ui.button("↷").on_hover_text("Повторить Ctrl+Y").clicked() {
                    self.redo();
                }
            });
        });

        // ===== Панель слоёв =====
        egui::SidePanel::left("layers")
            .default_width(210.0)
            .show(ctx, |ui| {
                ui.heading("Слои");

                ui.horizontal(|ui| {
                    if ui.button("+").clicked() {
                        self.history.save(&self.canvas);
                        self.canvas.add_layer();
                    }

                    if ui.button("Удалить").clicked() {
                        if self.canvas.layers.len() > 1 {
                            self.history.save(&self.canvas);
                            self.canvas.remove_active_layer();
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("↑").clicked() {
                        self.history.save(&self.canvas);
                        self.canvas.move_layer_up();
                    }
                    if ui.button("↓").clicked() {
                        self.history.save(&self.canvas);
                        self.canvas.move_layer_down();
                    }
                });

                ui.separator();

                for index in (0..self.canvas.layers.len()).rev() {
                    let mut select = false;

                    ui.horizontal(|ui| {
                        let layer = &mut self.canvas.layers[index];

                        ui.checkbox(&mut layer.visible, "");

                        if ui.selectable_label(
                            self.canvas.active_layer == index,
                            &layer.name
                        ).clicked() {
                            select = true;
                        }
                    });

                    if select {
                        self.canvas.active_layer = index;
                    }
                }

                ui.separator();
                ui.label("Название активного слоя");

                if let Some(layer) =
                    self.canvas.layers.get_mut(self.canvas.active_layer)
                {
                    ui.text_edit_singleline(&mut layer.name);
                }

                ui.separator();
                ui.heading("Размер холста");

                ui.horizontal(|ui| {
                    ui.label("Ш:");
                    ui.add(egui::DragValue::new(&mut self.width_input)
                        .range(1..=8192));
                });

                ui.horizontal(|ui| {
                    ui.label("В:");
                    ui.add(egui::DragValue::new(&mut self.height_input)
                        .range(1..=8192));
                });

                if ui.button("Применить размер").clicked() {
                    self.history.save(&self.canvas);
                    self.canvas.resize(self.width_input, self.height_input);
                }

                ui.separator();
                ui.label(format!(
                    "{} × {}",
                    self.canvas.width,
                    self.canvas.height
                ));
            });

        // ===== Рабочая область =====
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("ЛКМ: рисование | СКМ: перемещение | Ctrl+Z: отмена");

            let viewport_size = ui.available_size().max(egui::Vec2::splat(1.0));

            let (viewport, response) = ui.allocate_exact_size(
                viewport_size,
                egui::Sense::click_and_drag(),
            );

            let canvas_rect = egui::Rect::from_min_size(
                viewport.min + self.pan,
                egui::vec2(
                    self.canvas.width as f32,
                    self.canvas.height as f32,
                ),
            );

            let (middle, primary, pressed, released, pointer, delta) =
                ctx.input(|i| (
                    i.pointer.button_down(egui::PointerButton::Middle),
                    i.pointer.button_down(egui::PointerButton::Primary),
                    i.pointer.button_pressed(egui::PointerButton::Primary),
                    i.pointer.button_released(egui::PointerButton::Primary),
                    i.pointer.interact_pos(),
                    i.pointer.delta(),
                ));

            if response.hovered() && middle {
                self.pan += delta;
                self.drawing = false;
                ctx.request_repaint();
            }

            if pressed && response.hovered() && !middle {
                if let Some(pos) = pointer {
                    if canvas_rect.contains(pos) {
                        self.history.save(&self.canvas);

                        let local = pos - canvas_rect.min;
                        self.canvas.begin_stroke(
                            local.x,
                            local.y,
                            self.color,
                            self.brush_size,
                            self.tool,
                        );

                        self.drawing = true;
                    }
                }
            }

            if primary && self.drawing && !middle {
                if let Some(pos) = pointer {
                    if canvas_rect.contains(pos) && viewport.contains(pos) {
                        let local = pos - canvas_rect.min;
                        self.canvas.continue_stroke(local.x, local.y);
                    }
                }

                ctx.request_repaint();
            }

            if released {
                self.drawing = false;
            }

            let painter = ui.painter_at(viewport);

            let bg = self.canvas.background;

            painter.rect_filled(
                canvas_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(
                    bg[0], bg[1], bg[2], bg[3],
                ),
            );

            for layer in &self.canvas.layers {
                if !layer.visible {
                    continue;
                }

                for stroke in &layer.strokes {
                    let color = egui::Color32::from_rgba_unmultiplied(
                        stroke.color[0],
                        stroke.color[1],
                        stroke.color[2],
                        stroke.color[3],
                    );

                    for pair in stroke.points.windows(2) {
                        let a = canvas_rect.min + egui::vec2(
                            pair[0].0, pair[0].1
                        );
                        let b = canvas_rect.min + egui::vec2(
                            pair[1].0, pair[1].1
                        );

                        painter.line_segment(
                            [a, b],
                            egui::Stroke::new(stroke.size, color),
                        );
                    }

                    if let Some(&(x, y)) = stroke.points.first() {
                        painter.circle_filled(
                            canvas_rect.min + egui::vec2(x, y),
                            stroke.size / 2.0,
                            color,
                        );
                    }
                }
            }

            painter.rect_stroke(
                canvas_rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::GRAY),
                egui::StrokeKind::Inside,
            );
        });

        // ===== Строка состояния =====
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 850.0])
            .with_min_inner_size([700.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rusted Paint",
        options,
        Box::new(|_cc| Ok(Box::new(RustedPaint::default()))),
    )
}
