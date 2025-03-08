use core::mem::MaybeUninit;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use app_state::{AppState, SpriteInfo};
use bg_worker::{
    BackgroundWork, BackgroundWorker, BackgroundWorkerEnqueueAccess,
    BackgroundWorkerEnqueueWeakAccess, BackgroundWorkerViewFeedback,
};
use color_factory::{
    d2d1_color_f_from_hex_argb, d2d1_color_f_from_hex_rgb, d2d1_color_f_from_websafe_hex_argb,
    d2d1_color_f_from_websafe_hex_rgb, ui_color_from_hex_rgb,
    ui_color_from_websafe_hex_rgb_with_alpha,
};
use component::{app_header::AppHeaderPresenter, dnd_overlay::FileDragAndDropOverlayView};
use composition_element_builder::{
    CompositionMaskBrushParams, CompositionNineGridBrushParams, CompositionSurfaceBrushParams,
    ContainerVisualParams, SimpleImplicitAnimationParams, SimpleScalarAnimationParams,
    SpriteVisualParams,
};
use coordinate::*;
use effect_builder::{
    ColorSourceEffectParams, CompositeEffectParams, GaussianBlurEffectParams, TintEffectParams,
};
use hittest::HitTestTreeActionHandler;
use hittest::*;
use image::EncodableLayout;
use input::*;
use native_wrapper::NativeEvent;
use parking_lot::RwLock;
use subsystem::Subsystem;
use surface_helper::draw_2d;
use timespan_helper::timespan_ms;
use windows::{
    Foundation::{Size, TimeSpan},
    Graphics::Effects::IGraphicsEffect,
    Storage::{
        FileAccessMode,
        Streams::{IRandomAccessStream, RandomAccessStream},
    },
    UI::{
        Color,
        Composition::{
            CompositionAnimationGroup, CompositionBrush, CompositionDrawingSurface,
            CompositionEasingFunction, CompositionEasingFunctionMode, CompositionEffectBrush,
            CompositionEffectSourceParameter, CompositionPropertySet, CompositionStretch,
            ContainerVisual, Desktop::DesktopWindowTarget, ScalarKeyFrameAnimation, SpriteVisual,
            VisualCollection,
        },
    },
    Win32::{
        Foundation::{HGLOBAL, HWND, LPARAM, LRESULT, POINT, RECT, WAIT_OBJECT_0, WPARAM},
        Graphics::{
            Direct2D::{
                Common::{
                    D2D_POINT_2F, D2D_RECT_F, D2D_SIZE_F, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED,
                    D2D1_FIGURE_END_CLOSED,
                },
                D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_ROUNDED_RECT, D2D1_TRIANGLE,
                ID2D1DeviceContext5, ID2D1Multithread,
            },
            Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
            Direct3D11::{
                D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_VERTEX_BUFFER,
                D3D11_BLEND_DESC, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
                D3D11_BOX, D3D11_BUFFER_DESC, D3D11_COMPARISON_ALWAYS, D3D11_CPU_ACCESS_READ,
                D3D11_CPU_ACCESS_WRITE, D3D11_FILTER_MIN_MAG_MIP_POINT, D3D11_INPUT_ELEMENT_DESC,
                D3D11_INPUT_PER_INSTANCE_DATA, D3D11_INPUT_PER_VERTEX_DATA, D3D11_MAP_WRITE,
                D3D11_MAP_WRITE_DISCARD, D3D11_RENDER_TARGET_BLEND_DESC, D3D11_SAMPLER_DESC,
                D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_TEXTURE2D_DESC,
                D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_USAGE_IMMUTABLE,
                D3D11_USAGE_STAGING, D3D11_VIEWPORT, ID3D11BlendState, ID3D11Buffer, ID3D11Device,
                ID3D11DeviceContext, ID3D11InputLayout, ID3D11Multithread, ID3D11PixelShader,
                ID3D11SamplerState, ID3D11ShaderResourceView, ID3D11Texture2D, ID3D11VertexShader,
            },
            DirectWrite::{DWRITE_FONT_WEIGHT_MEDIUM, DWRITE_TEXT_RANGE, IDWriteTextLayout1},
            Dwm::{
                DWMWA_USE_IMMERSIVE_DARK_MODE, DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
            },
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM,
                    DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_SAMPLE_DESC,
                },
                DXGI_PRESENT, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIAdapter, IDXGIDevice2, IDXGIFactory2,
                IDXGISwapChain2,
            },
            Gdi::{HBRUSH, MapWindowPoints},
        },
        Storage::Packaging::Appx::PACKAGE_VERSION,
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, CoCreateInstance, DVASPECT_CONTENT, FORMATETC, IStream,
                STGMEDIUM, TYMED_HGLOBAL,
            },
            LibraryLoader::GetModuleHandleW,
            Memory::{GlobalLock, GlobalUnlock},
            Ole::{
                CF_HDROP, DROPEFFECT_LINK, IDropTarget, IDropTarget_Impl, OleInitialize,
                RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop,
            },
            Threading::{INFINITE, WaitForMultipleObjectsEx},
            WinRT::{
                CreateDispatcherQueueController, CreateRandomAccessStreamOnFile,
                CreateStreamOverRandomAccessStream, DQTAT_COM_ASTA, DQTYPE_THREAD_CURRENT,
                DispatcherQueueOptions,
            },
        },
        UI::{
            Controls::MARGINS,
            HiDpi::GetDpiForWindow,
            Shell::{CLSID_DragDropHelper, DragQueryFileW, HDROP, IDropTargetHelper},
            WindowsAndMessaging::{
                CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA,
                GetClientRect, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, HCURSOR,
                HTCLIENT, HTTOP, IDC_ARROW, IDC_SIZEWE, IDI_APPLICATION, LoadCursorW, LoadIconW,
                MsgWaitForMultipleObjects, NCCALCSIZE_PARAMS, PM_REMOVE, PeekMessageW,
                PostQuitMessage, QS_ALLINPUT, RegisterClassExW, SM_CXSIZEFRAME, SM_CYSIZEFRAME,
                SW_SHOW, SWP_FRAMECHANGED, SetCursor, SetWindowLongPtrW, SetWindowPos, ShowWindow,
                TranslateMessage, WM_ACTIVATE, WM_CREATE, WM_DESTROY, WM_DPICHANGED,
                WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCALCSIZE, WM_NCHITTEST, WM_QUIT,
                WM_SETCURSOR, WM_SIZE, WNDCLASS_STYLES, WNDCLASSEXW, WS_EX_APPWINDOW,
                WS_EX_NOREDIRECTIONBITMAP, WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
    core::{HRESULT, Interface, PCWSTR, h, s, w},
};
use windows_core::{BOOL, HSTRING, implement};
use windows_numerics::{Matrix3x2, Vector2, Vector3};

mod app_state;
mod bg_worker;
mod color_factory;
mod component;
mod composition_element_builder;
mod coordinate;
mod effect_builder;
mod extra_bindings;
mod hittest;
mod input;
mod native_wrapper;
mod peridot;
mod source_reader;
mod subsystem;
mod surface_helper;
mod timespan_helper;

type AppHitTestTreeManager = HitTestTreeManager<AppState>;

#[macro_export]
macro_rules! scoped_try {
    ($label: tt, $x: expr) => {
        match $x {
            Ok(v) => v,
            Err(e) => break $label Err(e)
        }
    }
}

// windows app sdk bootstrapping
#[repr(C)]
#[derive(Clone, Copy)]
enum MddBootstrapInitializeOptions {
    ShowUI = 0x08,
}
// copy from WindowsAppSDK-VersionInfo.h
const APP_SDK_VERSION_U64: u64 = 0;

#[link(name = "Microsoft.WindowsAppRuntime.Bootstrap")]
unsafe extern "system" {
    unsafe fn MddBootstrapInitialize2(
        majorMinorVersion: u32,
        versionTag: PCWSTR,
        minVersion: PACKAGE_VERSION,
        options: MddBootstrapInitializeOptions,
    ) -> HRESULT;

    unsafe fn MddBootstrapShutdown();
}

struct AppRuntime;
impl AppRuntime {
    #[inline(always)]
    pub fn init() -> windows::core::Result<Self> {
        unsafe {
            MddBootstrapInitialize2(
                0x00010006,
                w!(""),
                core::mem::transmute(APP_SDK_VERSION_U64),
                MddBootstrapInitializeOptions::ShowUI,
            )
            .ok()?;
        }

        Ok(Self)
    }
}
impl Drop for AppRuntime {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            MddBootstrapShutdown();
        }
    }
}

struct D3D11CriticalSectionGuard<'a>(&'a ID3D11Multithread);
impl<'a> Drop for D3D11CriticalSectionGuard<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.0.Leave();
        }
    }
}
impl<'a> D3D11CriticalSectionGuard<'a> {
    #[inline(always)]
    pub fn enter(mt: &'a ID3D11Multithread) -> Self {
        unsafe {
            mt.Enter();
        }
        Self(mt)
    }
}

struct D2D1CriticalSectionGuard<'a>(&'a ID2D1Multithread);
impl<'a> Drop for D2D1CriticalSectionGuard<'a> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            self.0.Leave();
        }
    }
}
impl<'a> D2D1CriticalSectionGuard<'a> {
    #[inline(always)]
    pub fn enter(mt: &'a ID2D1Multithread) -> Self {
        unsafe {
            mt.Enter();
        }
        Self(mt)
    }
}

const BG_COLOR: Color = ui_color_from_hex_rgb(0x202030);

pub trait DpiHandler {
    #[allow(unused_variables)]
    fn on_dpi_changed(&self, new_dpi: f32) {}
}

pub struct PresenterInitContext<'r> {
    pub for_view: ViewInitContext<'r>,
    pub dpi_handlers: &'r mut Vec<std::rc::Weak<dyn DpiHandler>>,
    pub app_state: &'r Rc<RefCell<AppState>>,
}
pub struct ViewInitContext<'r> {
    pub subsystem: &'r Rc<Subsystem>,
    pub ht: &'r Rc<RefCell<AppHitTestTreeManager>>,
    pub dpi: f32,
    pub background_worker_enqueue_access: &'r BackgroundWorkerEnqueueAccess,
    pub background_worker_view_update_callback:
        &'r Rc<RefCell<Vec<Box<dyn FnMut(&[Option<String>])>>>>,
}
impl ViewInitContext<'_> {
    #[inline(always)]
    pub const fn dip_to_pixels(&self, dip: f32) -> f32 {
        dip_to_pixels(dip, self.dpi)
    }

    #[inline(always)]
    pub const fn signed_pixels_to_dip(&self, pixels: i32) -> f32 {
        signed_pixels_to_dip(pixels, self.dpi)
    }
}

#[repr(C, align(16))]
pub struct AtlasBaseGridRenderParams {
    pub pixel_size: [f32; 2],
    pub grid_offset: [f32; 2],
    pub grid_size: f32,
}

pub struct SimpleTextureAtlas {
    pub resource: ID3D11Texture2D,
    pub srv: ID3D11ShaderResourceView,
    pub current_top: u32,
    pub current_left: u32,
    pub max_height: u32,
}
impl SimpleTextureAtlas {
    const SIZE: u32 = 4096;

    pub fn new(d3d11: &ID3D11Device) -> Self {
        let mut resource = core::mem::MaybeUninit::uninit();
        let mut srv = MaybeUninit::uninit();
        unsafe {
            d3d11
                .CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Width: Self::SIZE,
                        Height: Self::SIZE,
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as _,
                        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as _,
                        MiscFlags: 0,
                    },
                    None,
                    Some(resource.as_mut_ptr()),
                )
                .unwrap();
            d3d11
                .CreateShaderResourceView(
                    resource.assume_init_ref().as_ref().unwrap(),
                    None,
                    Some(srv.as_mut_ptr()),
                )
                .unwrap();
        }
        let resource = unsafe { resource.assume_init().unwrap() };
        let srv = unsafe { srv.assume_init().unwrap() };

        Self {
            resource,
            srv,
            current_top: 0,
            current_left: 0,
            max_height: 0,
        }
    }

    pub fn alloc(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if (Self::SIZE - self.current_top) < height {
            // 高さが足りない
            return None;
        }

        if width <= (Self::SIZE - self.current_left) {
            // まだ入る
            let o = (self.current_left, self.current_top);
            self.current_left += width;
            self.max_height = self.max_height.max(height);
            Some(o)
        } else {
            // 改行が必要
            if (Self::SIZE - self.current_top - self.max_height) < height {
                // 改行すると入らなくなる
                return None;
            }

            let o = (self.current_left, self.current_top);
            self.current_top += self.max_height;
            self.current_left = 0;
            self.max_height = height;
            Some(o)
        }
    }
}

#[repr(C)]
pub struct SpriteInstance {
    pub pos_st: [f32; 4],
    pub uv_st: [f32; 4],
}

pub struct SpriteInstanceBuffer {
    buffer: ID3D11Buffer,
    staging: ID3D11Buffer,
    capacity: usize,
    count: usize,
    is_dirty: bool,
}

pub struct AtlasBaseGridView {
    root: SpriteVisual,
    swapchain: IDXGISwapChain2,
    vsh: ID3D11VertexShader,
    psh: ID3D11PixelShader,
    render_params_cb: ID3D11Buffer,
    texture_preview_vb: ID3D11Buffer,
    texture_preview_cb: ID3D11Buffer,
    texture_preview_vsh: ID3D11VertexShader,
    texture_preview_psh: ID3D11PixelShader,
    sprite_instance_vsh: ID3D11VertexShader,
    sprite_instance_psh: ID3D11PixelShader,
    sprite_instance_base_vb: ID3D11Buffer,
    sprite_instance_input_layout: ID3D11InputLayout,
    sprite_instance_buffer: RwLock<SpriteInstanceBuffer>,
    atlas_bg_vsh: ID3D11VertexShader,
    atlas_bg_psh: ID3D11PixelShader,
    atlas_bg_input_layout: ID3D11InputLayout,
    atlas_bg_vb: ID3D11Buffer,
    atlas_bg_vertices: RwLock<[[f32; 2]; 4]>,
    atlas_bg_vertices_dirty: RwLock<bool>,
    tex_sampler: ID3D11SamplerState,
    size_pixels: RwLock<(u32, u32)>,
    resize_order: RwLock<Option<(u32, u32)>>,
    offset_pixels: RwLock<(f32, f32)>,
    background_worker_enqueue_access: BackgroundWorkerEnqueueWeakAccess,
    simple_atlas: RwLock<SimpleTextureAtlas>,
    sprite_source_offset: RwLock<HashMap<PathBuf, (u32, u32)>>,
    d3d11_device: ID3D11Device,
    d3d11_device_context: ID3D11DeviceContext,
    d3d11_mt: ID3D11Multithread,
    d2d1_mt: ID2D1Multithread,
    premul_blend_state: ID3D11BlendState,
}
impl AtlasBaseGridView {
    const SPRITE_INSTANCE_CAPACITY_UNIT: usize = 128;

    pub fn new(
        init: &mut ViewInitContext,
        init_width_pixels: u32,
        init_height_pixels: u32,
    ) -> Self {
        let sc = unsafe {
            init.subsystem
                .d3d11_device
                .cast::<IDXGIDevice2>()
                .unwrap()
                .GetParent::<IDXGIAdapter>()
                .unwrap()
                .GetParent::<IDXGIFactory2>()
                .unwrap()
                .CreateSwapChainForComposition(
                    &init.subsystem.d3d11_device,
                    &DXGI_SWAP_CHAIN_DESC1 {
                        Width: init_width_pixels,
                        Height: init_height_pixels,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        Stereo: BOOL(0),
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                        BufferCount: 2,
                        // Composition向けのSwapchainはStretchじゃないとだめらしい
                        Scaling: DXGI_SCALING_STRETCH,
                        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                        AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                        Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as _,
                    },
                    None,
                )
                .unwrap()
        };
        let sc = sc.cast::<IDXGISwapChain2>().unwrap();
        let sc_surface = unsafe {
            init.subsystem
                .compositor_interop
                .CreateCompositionSurfaceForSwapChain(&sc)
                .unwrap()
        };

        let root = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&sc_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size(Vector2 {
            X: init_width_pixels as _,
            Y: init_height_pixels as _,
        })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let mut vsh = core::mem::MaybeUninit::uninit();
        let mut psh = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateVertexShader(
                    &std::fs::read("./resources/grid/vsh.fxc").unwrap(),
                    None,
                    Some(vsh.as_mut_ptr()),
                )
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreatePixelShader(
                    &std::fs::read("./resources/grid/psh.fxc").unwrap(),
                    None,
                    Some(psh.as_mut_ptr()),
                )
                .unwrap();
        }
        let vsh = unsafe { vsh.assume_init().unwrap() };
        let psh = unsafe { psh.assume_init().unwrap() };

