use core::{
    cell::{Cell, RefCell},
    hash::Hash,
};
use std::collections::HashMap;
use windows::{
    Foundation::Size,
    Graphics::DirectX::{DirectXAlphaMode, DirectXPixelFormat},
    UI::Composition::{CompositionDrawingSurface, CompositionGraphicsDevice, Compositor},
    Win32::{
        Foundation::HMODULE,
        Graphics::{
            CompositionSwapchain::{
                CreatePresentationFactory, IPresentationFactory, IPresentationManager,
            },
            Direct2D::{
                Common::{D2D_POINT_2F, D2D_RECT_F, D2D1_COLOR_F},
                D2D1_DEBUG_LEVEL_WARNING, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_OPTIONS,
                D2D1_FACTORY_TYPE_MULTI_THREADED, D2D1_ROUNDED_RECT, D2D1CreateFactory,
                ID2D1Device, ID2D1Factory1,
            },
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_DEBUG, D3D11_SDK_VERSION,
                D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext,
            },
            DirectWrite::{
                DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_WEIGHT_NORMAL, DWriteCreateFactory, IDWriteFactory, IDWriteFactory1,
                IDWriteFontCollection, IDWriteFontCollectionLoader,
                IDWriteFontCollectionLoader_Impl, IDWriteFontFileEnumerator,
                IDWriteFontFileEnumerator_Impl, IDWriteTextFormat, IDWriteTextLayout,
            },
            Dxgi::IDXGIDevice,
        },
        System::WinRT::Composition::{ICompositorDesktopInterop, ICompositorInterop},
    },
    core::{Interface, h, implement, w},
};
use windows_core::{BOOL, HSTRING};
use windows_numerics::Matrix3x2;

use crate::{
    coordinate::{dip_to_pixels, signed_pixels_to_dip, size_sq},
    surface_helper::draw_2d,
};

#[implement(IDWriteFontCollectionLoader)]
struct AppFontCollectionLoader;
impl IDWriteFontCollectionLoader_Impl for AppFontCollectionLoader_Impl {
    fn CreateEnumeratorFromKey(
        &self,
        factory: windows::core::Ref<'_, IDWriteFactory>,
        _collectionkey: *const core::ffi::c_void,
        _collectionkeysize: u32,
    ) -> windows::core::Result<IDWriteFontFileEnumerator> {
        Ok(AppFontFileEnumerator {
            factory: factory.unwrap().clone(),
            ptr: Cell::new(0),
        }
        .into())
    }
}

#[implement(IDWriteFontFileEnumerator)]
struct AppFontFileEnumerator {
    factory: IDWriteFactory,
    ptr: Cell<usize>,
}
impl IDWriteFontFileEnumerator_Impl for AppFontFileEnumerator_Impl {
    fn GetCurrentFontFile(
        &self,
    ) -> windows_core::Result<windows::Win32::Graphics::DirectWrite::IDWriteFontFile> {
        match self.ptr.get() {
            1 => unsafe {
                self.factory
                    .CreateFontFileReference(w!("./resources/inter.ttc"), None)
            },
            _ => Err(windows_core::Error::from_hresult(
                windows::Win32::Foundation::E_FAIL,
            )),
        }
    }

    fn MoveNext(&self) -> windows_core::Result<BOOL> {
        self.ptr.set(self.ptr.get() + 1);
        Ok(BOOL(if self.ptr.get() <= 1 { 1 } else { 0 }))
    }
}

#[derive(Debug, Clone)]
pub struct TextFormatCacheKey {
    family_name: HSTRING,
    size: f32,
}
impl PartialEq for TextFormatCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.family_name.eq(&other.family_name) && self.size.to_bits().eq(&other.size.to_bits())
    }
}
impl Eq for TextFormatCacheKey {}
impl Hash for TextFormatCacheKey {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.family_name.hash(state);
        self.size.to_bits().hash(state);
    }
}

