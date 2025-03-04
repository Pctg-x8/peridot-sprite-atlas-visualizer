use windows::Graphics::Effects::IGraphicsEffectSource;
use windows_core::HSTRING;

use crate::extra_bindings::Microsoft::Graphics::Canvas::{
    CanvasComposite,
    Effects::{ColorSourceEffect, CompositeEffect, GaussianBlurEffect, TintEffect},
};

pub struct GaussianBlurEffectParams<'s, Source> {
    pub source: Source,
    pub blur_amount: Option<f32>,
    pub name: Option<&'s HSTRING>,
}
impl<'s, Source> GaussianBlurEffectParams<'s, Source> {
    pub const fn new(source: Source) -> Self {
        Self {
            source,
            blur_amount: None,
            name: None,
        }
    }

    pub const fn blur_amount(mut self, amount: f32) -> Self {
        self.blur_amount = Some(amount);
        self
    }

    pub const fn blur_amount_px(self, amount_px: f32) -> Self {
        self.blur_amount(amount_px / 3.0)
    }

    pub const fn name(mut self, name: &'s HSTRING) -> Self {
        self.name = Some(name);
        self
    }
}
impl<'s, Source> GaussianBlurEffectParams<'s, Source>
where
    Source: windows_core::Param<IGraphicsEffectSource>,
{
    #[inline]
    pub fn instantiate(self) -> windows::core::Result<GaussianBlurEffect> {
        let x = GaussianBlurEffect::new()?;
        x.SetSource(self.source)?;
        if let Some(p) = self.blur_amount {
            x.SetBlurAmount(p)?;
        }
        if let Some(p) = self.name {
            x.SetName(p)?;
        }

        Ok(x)
    }
}

pub struct ColorSourceEffectParams {
    pub color: Option<windows::UI::Color>,
}
impl ColorSourceEffectParams {
    #[inline]
    pub fn instantiate(self) -> windows::core::Result<ColorSourceEffect> {
        let x = ColorSourceEffect::new()?;
        if let Some(p) = self.color {
            x.SetColor(p)?;
        }

        Ok(x)
    }
}

pub struct CompositeEffectParams<'a> {
    pub sources: &'a [IGraphicsEffectSource],
    pub mode: Option<CanvasComposite>,
}
impl<'a> CompositeEffectParams<'a> {
    pub const fn new(sources: &'a [IGraphicsEffectSource]) -> Self {
        Self {
            sources,
            mode: None,
        }
    }

    pub const fn mode(mut self, mode: CanvasComposite) -> Self {
        self.mode = Some(mode);
        self
    }
}
impl CompositeEffectParams<'_> {
    #[inline]
    pub fn instantiate(self) -> windows::core::Result<CompositeEffect> {
        let x = CompositeEffect::new()?;
        if !self.sources.is_empty() {
            let sources = x.Sources()?;
            for p in self.sources {
                sources.Append(p)?;
            }
        }
        if let Some(p) = self.mode {
            x.SetMode(p)?;
        }

        Ok(x)
    }
}

pub struct TintEffectParams<Source> {
    pub source: Source,
    pub color: Option<windows::UI::Color>,
}
impl<Source> TintEffectParams<Source>
where
    Source: windows_core::Param<IGraphicsEffectSource>,
{
    #[inline]
    pub fn instantiate(self) -> windows_core::Result<TintEffect> {
        let x = TintEffect::new()?;
        x.SetSource(self.source)?;
        if let Some(p) = self.color {
            x.SetColor(p)?;
        }

        Ok(x)
    }
}
