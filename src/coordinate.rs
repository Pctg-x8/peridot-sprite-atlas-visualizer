use windows::Foundation::Size;

pub const fn pixels_to_dip(pixels: u32, dpi: f32) -> f32 {
    pixels as f32 * 96.0 / dpi
}

pub const fn signed_pixels_to_dip(pixels: i32, dpi: f32) -> f32 {
    pixels as f32 * 96.0 / dpi
}

pub const fn dip_to_pixels(dip: f32, dpi: f32) -> f32 {
    dip * dpi / 96.0
}

pub const fn size_sq(x: f32) -> Size {
    Size {
        Width: x,
        Height: x,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizePixels {
    pub width: u32,
    pub height: u32,
}
impl SizePixels {
    pub const fn to_dip(&self, dpi: f32) -> Size {
        Size {
            Width: pixels_to_dip(self.width, dpi),
            Height: pixels_to_dip(self.height, dpi),
        }
    }
}

pub struct PointDIP {
    pub x: f32,
    pub y: f32,
}
impl PointDIP {
    pub const fn make_rel_from(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

pub struct RectDIP {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl RectDIP {
    pub const fn contains(&self, p: &PointDIP) -> bool {
        self.left <= p.x && p.x <= self.right && self.top <= p.y && p.y <= self.bottom
    }
}