pub struct TextFormatStore {
    factory: IDWriteFactory1,
    app_fc: IDWriteFontCollection,
    cache: RefCell<HashMap<TextFormatCacheKey, IDWriteTextFormat>>,
}
impl TextFormatStore {
    pub fn new(factory: IDWriteFactory1, app_fc: IDWriteFontCollection) -> Self {
        Self {
            factory,
            app_fc,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(
        &self,
        family_name: &HSTRING,
        size: f32,
    ) -> windows::core::Result<IDWriteTextFormat> {
        let key = TextFormatCacheKey {
            family_name: family_name.clone(),
            size,
        };

        if let Some(x) = self.cache.borrow().get(&key).cloned() {
            // ある
            return Ok(x);
        }

        let x = unsafe {
            self.factory.CreateTextFormat(
                &key.family_name,
                Some(&self.app_fc),
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                key.size,
                w!("ja-JP"),
            )?
        };
        self.cache.borrow_mut().insert(key, x.clone());
        Ok(x)
    }
}

pub struct Subsystem {
    pub d3d11_device: ID3D11Device,
    pub d3d11_imm_context: ID3D11DeviceContext,
    pub d2d1_factory: ID2D1Factory1,
    pub d2d1_device: ID2D1Device,
    pub dwrite_factory: IDWriteFactory1,
    pub text_format_store: TextFormatStore,
    pub default_ui_format: IDWriteTextFormat,
    pub compositor: Compositor,
    pub compositor_interop: ICompositorInterop,
    pub compositor_desktop_interop: ICompositorDesktopInterop,
    pub composition_2d_graphics_device: CompositionGraphicsDevice,
    pub presentation_factory: IPresentationFactory,
    pub presentation_manager: IPresentationManager,
}
impl Subsystem {
    pub fn new() -> Self {
        let mut d3d11_device = core::mem::MaybeUninit::uninit();
        let mut feature_level = core::mem::MaybeUninit::uninit();
        let mut d3d11_imm_context = core::mem::MaybeUninit::uninit();
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE(core::ptr::null_mut()),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG,
                None,
                D3D11_SDK_VERSION,
                Some(d3d11_device.as_mut_ptr()),
                Some(feature_level.as_mut_ptr()),
                Some(d3d11_imm_context.as_mut_ptr()),
            )
            .expect("Failed to create D3D11 Device");
        }
        let d3d11_device = unsafe {
            d3d11_device
                .assume_init()
                .expect("no d3d11 device provided?")
        };
        let feature_level = unsafe { feature_level.assume_init() };
        let d3d11_imm_context = unsafe {
            d3d11_imm_context
                .assume_init()
                .expect("no d3d11 imm context provided?")
        };
        println!("d3d11 feature level = {feature_level:?}");

        let d2d1_factory: ID2D1Factory1 = unsafe {
            D2D1CreateFactory(
                D2D1_FACTORY_TYPE_MULTI_THREADED,
                Some(&D2D1_FACTORY_OPTIONS {
                    debugLevel: D2D1_DEBUG_LEVEL_WARNING,
                }),
            )
            .expect("Failed to create d2d1 factory")
        };
        let d2d1_device = unsafe {
            d2d1_factory
                .CreateDevice(&d3d11_device.cast::<IDXGIDevice>().expect("no dxgi device?"))
                .expect("Failed to create d2d1 device")
        };

        let dwrite_factory: IDWriteFactory1 = unsafe {
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .expect("Failed to create dwrite factory")
        };
        let app_fc_loader: IDWriteFontCollectionLoader = AppFontCollectionLoader.into();
        unsafe {
            dwrite_factory
                .RegisterFontCollectionLoader(&app_fc_loader)
                .unwrap();
        }
        let key = 0u32;
        let app_font_collection = unsafe {
            dwrite_factory
                .CreateCustomFontCollection(
                    &app_fc_loader,
                    &key as *const _ as _,
                    core::mem::size_of_val(&key) as _,
                )
                .unwrap()
        };
        let text_format_store = TextFormatStore::new(dwrite_factory.clone(), app_font_collection);
        let default_ui_format = text_format_store.get(h!("Inter Display"), 12.0).unwrap();

        let compositor = Compositor::new().expect("Failed to create ui compositor");
        let compositor_interop = compositor
            .cast::<ICompositorInterop>()
            .expect("no compositor interop support");
        let compositor_desktop_interop = compositor
            .cast::<ICompositorDesktopInterop>()
            .expect("no compositor desktop interop");
        let composition_2d_graphics_device = unsafe {
            compositor_interop
                .CreateGraphicsDevice(&d2d1_device)
                .expect("Failed to create composition 2d graphics device")
        };

        let presentation_factory: IPresentationFactory =
            unsafe { CreatePresentationFactory(&d3d11_device).unwrap() };
        if unsafe { presentation_factory.IsPresentationSupportedWithIndependentFlip() == 0 } {
            unimplemented!("Presentation with independent flip is not supported on this machine");
        }
        let presentation_manager =
            unsafe { presentation_factory.CreatePresentationManager().unwrap() };

        Self {
            d3d11_device,
            d3d11_imm_context,
            d2d1_factory,
            d2d1_device,
            dwrite_factory,
            text_format_store,
            default_ui_format,
            compositor,
            compositor_interop,
            compositor_desktop_interop,
            composition_2d_graphics_device,
            presentation_factory,
            presentation_manager,
        }
    }

