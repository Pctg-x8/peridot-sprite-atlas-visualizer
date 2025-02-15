use windows::Graphics::Effects::IGraphicsEffectSource;

use crate::extra_bindings::Microsoft::Graphics::Canvas::{
    CanvasComposite,
    Effects::{ColorSourceEffect, CompositeEffect, GaussianBlurEffect},
};

pub struct GaussianBlurEffectParams<Source> {
    pub source: Source,
    pub blur_amount: Option<f32>,
}
impl<Source> GaussianBlurEffectParams<Source>
where
    Source: windows::core::Param<IGraphicsEffectSource>,
{
    #[inline]
    pub fn instantiate(self) -> windows::core::Result<GaussianBlurEffect> {
        let x = GaussianBlurEffect::new()?;
        x.SetSource(self.source)?;
        if let Some(p) = self.blur_amount {
            x.SetBlurAmount(p)?;
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