        let mut render_params_cb = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: core::mem::size_of::<AtlasBaseGridRenderParams>() as _,
                        Usage: D3D11_USAGE_DYNAMIC,
                        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as _,
                        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as _,
                        MiscFlags: 0,
                        StructureByteStride: core::mem::size_of::<AtlasBaseGridRenderParams>() as _,
                    },
                    None,
                    Some(render_params_cb.as_mut_ptr()),
                )
                .unwrap();
        }
        let render_params_cb = unsafe { render_params_cb.assume_init().unwrap() };

        let mut premul_blend_state = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateBlendState(
                    &D3D11_BLEND_DESC {
                        AlphaToCoverageEnable: BOOL(0),
                        IndependentBlendEnable: BOOL(0),
                        RenderTarget: [
                            D3D11_RENDER_TARGET_BLEND_DESC {
                                BlendEnable: BOOL(1),
                                SrcBlend: D3D11_BLEND_ONE,
                                DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
                                BlendOp: D3D11_BLEND_OP_ADD,
                                SrcBlendAlpha: D3D11_BLEND_ONE,
                                DestBlendAlpha: D3D11_BLEND_INV_SRC_ALPHA,
                                BlendOpAlpha: D3D11_BLEND_OP_ADD,
                                RenderTargetWriteMask: 0x0f,
                            },
                            D3D11_RENDER_TARGET_BLEND_DESC::default(),
                            D3D11_RENDER_TARGET_BLEND_DESC::default(),
                            D3D11_RENDER_TARGET_BLEND_DESC::default(),
                            D3D11_RENDER_TARGET_BLEND_DESC::default(),
                            D3D11_RENDER_TARGET_BLEND_DESC::default(),
                            D3D11_RENDER_TARGET_BLEND_DESC::default(),
                            D3D11_RENDER_TARGET_BLEND_DESC::default(),
                        ],
                    },
                    Some(premul_blend_state.as_mut_ptr()),
                )
                .unwrap();
        }
        let premul_blend_state = unsafe { premul_blend_state.assume_init().unwrap() };

        let mut tex_sampler = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateSamplerState(
                    &D3D11_SAMPLER_DESC {
                        Filter: D3D11_FILTER_MIN_MAG_MIP_POINT,
                        AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
                        AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
                        AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
                        MipLODBias: 0.0,
                        MaxAnisotropy: 1,
                        ComparisonFunc: D3D11_COMPARISON_ALWAYS,
                        BorderColor: [0.0; 4],
                        MinLOD: 0.0,
                        MaxLOD: 0.0,
                    },
                    Some(tex_sampler.as_mut_ptr()),
                )
                .unwrap();
        }
        let tex_sampler = unsafe { tex_sampler.assume_init().unwrap() };

        let simple_atlas = RwLock::new(SimpleTextureAtlas::new(&init.subsystem.d3d11_device));

        let d3d11_mt: ID3D11Multithread = init.subsystem.d3d11_imm_context.cast().unwrap();
        unsafe {
            let _ = d3d11_mt.SetMultithreadProtected(true);
        }

        let mut sprite_instance_vsh = core::mem::MaybeUninit::uninit();
        let mut sprite_instance_psh = core::mem::MaybeUninit::uninit();
        let mut sprite_instance_input_layout = core::mem::MaybeUninit::uninit();
        let vsh_code = std::fs::read("resources/sprite_instance/vsh.fxc").unwrap();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateVertexShader(&vsh_code, None, Some(sprite_instance_vsh.as_mut_ptr()))
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreatePixelShader(
                    &std::fs::read("resources/sprite_instance/psh.fxc").unwrap(),
                    None,
                    Some(sprite_instance_psh.as_mut_ptr()),
                )
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreateInputLayout(
                    &[
                        D3D11_INPUT_ELEMENT_DESC {
                            SemanticName: s!("POSITION"),
                            SemanticIndex: 0,
                            Format: DXGI_FORMAT_R32G32_FLOAT,
                            InputSlot: 0,
                            AlignedByteOffset: 0,
                            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                            InstanceDataStepRate: 0,
                        },
                        D3D11_INPUT_ELEMENT_DESC {
                            SemanticName: s!("POSITION"),
                            SemanticIndex: 1,
                            Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                            InputSlot: 1,
                            AlignedByteOffset: core::mem::offset_of!(SpriteInstance, pos_st) as _,
                            InputSlotClass: D3D11_INPUT_PER_INSTANCE_DATA,
                            InstanceDataStepRate: 1,
                        },
                        D3D11_INPUT_ELEMENT_DESC {
                            SemanticName: s!("TEXCOORD"),
                            SemanticIndex: 0,
                            Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                            InputSlot: 1,
                            AlignedByteOffset: core::mem::offset_of!(SpriteInstance, uv_st) as _,
                            InputSlotClass: D3D11_INPUT_PER_INSTANCE_DATA,
                            InstanceDataStepRate: 1,
                        },
                    ],
                    &vsh_code,
                    Some(sprite_instance_input_layout.as_mut_ptr()),
                )
                .unwrap();
        }
        let sprite_instance_vsh = unsafe { sprite_instance_vsh.assume_init().unwrap() };
        let sprite_instance_psh = unsafe { sprite_instance_psh.assume_init().unwrap() };
        let sprite_instance_input_layout =
            unsafe { sprite_instance_input_layout.assume_init().unwrap() };

        let mut sprite_instance_base_vb = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: core::mem::size_of::<[[f32; 2]; 4]>() as _,
                        Usage: D3D11_USAGE_IMMUTABLE,
                        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as _,
                        CPUAccessFlags: 0,
                        MiscFlags: 0,
                        StructureByteStride: core::mem::size_of::<[f32; 2]>() as _,
                    },
                    Some(&D3D11_SUBRESOURCE_DATA {
                        pSysMem: [
                            [0.0f32, 0.0f32],
                            [1.0f32, 0.0f32],
                            [0.0f32, 1.0f32],
                            [1.0f32, 1.0f32],
                        ]
                        .as_ptr() as *const _,
                        SysMemPitch: 0,
                        SysMemSlicePitch: 0,
                    }),
                    Some(sprite_instance_base_vb.as_mut_ptr()),
                )
                .unwrap();
        }
        let sprite_instance_base_vb = unsafe { sprite_instance_base_vb.assume_init().unwrap() };

        let sprite_instance_buffer_capacity = Self::SPRITE_INSTANCE_CAPACITY_UNIT;
        let mut sprite_instance_buffer = core::mem::MaybeUninit::uninit();
        let mut sprite_instance_buffer_staging = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: (core::mem::size_of::<SpriteInstance>()
                            * sprite_instance_buffer_capacity)
                            as _,
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as _,
                        CPUAccessFlags: 0,
                        MiscFlags: 0,
                        StructureByteStride: core::mem::size_of::<SpriteInstance>() as _,
                    },
                    None,
                    Some(sprite_instance_buffer.as_mut_ptr()),
                )
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: (core::mem::size_of::<SpriteInstance>()
                            * sprite_instance_buffer_capacity)
                            as _,
                        Usage: D3D11_USAGE_STAGING,
                        BindFlags: 0,
                        CPUAccessFlags: (D3D11_CPU_ACCESS_WRITE | D3D11_CPU_ACCESS_READ).0 as _,
                        MiscFlags: 0,
                        StructureByteStride: core::mem::size_of::<SpriteInstance>() as _,
                    },
                    None,
                    Some(sprite_instance_buffer_staging.as_mut_ptr()),
                )
                .unwrap();
        }
        let sprite_instance_buffer = unsafe { sprite_instance_buffer.assume_init().unwrap() };
        let sprite_instance_buffer_staging =
            unsafe { sprite_instance_buffer_staging.assume_init().unwrap() };

        let atlas_bg_vertices = [[0.0, 0.0], [128.0, 0.0], [0.0, 128.0], [128.0, 128.0]];
        let mut atlas_bg_vsh = MaybeUninit::uninit();
        let mut atlas_bg_psh = MaybeUninit::uninit();
        let mut atlas_bg_input_layout = MaybeUninit::uninit();
        let mut atlas_bg_vb = MaybeUninit::uninit();
        let atlas_bg_vsh_code = std::fs::read("resources/atlas_bg/vsh.fxc").unwrap();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateVertexShader(&atlas_bg_vsh_code, None, Some(atlas_bg_vsh.as_mut_ptr()))
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreatePixelShader(
                    &std::fs::read("resources/atlas_bg/psh.fxc").unwrap(),
                    None,
                    Some(atlas_bg_psh.as_mut_ptr()),
                )
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreateInputLayout(
                    &[D3D11_INPUT_ELEMENT_DESC {
                        SemanticName: s!("POSITION"),
                        SemanticIndex: 0,
                        Format: DXGI_FORMAT_R32G32_FLOAT,
                        InputSlot: 0,
                        AlignedByteOffset: 0,
                        InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                        InstanceDataStepRate: 0,
                    }],
                    &atlas_bg_vsh_code,
                    Some(atlas_bg_input_layout.as_mut_ptr()),
                )
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: (core::mem::size_of::<[f32; 2]>() * 4) as _,
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as _,
                        CPUAccessFlags: 0,
                        MiscFlags: 0,
                        StructureByteStride: core::mem::size_of::<[f32; 2]>() as _,
                    },
                    Some(&D3D11_SUBRESOURCE_DATA {
                        pSysMem: atlas_bg_vertices.as_ptr() as _,
                        SysMemPitch: 0,
                        SysMemSlicePitch: 0,
                    }),
                    Some(atlas_bg_vb.as_mut_ptr()),
                )
                .unwrap();
        }
        let atlas_bg_vsh = unsafe { atlas_bg_vsh.assume_init().unwrap() };
        let atlas_bg_psh = unsafe { atlas_bg_psh.assume_init().unwrap() };
        let atlas_bg_input_layout = unsafe { atlas_bg_input_layout.assume_init().unwrap() };
        let atlas_bg_vb = unsafe { atlas_bg_vb.assume_init().unwrap() };

        let mut texture_preview_vb = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: core::mem::size_of::<[[f32; 2]; 4]>() as _,
                        Usage: D3D11_USAGE_IMMUTABLE,
                        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as _,
                        CPUAccessFlags: 0,
                        MiscFlags: 0,
                        StructureByteStride: core::mem::size_of::<[f32; 2]>() as _,
                    },
                    Some(&D3D11_SUBRESOURCE_DATA {
                        pSysMem: [
                            [0.0f32, 0.0f32],
                            [1.0f32, 0.0f32],
                            [0.0f32, 1.0f32],
                            [1.0f32, 1.0f32],
                        ]
                        .as_ptr() as *const _,
                        SysMemPitch: 0,
                        SysMemSlicePitch: 0,
                    }),
                    Some(texture_preview_vb.as_mut_ptr()),
                )
                .unwrap();
        }
        let texture_preview_vb = unsafe { texture_preview_vb.assume_init().unwrap() };
        let mut texture_preview_cb = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: core::mem::size_of::<[[f32; 2]; 2]>() as _,
                        Usage: D3D11_USAGE_DYNAMIC,
                        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as _,
                        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as _,
                        MiscFlags: 0,
                        StructureByteStride: core::mem::size_of::<[[f32; 2]; 2]>() as _,
                    },
                    None,
                    Some(texture_preview_cb.as_mut_ptr()),
                )
                .unwrap();
        }
        let texture_preview_cb = unsafe { texture_preview_cb.assume_init().unwrap() };

        let mut texture_preview_vsh = core::mem::MaybeUninit::uninit();
        let mut texture_preview_psh = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateVertexShader(
                    &std::fs::read("./resources/grid/vsh_plane.fxc").unwrap(),
                    None,
                    Some(texture_preview_vsh.as_mut_ptr()),
                )
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreatePixelShader(
                    &std::fs::read("./resources/grid/psh_tex.fxc").unwrap(),
                    None,
                    Some(texture_preview_psh.as_mut_ptr()),
                )
                .unwrap();
        }
        let texture_preview_vsh = unsafe { texture_preview_vsh.assume_init().unwrap() };
        let texture_preview_psh = unsafe { texture_preview_psh.assume_init().unwrap() };

        Self {
            root,
            swapchain: sc,
            vsh,
            psh,
            render_params_cb,
            texture_preview_vb,
            texture_preview_cb,
            texture_preview_vsh,
            texture_preview_psh,
            sprite_instance_vsh,
            sprite_instance_psh,
            sprite_instance_input_layout,
            sprite_instance_base_vb,
            sprite_instance_buffer: RwLock::new(SpriteInstanceBuffer {
                buffer: sprite_instance_buffer,
                staging: sprite_instance_buffer_staging,
                capacity: sprite_instance_buffer_capacity,
                count: 0,
                is_dirty: false,
            }),
            atlas_bg_vsh,
            atlas_bg_psh,
            atlas_bg_input_layout,
            atlas_bg_vb,
            atlas_bg_vertices: RwLock::new(atlas_bg_vertices),
            atlas_bg_vertices_dirty: RwLock::new(false),
            tex_sampler,
            size_pixels: RwLock::new((init_width_pixels, init_height_pixels)),
            resize_order: RwLock::new(None),
            offset_pixels: RwLock::new((0.0, 0.0)),
            background_worker_enqueue_access: init.background_worker_enqueue_access.downgrade(),
            simple_atlas,
            sprite_source_offset: RwLock::new(HashMap::new()),
            d3d11_device: init.subsystem.d3d11_device.clone(),
            d3d11_device_context: init.subsystem.d3d11_imm_context.clone(),
            d3d11_mt,
            d2d1_mt: init.subsystem.d2d1_factory.cast().unwrap(),
            premul_blend_state,
        }
    }

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn unmount(&self) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
    }

    pub fn resize(&self, new_width_px: u32, new_height_px: u32) {
        self.root
            .SetSize(Vector2 {
                X: new_width_px as _,
                Y: new_height_px as _,
            })
            .unwrap();

        *self.resize_order.write() = Some((new_width_px, new_height_px));
    }

    pub fn set_atlas_size(&self, width_pixels: u32, height_pixels: u32) {
        *self.atlas_bg_vertices.write() = [
            [0.0, 0.0],
            [width_pixels as _, 0.0],
            [0.0, height_pixels as _],
            [width_pixels as _, height_pixels as _],
        ];
        *self.atlas_bg_vertices_dirty.write() = true;
    }

    pub fn update_sprite_offset(&self, index: usize, left_pixels: f32, top_pixels: f32) {
        let c = D3D11CriticalSectionGuard::enter(&self.d3d11_mt);
        let mut sprite_instance_buffer = self.sprite_instance_buffer.write();

        let mut mapped = MaybeUninit::uninit();
        unsafe {
            self.d3d11_device_context
                .Map(
                    &sprite_instance_buffer.staging,
                    0,
                    D3D11_MAP_WRITE,
                    0,
                    Some(mapped.as_mut_ptr()),
                )
                .unwrap();
        }
        let mapped = unsafe { mapped.assume_init() };
        unsafe {
            let instance_ptr = (mapped.pData as *mut SpriteInstance).add(index);
            core::ptr::write(
                core::ptr::addr_of_mut!((*instance_ptr).pos_st[2]),
                left_pixels,
            );
            core::ptr::write(
                core::ptr::addr_of_mut!((*instance_ptr).pos_st[3]),
                top_pixels,
            );

            sprite_instance_buffer.is_dirty = true;
        }
        unsafe {
            self.d3d11_device_context
                .Unmap(&sprite_instance_buffer.staging, 0);
        }

        drop(c);
    }

    pub fn update_sprites(&self, sprites: &[SpriteInfo]) {
        let Some(background_worker_enqueue_access) =
            self.background_worker_enqueue_access.upgrade()
        else {
            // app teardown-ed
            return;
        };

        let c = D3D11CriticalSectionGuard::enter(&self.d3d11_mt);
        let mut sprite_instance_buffer = self.sprite_instance_buffer.write();

        if sprite_instance_buffer.capacity < sprites.len() {
            // たりない
            let new_capacity =
                sprite_instance_buffer.capacity + Self::SPRITE_INSTANCE_CAPACITY_UNIT;
            let mut new_buf = MaybeUninit::uninit();
            unsafe {
                self.d3d11_device
                    .CreateBuffer(
                        &D3D11_BUFFER_DESC {
                            ByteWidth: (core::mem::size_of::<SpriteInstance>() * new_capacity) as _,
                            Usage: D3D11_USAGE_DEFAULT,
                            BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as _,
                            CPUAccessFlags: 0,
                            MiscFlags: 0,
                            StructureByteStride: core::mem::size_of::<SpriteInstance>() as _,
                        },
                        None,
                        Some(new_buf.as_mut_ptr()),
                    )
                    .unwrap();
            }
            sprite_instance_buffer.buffer = unsafe { new_buf.assume_init().unwrap() };
            let mut new_buf_stg = MaybeUninit::uninit();
            unsafe {
                self.d3d11_device
                    .CreateBuffer(
                        &D3D11_BUFFER_DESC {
                            ByteWidth: (core::mem::size_of::<SpriteInstance>() * new_capacity) as _,
                            Usage: D3D11_USAGE_STAGING,
                            BindFlags: 0,
                            CPUAccessFlags: (D3D11_CPU_ACCESS_WRITE | D3D11_CPU_ACCESS_READ).0 as _,
                            MiscFlags: 0,
                            StructureByteStride: core::mem::size_of::<SpriteInstance>() as _,
                        },
                        None,
                        Some(new_buf_stg.as_mut_ptr()),
                    )
                    .unwrap();
                let old_stg = core::mem::replace(
                    &mut sprite_instance_buffer.staging,
                    new_buf_stg.assume_init().unwrap(),
                );
                self.d3d11_mt.Enter();
                self.d3d11_device_context
                    .CopyResource(&sprite_instance_buffer.staging, &old_stg);
                self.d3d11_device_context.Flush();
                self.d3d11_mt.Leave();
            }
            sprite_instance_buffer.capacity = new_capacity;
            sprite_instance_buffer.is_dirty = true;
        }

        let mut mapped = MaybeUninit::uninit();
        unsafe {
            self.d3d11_device_context
                .Map(
                    &sprite_instance_buffer.staging,
                    0,
                    D3D11_MAP_WRITE,
                    0,
                    Some(mapped.as_mut_ptr()),
                )
                .unwrap();
        }
        let mapped = unsafe { mapped.assume_init() };
        for (n, x) in sprites.iter().enumerate() {
            let (ox, oy) = match self
                .sprite_source_offset
                .write()
                .entry(x.source_path.clone())
            {
                // ロード済み
                std::collections::hash_map::Entry::Occupied(o) => *o.get(),
                std::collections::hash_map::Entry::Vacant(e) => {
                    let Some((ox, oy)) = self.simple_atlas.write().alloc(x.width, x.height) else {
                        tracing::warn!("no suitable region(realloc or alloc page here...)");
                        continue;
                    };
                    e.insert((ox, oy));

                    background_worker_enqueue_access.enqueue(BackgroundWork::LoadSpriteSource(
                        x.source_path.clone(),
                        Box::new({
                            let d3d11_device_context = self.d3d11_device_context.clone();
                            let d3d11_mt = self.d3d11_mt.clone();
                            let simple_atlas_resource = self.simple_atlas.read().resource.clone();
                            let (width, height) = (x.width, x.height);

                            move |path, di| {
                                // TODO: HDR対応
                                let img_formatted = di.to_rgba8();
                                unsafe {
                                    let c = D3D11CriticalSectionGuard::enter(&d3d11_mt);

                                    d3d11_device_context.UpdateSubresource(
                                        &simple_atlas_resource,
                                        0,
                                        Some(&D3D11_BOX {
                                            left: ox,
                                            top: oy,
                                            front: 0,
                                            right: ox + width,
                                            bottom: oy + height,
                                            back: 1,
                                        }),
                                        img_formatted.as_bytes().as_ptr() as *const _,
                                        img_formatted.width() * 4,
                                        0,
                                    );

                                    drop(c);
                                }
                                tracing::info!({?path, ox, oy}, "LoadSpriteComplete");
                            }
                        }),
                    ));

                    (ox, oy)
                }
            };

            unsafe {
                let instance_ptr = (mapped.pData as *mut SpriteInstance).add(n);
                core::ptr::write(
                    core::ptr::addr_of_mut!((*instance_ptr).pos_st),
                    [x.width as f32, x.height as f32, x.left as f32, x.top as f32],
                );
                core::ptr::write(
                    core::ptr::addr_of_mut!((*instance_ptr).uv_st),
                    [
                        x.width as f32 / SimpleTextureAtlas::SIZE as f32,
                        x.height as f32 / SimpleTextureAtlas::SIZE as f32,
                        ox as f32 / SimpleTextureAtlas::SIZE as f32,
                        oy as f32 / SimpleTextureAtlas::SIZE as f32,
                    ],
                );

                sprite_instance_buffer.is_dirty = true;
            }
        }
        unsafe {
            self.d3d11_device_context
                .Unmap(&sprite_instance_buffer.staging, 0);
        }

        sprite_instance_buffer.count = sprites.len();
        drop(c);
    }

    pub fn set_offset(&self, offset_x: f32, offset_y: f32) {
        *self.offset_pixels.write() = (offset_x, offset_y);
    }

    pub fn update_content(&self) {
        if let Some((req_width_px, req_height_px)) = self.resize_order.write().take() {
            unsafe {
                self.swapchain
                    .ResizeBuffers(
                        2,
                        req_width_px,
                        req_height_px,
                        DXGI_FORMAT_B8G8R8A8_UNORM,
                        DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
                    )
                    .unwrap();
            }

            *self.size_pixels.write() = (req_width_px, req_height_px);
        }

        let (width_px, height_px) = *self.size_pixels.read();
        let (offset_x, offset_y) = *self.offset_pixels.read();

        let c = D3D11CriticalSectionGuard::enter(&self.d3d11_mt);
        let mut mapped = core::mem::MaybeUninit::uninit();
        unsafe {
            self.d3d11_device_context
                .Map(
                    &self.render_params_cb,
                    0,
                    D3D11_MAP_WRITE_DISCARD,
                    0,
                    Some(mapped.as_mut_ptr()),
                )
                .unwrap();
        }
        let mapped = unsafe { mapped.assume_init() };
        unsafe {
            core::ptr::write(
                mapped.pData as _,
                AtlasBaseGridRenderParams {
                    pixel_size: [width_px as _, height_px as _],
                    grid_offset: [offset_x, offset_y],
                    grid_size: 64.0,
                },
            );
        }
        unsafe {
            self.d3d11_device_context.Unmap(&self.render_params_cb, 0);
        }

        let mut mapped = core::mem::MaybeUninit::uninit();
        unsafe {
            self.d3d11_device_context
                .Map(
                    &self.texture_preview_cb,
                    0,
                    D3D11_MAP_WRITE_DISCARD,
                    0,
                    Some(mapped.as_mut_ptr()),
                )
                .unwrap();
        }
        let mapped = unsafe { mapped.assume_init() };
        unsafe {
            core::ptr::write(
                mapped.pData as _,
                [[width_px as f32, height_px as f32], [offset_x, offset_y]],
            );
        }
        unsafe {
            self.d3d11_device_context.Unmap(&self.texture_preview_cb, 0);
        }

        let mut sprite_instance_buffer = self.sprite_instance_buffer.write();
        if core::mem::replace(&mut sprite_instance_buffer.is_dirty, false) {
            unsafe {
                self.d3d11_device_context.CopyResource(
                    &sprite_instance_buffer.buffer,
                    &sprite_instance_buffer.staging,
                );
            }
        }

        let sprite_instance_buffer1 = sprite_instance_buffer.buffer.clone();
        let sprite_instance_count = sprite_instance_buffer.count;
        drop(sprite_instance_buffer);

        if core::mem::replace(&mut *self.atlas_bg_vertices_dirty.write(), false) {
            unsafe {
                self.d3d11_device_context.UpdateSubresource(
                    &self.atlas_bg_vb,
                    0,
                    None,
                    self.atlas_bg_vertices.data_ptr() as _,
                    0,
                    0,
                );
            }
        }

        drop(c);

        let bb = unsafe { self.swapchain.GetBuffer::<ID3D11Texture2D>(0).unwrap() };
        let mut rtv = core::mem::MaybeUninit::uninit();
        unsafe {
            self.d3d11_device
                .CreateRenderTargetView(&bb, None, Some(rtv.as_mut_ptr()))
                .unwrap()
        };
        let rtv = unsafe { rtv.assume_init().unwrap() };

        let c = D2D1CriticalSectionGuard::enter(&self.d2d1_mt);
        unsafe {
            self.d3d11_device_context
                .OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            self.d3d11_device_context
                .RSSetViewports(Some(&[D3D11_VIEWPORT {
                    TopLeftX: 0.0,
                    TopLeftY: 0.0,
                    Width: width_px as _,
                    Height: height_px as _,
                    MinDepth: 0.0,
                    MaxDepth: 1.0,
                }]));
            self.d3d11_device_context.RSSetScissorRects(Some(&[RECT {
                left: 0,
                top: 0,
                right: width_px as _,
                bottom: height_px as _,
            }]));

            self.d3d11_device_context.VSSetShader(&self.vsh, None);
            self.d3d11_device_context.PSSetShader(&self.psh, None);
            self.d3d11_device_context
                .PSSetConstantBuffers(0, Some(&[Some(self.render_params_cb.clone())]));
            self.d3d11_device_context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            self.d3d11_device_context.Draw(4, 0);

            self.d3d11_device_context
                .VSSetShader(&self.atlas_bg_vsh, None);
            self.d3d11_device_context
                .PSSetShader(&self.atlas_bg_psh, None);
            self.d3d11_device_context
                .VSSetConstantBuffers(0, Some(&[Some(self.render_params_cb.clone())]));
            self.d3d11_device_context
                .IASetInputLayout(&self.atlas_bg_input_layout);
            self.d3d11_device_context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            self.d3d11_device_context.IASetVertexBuffers(
                0,
                1,
                Some([Some(self.atlas_bg_vb.clone())].as_ptr()),
                Some([core::mem::size_of::<[f32; 2]>() as u32].as_ptr()),
                Some([0u32].as_ptr()),
            );
            self.d3d11_device_context.Draw(4, 0);

            self.d3d11_device_context
                .OMSetBlendState(&self.premul_blend_state, None, u32::MAX);
            self.d3d11_device_context
                .VSSetShader(&self.sprite_instance_vsh, None);
            self.d3d11_device_context
                .PSSetShader(&self.sprite_instance_psh, None);
            self.d3d11_device_context
                .VSSetConstantBuffers(0, Some(&[Some(self.texture_preview_cb.clone())]));
            self.d3d11_device_context
                .PSSetShaderResources(0, Some(&[Some(self.simple_atlas.read().srv.clone())]));
            self.d3d11_device_context
                .PSSetSamplers(0, Some(&[Some(self.tex_sampler.clone())]));
            self.d3d11_device_context
                .IASetInputLayout(&self.sprite_instance_input_layout);
            self.d3d11_device_context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            self.d3d11_device_context.IASetVertexBuffers(
                0,
                2,
                Some(
                    [
                        Some(self.sprite_instance_base_vb.clone()),
                        Some(sprite_instance_buffer1),
                    ]
                    .as_ptr(),
                ),
                Some(
                    [
                        core::mem::size_of::<[f32; 2]>() as u32,
                        core::mem::size_of::<SpriteInstance>() as u32,
                    ]
                    .as_ptr(),
                ),
                Some([0u32, 0u32].as_ptr()),
            );
            self.d3d11_device_context
                .DrawInstanced(4, sprite_instance_count as _, 0, 0);

            /*self.d3d11_device_context
                .VSSetShader(&self.texture_preview_vsh, None);
            self.d3d11_device_context
                .PSSetShader(&self.texture_preview_psh, None);
            self.d3d11_device_context
                .VSSetConstantBuffers(0, Some(&[Some(self.texture_preview_cb.clone())]));
            self.d3d11_device_context
                .PSSetShaderResources(0, Some(&[Some(self.texture_preview_srv.clone())]));
            self.d3d11_device_context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            self.d3d11_device_context.IASetVertexBuffers(
                0,
                1,
                Some([Some(self.texture_preview_vb.clone())].as_ptr()),
                Some(&(core::mem::size_of::<[f32; 2]>() as u32) as *const _),
                Some(&0u32 as *const _),
            );
            self.d3d11_device_context.Draw(4, 0);*/
            self.d3d11_device_context.Flush();
            self.d3d11_device_context.ClearState();
        }

        unsafe {
            self.swapchain.Present(0, DXGI_PRESENT(0)).ok().unwrap();
        }

        drop(c);
    }
}

