use serde::Serialize;


#[derive(Debug, Serialize, Clone, Copy)]
pub struct BBox {
    pub x_min  : f64,
    pub y_min  : f64,
    pub x_max  : f64,
    pub y_max  : f64,
}

impl BBox {
    pub fn new(x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self { Self { x_min, y_min, x_max, y_max } }

    /// This assumes the baseline is at y = 0
    pub fn from_typographic(x_min: f64, depth: f64, x_max: f64, height: f64) -> Self { 
        // height is signed distance from baseline to top of the glyph's bounding box
        // height > 0 means that top of bouding box is above baseline (i.e. y_min)
        // above in the screen's coordinate system means Y < 0
        // So y_min = - height
        // Similar reasoning for depth
        Self { x_min, y_min : -height, x_max, y_max : -depth } 
    }

    #[inline]
    pub fn width(&self) -> f64 { self.x_max - self.x_min }

    #[inline]
    pub fn height(&self) -> f64 { self.y_max - self.y_min }
}


#[derive(Debug, Serialize)]
pub struct Metrics {
    pub bbox      : BBox,
    pub baseline  : f64,
    pub font_size : f64,
}

