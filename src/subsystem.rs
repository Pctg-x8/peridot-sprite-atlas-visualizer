use core::{
    cell::{Cell, RefCell},
    hash::Hash,
};
use std::collections::HashMap;
use windows::{
    core::{h, implement, w, Interface},
    Win32::{
        Foundation::{BOOL, HMODULE},
        Graphics::{
            CompositionSwapchain::{
                CreatePresentationFactory, IPresentationFactory, IPresentationManager,
            },
            Direct2D::{
                D2D1CreateFactory, ID2D1Device, ID2D1Factory1, D2D1_DEBUG_LEVEL_WARNING,
                D2D1_FACTORY_OPTIONS, D2D1_FACTORY_TYPE_SINGLE_THREADED,
            },
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_DEBUG, D3D11_SDK_VERSION,
            },
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteFactory1, IDWriteFontCollection,
                IDWriteFontCollectionLoader, IDWriteFontCollectionLoader_Impl,
                IDWriteFontFileEnumerator, IDWriteFontFileEnumerator_Impl, IDWriteTextFormat,
                DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_WEIGHT_NORMAL,
            },
            Dxgi::IDXGIDevice,
        },
        System::WinRT::Composition::{ICompositorDesktopInterop, ICompositorInterop},
    },
    UI::Composition::{CompositionGraphicsDevice, Compositor},
};
use windows_core::HSTRING;

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
        unsafe {
            self.factory
                .CreateFontFileReference(w!("./resources/inter.ttc"), None)
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
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
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
}