#[inline]
fn create_instant_effect_brush(
    subsystem: &Subsystem,
    effect: impl windows::core::Param<IGraphicsEffect>,
    source_params: &[(&HSTRING, CompositionBrush)],
) -> windows::core::Result<CompositionEffectBrush> {
    let x = subsystem
        .compositor
        .CreateEffectFactory(effect)?
        .CreateBrush()?;
    for (n, s) in source_params {
        x.SetSourceParameter(n, s)?;
    }

    Ok(x)
}

const D2D1_COLOR_F_WHITE: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub struct CurrentSelectedSpriteMarkerView {
    root: SpriteVisual,
    composition_properties: CompositionPropertySet,
    focus_animation: CompositionAnimationGroup,
    hide_animation: ScalarKeyFrameAnimation,
}
impl CurrentSelectedSpriteMarkerView {
    const CORNER_RADIUS: f32 = 4.0;
    const THICKNESS: f32 = 2.0;
    const COLOR: D2D1_COLOR_F = d2d1_color_f_from_hex_rgb(0x00ff00);

    pub fn new(init: &mut ViewInitContext) -> Self {
        let surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(Self::CORNER_RADIUS * 2.0 + 1.0),
                Height: init.dip_to_pixels(Self::CORNER_RADIUS * 2.0 + 1.0),
            })
            .unwrap();
        draw_2d(&surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            let brush = unsafe { dc.CreateSolidColorBrush(&Self::COLOR, None)? };

            unsafe {
                dc.Clear(None);
                dc.DrawRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: Self::THICKNESS * 0.5,
                            top: Self::THICKNESS * 0.5,
                            right: Self::CORNER_RADIUS * 2.0 + 1.0 - Self::THICKNESS * 0.5,
                            bottom: Self::CORNER_RADIUS * 2.0 + 1.0 - Self::THICKNESS * 0.5,
                        },
                        radiusX: Self::CORNER_RADIUS,
                        radiusY: Self::CORNER_RADIUS,
                    },
                    &brush,
                    Self::THICKNESS,
                    None,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let root = SpriteVisualParams::new(
            &CompositionNineGridBrushParams::new(
                &CompositionSurfaceBrushParams::new(&surface)
                    .stretch(CompositionStretch::Fill)
                    .instantiate(&init.subsystem.compositor)
                    .unwrap(),
            )
            .insets(init.dip_to_pixels(Self::CORNER_RADIUS))
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
        )
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let focus_scale_animation = init
            .subsystem
            .compositor
            .CreateVector3KeyFrameAnimation()
            .unwrap();
        focus_scale_animation
            .InsertKeyFrame(
                0.0,
                Vector3 {
                    X: 1.3,
                    Y: 1.3,
                    Z: 1.0,
                },
            )
            .unwrap();
        focus_scale_animation
            .InsertKeyFrameWithEasingFunction(
                1.0,
                Vector3 {
                    X: 1.0,
                    Y: 1.0,
                    Z: 1.0,
                },
                &init
                    .subsystem
                    .compositor
                    .CreateCubicBezierEasingFunction(
                        Vector2 { X: 0.0, Y: 0.0 },
                        Vector2 { X: 0.0, Y: 1.0 },
                    )
                    .unwrap(),
            )
            .unwrap();
        focus_scale_animation.SetTarget(h!("Scale")).unwrap();

        let focus_opacity_animation = init
            .subsystem
            .compositor
            .CreateScalarKeyFrameAnimation()
            .unwrap();
        focus_opacity_animation.InsertKeyFrame(0.0, 0.0).unwrap();
        focus_opacity_animation
            .InsertKeyFrameWithEasingFunction(
                1.0,
                1.0,
                &init
                    .subsystem
                    .compositor
                    .CreateLinearEasingFunction()
                    .unwrap(),
            )
            .unwrap();
        focus_opacity_animation.SetTarget(h!("Opacity")).unwrap();

        focus_scale_animation.SetDuration(timespan_ms(150)).unwrap();
        focus_opacity_animation
            .SetDuration(timespan_ms(150))
            .unwrap();
        let focus_animation = init.subsystem.compositor.CreateAnimationGroup().unwrap();
        focus_animation.Add(&focus_scale_animation).unwrap();
        focus_animation.Add(&focus_opacity_animation).unwrap();

        let composition_properties = init.subsystem.compositor.CreatePropertySet().unwrap();
        composition_properties
            .InsertVector3(h!("GlobalPos"), Vector3::zero())
            .unwrap();
        composition_properties
            .InsertVector3(h!("ViewOffset"), Vector3::zero())
            .unwrap();

        let root_offset_expr = init
            .subsystem
            .compositor
            .CreateExpressionAnimationWithExpression(h!("cp.GlobalPos + cp.ViewOffset"))
            .unwrap();
        root_offset_expr
            .SetExpressionReferenceParameter(h!("cp"), &composition_properties)
            .unwrap();
        root.StartAnimation(h!("Offset"), &root_offset_expr)
            .unwrap();

        let hide_animation = SimpleScalarAnimationParams::new(
            1.0,
            0.0,
            &init
                .subsystem
                .compositor
                .CreateLinearEasingFunction()
                .unwrap(),
        )
        .target(h!("Opacity"))
        .duration(timespan_ms(150))
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        Self {
            root,
            composition_properties,
            focus_animation,
            hide_animation,
        }
    }

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn focus(&self, x_pixels: f32, y_pixels: f32, width_pixels: f32, height_pixels: f32) {
        self.composition_properties
            .InsertVector3(
                h!("GlobalPos"),
                Vector3 {
                    X: x_pixels,
                    Y: y_pixels,
                    Z: 0.0,
                },
            )
            .unwrap();
        self.root
            .SetSize(Vector2 {
                X: width_pixels,
                Y: height_pixels,
            })
            .unwrap();
        self.root
            .SetCenterPoint(Vector3 {
                X: width_pixels * 0.5,
                Y: height_pixels * 0.5,
                Z: 0.0,
            })
            .unwrap();
        self.root
            .StartAnimationGroup(&self.focus_animation)
            .unwrap();
    }

    pub fn hide(&self) {
        self.root.StopAnimationGroup(&self.focus_animation).unwrap();
        self.root
            .StartAnimation(h!("Opacity"), &self.hide_animation)
            .unwrap();
    }

    pub fn set_view_offset(&self, offset_x_pixels: f32, offset_y_pixels: f32) {
        self.composition_properties
            .InsertVector3(
                h!("ViewOffset"),
                Vector3 {
                    X: -offset_x_pixels,
                    Y: -offset_y_pixels,
                    Z: 0.0,
                },
            )
            .unwrap();
    }
}

