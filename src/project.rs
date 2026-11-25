use serde::{Deserialize, Serialize};
use crate::layer::Layer;

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub width: usize,
    pub height: usize,
    pub background: [u8; 4],
    pub layers: Vec<Layer>,
    pub active_layer: usize,
}

impl Project {
    pub fn from_canvas(canvas: &crate::canvas::Canvas) -> Self {
        Self {
            version: 3,
            width: canvas.width,
            height: canvas.height,
            background: canvas.background,
            layers: canvas.layers.clone(),
            active_layer: canvas.active_layer,
        }
    }

    pub fn into_canvas(self) -> crate::canvas::Canvas {
        let mut canvas = crate::canvas::Canvas::new(
            self.width,
            self.height,
        );

        canvas.background = self.background;
        canvas.layers = self.layers;
        canvas.active_layer = self.active_layer;
        canvas.ensure_valid();

        canvas
    }
}