    #[inline]
    pub fn new_text_layout_unrestricted(
        &self,
        text: &str,
        format: impl windows_core::Param<IDWriteTextFormat>,
    ) -> windows_core::Result<IDWriteTextLayout> {
        unsafe {
            self.dwrite_factory.CreateTextLayout(
                &text.encode_utf16().collect::<Vec<_>>(),
                format,
                f32::MAX,
                f32::MAX,
            )
        }
    }

    #[inline]
    pub fn new_2d_drawing_surface(
        &self,
        size: Size,
    ) -> windows_core::Result<CompositionDrawingSurface> {
        tracing::trace!("gen {}x{} 2d surface", size.Width, size.Height);

        self.composition_2d_graphics_device.CreateDrawingSurface(
            size,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            DirectXAlphaMode::Premultiplied,
        )
    }

    #[inline]
    pub fn new_2d_mask_surface(
        &self,
        size: Size,
    ) -> windows_core::Result<CompositionDrawingSurface> {
        tracing::trace!("gen {}x{} mask surface", size.Width, size.Height);

        self.composition_2d_graphics_device.CreateDrawingSurface(
            size,
            DirectXPixelFormat::A8UIntNormalized,
            DirectXAlphaMode::Premultiplied,
        )
    }

    pub fn gen_text_surface(
        &self,
        dpi: f32,
        text: &IDWriteTextLayout,
        color: &D2D1_COLOR_F,
    ) -> windows_core::Result<CompositionDrawingSurface> {
        let mut metrics = core::mem::MaybeUninit::uninit();
        let metrics = unsafe {
            text.GetMetrics(metrics.as_mut_ptr())?;
            metrics.assume_init()
        };

        let surface = self.new_2d_drawing_surface(Size {
            Width: dip_to_pixels(metrics.width, dpi),
            Height: dip_to_pixels(metrics.height, dpi),
        })?;
        draw_2d(&surface, |dc, offset| {
            unsafe {
                dc.SetDpi(dpi, dpi);

                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: signed_pixels_to_dip(offset.x, dpi),
                        y: signed_pixels_to_dip(offset.y, dpi),
                    },
                    text,
                    &dc.CreateSolidColorBrush(color, None)?,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })?;

        Ok(surface)
    }

    pub fn rounded_rect_mask_surface(
        &self,
        dpi: f32,
        round_dip: f32,
    ) -> windows_core::Result<CompositionDrawingSurface> {
        let surface =
            self.new_2d_mask_surface(size_sq(dip_to_pixels(round_dip * 2.0 + 1.0, dpi)))?;
        draw_2d(&surface, |dc, offset| {
            unsafe {
                dc.SetDpi(dpi, dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    signed_pixels_to_dip(offset.x, dpi),
                    signed_pixels_to_dip(offset.y, dpi),
                ));

                dc.Clear(None);
                dc.FillRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: 0.0,
                            top: 0.0,
                            right: round_dip * 2.0 + 1.0,
                            bottom: round_dip * 2.0 + 1.0,
                        },
                        radiusX: round_dip,
                        radiusY: round_dip,
                    },
                    &dc.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            a: 1.0,
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                        },
                        None,
                    )?,
                );
            }

            Ok::<_, windows_core::Error>(())
        })?;

        Ok(surface)
    }
}