pub struct SpriteAtlasBorderView {
    root: SpriteVisual,
}
impl SpriteAtlasBorderView {
    pub fn new(init: &mut ViewInitContext) -> Self {
        let frame_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: 4.0,
                Height: 4.0,
            })
            .unwrap();
        draw_2d(&frame_surface, |dc, offset| {
            unsafe {
                dc.Clear(None);
                dc.DrawRectangle(
                    &D2D_RECT_F {
                        left: offset.x as f32 + 0.5,
                        top: offset.y as f32 + 0.5,
                        right: offset.x as f32 + 4.0 - 0.5,
                        bottom: offset.y as f32 + 4.0 - 0.5,
                    },
                    &dc.CreateSolidColorBrush(&d2d1_color_f_from_websafe_hex_rgb(0xf00), None)?,
                    1.0,
                    None,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let root = SpriteVisualParams::new(
            &CompositionNineGridBrushParams::new(
                &CompositionSurfaceBrushParams::new(&frame_surface)
                    .stretch(CompositionStretch::Fill)
                    .instantiate(&init.subsystem.compositor)
                    .unwrap(),
            )
            .insets(1.0)
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
        )
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        Self { root }
    }

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn set_size(&self, size: SizePixels) {
        self.root
            .SetSize(Vector2 {
                X: size.width as _,
                Y: size.height as _,
            })
            .unwrap();
    }

    pub fn set_view_offset(&self, offset_x_pixels: f32, offset_y_pixels: f32) {
        self.root
            .SetOffset(Vector3 {
                X: -offset_x_pixels,
                Y: -offset_y_pixels,
                Z: 0.0,
            })
            .unwrap();
    }
}

pub struct SpriteListToggleButtonView {
    root: ContainerVisual,
    bg: SpriteVisual,
    icon: SpriteVisual,
    ht_root: HitTestTreeRef,
    dpi: Cell<f32>,
    hovering: Cell<bool>,
    pressing: Cell<bool>,
}
impl SpriteListToggleButtonView {
    const BUTTON_SIZE: f32 = 20.0;
    const ICON_SIZE: f32 = 7.0;
    const ICON_THICKNESS: f32 = 1.25;
    const ICON_COLOR: D2D1_COLOR_F = d2d1_color_f_from_hex_rgb(0xf0f0f0);

    pub fn new(init: &mut ViewInitContext) -> Self {
        let button_size_px = init.dip_to_pixels(Self::BUTTON_SIZE);
        let icon_size_px = init.dip_to_pixels(Self::ICON_SIZE);

        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(icon_size_px))
            .unwrap();
        draw_2d(&icon_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            let brush = unsafe { dc.CreateSolidColorBrush(&Self::ICON_COLOR, None)? };

            unsafe {
                dc.Clear(None);
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE * 0.4,
                        y: 0.0,
                    },
                    D2D_POINT_2F {
                        x: 0.0,
                        y: Self::ICON_SIZE * 0.5,
                    },
                    &brush,
                    Self::ICON_THICKNESS,
                    None,
                );
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: 0.0,
                        y: Self::ICON_SIZE * 0.5,
                    },
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE * 0.4,
                        y: Self::ICON_SIZE,
                    },
                    &brush,
                    Self::ICON_THICKNESS,
                    None,
                );
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE,
                        y: 0.0,
                    },
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE * 0.6,
                        y: Self::ICON_SIZE * 0.5,
                    },
                    &brush,
                    Self::ICON_THICKNESS,
                    None,
                );
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE * 0.6,
                        y: Self::ICON_SIZE * 0.5,
                    },
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE,
                        y: Self::ICON_SIZE,
                    },
                    &brush,
                    Self::ICON_THICKNESS,
                    None,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let frame_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: button_size_px,
                Height: button_size_px,
            })
            .unwrap();
        draw_2d(&frame_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));

                dc.Clear(None);
                dc.FillEllipse(
                    &D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: Self::BUTTON_SIZE * 0.5,
                            y: Self::BUTTON_SIZE * 0.5,
                        },
                        radiusX: Self::BUTTON_SIZE * 0.5,
                        radiusY: Self::BUTTON_SIZE * 0.5,
                    },
                    &dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None)?,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let root = ContainerVisualParams::new()
            .size_sq(button_size_px)
            .relative_offset_adjustment_xy(Vector2 { X: 1.0, Y: 0.0 })
            .offset_xy(Vector2 {
                X: -button_size_px - init.dip_to_pixels(8.0),
                Y: init.dip_to_pixels(8.0),
            })
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let bg = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&frame_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .expand()
        .opacity(0.0)
        .instantiate(&init.subsystem.compositor)
        .unwrap();
        let icon = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&icon_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size_sq(icon_size_px)
        .relative_offset_adjustment_xy(Vector2 { X: 0.5, Y: 0.5 })
        .anchor_point(Vector2 { X: 0.5, Y: 0.5 })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&icon).unwrap();

        let implicit_easing_animation = SimpleImplicitAnimationParams::new(
            &init
                .subsystem
                .compositor
                .CreateCubicBezierEasingFunction(
                    Vector2 { X: 0.5, Y: 0.0 },
                    Vector2 { X: 0.5, Y: 1.0 },
                )
                .unwrap(),
            h!("Opacity"),
            timespan_ms(100),
        )
        .instantiate_scalar(&init.subsystem.compositor)
        .unwrap();
        let implicit_move_animation = SimpleImplicitAnimationParams::new(
            &init
                .subsystem
                .compositor
                .CreateCubicBezierEasingFunction(
                    Vector2 { X: 0.5, Y: 0.0 },
                    Vector2 { X: 0.5, Y: 1.0 },
                )
                .unwrap(),
            h!("Offset"),
            SpriteListPaneView::TRANSITION_DURATION,
        )
        .instantiate_vector3(&init.subsystem.compositor)
        .unwrap();

        let bg_implicit_animations = init
            .subsystem
            .compositor
            .CreateImplicitAnimationCollection()
            .unwrap();
        bg_implicit_animations
            .Insert(h!("Opacity"), &implicit_easing_animation)
            .unwrap();
        bg.SetImplicitAnimations(&bg_implicit_animations).unwrap();

        let root_implicit_animations = init
            .subsystem
            .compositor
            .CreateImplicitAnimationCollection()
            .unwrap();
        root_implicit_animations
            .Insert(h!("Offset"), &implicit_move_animation)
            .unwrap();
        root.SetImplicitAnimations(&root_implicit_animations)
            .unwrap();

        let ht_root = init.ht.borrow_mut().alloc(HitTestTreeData {
            left: -Self::BUTTON_SIZE - 8.0,
            top: 8.0,
            left_adjustment_factor: 1.0,
            top_adjustment_factor: 0.0,
            width: Self::BUTTON_SIZE,
            height: Self::BUTTON_SIZE,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 0.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });

        Self {
            root,
            icon,
            bg,
            ht_root,
            dpi: Cell::new(init.dpi),
            hovering: Cell::new(false),
            pressing: Cell::new(false),
        }
    }

    pub fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut AppHitTestTreeManager,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    fn update_bg_opacity(&self) {
        match (self.hovering.get(), self.pressing.get()) {
            (false, _) => {
                self.bg.SetOpacity(0.0).unwrap();
            }
            (true, false) => {
                self.bg.SetOpacity(0.15).unwrap();
            }
            (true, true) => {
                self.bg.SetOpacity(0.3).unwrap();
            }
        }
    }

    pub fn on_hover(&self) {
        self.hovering.set(true);
        self.update_bg_opacity();
    }

    pub fn on_press(&self) {
        self.pressing.set(true);
        self.update_bg_opacity();
    }

    pub fn on_release(&self) {
        self.pressing.set(false);
        self.update_bg_opacity();
    }

    pub fn on_hover_leave(&self) {
        self.hovering.set(false);
        self.pressing.set(false);
        self.update_bg_opacity();
    }

    pub fn transit_hidden(&self, ht: &mut AppHitTestTreeManager) {
        let dpi = self.dpi.get();

        self.icon
            .SetScale(Vector3 {
                X: -1.0,
                Y: 1.0,
                Z: 1.0,
            })
            .unwrap();
        self.root
            .SetOffset(Vector3 {
                X: dip_to_pixels(8.0 + SpriteListPaneView::SPACING, dpi),
                Y: dip_to_pixels(8.0, dpi),
                Z: 0.0,
            })
            .unwrap();
        ht.get_mut(self.ht_root).left = 8.0 + SpriteListPaneView::SPACING;
    }

    pub fn transit_shown(&self, ht: &mut AppHitTestTreeManager) {
        let dpi = self.dpi.get();

        self.icon
            .SetScale(Vector3 {
                X: 1.0,
                Y: 1.0,
                Z: 1.0,
            })
            .unwrap();
        self.root
            .SetOffset(Vector3 {
                X: dip_to_pixels(-Self::BUTTON_SIZE - 8.0, dpi),
                Y: dip_to_pixels(8.0, dpi),
                Z: 0.0,
            })
            .unwrap();
        ht.get_mut(self.ht_root).left = -Self::BUTTON_SIZE - 8.0;
    }
}

pub struct SpriteListCellView {
    root: ContainerVisual,
    bg: SpriteVisual,
    bg_select: SpriteVisual,
    label: SpriteVisual,
    top: Cell<f32>,
    dpi: Cell<f32>,
}
impl SpriteListCellView {
    const FRAME_TEX_SIZE: f32 = 24.0;
    const CORNER_RADIUS: f32 = 8.0;
    const CELL_HEIGHT: f32 = 20.0;
    const LABEL_COLOR: D2D1_COLOR_F = D2D1_COLOR_F_WHITE;
    const BG_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0xccc, 32);
    const ACTIVE_BG_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0x4af, 64);

    fn gen_frame_tex(subsystem: &Subsystem, dpi: f32) -> CompositionDrawingSurface {
        let s = subsystem
            .new_2d_drawing_surface(Size {
                Width: dip_to_pixels(Self::FRAME_TEX_SIZE, dpi),
                Height: dip_to_pixels(Self::FRAME_TEX_SIZE, dpi),
            })
            .unwrap();
        draw_2d(&s, |dc, offset| {
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
                            right: Self::FRAME_TEX_SIZE,
                            bottom: Self::FRAME_TEX_SIZE,
                        },
                        radiusX: Self::CORNER_RADIUS,
                        radiusY: Self::CORNER_RADIUS,
                    },
                    &dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None)?,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        s
    }

    pub fn new(init: &mut ViewInitContext, label: &str, init_top: f32) -> Self {
        let frame_tex = Self::gen_frame_tex(init.subsystem, init.dpi);

        let tl = init
            .subsystem
            .new_text_layout_unrestricted(label, &init.subsystem.default_ui_format)
            .unwrap();
        let mut tm = core::mem::MaybeUninit::uninit();
        unsafe {
            tl.GetMetrics(tm.as_mut_ptr()).unwrap();
        }
        let tm = unsafe { tm.assume_init() };
        let label_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(tm.width),
                Height: init.dip_to_pixels(tm.height),
            })
            .unwrap();
        draw_2d(&label_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);

                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: init.signed_pixels_to_dip(offset.x),
                        y: init.signed_pixels_to_dip(offset.y),
                    },
                    &tl,
                    &dc.CreateSolidColorBrush(&Self::LABEL_COLOR, None)?,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let bg_base_brush = CompositionNineGridBrushParams::new(
            &CompositionSurfaceBrushParams::new(&frame_tex)
                .stretch(CompositionStretch::Fill)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .insets(init.dip_to_pixels(Self::CORNER_RADIUS))
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let root = ContainerVisualParams::new()
            .size(Vector2 {
                X: init.dip_to_pixels(
                    -SpriteListPaneView::CELL_AREA_PADDINGS.right
                        - SpriteListPaneView::CELL_AREA_PADDINGS.left,
                ),
                Y: init.dip_to_pixels(Self::CELL_HEIGHT),
            })
            .expand_width()
            .offset_xy(Vector2 {
                X: init.dip_to_pixels(SpriteListPaneView::CELL_AREA_PADDINGS.left),
                Y: init.dip_to_pixels(init_top),
            })
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let bg = SpriteVisualParams::new(
            &create_instant_effect_brush(
                &init.subsystem,
                &TintEffectParams {
                    source: &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                    color: Some(Self::BG_COLOR),
                }
                .instantiate()
                .unwrap(),
                &[(h!("source"), bg_base_brush.cast().unwrap())],
            )
            .unwrap(),
        )
        .expand()
        .opacity(0.0)
        .instantiate(&init.subsystem.compositor)
        .unwrap();
        let bg_select = SpriteVisualParams::new(
            &create_instant_effect_brush(
                &init.subsystem,
                &TintEffectParams {
                    source: &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                    color: Some(Self::ACTIVE_BG_COLOR),
                }
                .instantiate()
                .unwrap(),
                &[(h!("source"), bg_base_brush.cast().unwrap())],
            )
            .unwrap(),
        )
        .expand()
        .opacity(0.0)
        .instantiate(&init.subsystem.compositor)
        .unwrap();
        let label = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&label_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size(Vector2 {
            X: init.dip_to_pixels(tm.width),
            Y: init.dip_to_pixels(tm.height),
        })
        .offset_xy(Vector2 {
            X: init.dip_to_pixels(8.0),
            Y: init.dip_to_pixels(-tm.height * 0.5),
        })
        .relative_vertical_offset_adjustment(0.5)
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg_select).unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&label).unwrap();

        let opacity_transition = SimpleImplicitAnimationParams::new(
            &init
                .subsystem
                .compositor
                .CreateCubicBezierEasingFunction(
                    Vector2 { X: 0.5, Y: 0.0 },
                    Vector2 { X: 0.5, Y: 1.0 },
                )
                .unwrap(),
            h!("Opacity"),
            timespan_ms(100),
        )
        .instantiate_scalar(&init.subsystem.compositor)
        .unwrap();

        let bg_implicit_animations = init
            .subsystem
            .compositor
            .CreateImplicitAnimationCollection()
            .unwrap();
        bg_implicit_animations
            .Insert(h!("Opacity"), &opacity_transition)
            .unwrap();
        bg.SetImplicitAnimations(&bg_implicit_animations).unwrap();
        bg_select
            .SetImplicitAnimations(&bg_implicit_animations)
            .unwrap();

        Self {
            root,
            bg,
            bg_select,
            label,
            top: Cell::new(init_top),
            dpi: Cell::new(init.dpi),
        }
    }

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn unmount(&self) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
    }

    pub fn on_hover(&self) {
        self.bg.SetOpacity(1.0).unwrap();
    }

    pub fn on_leave(&self) {
        self.bg.SetOpacity(0.0).unwrap();
    }

    pub fn on_select(&self) {
        self.bg_select.SetOpacity(1.0).unwrap();
    }

    pub fn on_deselect(&self) {
        self.bg_select.SetOpacity(0.0).unwrap();
    }

    pub fn set_top(&self, top: f32) {
        let dpi = self.dpi.get();

        self.root
            .SetOffset(Vector3 {
                X: dip_to_pixels(SpriteListPaneView::CELL_AREA_PADDINGS.left, dpi),
                Y: dip_to_pixels(top, dpi),
                Z: 0.0,
            })
            .unwrap();
        self.top.set(top);
    }

    pub fn set_name(&self, name: &str, subsystem: &Subsystem) {
        let dpi = self.dpi.get();

        let tl = subsystem
            .new_text_layout_unrestricted(name, &subsystem.default_ui_format)
            .unwrap();
        let mut tm = core::mem::MaybeUninit::uninit();
        unsafe {
            tl.GetMetrics(tm.as_mut_ptr()).unwrap();
        }
        let tm = unsafe { tm.assume_init() };
        let label_surface = subsystem
            .new_2d_drawing_surface(Size {
                Width: dip_to_pixels(tm.width, dpi),
                Height: dip_to_pixels(tm.height, dpi),
            })
            .unwrap();
        draw_2d(&label_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(dpi, dpi);

                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: signed_pixels_to_dip(offset.x, dpi),
                        y: signed_pixels_to_dip(offset.y, dpi),
                    },
                    &tl,
                    &dc.CreateSolidColorBrush(&Self::LABEL_COLOR, None)?,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        self.label
            .SetBrush(
                &CompositionSurfaceBrushParams::new(&label_surface)
                    .instantiate(&subsystem.compositor)
                    .unwrap(),
            )
            .unwrap();
        self.label
            .SetSize(Vector2 {
                X: dip_to_pixels(tm.width, dpi),
                Y: dip_to_pixels(tm.height, dpi),
            })
            .unwrap();
    }
}

pub struct SpriteListPaneView {
    root: ContainerVisual,
    ht_root: HitTestTreeRef,
    ht_adjust_area: HitTestTreeRef,
    ht_cell_area: HitTestTreeRef,
    composition_properties: CompositionPropertySet,
    hide_animation: ScalarKeyFrameAnimation,
    show_animation: ScalarKeyFrameAnimation,
    dpi: f32,
    top: Cell<f32>,
    width: Cell<f32>,
}
impl SpriteListPaneView {
    const CORNER_RADIUS: f32 = 12.0;
    const FRAME_TEX_SIZE: f32 = 32.0;
    const BLUR_AMOUNT: f32 = 27.0;
    const SURFACE_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0xfff, 24);
    const SPACING: f32 = 8.0;
    const ADJUST_AREA_THICKNESS: f32 = 4.0;
    const INIT_WIDTH: f32 = 280.0;
    const HEADER_LABEL_MAIN_COLOR: D2D1_COLOR_F = d2d1_color_f_from_websafe_hex_rgb(0xfff);
    const HEADER_LABEL_SHADOW_COLOR: D2D1_COLOR_F = d2d1_color_f_from_websafe_hex_rgb(0x111);
    const TRANSITION_DURATION: TimeSpan = timespan_ms(250);
    const CELL_AREA_PADDINGS: RectDIP = RectDIP {
        left: 16.0,
        top: 32.0,
        right: 16.0,
        bottom: 16.0,
    };

    fn gen_frame_tex(subsystem: &Subsystem, dpi: f32) -> CompositionDrawingSurface {
        let s = subsystem
            .new_2d_drawing_surface(size_sq(dip_to_pixels(Self::FRAME_TEX_SIZE, dpi)))
            .unwrap();
        draw_2d(&s, |dc, offset| {
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
                            right: Self::FRAME_TEX_SIZE,
                            bottom: Self::FRAME_TEX_SIZE,
                        },
                        radiusX: Self::CORNER_RADIUS,
                        radiusY: Self::CORNER_RADIUS,
                    },
                    &dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None)?,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        s
    }

    pub fn new(init: &mut ViewInitContext) -> Self {
        let frame_surface = Self::gen_frame_tex(init.subsystem, init.dpi);

        let root = ContainerVisualParams::new()
            .left(init.dip_to_pixels(8.0))
            .width(init.dip_to_pixels(Self::INIT_WIDTH))
            .expand_height()
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let bg_base_brush = create_instant_effect_brush(
            init.subsystem,
            &CompositeEffectParams::new(&[
                GaussianBlurEffectParams::new(
                    &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                )
                .blur_amount_px(Self::BLUR_AMOUNT)
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
                ColorSourceEffectParams {
                    color: Some(Self::SURFACE_COLOR),
                }
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
            ])
            .instantiate()
            .unwrap(),
            &[(
                h!("source"),
                init.subsystem
                    .compositor
                    .CreateBackdropBrush()
                    .unwrap()
                    .cast()
                    .unwrap(),
            )],
        )
        .unwrap();

        let bg = SpriteVisualParams::new(
            &CompositionMaskBrushParams {
                source: &bg_base_brush,
                mask: &CompositionNineGridBrushParams::new(
                    &CompositionSurfaceBrushParams::new(&frame_surface)
                        .stretch(CompositionStretch::Fill)
                        .instantiate(&init.subsystem.compositor)
                        .unwrap(),
                )
                .insets(init.dip_to_pixels(Self::CORNER_RADIUS))
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
        )
        .expand()
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let tl = init
            .subsystem
            .new_text_layout_unrestricted("Sprites", &init.subsystem.default_ui_format)
            .unwrap();
        unsafe {
            tl.SetFontWeight(
                DWRITE_FONT_WEIGHT_MEDIUM,
                DWRITE_TEXT_RANGE {
                    startPosition: 0,
                    length: 7,
                },
            )
            .unwrap();
            tl.cast::<IDWriteTextLayout1>()
                .unwrap()
                .SetCharacterSpacing(
                    0.2,
                    0.2,
                    0.1,
                    DWRITE_TEXT_RANGE {
                        startPosition: 0,
                        length: 7,
                    },
                )
                .unwrap();
        }
        let mut tm = core::mem::MaybeUninit::uninit();
        unsafe {
            tl.GetMetrics(tm.as_mut_ptr()).unwrap();
        }
        let tm = unsafe { tm.assume_init() };
        let header_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(tm.width),
                Height: init.dip_to_pixels(tm.height),
            })
            .unwrap();
        draw_2d(&header_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);

                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: init.signed_pixels_to_dip(offset.x),
                        y: init.signed_pixels_to_dip(offset.y),
                    },
                    &tl,
                    &dc.CreateSolidColorBrush(&Self::HEADER_LABEL_MAIN_COLOR, None)?,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();
        let header_surface_w = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(tm.width + 18.0),
                Height: init.dip_to_pixels(tm.height + 18.0),
            })
            .unwrap();
        draw_2d(&header_surface_w, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);

                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: init.signed_pixels_to_dip(offset.x) + 9.0,
                        y: init.signed_pixels_to_dip(offset.y) + 9.0,
                    },
                    &tl,
                    &dc.CreateSolidColorBrush(&Self::HEADER_LABEL_SHADOW_COLOR, None)?,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let header = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&header_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size(Vector2 {
            X: init.dip_to_pixels(tm.width),
            Y: init.dip_to_pixels(tm.height),
        })
        .offset_xy(Vector2 {
            X: init.dip_to_pixels(-tm.width * 0.5),
            Y: init.dip_to_pixels(Self::CORNER_RADIUS),
        })
        .relative_horizontal_offset_adjustment(0.5)
        .instantiate(&init.subsystem.compositor)
        .unwrap();
        let header_bg = SpriteVisualParams::new(
            &create_instant_effect_brush(
                init.subsystem,
                &GaussianBlurEffectParams::new(
                    &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                )
                .blur_amount_px(18.0)
                .instantiate()
                .unwrap(),
                &[(
                    h!("source"),
                    CompositionSurfaceBrushParams::new(&header_surface_w)
                        .instantiate(&init.subsystem.compositor)
                        .unwrap()
                        .cast()
                        .unwrap(),
                )],
            )
            .unwrap(),
        )
        .size(Vector2 {
            X: init.dip_to_pixels(tm.width + 18.0),
            Y: init.dip_to_pixels(tm.height + 18.0),
        })
        .offset_xy(Vector2 {
            X: init.dip_to_pixels(-(tm.width + 18.0) * 0.5),
            Y: init.dip_to_pixels(Self::CORNER_RADIUS - 9.0),
        })
        .relative_horizontal_offset_adjustment(0.5)
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&header_bg).unwrap();
        children.InsertAtTop(&header).unwrap();

        let composition_properties = init.subsystem.compositor.CreatePropertySet().unwrap();
        composition_properties
            .InsertScalar(h!("ShownRate"), 1.0)
            .unwrap();
        composition_properties
            .InsertScalar(h!("DPI"), init.dpi)
            .unwrap();
        composition_properties
            .InsertScalar(h!("TopOffset"), 0.0)
            .unwrap();

        let offset_expr = format!(
            "Vector3(-this.Target.Size.X - ({spc} * compositionProperties.DPI / 96.0) + (this.Target.Size.X + ({spc} * 2.0 * compositionProperties.DPI / 96.0)) * compositionProperties.ShownRate, compositionProperties.TopOffset * compositionProperties.DPI / 96.0, 0.0)",
            spc = Self::SPACING
        );
        let root_offset = init
            .subsystem
            .compositor
            .CreateExpressionAnimationWithExpression(&offset_expr.into())
            .unwrap();
        root_offset
            .SetExpressionReferenceParameter(h!("compositionProperties"), &composition_properties)
            .unwrap();
        root.StartAnimation(h!("Offset"), &root_offset).unwrap();

        let easing = CompositionEasingFunction::CreateBackEasingFunction(
            &init.subsystem.compositor,
            CompositionEasingFunctionMode::Out,
            0.1,
        )
        .unwrap();
        let hide_animation = SimpleScalarAnimationParams::new(1.0, 0.0, &easing)
            .duration(Self::TRANSITION_DURATION)
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let show_animation = SimpleScalarAnimationParams::new(0.0, 1.0, &easing)
            .duration(Self::TRANSITION_DURATION)
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let ht_root = init.ht.borrow_mut().alloc(HitTestTreeData {
            left: Self::SPACING,
            top: 0.0,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: Self::INIT_WIDTH,
            height: -Self::SPACING,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });
        let ht_adjust_area = init.ht.borrow_mut().alloc(HitTestTreeData {
            left: -Self::ADJUST_AREA_THICKNESS * 0.5,
            top: 0.0,
            left_adjustment_factor: 1.0,
            top_adjustment_factor: 0.0,
            width: Self::ADJUST_AREA_THICKNESS,
            height: 0.0,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });
        let ht_cell_area = init.ht.borrow_mut().alloc(HitTestTreeData {
            left: Self::CELL_AREA_PADDINGS.left,
            top: Self::CELL_AREA_PADDINGS.top,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: -Self::CELL_AREA_PADDINGS.right - Self::CELL_AREA_PADDINGS.left,
            height: -Self::CELL_AREA_PADDINGS.bottom - Self::CELL_AREA_PADDINGS.top,
            width_adjustment_factor: 1.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });
        init.ht.borrow_mut().add_child(ht_root, ht_cell_area);
        init.ht.borrow_mut().add_child(ht_root, ht_adjust_area);

        Self {
            root,
            ht_root,
            ht_adjust_area,
            ht_cell_area,
            composition_properties,
            hide_animation,
            show_animation,
            dpi: init.dpi,
            top: Cell::new(0.0),
            width: Cell::new(Self::INIT_WIDTH),
        }
    }

    pub fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut AppHitTestTreeManager,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    pub fn unmount(&self, ht: &mut AppHitTestTreeManager) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
        ht.remove_child(self.ht_root);
    }

    pub fn shutdown(&self, ht: &mut AppHitTestTreeManager) {
        ht.free_rec(self.ht_root);
    }

    pub fn set_top(&self, ht: &mut AppHitTestTreeManager, top: f32) {
        self.composition_properties
            .InsertScalar(h!("TopOffset"), top)
            .unwrap();
        self.root
            .SetSize(Vector2 {
                X: dip_to_pixels(self.width.get(), self.dpi),
                Y: dip_to_pixels(-top - Self::SPACING, self.dpi),
            })
            .unwrap();
        ht.get_mut(self.ht_root).top = top;
        ht.get_mut(self.ht_root).height = -top - Self::SPACING;

        self.top.set(top);
    }

    pub fn set_width(&self, ht: &mut AppHitTestTreeManager, width: f32) {
        self.root
            .SetSize(Vector2 {
                X: dip_to_pixels(width, self.dpi),
                Y: dip_to_pixels(-self.top.get() - Self::SPACING, self.dpi),
            })
            .unwrap();
        ht.get_mut(self.ht_root).width = width;

        self.width.set(width);
    }

    pub fn transit_hidden(&self, ht: &mut AppHitTestTreeManager) {
        self.composition_properties
            .StartAnimation(h!("ShownRate"), &self.hide_animation)
            .unwrap();
        ht.get_mut(self.ht_root).left = -self.width.get();
    }

    pub fn transit_shown(&self, ht: &mut AppHitTestTreeManager) {
        self.composition_properties
            .StartAnimation(h!("ShownRate"), &self.show_animation)
            .unwrap();
        ht.get_mut(self.ht_root).left = Self::SPACING;
    }
}

