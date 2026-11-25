use egui::Color32;
use serde::{Deserialize, Serialize};
use crate::layer::Layer;

#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Brush,
    Eraser,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub points: Vec<(f32, f32)>,
    pub color: [u8; 4],
    pub size: f32,
}

#[derive(Clone)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub background: [u8; 4],
    pub layers: Vec<Layer>,
    pub active_layer: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width: width.clamp(1, 8192),
            height: height.clamp(1, 8192),
            background: [255, 255, 255, 255],
            layers: vec![Layer::new("Слой 1")],
            active_layer: 0,
        }
    }

    pub fn begin_stroke(
        &mut self,
        x: f32,
        y: f32,
        color: Color32,
        size: f32,
        tool: Tool,
    ) {
        let color = match tool {
            Tool::Brush => color,
            Tool::Eraser => Color32::from_rgba_unmultiplied(
                self.background[0],
                self.background[1],
                self.background[2],
                self.background[3],
            ),
        };

        let rgba = [color.r(), color.g(), color.b(), color.a()];

        if let Some(layer) = self.layers.get_mut(self.active_layer) {
            layer.strokes.push(Stroke {
                points: vec![(x, y)],
                color: rgba,
                size: size.clamp(1.0, 256.0),
            });
        }
    }

    pub fn continue_stroke(&mut self, x: f32, y: f32) {
        if let Some(layer) = self.layers.get_mut(self.active_layer) {
            if let Some(last) = layer.strokes.last_mut() {
                let point = (x, y);

                if last.points.last() != Some(&point) {
                    last.points.push(point);
                }
            }
        }
    }

    pub fn active_strokes(&self) -> &[Stroke] {
        self.layers
            .get(self.active_layer)
            .map(|l| l.strokes.as_slice())
            .unwrap_or(&[])
    }

    pub fn clear(&mut self) {
        if let Some(layer) = self.layers.get_mut(self.active_layer) {
            layer.strokes.clear();
        }
    }

    pub fn add_layer(&mut self) {
        let number = self.layers.len() + 1;
        self.layers.push(Layer::new(format!("Слой {number}")));
        self.active_layer = self.layers.len() - 1;
    }

    pub fn remove_active_layer(&mut self) {
        if self.layers.len() <= 1 {
            return;
        }

        self.layers.remove(self.active_layer);
        self.active_layer = self.active_layer.min(self.layers.len() - 1);
    }

    pub fn move_layer_up(&mut self) {
        if self.active_layer + 1 < self.layers.len() {
            self.layers.swap(self.active_layer, self.active_layer + 1);
            self.active_layer += 1;
        }
    }

    pub fn move_layer_down(&mut self) {
        if self.active_layer > 0 {
            self.layers.swap(self.active_layer, self.active_layer - 1);
            self.active_layer -= 1;
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width.clamp(1, 8192);
        self.height = height.clamp(1, 8192);
    }

    pub fn ensure_valid(&mut self) {
        if self.layers.is_empty() {
            self.layers.push(Layer::new("Слой 1"));
        }

        self.active_layer = self.active_layer.min(self.layers.len() - 1);
        self.width = self.width.clamp(1, 8192);
        self.height = self.height.clamp(1, 8192);
    }
}
