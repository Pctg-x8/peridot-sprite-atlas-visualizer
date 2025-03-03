use std::path::PathBuf;

#[derive(Debug)]
pub struct SpriteInfo {
    pub name: String,
    pub source_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub left: u32,
    pub top: u32,
    pub left_slice: u32,
    pub right_slice: u32,
    pub top_slice: u32,
    pub bottom_slice: u32,
    pub selected: bool,
}

pub struct AppState {
    pub atlas_width: u32,
    pub atlas_height: u32,
    sprites: Vec<SpriteInfo>,
    sprites_view_feedbacks: Vec<Box<dyn FnMut(&[SpriteInfo])>>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            atlas_width: 32,
            atlas_height: 32,
            sprites: Vec::new(),
            sprites_view_feedbacks: Vec::new(),
        }
    }

    pub fn add_sprites(&mut self, sprites: impl IntoIterator<Item = SpriteInfo>) {
        self.sprites.extend(sprites);

        for cb in self.sprites_view_feedbacks.iter_mut() {
            cb(&self.sprites);
        }
    }

    pub fn select_sprite(&mut self, index: usize) {
        for (n, x) in self.sprites.iter_mut().enumerate() {
            x.selected = n == index;
        }

        for cb in self.sprites_view_feedbacks.iter_mut() {
            cb(&self.sprites);
        }
    }

    // TODO: unregister
    pub fn register_sprites_view_feedback(&mut self, fb: impl FnMut(&[SpriteInfo]) + 'static) {
        let boxed = Box::new(fb);
        self.sprites_view_feedbacks.push(boxed);
    }
}