pub struct SpriteListPaneHitActionHandler {
    pub view: Rc<SpriteListPaneView>,
    pub toggle_button_view: Rc<SpriteListToggleButtonView>,
    pub cell_views: Rc<RefCell<Vec<SpriteListCellView>>>,
    pub active_cell_index: Cell<Option<usize>>,
    pub hidden: Cell<bool>,
    adjust_drag_state: Cell<Option<(f32, f32)>>,
}
impl HitTestTreeActionHandler for SpriteListPaneHitActionHandler {
    type Context = AppState;

    fn cursor(&self, sender: HitTestTreeRef, _context: &mut AppState) -> Option<HCURSOR> {
        if sender == self.view.ht_adjust_area && !self.hidden.get() {
            // TODO: 必要そうならキャッシュする
            return Some(unsafe { LoadCursorW(None, IDC_SIZEWE).unwrap() });
        }

        None
    }

    fn on_pointer_enter(
        &self,
        sender: HitTestTreeRef,
        _context: &mut AppState,
        ht: &mut AppHitTestTreeManager,
        client_x: f32,
        client_y: f32,
        client_width: f32,
        client_height: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.toggle_button_view.ht_root {
            self.toggle_button_view.on_hover();

            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.view.ht_cell_area {
            let (_, local_y, _, _) = ht.translate_client_to_tree_local(
                sender,
                client_x,
                client_y,
                client_width,
                client_height,
            );

            let index = (local_y / SpriteListCellView::CELL_HEIGHT).trunc();
            if 0.0 <= index && index < self.cell_views.borrow().len() as f32 {
                let index = index as usize;
                self.cell_views.borrow()[index].on_hover();
                self.active_cell_index.set(Some(index));
            } else {
                self.active_cell_index.set(None);
            }

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_leave(
        &self,
        sender: HitTestTreeRef,
        _context: &mut AppState,
        _ht: &mut AppHitTestTreeManager,
        _client_x: f32,
        _client_y: f32,
        _client_width: f32,
        _client_height: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.toggle_button_view.ht_root {
            self.toggle_button_view.on_hover_leave();

            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.view.ht_cell_area {
            if let Some(x) = self.active_cell_index.replace(None) {
                self.cell_views.borrow()[x].on_leave();
            }

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_down(
        &self,
        sender: HitTestTreeRef,
        _context: &mut AppState,
        _ht: &mut AppHitTestTreeManager,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.view.ht_adjust_area && !self.hidden.get() {
            self.adjust_drag_state
                .set(Some((client_x, self.view.width.get())));

            return EventContinueControl::CAPTURE_ELEMENT | EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.toggle_button_view.ht_root {
            self.toggle_button_view.on_press();

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_move(
        &self,
        sender: HitTestTreeRef,
        _context: &mut AppState,
        ht: &mut AppHitTestTreeManager,
        client_x: f32,
        client_y: f32,
        client_width: f32,
        client_height: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.view.ht_adjust_area && !self.hidden.get() {
            if let Some((base_x, base_width)) = self.adjust_drag_state.get() {
                let new_width = (base_width + (client_x - base_x)).max(10.0);
                self.view.set_width(ht, new_width);
            }

            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.view.ht_cell_area {
            let (_, local_y, _, _) = ht.translate_client_to_tree_local(
                sender,
                client_x,
                client_y,
                client_width,
                client_height,
            );

            let new_index = (local_y / SpriteListCellView::CELL_HEIGHT).trunc();
            let new_index = if 0.0 <= new_index && new_index < self.cell_views.borrow().len() as f32
            {
                Some(new_index as usize)
            } else {
                None
            };

            if self.active_cell_index.get() != new_index {
                // active changed
                if let Some(n) = self.active_cell_index.replace(new_index) {
                    self.cell_views.borrow()[n].on_leave();
                }

                if let Some(n) = new_index {
                    self.cell_views.borrow()[n].on_hover();
                }
            }

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_up(
        &self,
        sender: HitTestTreeRef,
        _context: &mut AppState,
        ht: &mut AppHitTestTreeManager,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.view.ht_adjust_area && !self.hidden.get() {
            if let Some((base_x, base_width)) = self.adjust_drag_state.replace(None) {
                let new_width = (base_width + (client_x - base_x)).max(10.0);
                self.view.set_width(ht, new_width);
            }

            return EventContinueControl::RELEASE_CAPTURE_ELEMENT
                | EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.toggle_button_view.ht_root {
            self.toggle_button_view.on_release();

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_click(
        &self,
        sender: HitTestTreeRef,
        context: &mut AppState,
        ht: &mut AppHitTestTreeManager,
        client_x: f32,
        client_y: f32,
        client_width: f32,
        client_height: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.toggle_button_view.ht_root {
            let hidden = !self.hidden.get();
            self.hidden.set(hidden);

            if hidden {
                self.toggle_button_view.transit_hidden(ht);
                self.view.transit_hidden(ht);
            } else {
                self.toggle_button_view.transit_shown(ht);
                self.view.transit_shown(ht);
            }

            return EventContinueControl::STOP_PROPAGATION
                | EventContinueControl::RECOMPUTE_POINTER_ENTER;
        }

        if sender == self.view.ht_cell_area {
            let (_, local_y, _, _) = ht.translate_client_to_tree_local(
                sender,
                client_x,
                client_y,
                client_width,
                client_height,
            );

            let click_index = (local_y / SpriteListCellView::CELL_HEIGHT).trunc();
            let click_index =
                if 0.0 <= click_index && click_index < self.cell_views.borrow().len() as f32 {
                    Some(click_index as usize)
                } else {
                    None
                };

            if let Some(x) = click_index {
                context.select_sprite(x);
            }

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }
}

pub struct SpriteListPanePresenter {
    view: Rc<SpriteListPaneView>,
    _ht_action_handler: Rc<SpriteListPaneHitActionHandler>,
}
impl SpriteListPanePresenter {
    pub fn new(init: &mut PresenterInitContext) -> Self {
        let view = Rc::new(SpriteListPaneView::new(&mut init.for_view));
        let toggle_button_view = Rc::new(SpriteListToggleButtonView::new(&mut init.for_view));
        let mut sprite_list_contents = Vec::new();
        let sprite_list_cells = Rc::new(RefCell::new(Vec::new()));

        toggle_button_view.mount(
            &view.root.Children().unwrap(),
            &mut init.for_view.ht.borrow_mut(),
            view.ht_root,
        );

        init.app_state.borrow_mut().register_sprites_view_feedback({
            let subsystem = Rc::downgrade(init.for_view.subsystem);
            let ht = Rc::downgrade(init.for_view.ht);
            let view = Rc::downgrade(&view);
            let sprite_list_cells = Rc::downgrade(&sprite_list_cells);
            let background_worker_enqueue_access =
                init.for_view.background_worker_enqueue_access.downgrade();
            let background_worker_view_update_callback =
                Rc::downgrade(init.for_view.background_worker_view_update_callback);

            move |sprites| {
                let Some(subsystem) = subsystem.upgrade() else {
                    // app teardown-ed
                    return;
                };
                let Some(background_worker_enqueue_access) =
                    background_worker_enqueue_access.upgrade()
                else {
                    // app teardown-ed
                    return;
                };
                let Some(background_worker_view_update_callback) =
                    background_worker_view_update_callback.upgrade()
                else {
                    // app teardown-ed
                    return;
                };
                let Some(ht) = ht.upgrade() else {
                    // parent teardown-ed
                    return;
                };
                let Some(view) = view.upgrade() else {
                    // parent teardown-ed
                    return;
                };
                let Some(sprite_list_cells) = sprite_list_cells.upgrade() else {
                    // parent teardown-ed
                    return;
                };

                sprite_list_contents.clear();
                sprite_list_contents.extend(sprites.iter().map(|x| (x.name.clone(), x.selected)));
                let visible_contents = &sprite_list_contents[..];
                for (n, &(ref c, sel)) in visible_contents.iter().enumerate() {
                    if sprite_list_cells.borrow().len() == n {
                        // create new one
                        let new_cell = SpriteListCellView::new(
                            &mut ViewInitContext {
                                subsystem: &subsystem,
                                ht: &ht,
                                dpi: view.dpi,
                                background_worker_enqueue_access: &background_worker_enqueue_access,
                                background_worker_view_update_callback:
                                    &background_worker_view_update_callback,
                            },
                            &c,
                            SpriteListPaneView::CELL_AREA_PADDINGS.top
                                + n as f32 * SpriteListCellView::CELL_HEIGHT,
                        );
                        new_cell.mount(&view.root.Children().unwrap());
                        if sel {
                            new_cell.on_select();
                        }
                        sprite_list_cells.borrow_mut().push(new_cell);
                        continue;
                    }

                    sprite_list_cells.borrow()[n].set_name(&c, &subsystem);
                    sprite_list_cells.borrow()[n].set_top(
                        SpriteListPaneView::CELL_AREA_PADDINGS.top
                            + n as f32 * SpriteListCellView::CELL_HEIGHT,
                    );
                    if sel {
                        sprite_list_cells.borrow()[n].on_select();
                    } else {
                        sprite_list_cells.borrow()[n].on_deselect();
                    }
                }
            }
        });

        let ht_action_handler = Rc::new(SpriteListPaneHitActionHandler {
            view: view.clone(),
            toggle_button_view: toggle_button_view.clone(),
            cell_views: sprite_list_cells,
            active_cell_index: Cell::new(None),
            hidden: Cell::new(false),
            adjust_drag_state: Cell::new(None),
        });
        init.for_view
            .ht
            .borrow_mut()
            .get_mut(view.ht_root)
            .action_handler = Some(Rc::downgrade(&ht_action_handler) as _);
        init.for_view
            .ht
            .borrow_mut()
            .get_mut(view.ht_adjust_area)
            .action_handler = Some(Rc::downgrade(&ht_action_handler) as _);
        init.for_view
            .ht
            .borrow_mut()
            .get_mut(view.ht_cell_area)
            .action_handler = Some(Rc::downgrade(&ht_action_handler) as _);
        init.for_view
            .ht
            .borrow_mut()
            .get_mut(toggle_button_view.ht_root)
            .action_handler = Some(Rc::downgrade(&ht_action_handler) as _);

        Self {
            view,
            _ht_action_handler: ht_action_handler,
        }
    }

    pub fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut AppHitTestTreeManager,
        ht_parent: HitTestTreeRef,
    ) {
        self.view.mount(children, ht, ht_parent);
    }

    pub fn unmount(&self, ht: &mut AppHitTestTreeManager) {
        self.view.unmount(ht);
    }

    pub fn shutdown(&self, ht: &mut AppHitTestTreeManager) {
        self.view.shutdown(ht);
    }

    pub fn set_top(&self, ht: &mut AppHitTestTreeManager, top: f32) {
        self.view.set_top(ht, top);
    }
}

struct AppMenuView {
    root: ContainerVisual,
    window_root: ContainerVisual,
    composition_params: CompositionPropertySet,
    show_animation: ScalarKeyFrameAnimation,
    hide_animation: ScalarKeyFrameAnimation,
    ht_root: HitTestTreeRef,
    top_offset: Cell<f32>,
    dpi: Cell<f32>,
}
impl AppMenuView {
    const CORNER_RADIUS: f32 = 8.0;
    const BEAK_SIZE: f32 = 8.0;
    const BEAK_LEFT_OFFSET: f32 = 8.0;
    const FRAME_BASE_COLOR: windows::UI::Color =
        ui_color_from_websafe_hex_rgb_with_alpha(0xfff, 64);
    const MASK_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0x000, 128);

    pub fn new(init: &mut ViewInitContext, top_offset: f32) -> Self {
        let beak_path = unsafe { init.subsystem.d2d1_factory.CreatePathGeometry().unwrap() };
        let beak_path_sink = unsafe { beak_path.Open().unwrap() };
        unsafe {
            beak_path_sink.BeginFigure(
                D2D_POINT_2F {
                    x: 0.0,
                    y: Self::BEAK_SIZE,
                },
                D2D1_FIGURE_BEGIN_FILLED,
            );
            beak_path_sink.AddLines(&[
                D2D_POINT_2F {
                    x: Self::BEAK_SIZE,
                    y: 0.0,
                },
                D2D_POINT_2F {
                    x: Self::BEAK_SIZE * 2.0,
                    y: Self::BEAK_SIZE,
                },
            ]);
            beak_path_sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            beak_path_sink.Close().unwrap();
            drop(beak_path_sink);
        }

        let beak_mask_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(Self::BEAK_SIZE * 2.0),
                Height: init.dip_to_pixels(Self::BEAK_SIZE),
            })
            .unwrap();
        draw_2d(&beak_mask_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            unsafe {
                dc.Clear(None);
                dc.FillGeometry(
                    &beak_path,
                    &dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None)?,
                    None,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let frame_mask_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(init.dip_to_pixels(Self::CORNER_RADIUS * 2.0 + 1.0)))
            .unwrap();
        draw_2d(&frame_mask_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));

                dc.Clear(None);
                dc.FillRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: 0.0,
                            top: 0.0,
                            right: Self::CORNER_RADIUS * 2.0 + 1.0,
                            bottom: Self::CORNER_RADIUS * 2.0 + 1.0,
                        },
                        radiusX: Self::CORNER_RADIUS,
                        radiusY: Self::CORNER_RADIUS,
                    },
                    &dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None)?,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let window_base_brush = create_instant_effect_brush(
            &init.subsystem,
            &CompositeEffectParams::new(&[
                GaussianBlurEffectParams::new(
                    &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                )
                .blur_amount_px(9.0)
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
                ColorSourceEffectParams {
                    color: Some(Self::FRAME_BASE_COLOR),
                }
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
            ])
            .instantiate()
            .unwrap(),
            &[(
                h!("source"),
                init.subsystem
                    .compositor
                    .CreateBackdropBrush()
                    .unwrap()
                    .cast()
                    .unwrap(),
            )],
        )
        .unwrap();

        let root = ContainerVisualParams::new()
            .expand()
            .top(init.dip_to_pixels(top_offset))
            .height(init.dip_to_pixels(-top_offset))
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let mask = SpriteVisualParams::new(
            &init
                .subsystem
                .compositor
                .CreateColorBrushWithColor(Self::MASK_COLOR)
                .unwrap(),
        )
        .expand()
        .instantiate(&init.subsystem.compositor)
        .unwrap();
        let window_root = ContainerVisualParams::new()
            .size_sq(init.dip_to_pixels(128.0))
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let beak = SpriteVisualParams::new(
            &CompositionMaskBrushParams {
                source: &window_base_brush,
                mask: &CompositionSurfaceBrushParams::new(&beak_mask_surface)
                    .instantiate(&init.subsystem.compositor)
                    .unwrap(),
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
        )
        .size(Vector2 {
            X: init.dip_to_pixels(Self::BEAK_SIZE * 2.0),
            Y: init.dip_to_pixels(Self::BEAK_SIZE),
        })
        .offset_xy(Vector2 {
            X: init.dip_to_pixels(Self::BEAK_LEFT_OFFSET),
            Y: init.dip_to_pixels(-Self::BEAK_SIZE),
        })
        .instantiate(&init.subsystem.compositor)
        .unwrap();
        let window_frame = SpriteVisualParams::new(
            &CompositionMaskBrushParams {
                source: &window_base_brush,
                mask: &CompositionNineGridBrushParams::new(
                    &CompositionSurfaceBrushParams::new(&frame_mask_surface)
                        .stretch(CompositionStretch::Fill)
                        .instantiate(&init.subsystem.compositor)
                        .unwrap(),
                )
                .insets(init.dip_to_pixels(Self::CORNER_RADIUS))
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
        )
        .expand()
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        window_root.Children().unwrap().InsertAtTop(&beak).unwrap();
        window_root
            .Children()
            .unwrap()
            .InsertAtTop(&window_frame)
            .unwrap();

        root.Children().unwrap().InsertAtTop(&mask).unwrap();
        root.Children().unwrap().InsertAtTop(&window_root).unwrap();

        let composition_params = init.subsystem.compositor.CreatePropertySet().unwrap();
        composition_params
            .InsertScalar(h!("VisibleRate"), 0.0)
            .unwrap();
        composition_params
            .InsertVector2(h!("WindowOffset"), Vector2 { X: 0.0, Y: 0.0 })
            .unwrap();
        composition_params
            .InsertScalar(h!("DPI"), init.dpi)
            .unwrap();

        let opacity_expression = init
            .subsystem
            .compositor
            .CreateExpressionAnimationWithExpression(h!("cp.VisibleRate"))
            .unwrap();
        opacity_expression
            .SetExpressionReferenceParameter(h!("cp"), &composition_params)
            .unwrap();
        opacity_expression.SetTarget(h!("Opacity")).unwrap();
        mask.StartAnimation(h!("Opacity"), &opacity_expression)
            .unwrap();
        window_root
            .StartAnimation(h!("Opacity"), &opacity_expression)
            .unwrap();

        let window_offset_expression = init
            .subsystem
            .compositor
            .CreateExpressionAnimationWithExpression(h!(
                "Vector3(cp.WindowOffset.X, cp.WindowOffset.Y - (16.0 * (1.0 - cp.VisibleRate) * cp.DPI / 96.0), 0.0)"
            ))
            .unwrap();
        window_offset_expression
            .SetExpressionReferenceParameter(h!("cp"), &composition_params)
            .unwrap();
        window_offset_expression.SetTarget(h!("Offset")).unwrap();
        window_root
            .StartAnimation(h!("Offset"), &window_offset_expression)
            .unwrap();

        let easing = init
            .subsystem
            .compositor
            .CreateCubicBezierEasingFunction(
                Vector2 { X: 0.0, Y: 0.0 },
                Vector2 { X: 0.25, Y: 1.0 },
            )
            .unwrap();
        let show_animation = SimpleScalarAnimationParams::new(0.0, 1.0, &easing)
            .duration(timespan_ms(200))
            .target(h!("VisibleRate"))
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let hide_animation = SimpleScalarAnimationParams::new(1.0, 0.0, &easing)
            .duration(timespan_ms(200))
            .target(h!("VisibleRate"))
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let ht_root = init.ht.borrow_mut().alloc(HitTestTreeData {
            left: 0.0,
            top: top_offset,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: 0.0,
            height: -top_offset,
            width_adjustment_factor: 1.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });

        let icon_svg_stream: IRandomAccessStream = unsafe {
            CreateRandomAccessStreamOnFile(
                w!("./resources/file_open.svg"),
                FileAccessMode::Read.0 as _,
            )
            .unwrap()
        };
        let icon_svg_stream: IStream =
            unsafe { CreateStreamOverRandomAccessStream(&icon_svg_stream).unwrap() };
        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(init.dip_to_pixels(20.0)))
            .unwrap();
        draw_2d(&icon_surface, |dc, offset| {
            let dc5: ID2D1DeviceContext5 = dc.cast()?;

            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            let svg = unsafe {
                dc5.CreateSvgDocument(
                    &icon_svg_stream,
                    D2D_SIZE_F {
                        width: pixels_to_dip(20, init.dpi),
                        height: pixels_to_dip(20, init.dpi),
                    },
                )?
            };

            unsafe {
                dc.Clear(None);
                dc5.DrawSvgDocument(&svg);
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let open_cell = ContainerVisualParams::new()
            .expand_width()
            .height(init.dip_to_pixels(20.0))
            .offset_xy(Vector2 {
                X: init.dip_to_pixels(0.0),
                Y: init.dip_to_pixels(Self::CORNER_RADIUS),
            })
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let icon = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&icon_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size_sq(init.dip_to_pixels(20.0))
        .left(init.dip_to_pixels(Self::CORNER_RADIUS))
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let cell_label_layout = init
            .subsystem
            .new_text_layout_unrestricted("開く", &init.subsystem.default_ui_format)
            .unwrap();
        let mut cell_label_metrics = MaybeUninit::uninit();
        unsafe {
            cell_label_layout
                .GetMetrics(cell_label_metrics.as_mut_ptr())
                .unwrap();
        }
        let cell_label_metrics = unsafe { cell_label_metrics.assume_init() };
        let cell_label = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(cell_label_metrics.width),
                Height: init.dip_to_pixels(cell_label_metrics.height),
            })
            .unwrap();
        draw_2d(&cell_label, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);

                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: init.signed_pixels_to_dip(offset.x),
                        y: init.signed_pixels_to_dip(offset.y),
                    },
                    &cell_label_layout,
                    &dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None)?,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();
        let cell_label = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&cell_label)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .anchor_point(Vector2 { X: 0.0, Y: 0.5 })
        .relative_offset_adjustment_xy(Vector2 { X: 0.0, Y: 0.5 })
        .offset_xy(Vector2 {
            X: init.dip_to_pixels(Self::CORNER_RADIUS + 20.0 + 4.0),
            Y: 0.0,
        })
        .size(Vector2 {
            X: init.dip_to_pixels(cell_label_metrics.width),
            Y: init.dip_to_pixels(cell_label_metrics.height),
        })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let icon_svg_stream: IRandomAccessStream = unsafe {
            CreateRandomAccessStreamOnFile(
                w!("./resources/file_save.svg"),
                FileAccessMode::Read.0 as _,
            )
            .unwrap()
        };
        let icon_svg_stream: IStream =
            unsafe { CreateStreamOverRandomAccessStream(&icon_svg_stream).unwrap() };
        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(init.dip_to_pixels(20.0)))
            .unwrap();
        draw_2d(&icon_surface, |dc, offset| {
            let dc5: ID2D1DeviceContext5 = dc.cast()?;

            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            let svg = unsafe {
                dc5.CreateSvgDocument(
                    &icon_svg_stream,
                    D2D_SIZE_F {
                        width: pixels_to_dip(20, init.dpi),
                        height: pixels_to_dip(20, init.dpi),
                    },
                )?
            };

            unsafe {
                dc.Clear(None);
                dc5.DrawSvgDocument(&svg);
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        open_cell.Children().unwrap().InsertAtTop(&icon).unwrap();
        open_cell
            .Children()
            .unwrap()
            .InsertAtTop(&cell_label)
            .unwrap();

        let save_cell = ContainerVisualParams::new()
            .expand_width()
            .height(init.dip_to_pixels(20.0))
            .offset_xy(Vector2 {
                X: init.dip_to_pixels(0.0),
                Y: init.dip_to_pixels(Self::CORNER_RADIUS + 24.0),
            })
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let icon = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&icon_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size_sq(init.dip_to_pixels(20.0))
        .left(init.dip_to_pixels(Self::CORNER_RADIUS))
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let cell_label_layout = init
            .subsystem
            .new_text_layout_unrestricted("保存", &init.subsystem.default_ui_format)
            .unwrap();
        let mut cell_label_metrics = MaybeUninit::uninit();
        unsafe {
            cell_label_layout
                .GetMetrics(cell_label_metrics.as_mut_ptr())
                .unwrap();
        }
        let cell_label_metrics = unsafe { cell_label_metrics.assume_init() };
        let cell_label = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(cell_label_metrics.width),
                Height: init.dip_to_pixels(cell_label_metrics.height),
            })
            .unwrap();
        draw_2d(&cell_label, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);

                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: init.signed_pixels_to_dip(offset.x),
                        y: init.signed_pixels_to_dip(offset.y),
                    },
                    &cell_label_layout,
                    &dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None)?,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();
        let cell_label = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&cell_label)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .anchor_point(Vector2 { X: 0.0, Y: 0.5 })
        .relative_offset_adjustment_xy(Vector2 { X: 0.0, Y: 0.5 })
        .offset_xy(Vector2 {
            X: init.dip_to_pixels(Self::CORNER_RADIUS + 20.0 + 4.0),
            Y: 0.0,
        })
        .size(Vector2 {
            X: init.dip_to_pixels(cell_label_metrics.width),
            Y: init.dip_to_pixels(cell_label_metrics.height),
        })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        save_cell.Children().unwrap().InsertAtTop(&icon).unwrap();
        save_cell
            .Children()
            .unwrap()
            .InsertAtTop(&cell_label)
            .unwrap();

        window_root
            .Children()
            .unwrap()
            .InsertAtTop(&open_cell)
            .unwrap();
        window_root
            .Children()
            .unwrap()
            .InsertAtTop(&save_cell)
            .unwrap();

        Self {
            root,
            window_root,
            composition_params,
            show_animation,
            hide_animation,
            ht_root,
            top_offset: Cell::new(top_offset),
            dpi: Cell::new(init.dpi),
        }
    }

    pub fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut AppHitTestTreeManager,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    pub fn set_window_offset_by_beak_peak(&self, peak_x: f32, peak_y: f32) {
        let dpi = self.dpi.get();
        let top_offset = self.top_offset.get();

        self.composition_params
            .InsertVector2(
                h!("WindowOffset"),
                Vector2 {
                    X: dip_to_pixels(peak_x - Self::BEAK_LEFT_OFFSET - Self::BEAK_SIZE, dpi),
                    Y: dip_to_pixels(peak_y + Self::BEAK_SIZE - top_offset, dpi),
                },
            )
            .unwrap();
    }

    pub fn show(&self, immediate: bool) {
        if immediate {
            self.composition_params
                .InsertScalar(h!("VisibleRate"), 1.0)
                .unwrap();
        } else {
            self.composition_params
                .StartAnimation(h!("VisibleRate"), &self.show_animation)
                .unwrap();
        }
    }

    pub fn hide(&self, immediate: bool) {
        if immediate {
            self.composition_params
                .InsertScalar(h!("VisibleRate"), 0.0)
                .unwrap();
        } else {
            self.composition_params
                .StartAnimation(h!("VisibleRate"), &self.hide_animation)
                .unwrap();
        }
    }
}

pub struct QuadTreeElementIndexIter<'a> {
    qt: &'a QuadTree,
    index: u64,
    current_level: usize,
    current_internal_iter: Option<std::collections::hash_set::Iter<'a, usize>>,
}
impl Iterator for QuadTreeElementIndexIter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if let Some(a) = self.current_internal_iter.as_mut() {
            if let Some(&x) = a.next() {
                return Some(x);
            }

            // 全部回りきった
            self.current_internal_iter = None;
            self.current_level += 1;
        }

        // 下層の要素を探す
        while self.current_level < 32 {
            let index = if self.current_level == 0 {
                0
            } else {
                self.index >> (64 - self.current_level * 2)
            };
            let elements = &self
                .qt
                .element_index_for_region
                .get(self.current_level)
                .and_then(|xs| xs.get(index as usize));
            if let Some(elements) = elements.filter(|x| !x.is_empty()) {
                // この層には要素がある
                self.current_internal_iter = Some(elements.iter());
                return self.next();
            }

            self.current_level += 1;
        }

        // もうない
        None
    }
}

/// ビットを一つおきに分散させる
/// 例: 0b11000110 => 0b01_01_00_00_00_01_01_00
const fn interleave(bits: u64) -> u64 {
    let bits = (bits | (bits << 32)) & 0xffff_ffff_ffff_ffff;
    let bits = (bits | (bits << 16)) & 0x0000_ffff_0000_ffff;
    let bits = (bits | (bits << 8)) & 0x00ff_00ff_00ff_00ff;
    let bits = (bits | (bits << 4)) & 0x0f0f_0f0f_0f0f_0f0f;
    let bits = (bits | (bits << 2)) & 0x3333_3333_3333_3333;
    let bits = (bits | (bits << 1)) & 0x5555_5555_5555_5555;

    bits
}

// http://marupeke296.com/COL_2D_No8_QuadTree.html だいたいこれの実装
pub struct QuadTree {
    pub element_index_for_region: Vec<Vec<HashSet<usize>>>,
}
impl QuadTree {
    pub fn new() -> Self {
        Self {
            element_index_for_region: Vec::new(),
        }
    }

    pub fn bind(&mut self, level: usize, index: u64, n: usize) {
        while self.element_index_for_region.len() <= level {
            self.element_index_for_region.push(Vec::new());
        }

        while self.element_index_for_region[level].len() <= index as _ {
            self.element_index_for_region[level].push(HashSet::new());
        }

        self.element_index_for_region[level][index as usize].insert(n);
    }

    pub const fn iter_possible_element_indices(
        &self,
        x_pixels: u32,
        y_pixels: u32,
    ) -> impl Iterator<Item = usize> {
        QuadTreeElementIndexIter {
            qt: self,
            index: Self::compute_location_index(x_pixels, y_pixels),
            current_level: 0,
            current_internal_iter: None,
        }
    }

    pub const fn compute_location_index(location_x_pixels: u32, location_y_pixels: u32) -> u64 {
        // 一旦一律16(2^4)px角まで分割する
        let (xv, yv) = (
            (location_x_pixels >> 4) as u64,
            (location_y_pixels >> 4) as u64,
        );

        // のちのシフト操作で情報が欠けないように検査いれる
        assert!(xv.leading_zeros() >= 32, "too many divisions!");
        assert!(yv.leading_zeros() >= 32, "too many divisions!");

        interleave(xv) | (interleave(yv) << 1)
    }

    pub const fn rect_index_and_level(
        left: u32,
        top: u32,
        right: u32,
        bottom: u32,
    ) -> (u64, usize) {
        let lt_location = Self::compute_location_index(left, top);
        let rb_location = Self::compute_location_index(right, bottom);
        // xorをとるとズレているレベルの2bitが00にならないので、それでどの分割レベルで跨いでいないか（どのレベルの所属インデックスまでが一致しているか）を判定できるっぽい
        let xor = lt_location ^ rb_location;

        // 先頭の0のビット数を数えて、
        // 0, 1...Lv0(root, 分割無し)
        // 2, 3...Lv1(全体を4分割したうちのどこか)
        // 4, 5...Lv2(全体を16分割したうちのどこか)
        // ...となるように計算する
        let level = (xor.leading_zeros() / 2) as usize;
        // 符号なし整数なので右シフト後は上が0で埋まるはず
        let index = lt_location >> (64 - level * 2);

        (index, level)
    }
}

enum DragState {
    None,
    Grid {
        base_x_pixels: f32,
        base_y_pixels: f32,
        drag_start_client_x_pixels: f32,
        drag_start_client_y_pixels: f32,
    },
    Sprite {
        index: usize,
        base_x_pixels: f32,
        base_y_pixels: f32,
        base_width_pixels: f32,
        base_height_pixels: f32,
        drag_start_client_x_pixels: f32,
        drag_start_client_y_pixels: f32,
    },
}

struct AppWindowHitTestTreeActionHandler {
    grid_view: Arc<AtlasBaseGridView>,
    sprite_atlas_border_view: Rc<SpriteAtlasBorderView>,
    selected_sprite_marker_view: Rc<CurrentSelectedSpriteMarkerView>,
    menu_view: Rc<AppMenuView>,
    qt: RefCell<QuadTree>,
    sprite_rect_cached: RefCell<Vec<(u32, u32, u32, u32)>>,
    drag_data: RefCell<DragState>,
    dpi: Cell<f32>,
    ht_root: HitTestTreeRef,
}
impl HitTestTreeActionHandler for AppWindowHitTestTreeActionHandler {
    type Context = AppState;

    fn hit_active(&self, sender: HitTestTreeRef, context: &Self::Context) -> bool {
        if sender == self.menu_view.ht_root && !context.is_visible_menu() {
            // AppMenuが表示されていないときはAppMenuのヒットテストを無効化する
            return false;
        }

        true
    }

    fn on_pointer_down(
        &self,
        sender: HitTestTreeRef,
        context: &mut AppState,
        _ht: &mut AppHitTestTreeManager,
        client_x: f32,
        client_y: f32,
    ) -> EventContinueControl {
        if sender == self.menu_view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.ht_root {
            let dpi = self.dpi.get();
            let (current_offset_x, current_offset_y) = *self.grid_view.offset_pixels.read();

            let (pointing_x, pointing_y) = (
                dip_to_pixels(client_x, dpi) + current_offset_x,
                dip_to_pixels(client_y, dpi) + current_offset_y,
            );
            let sprite_drag_target_index =
                context.selected_sprites_with_index().rev().find(|(_, x)| {
                    x.left as f32 <= pointing_x
                        && pointing_x <= x.right() as f32
                        && x.top as f32 <= pointing_y
                        && pointing_y <= x.bottom() as f32
                });
            if let Some((sprite_drag_target_index, target_sprite_ref)) = sprite_drag_target_index {
                // 選択中のスプライトの上で操作が開始された
                self.selected_sprite_marker_view.hide();
                *self.drag_data.borrow_mut() = DragState::Sprite {
                    index: sprite_drag_target_index,
                    base_x_pixels: target_sprite_ref.left as f32,
                    base_y_pixels: target_sprite_ref.top as f32,
                    base_width_pixels: target_sprite_ref.width as f32,
                    base_height_pixels: target_sprite_ref.height as f32,
                    drag_start_client_x_pixels: dip_to_pixels(client_x, dpi),
                    drag_start_client_y_pixels: dip_to_pixels(client_y, dpi),
                };
            } else {
                *self.drag_data.borrow_mut() = DragState::Grid {
                    base_x_pixels: current_offset_x,
                    base_y_pixels: current_offset_y,
                    drag_start_client_x_pixels: dip_to_pixels(client_x, dpi),
                    drag_start_client_y_pixels: dip_to_pixels(client_y, dpi),
                };
            }

            return EventContinueControl::STOP_PROPAGATION | EventContinueControl::CAPTURE_ELEMENT;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_move(
        &self,
        sender: HitTestTreeRef,
        _context: &mut AppState,
        _ht: &mut AppHitTestTreeManager,
        client_x: f32,
        client_y: f32,
        _client_width: f32,
        _client_height: f32,
    ) -> EventContinueControl {
        if sender == self.menu_view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.ht_root {
            match &*self.drag_data.borrow() {
                DragState::None => {}
                &DragState::Grid {
                    base_x_pixels,
                    base_y_pixels,
                    drag_start_client_x_pixels,
                    drag_start_client_y_pixels,
                } => {
                    let dpi = self.dpi.get();
                    let (dx, dy) = (
                        drag_start_client_x_pixels - dip_to_pixels(client_x, dpi),
                        drag_start_client_y_pixels - dip_to_pixels(client_y, dpi),
                    );
                    self.grid_view
                        .set_offset(base_x_pixels + dx, base_y_pixels + dy);
                    self.sprite_atlas_border_view
                        .set_view_offset(base_x_pixels + dx, base_y_pixels + dy);
                    self.selected_sprite_marker_view
                        .set_view_offset(base_x_pixels + dx, base_y_pixels + dy);

                    return EventContinueControl::STOP_PROPAGATION;
                }
                &DragState::Sprite {
                    index,
                    base_x_pixels,
                    base_y_pixels,
                    drag_start_client_x_pixels,
                    drag_start_client_y_pixels,
                    ..
                } => {
                    let dpi = self.dpi.get();
                    let (dx, dy) = (
                        dip_to_pixels(client_x, dpi) - drag_start_client_x_pixels,
                        dip_to_pixels(client_y, dpi) - drag_start_client_y_pixels,
                    );
                    let (sx, sy) = (
                        (base_x_pixels + dx).max(0.0) as u32,
                        (base_y_pixels + dy).max(0.0) as u32,
                    );
                    self.grid_view.update_sprite_offset(index, sx as _, sy as _);

                    return EventContinueControl::STOP_PROPAGATION;
                }
            }
        }

        EventContinueControl::empty()
    }

    fn on_pointer_up(
        &self,
        sender: HitTestTreeRef,
        context: &mut AppState,
        _ht: &mut AppHitTestTreeManager,
        client_x: f32,
        client_y: f32,
    ) -> EventContinueControl {
        if sender == self.menu_view.ht_root {
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.ht_root {
            match core::mem::replace(&mut *self.drag_data.borrow_mut(), DragState::None) {
                DragState::None => {}
                DragState::Grid {
                    base_x_pixels,
                    base_y_pixels,
                    drag_start_client_x_pixels,
                    drag_start_client_y_pixels,
                } => {
                    let dpi = self.dpi.get();
                    let (dx, dy) = (
                        drag_start_client_x_pixels - dip_to_pixels(client_x, dpi),
                        drag_start_client_y_pixels - dip_to_pixels(client_y, dpi),
                    );
                    self.grid_view
                        .set_offset(base_x_pixels + dx, base_y_pixels + dy);
                    self.sprite_atlas_border_view
                        .set_view_offset(base_x_pixels + dx, base_y_pixels + dy);
                    self.selected_sprite_marker_view
                        .set_view_offset(base_x_pixels + dx, base_y_pixels + dy);
                }
                DragState::Sprite {
                    index,
                    base_x_pixels,
                    base_y_pixels,
                    base_width_pixels,
                    base_height_pixels,
                    drag_start_client_x_pixels,
                    drag_start_client_y_pixels,
                } => {
                    let dpi = self.dpi.get();
                    let (dx, dy) = (
                        dip_to_pixels(client_x, dpi) - drag_start_client_x_pixels,
                        dip_to_pixels(client_y, dpi) - drag_start_client_y_pixels,
                    );
                    let (sx, sy) = (
                        (base_x_pixels + dx).max(0.0) as u32,
                        (base_y_pixels + dy).max(0.0) as u32,
                    );
                    context.set_sprite_offset(index, sx, sy);

                    // 選択インデックスが変わるわけではないのでここで選択枠Viewを復帰させる
                    self.selected_sprite_marker_view.focus(
                        sx as _,
                        sy as _,
                        base_width_pixels,
                        base_height_pixels,
                    );
                }
            }

            return EventContinueControl::STOP_PROPAGATION
                | EventContinueControl::RELEASE_CAPTURE_ELEMENT;
        }

        EventContinueControl::empty()
    }

    fn on_click(
        &self,
        sender: HitTestTreeRef,
        context: &mut Self::Context,
        _ht: &mut HitTestTreeManager<Self::Context>,
        client_x: f32,
        client_y: f32,
        _client_width: f32,
        _client_height: f32,
    ) -> EventContinueControl {
        if sender == self.menu_view.ht_root {
            context.toggle_menu();
            return EventContinueControl::STOP_PROPAGATION;
        }

        if sender == self.ht_root {
            let dpi = self.dpi.get();
            let x = dip_to_pixels(client_x, dpi) + self.grid_view.offset_pixels.read().0;
            let y = dip_to_pixels(client_y, dpi) + self.grid_view.offset_pixels.read().1;

            let mut max_index = None;
            for n in self
                .qt
                .borrow()
                .iter_possible_element_indices(x as _, y as _)
            {
                let (left, top, right, bottom) = self.sprite_rect_cached.borrow()[n];
                if left as f32 <= x && x <= right as f32 && top as f32 <= y && y <= bottom as f32 {
                    // 大きいインデックスのものが最前面にいるのでmaxをとる
                    max_index = Some(max_index.map_or(n, |x: usize| x.max(n)));
                }
            }

            if let Some(mx) = max_index {
                context.select_sprite(mx);
            } else {
                context.deselect_sprite();
            }

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }
}

pub struct AppWindowDpiHandler {
    ht_action_handler: Rc<AppWindowHitTestTreeActionHandler>,
}
impl DpiHandler for AppWindowDpiHandler {
    fn on_dpi_changed(&self, new_dpi: f32) {
        self.ht_action_handler.dpi.set(new_dpi);
    }
}

struct AppWindowPresenter {
    root: ContainerVisual,
    ht_root: HitTestTreeRef,
    grid_view: Arc<AtlasBaseGridView>,
    _sprite_atlas_border_view: Rc<SpriteAtlasBorderView>,
    _selected_sprite_marker_view: Rc<CurrentSelectedSpriteMarkerView>,
    sprite_list_pane: SpriteListPanePresenter,
    header: AppHeaderPresenter,
    _menu_view: Rc<AppMenuView>,
    file_dnd_overlay: Rc<FileDragAndDropOverlayView>,
    _ht_action_handler: Rc<AppWindowHitTestTreeActionHandler>,
    _dpi_handler: Rc<AppWindowDpiHandler>,
}
impl AppWindowPresenter {
    pub fn new(init: &mut PresenterInitContext, init_client_size_pixels: &SizePixels) -> Self {
        let root = ContainerVisualParams::new()
            .expand()
            .instantiate(&init.for_view.subsystem.compositor)
            .unwrap();

        let bg = SpriteVisualParams::new(
            &init
                .for_view
                .subsystem
                .compositor
                .CreateColorBrushWithColor(BG_COLOR)
                .unwrap(),
        )
        .expand()
        .instantiate(&init.for_view.subsystem.compositor)
        .unwrap();

        let ht_root = init.for_view.ht.borrow_mut().alloc(HitTestTreeData {
            left: 0.0,
            top: 0.0,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: 0.0,
            height: 0.0,
            width_adjustment_factor: 1.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: vec![],
            action_handler: None,
        });

        let grid_view = Arc::new(AtlasBaseGridView::new(&mut init.for_view, 128, 128));
        grid_view.resize(
            init_client_size_pixels.width,
            init_client_size_pixels.height,
        );

        let sprite_atlas_border_view = Rc::new(SpriteAtlasBorderView::new(&mut init.for_view));

        let selected_sprite_marker_view =
            Rc::new(CurrentSelectedSpriteMarkerView::new(&mut init.for_view));

        let sprite_list_pane = SpriteListPanePresenter::new(init);

        let header = AppHeaderPresenter::new(init, "Peridot SpriteAtlas Visualizer/Editor");

        let menu_view = Rc::new(AppMenuView::new(&mut init.for_view, header.height()));
        menu_view.set_window_offset_by_beak_peak(
            header.height() * 0.5,
            header.height() * 0.5 + 6.0 + 2.0,
        );

        let file_dnd_overlay = Rc::new(FileDragAndDropOverlayView::new(&mut init.for_view));

        sprite_list_pane.set_top(&mut init.for_view.ht.borrow_mut(), header.height());
        grid_view.set_offset(0.0, -init.for_view.dip_to_pixels(header.height()));

        root.Children().unwrap().InsertAtBottom(&bg).unwrap();
        grid_view.mount(&root.Children().unwrap());
        sprite_atlas_border_view.mount(&root.Children().unwrap());
        selected_sprite_marker_view.mount(&root.Children().unwrap());
        sprite_list_pane.mount(
            &root.Children().unwrap(),
            &mut init.for_view.ht.borrow_mut(),
            ht_root,
        );
        header.mount(
            &root.Children().unwrap(),
            &mut init.for_view.ht.borrow_mut(),
            ht_root,
        );
        menu_view.mount(
            &root.Children().unwrap(),
            &mut init.for_view.ht.borrow_mut(),
            ht_root,
        );
        file_dnd_overlay.mount(&root.Children().unwrap());

        let ht_action_handler = Rc::new(AppWindowHitTestTreeActionHandler {
            grid_view: grid_view.clone(),
            sprite_atlas_border_view: sprite_atlas_border_view.clone(),
            selected_sprite_marker_view: selected_sprite_marker_view.clone(),
            menu_view: menu_view.clone(),
            qt: RefCell::new(QuadTree::new()),
            sprite_rect_cached: RefCell::new(Vec::new()),
            drag_data: RefCell::new(DragState::None),
            dpi: Cell::new(init.for_view.dpi),
            ht_root,
        });
        init.for_view
            .ht
            .borrow_mut()
            .get_mut(ht_root)
            .action_handler = Some(Rc::downgrade(&ht_action_handler) as _);
        init.for_view
            .ht
            .borrow_mut()
            .get_mut(menu_view.ht_root)
            .action_handler = Some(Rc::downgrade(&ht_action_handler) as _);

        let dpi_handler = Rc::new(AppWindowDpiHandler {
            ht_action_handler: ht_action_handler.clone(),
        });
        init.dpi_handlers.push(Rc::downgrade(&dpi_handler) as _);

        init.app_state.borrow_mut().register_sprites_view_feedback({
            let grid_view = Arc::downgrade(&grid_view);
            let selected_sprite_marker_view = Rc::downgrade(&selected_sprite_marker_view);
            let mut last_selected_index = None;
            let ht_action_handler = Rc::downgrade(&ht_action_handler);

            move |sprites| {
                let Some(grid_view) = grid_view.upgrade() else {
                    // parent teardown-ed
                    return;
                };
                let Some(selected_sprite_marker_view) = selected_sprite_marker_view.upgrade()
                else {
                    // parent teardown-ed
                    return;
                };
                let Some(ht_action_handler) = ht_action_handler.upgrade() else {
                    // parent teardown-ed
                    return;
                };

                while ht_action_handler.sprite_rect_cached.borrow().len() > sprites.len() {
                    // 削除分
                    let n = ht_action_handler.sprite_rect_cached.borrow().len() - 1;
                    let old = ht_action_handler
                        .sprite_rect_cached
                        .borrow_mut()
                        .pop()
                        .unwrap();
                    let (index, level) = QuadTree::rect_index_and_level(old.0, old.1, old.2, old.3);

                    ht_action_handler.qt.borrow_mut().element_index_for_region[level]
                        [index as usize]
                        .remove(&n);
                }
                for (n, (old, new)) in ht_action_handler
                    .sprite_rect_cached
                    .borrow_mut()
                    .iter_mut()
                    .zip(sprites.iter())
                    .enumerate()
                {
                    // 移動分
                    if old.0 == new.left
                        && old.1 == new.top
                        && old.2 == new.right()
                        && old.3 == new.bottom()
                    {
                        // 座標変化なし
                        continue;
                    }

                    let (old_index, old_level) =
                        QuadTree::rect_index_and_level(old.0, old.1, old.2, old.3);
                    let (new_index, new_level) = QuadTree::rect_index_and_level(
                        new.left,
                        new.top,
                        new.right(),
                        new.bottom(),
                    );
                    *old = (new.left, new.top, new.right(), new.bottom());

                    if old_level == new_level && old_index == new_index {
                        // 所属ブロックに変化なし
                        continue;
                    }

                    ht_action_handler.qt.borrow_mut().element_index_for_region[old_level]
                        [old_index as usize]
                        .remove(&n);
                    ht_action_handler
                        .qt
                        .borrow_mut()
                        .bind(new_level, new_index, n);
                }
                let new_base = ht_action_handler.sprite_rect_cached.borrow().len();
                for (n, new) in sprites.iter().enumerate().skip(new_base) {
                    // 追加分
                    let (index, level) = QuadTree::rect_index_and_level(
                        new.left,
                        new.top,
                        new.right(),
                        new.bottom(),
                    );
                    ht_action_handler.qt.borrow_mut().bind(level, index, n);
                    ht_action_handler.sprite_rect_cached.borrow_mut().push((
                        new.left,
                        new.top,
                        new.right(),
                        new.bottom(),
                    ));
                }

                grid_view.update_sprites(sprites);

                // TODO: Model的には複数選択できる形にしてるけどViewはどうしようか......
                let selected_index = sprites.iter().position(|x| x.selected);
                if selected_index != last_selected_index {
                    last_selected_index = selected_index;
                    if let Some(x) = selected_index {
                        selected_sprite_marker_view.focus(
                            sprites[x].left as _,
                            sprites[x].top as _,
                            sprites[x].width as _,
                            sprites[x].height as _,
                        );
                    } else {
                        selected_sprite_marker_view.hide();
                    }
                }
            }
        });
        init.app_state
            .borrow_mut()
            .register_atlas_size_view_feedback({
                let border_view = Rc::downgrade(&sprite_atlas_border_view);
                let grid_view = Arc::downgrade(&grid_view);

                move |size| {
                    let Some(border_view) = border_view.upgrade() else {
                        // parent teardown-ed
                        return;
                    };
                    let Some(grid_view) = grid_view.upgrade() else {
                        // parent teardown-ed
                        return;
                    };

                    border_view.set_size(*size);
                    grid_view.set_atlas_size(size.width, size.height);
                }
            });
        init.app_state
            .borrow_mut()
            .register_visible_menu_view_feedback({
                let menu_view = Rc::downgrade(&menu_view);

                move |visible, initial_fire| {
                    let Some(menu_view) = menu_view.upgrade() else {
                        // parent teardown-ed
                        return;
                    };

                    if visible {
                        menu_view.show(initial_fire);
                    } else {
                        menu_view.hide(initial_fire);
                    }
                }
            });

        Self {
            root,
            ht_root,
            grid_view,
            _sprite_atlas_border_view: sprite_atlas_border_view,
            _selected_sprite_marker_view: selected_sprite_marker_view,
            sprite_list_pane,
            header,
            _menu_view: menu_view,
            file_dnd_overlay,
            _ht_action_handler: ht_action_handler,
            _dpi_handler: dpi_handler,
        }
    }
}

struct AppWindowStateModel {
    ht: Rc<RefCell<AppHitTestTreeManager>>,
    client_size_pixels: SizePixels,
    dpi: f32,
    dpi_handlers: Vec<std::rc::Weak<dyn DpiHandler>>,
    pointer_input_manager: PointerInputManager,
    _composition_target: DesktopWindowTarget,
    root_presenter: AppWindowPresenter,
    app_state: Rc<RefCell<AppState>>,
}
impl AppWindowStateModel {
    pub fn new(
        subsystem: &Rc<Subsystem>,
        bound_hwnd: HWND,
        app_state: &Rc<RefCell<AppState>>,
        background_worker: &BackgroundWorker,
        background_worker_view_update_callback: &Rc<
            RefCell<Vec<Box<dyn FnMut(&[Option<String>])>>>,
        >,
    ) -> Self {
        let ht = Rc::new(RefCell::new(HitTestTreeManager::new()));
        let mut client_size_pixels = core::mem::MaybeUninit::uninit();
        unsafe {
            GetClientRect(bound_hwnd, client_size_pixels.as_mut_ptr()).unwrap();
        }
        let client_size_pixels = unsafe { client_size_pixels.assume_init_ref() };
        let client_size_pixels = SizePixels {
            width: (client_size_pixels.right - client_size_pixels.left)
                .try_into()
                .expect("negative size?"),
            height: (client_size_pixels.bottom - client_size_pixels.top)
                .try_into()
                .expect("negative size?"),
        };
        let dpi = unsafe { GetDpiForWindow(bound_hwnd) as f32 };
        let mut dpi_handlers = Vec::new();
        let pointer_input_manager = PointerInputManager::new();

        let composition_target = unsafe {
            subsystem
                .compositor_desktop_interop
                .CreateDesktopWindowTarget(bound_hwnd, true)
                .unwrap()
        };

        let root_presenter = AppWindowPresenter::new(
            &mut PresenterInitContext {
                for_view: ViewInitContext {
                    subsystem,
                    ht: &ht,
                    dpi,
                    background_worker_enqueue_access: &background_worker.enqueue_access(),
                    background_worker_view_update_callback,
                },
                dpi_handlers: &mut dpi_handlers,
                app_state,
            },
            &client_size_pixels,
        );
        composition_target.SetRoot(&root_presenter.root).unwrap();

        ht.borrow().dump(root_presenter.ht_root);

        tracing::info!({ dpi }, "window state initialized");

        Self {
            ht,
            client_size_pixels,
            dpi,
            dpi_handlers,
            pointer_input_manager,
            _composition_target: composition_target,
            root_presenter,
            app_state: app_state.clone(),
        }
    }

    pub fn shutdown(&mut self) {
        self.root_presenter
            .sprite_list_pane
            .shutdown(&mut self.ht.borrow_mut());
    }

    pub fn client_size_dip(&self) -> Size {
        self.client_size_pixels.to_dip(self.dpi)
    }

    pub fn nc_hittest(&self, hwnd: HWND, _wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        let resize_h = unsafe { GetSystemMetrics(SM_CYSIZEFRAME) };

        let (x, y) = (
            (lparam.0 & 0xffff) as i16 as i32,
            ((lparam.0 >> 16) & 0xffff) as i16 as i32,
        );
        let mut p = [POINT { x, y }];
        unsafe {
            MapWindowPoints(None, Some(hwnd), &mut p);
        }
        let [POINT { x, y }] = p;

        if 0 > x
            || x > self.client_size_pixels.width as i32
            || 0 > y
            || y > self.client_size_pixels.height as i32
        {
            // ウィンドウ範囲外はシステムにおまかせ
            return None;
        }

        if y < resize_h {
            // global override
            return Some(LRESULT(HTTOP as _));
        }

        let p = PointDIP {
            x: signed_pixels_to_dip(x, self.dpi),
            y: signed_pixels_to_dip(y, self.dpi),
        };
        let size = self.client_size_dip();

        if let Some(ht) = self.root_presenter.header.nc_hittest(&p, &size) {
            return Some(LRESULT(ht as _));
        }

        Some(LRESULT(HTCLIENT as _))
    }

    pub fn on_dpi_changed(&mut self, new_dpi: u16) {
        self.dpi = new_dpi as _;
        for x in self.dpi_handlers.iter() {
            if let Some(x) = x.upgrade() {
                x.on_dpi_changed(new_dpi as _);
            }
        }
    }

    pub fn resize(&mut self, new_width: u16, new_height: u16) {
        self.client_size_pixels.width = new_width as _;
        self.client_size_pixels.height = new_height as _;

        self.root_presenter.grid_view.resize(
            self.client_size_pixels.width,
            self.client_size_pixels.height,
        );
    }

    pub fn on_mouse_move(&mut self, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_move(
            &mut self.ht.borrow_mut(),
            &mut self.app_state.borrow_mut(),
            self.root_presenter.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );

        // WM_SETCURSORが飛ばないことがあるのでここで設定する
        if let Some(c) = self
            .pointer_input_manager
            .cursor(&self.ht.borrow(), &mut self.app_state.borrow_mut())
        {
            unsafe {
                SetCursor(Some(c));
            }
        }
    }

    pub fn on_mouse_left_down(&mut self, hwnd: HWND, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_left_down(
            hwnd,
            &mut self.ht.borrow_mut(),
            &mut self.app_state.borrow_mut(),
            self.root_presenter.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );
    }

    pub fn on_mouse_left_up(&mut self, hwnd: HWND, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_left_up(
            hwnd,
            &mut self.ht.borrow_mut(),
            &mut self.app_state.borrow_mut(),
            self.root_presenter.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );
    }

    pub fn handle_set_cursor(&mut self) -> bool {
        if let Some(c) = self
            .pointer_input_manager
            .cursor(&self.ht.borrow(), &mut self.app_state.borrow_mut())
        {
            unsafe {
                SetCursor(Some(c));
            }

            return true;
        }

        false
    }
}

#[implement(IDropTarget)]
pub struct DropTargetHandler {
    pub bound_hwnd: HWND,
    pub overlay_view: Rc<FileDragAndDropOverlayView>,
    pub dd_helper: IDropTargetHelper,
    pub app_state: std::rc::Weak<RefCell<AppState>>,
}
impl IDropTarget_Impl for DropTargetHandler_Impl {
    fn DragEnter(
        &self,
        pdataobj: windows_core::Ref<'_, windows::Win32::System::Com::IDataObject>,
        _grfkeystate: windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS,
        pt: &windows::Win32::Foundation::POINTL,
        pdweffect: *mut windows::Win32::System::Ole::DROPEFFECT,
    ) -> windows_core::Result<()> {
        unsafe {
            self.dd_helper.DragEnter(
                self.bound_hwnd,
                pdataobj.unwrap(),
                &POINT { x: pt.x, y: pt.y },
                core::ptr::read(pdweffect),
            )?;
        }
        self.overlay_view.show();
        unsafe {
            core::ptr::write(pdweffect, DROPEFFECT_LINK);
        }
        Ok(())
    }

    fn DragLeave(&self) -> windows_core::Result<()> {
        unsafe {
            self.dd_helper.DragLeave()?;
        }
        self.overlay_view.hide();
        Ok(())
    }

    fn DragOver(
        &self,
        _grfkeystate: windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS,
        pt: &windows::Win32::Foundation::POINTL,
        pdweffect: *mut windows::Win32::System::Ole::DROPEFFECT,
    ) -> windows_core::Result<()> {
        unsafe {
            self.dd_helper
                .DragOver(&POINT { x: pt.x, y: pt.y }, core::ptr::read(pdweffect))?;
        }
        unsafe {
            core::ptr::write(pdweffect, DROPEFFECT_LINK);
        }
        Ok(())
    }

    fn Drop(
        &self,
        pdataobj: windows_core::Ref<'_, windows::Win32::System::Com::IDataObject>,
        _grfkeystate: windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS,
        pt: &windows::Win32::Foundation::POINTL,
        pdweffect: *mut windows::Win32::System::Ole::DROPEFFECT,
    ) -> windows_core::Result<()> {
        let data = OwnedStgMedium(unsafe {
            pdataobj
                .unwrap()
                .GetData(&FORMATETC {
                    cfFormat: CF_HDROP.0,
                    ptd: core::ptr::null_mut(),
                    dwAspect: DVASPECT_CONTENT.0,
                    lindex: -1,
                    tymed: TYMED_HGLOBAL.0 as _,
                })
                .unwrap()
        });
        let glock = unsafe { LockedGlobal::acquire(data.hglobal_unchecked()) };
        let hdrop: HDROP = unsafe { core::mem::transmute(glock.ptr) };
        let file_count = unsafe { DragQueryFileW(hdrop, 0xffff_ffff, None) };
        let mut sprites = Vec::with_capacity(file_count as _);
        for n in 0..file_count {
            let len = unsafe { DragQueryFileW(hdrop, n, None) };
            let mut path = Vec::with_capacity((len + 1) as _);
            unsafe {
                path.set_len(path.capacity());
            }
            if unsafe { DragQueryFileW(hdrop, n, Some(&mut path)) } == 0 {
                panic!("DragQueryFileW(querying file path) failed");
            }

            // strip nul-character
            let path = PathBuf::from(OsString::from_wide(&path[..path.len() - 1]));
            if path.is_dir() {
                // process all files in directory(rec)
                for entry in walkdir::WalkDir::new(&path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if !path.is_file() {
                        // 自分自身を含むみたいなのでその場合は見逃す
                        continue;
                    }

                    let mut fs = std::fs::File::open(&path).unwrap();
                    let Some(png_meta) = source_reader::png::Metadata::try_read(&mut fs) else {
                        // PNGじゃないのは一旦見逃す
                        continue;
                    };

                    sprites.push(SpriteInfo::new(
                        path.file_stem().unwrap().to_str().unwrap().into(),
                        path.to_path_buf(),
                        png_meta.width,
                        png_meta.height,
                    ));
                }
            } else {
                let mut fs = std::fs::File::open(&path).unwrap();
                let png_meta = source_reader::png::Metadata::try_read(&mut fs).expect("not a png?");

                sprites.push(SpriteInfo::new(
                    path.file_stem().unwrap().to_str().unwrap().into(),
                    path.to_path_buf(),
                    png_meta.width,
                    png_meta.height,
                ));
            }
        }
        drop(glock);
        drop(data);

        unsafe {
            self.dd_helper.Drop(
                pdataobj.unwrap(),
                &POINT { x: pt.x, y: pt.y },
                core::ptr::read(pdweffect),
            )?;
        }
        self.overlay_view.hide();
        unsafe {
            core::ptr::write(pdweffect, DROPEFFECT_LINK);
        }

        if let Some(m) = self.app_state.upgrade() {
            m.borrow_mut().add_sprites(sprites);
        }

        Ok(())
    }
}

struct OwnedStgMedium(pub STGMEDIUM);
impl Drop for OwnedStgMedium {
    fn drop(&mut self) {
        unsafe {
            ReleaseStgMedium(&mut self.0);
        }
    }
}
impl OwnedStgMedium {
    pub unsafe fn hglobal_unchecked(&self) -> HGLOBAL {
        unsafe { self.0.u.hGlobal }
    }
}

struct LockedGlobal {
    handle: HGLOBAL,
    ptr: *mut core::ffi::c_void,
}
impl Drop for LockedGlobal {
    fn drop(&mut self) {
        if let Err(e) = unsafe { GlobalUnlock(self.handle) } {
            if e.code() != windows::Win32::Foundation::S_OK {
                // Note: なぜかErrなのに中身S_OKが返ってくることがあるらしい
                tracing::warn!({ ?e }, "GlobalUnlock failed");
            }
        }
    }
}
impl LockedGlobal {
    pub unsafe fn acquire(handle: HGLOBAL) -> Self {
        let ptr = unsafe { GlobalLock(handle) };

        Self { handle, ptr }
    }
}

fn main() {
    tracing_subscriber::fmt().pretty().init();
    unsafe {
        OleInitialize(None).unwrap();
    }

    let _ = AppRuntime::init().expect("Failed to initialize app runtime");

    let _dispatcher_queue_controller = unsafe {
        CreateDispatcherQueueController(DispatcherQueueOptions {
            dwSize: core::mem::size_of::<DispatcherQueueOptions>() as _,
            threadType: DQTYPE_THREAD_CURRENT,
            apartmentType: DQTAT_COM_ASTA,
        })
        .expect("Failed to create dispatcher queue")
    };
    let subsystem = Rc::new(Subsystem::new());
    let ui_thread_wakeup_event =
        Arc::new(NativeEvent::new(true, w!("UIThreadWakeupEvent")).unwrap());
    let background_worker = BackgroundWorker::new(&ui_thread_wakeup_event);

    let cls = WNDCLASSEXW {
        cbSize: core::mem::size_of::<WNDCLASSEXW>() as _,
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe {
            core::mem::transmute(GetModuleHandleW(None).expect("Failed to current module handle"))
        },
        hIcon: unsafe { LoadIconW(None, IDI_APPLICATION).expect("Failed to load default icon") },
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).expect("Failed to load default cursor") },
        hbrBackground: HBRUSH(core::ptr::null_mut()),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: w!("AppWindow"),
        hIconSm: unsafe { LoadIconW(None, IDI_APPLICATION).expect("Failed to load default icon") },
    };
    let ca = unsafe { RegisterClassExW(&cls) };
    if ca == 0 {
        panic!(
            "RegisterClassEx failed: {:?}",
            windows_core::Error::from_win32()
        );
    }

    let hw = unsafe {
        CreateWindowExW(
            WS_EX_APPWINDOW | WS_EX_OVERLAPPEDWINDOW | WS_EX_NOREDIRECTIONBITMAP,
            PCWSTR(ca as _),
            w!("Peridot SpriteAtlas Visualizer/Editor"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(cls.hInstance),
            None,
        )
        .expect("Failed to create app window")
    };
    unsafe {
        // set dark mode preference
        DwmSetWindowAttribute(
            hw,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &BOOL(true as _) as *const _ as _,
            core::mem::size_of::<BOOL>() as _,
        )
        .expect("Failed to set dark mode preference");
    }

    let mut bg_worker_vf = vec![None; background_worker.worker_count()];
    let bg_worker_vf_update_callback =
        Rc::new(RefCell::new(Vec::<Box<dyn FnMut(&[Option<String>])>>::new()));
    let app_state = Rc::new(RefCell::new(AppState::new()));

    let mut app_window_state_model = AppWindowStateModel::new(
        &subsystem,
        hw,
        &app_state,
        &background_worker,
        &bg_worker_vf_update_callback,
    );
    unsafe {
        RegisterDragDrop(
            hw,
            &IDropTarget::from(DropTargetHandler {
                bound_hwnd: hw,
                overlay_view: app_window_state_model
                    .root_presenter
                    .file_dnd_overlay
                    .clone(),
                dd_helper: CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER)
                    .unwrap(),
                app_state: Rc::downgrade(&app_state),
            }),
        )
        .unwrap();
    }

    unsafe {
        SetWindowLongPtrW(
            hw,
            GWLP_USERDATA,
            &mut app_window_state_model as *mut _ as _,
        );
    }

    let render_thread_close_event = Arc::new(NativeEvent::new(true, None).unwrap());
    let render_thread = {
        let close_event = render_thread_close_event.clone();
        let grid_view = app_window_state_model.root_presenter.grid_view.clone();

        std::thread::Builder::new()
            .name("RenderThread".into())
            .spawn(move || {
                let grid_view_render_waits = unsafe {
                    NativeEvent::from_raw(grid_view.swapchain.GetFrameLatencyWaitableObject())
                };

                loop {
                    let r = unsafe {
                        WaitForMultipleObjectsEx(
                            &[close_event.handle(), grid_view_render_waits.handle()],
                            false,
                            INFINITE,
                            false,
                        )
                    };

                    if r.0 == WAIT_OBJECT_0.0 {
                        // close
                        break;
                    }

                    if r.0 == WAIT_OBJECT_0.0 + 1 {
                        // render grid
                        grid_view.update_content();
                    }
                }
            })
            .unwrap()
    };

    unsafe {
        let _ = ShowWindow(hw, SW_SHOW);
    }

    let mut msg = core::mem::MaybeUninit::uninit();
    'app: loop {
        let r = unsafe {
            MsgWaitForMultipleObjects(
                Some(&[ui_thread_wakeup_event.handle()]),
                false,
                INFINITE,
                QS_ALLINPUT,
            )
        };

        if r == WAIT_OBJECT_0 {
            // notify to ui thread
            while let Some(vf) = background_worker.try_pop_view_feedback() {
                match vf {
                    BackgroundWorkerViewFeedback::BeginWork(n, msg) => {
                        tracing::info!("Thread #{n} has started a work: {msg}");
                        bg_worker_vf[n] = Some(msg);
                        for x in bg_worker_vf_update_callback.borrow_mut().iter_mut() {
                            x(&bg_worker_vf);
                        }
                    }
                    BackgroundWorkerViewFeedback::EndWork(n) => {
                        tracing::info!("Thread #{n} has finished a work");
                        bg_worker_vf[n] = None;
                        for x in bg_worker_vf_update_callback.borrow_mut().iter_mut() {
                            x(&bg_worker_vf);
                        }
                    }
                }
            }

            ui_thread_wakeup_event.reset();
            continue;
        }
        if r.0 == WAIT_OBJECT_0.0 + 1 {
            while unsafe { PeekMessageW(msg.as_mut_ptr(), None, 0, 0, PM_REMOVE).as_bool() } {
                if unsafe { msg.assume_init_ref().message == WM_QUIT } {
                    break 'app;
                }

                unsafe {
                    let _ = TranslateMessage(msg.as_ptr());
                    DispatchMessageW(msg.as_ptr());
                }
            }

            continue;
        }

        unreachable!();
    }

    render_thread_close_event.signal();
    render_thread.join().unwrap();
    background_worker.teardown();
    unsafe {
        SetWindowLongPtrW(hw, GWLP_USERDATA, 0);
    }
    app_window_state_model.shutdown();
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CREATE {
        // notify frame changed
        let mut rc = core::mem::MaybeUninit::uninit();
        unsafe {
            GetWindowRect(hwnd, rc.as_mut_ptr()).unwrap();
        }
        let rc = unsafe { rc.assume_init() };
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                rc.left,
                rc.top,
                rc.right - rc.left,
                rc.bottom - rc.top,
                SWP_FRAMECHANGED,
            )
            .expect("Failed to reset window frame");
        }

        return LRESULT(0);
    }

    if msg == WM_DESTROY {
        unsafe {
            RevokeDragDrop(hwnd).unwrap();
            PostQuitMessage(0);
        }
        return LRESULT(0);
    }

    if msg == WM_ACTIVATE {
        unsafe {
            DwmExtendFrameIntoClientArea(
                hwnd,
                &MARGINS {
                    cxLeftWidth: 1,
                    cxRightWidth: 1,
                    cyTopHeight: 1,
                    cyBottomHeight: 1,
                },
            )
            .expect("Failed to extend dwm frame");
        }

        return LRESULT(0);
    }

    if msg == WM_NCCALCSIZE {
        if wparam.0 == 1 {
            // remove non-client area

            let params = unsafe {
                core::mem::transmute::<_, *mut NCCALCSIZE_PARAMS>(lparam.0)
                    .as_mut()
                    .unwrap()
            };
            let w = unsafe { GetSystemMetrics(SM_CXSIZEFRAME) };
            let h = unsafe { GetSystemMetrics(SM_CYSIZEFRAME) };
            params.rgrc[0].left += w;
            params.rgrc[0].right -= w;
            params.rgrc[0].bottom -= h;
            // topはいじらない（他アプリもそんな感じになってるのでtopは自前でNCHITTESTしてリサイズ判定する）

            return LRESULT(0);
        }
    }

    if msg == WM_NCHITTEST {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        if let Some(x) = state.nc_hittest(hwnd, wparam, lparam) {
            return x;
        }
    }

    if msg == WM_DPICHANGED {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.on_dpi_changed((wparam.0 & 0xffff) as u16);
    }

    if msg == WM_SIZE {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.resize(
            (lparam.0 & 0xffff) as u16,
            ((lparam.0 >> 16) & 0xffff) as u16,
        );
        return LRESULT(0);
    }

    if msg == WM_MOUSEMOVE {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.on_mouse_move(
            (lparam.0 & 0xffff) as i16,
            ((lparam.0 >> 16) & 0xffff) as i16,
        );
        return LRESULT(0);
    }

    if msg == WM_LBUTTONDOWN {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.on_mouse_left_down(
            hwnd,
            (lparam.0 & 0xffff) as i16,
            ((lparam.0 >> 16) & 0xffff) as i16,
        );
        return LRESULT(0);
    }

    if msg == WM_LBUTTONUP {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.on_mouse_left_up(
            hwnd,
            (lparam.0 & 0xffff) as i16,
            ((lparam.0 >> 16) & 0xffff) as i16,
        );
        return LRESULT(0);
    }

    if msg == WM_SETCURSOR {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        if state.handle_set_cursor() {
            return LRESULT(1);
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
