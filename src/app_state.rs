use std::path::PathBuf;

use crate::coordinate::SizePixels;

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
impl SpriteInfo {
    pub const fn right(&self) -> u32 {
        self.left + self.width
    }

    pub const fn bottom(&self) -> u32 {
        self.top + self.height
    }
}

pub struct AppState {
    atlas_size: SizePixels,
    atlas_size_view_feedbacks: Vec<Box<dyn FnMut(&SizePixels)>>,
    sprites: Vec<SpriteInfo>,
    sprites_view_feedbacks: Vec<Box<dyn FnMut(&[SpriteInfo])>>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            atlas_size: SizePixels {
                width: 32,
                height: 32,
            },
            atlas_size_view_feedbacks: Vec::new(),
            sprites: Vec::new(),
            sprites_view_feedbacks: Vec::new(),
        }
    }

    pub fn add_sprites(&mut self, sprites: impl IntoIterator<Item = SpriteInfo>) {
        let mut iter = sprites.into_iter();
        self.sprites.reserve(iter.size_hint().0);
        let mut max_required_size = self.atlas_size;
        while let Some(n) = iter.next() {
            // Power of Twoに丸める（そうするとUV計算が正確になるため）
            max_required_size.width = max_required_size.width.max(n.right()).next_power_of_two();
            max_required_size.height = max_required_size.height.max(n.bottom()).next_power_of_two();

            self.sprites.push(n);
        }

        if max_required_size != self.atlas_size {
            self.atlas_size = max_required_size;
            for cb in self.atlas_size_view_feedbacks.iter_mut() {
                cb(&self.atlas_size);
            }
        }

        for cb in self.sprites_view_feedbacks.iter_mut() {
            cb(&self.sprites);
        }
    }

    pub fn selected_sprites_with_index(
        &self,
    ) -> impl DoubleEndedIterator<Item = (usize, &SpriteInfo)> {
        self.sprites.iter().enumerate().filter(|(_, x)| x.selected)
    }

    pub fn set_sprite_offset(&mut self, index: usize, left_pixels: u32, top_pixels: u32) {
        let target_sprite = &mut self.sprites[index];
        target_sprite.left = left_pixels;
        target_sprite.top = top_pixels;

        // Sprite Atlasのサイズ調整
        let mut max_required_size = self.atlas_size;
        // Power of Twoに丸める（そうするとUV計算が正確になるため）
        max_required_size.width = max_required_size
            .width
            .max(target_sprite.right())
            .next_power_of_two();
        max_required_size.height = max_required_size
            .height
            .max(target_sprite.bottom())
            .next_power_of_two();
        if max_required_size != self.atlas_size {
            self.atlas_size = max_required_size;
            for cb in self.atlas_size_view_feedbacks.iter_mut() {
                cb(&self.atlas_size);
            }
        }

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

    pub fn deselect_sprite(&mut self) {
        for x in self.sprites.iter_mut() {
            x.selected = false;
        }

        for cb in self.sprites_view_feedbacks.iter_mut() {
            cb(&self.sprites);
        }
    }

    // TODO: unregister
    pub fn register_sprites_view_feedback(&mut self, mut fb: impl FnMut(&[SpriteInfo]) + 'static) {
        fb(&self.sprites);
        self.sprites_view_feedbacks.push(Box::new(fb));
    }

    // TODO: unregister
    pub fn register_atlas_size_view_feedback(&mut self, mut fb: impl FnMut(&SizePixels) + 'static) {
        fb(&self.atlas_size);
        self.atlas_size_view_feedbacks.push(Box::new(fb));
    }
}
