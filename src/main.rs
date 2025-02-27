use std::{
    cell::{Cell, RefCell},
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    rc::Rc,
};

use component::app_header::AppHeaderView;
use composition_element_builder::{
    CompositionMaskBrushParams, CompositionNineGridBrushParams, CompositionSurfaceBrushParams,
    ContainerVisualParams, SimpleScalarAnimationParams, SpriteVisualParams,
};
use effect_builder::{ColorSourceEffectParams, CompositeEffectParams, GaussianBlurEffectParams};
use extra_bindings::Microsoft::Graphics::Canvas::{
    CanvasComposite,
    Effects::{CompositeEffect, GaussianBlurEffect},
};
use hittest::HitTestTreeActionHandler;
use subsystem::Subsystem;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use windows::{
    core::{h, w, Interface, HRESULT, PCWSTR},
    Foundation::{Size, TimeSpan},
    Graphics::Effects::IGraphicsEffect,
    Win32::{
        Foundation::{HGLOBAL, HWND, LPARAM, LRESULT, POINT, RECT, WAIT_OBJECT_0, WPARAM},
        Graphics::{
            Direct2D::{
                Common::{D2D1_COLOR_F, D2D_POINT_2F, D2D_RECT_F},
                ID2D1DeviceContext, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_ROUNDED_RECT,
            },
            Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
            Direct3D11::{
                ID3D11Buffer, ID3D11PixelShader, ID3D11Texture2D, ID3D11VertexShader,
                D3D11_BIND_CONSTANT_BUFFER, D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE,
                D3D11_MAP_WRITE_DISCARD, D3D11_RENDER_TARGET_VIEW_DESC,
                D3D11_RENDER_TARGET_VIEW_DESC_0, D3D11_RTV_DIMENSION_TEXTURE2D,
                D3D11_SUBRESOURCE_DATA, D3D11_TEX2D_RTV, D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT,
            },
            DirectWrite::{
                IDWriteTextLayout1, DWRITE_FONT_WEIGHT_MEDIUM, DWRITE_FONT_WEIGHT_SEMI_LIGHT,
                DWRITE_TEXT_RANGE,
            },
            Dwm::{
                DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE,
            },
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
                IDXGIAdapter, IDXGIDevice2, IDXGIFactory2, IDXGISwapChain2, DXGI_PRESENT,
                DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
            Gdi::{MapWindowPoints, HBRUSH},
        },
        Storage::Packaging::Appx::PACKAGE_VERSION,
        System::{
            Com::{
                CoCreateInstance, CLSCTX_INPROC_SERVER, DATADIR_GET, DVASPECT_CONTENT, FORMATETC,
                STGMEDIUM, TYMED_HGLOBAL,
            },
            LibraryLoader::GetModuleHandleW,
            Memory::{GlobalLock, GlobalUnlock},
            Ole::{
                IDropTarget, IDropTarget_Impl, OleInitialize, RegisterDragDrop, ReleaseStgMedium,
                RevokeDragDrop, CF_HDROP, DROPEFFECT_COPY, DROPEFFECT_LINK,
            },
            Threading::INFINITE,
            WinRT::{
                Composition::ICompositionDrawingSurfaceInterop, CreateDispatcherQueueController,
                DispatcherQueueOptions, DQTAT_COM_ASTA, DQTYPE_THREAD_CURRENT,
            },
        },
        UI::{
            Controls::MARGINS,
            HiDpi::GetDpiForWindow,
            Shell::{
                CLSID_DragDropHelper, DragAcceptFiles, DragQueryFileW, IDropTargetHelper, HDROP,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetSystemMetrics,
                GetWindowLongPtrW, GetWindowRect, LoadCursorW, LoadIconW,
                MsgWaitForMultipleObjects, PeekMessageW, PostQuitMessage, RegisterClassExW,
                SetCursor, SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage,
                CW_USEDEFAULT, GWLP_USERDATA, HCURSOR, HTCLIENT, HTTOP, IDC_ARROW, IDC_SIZEWE,
                IDI_APPLICATION, NCCALCSIZE_PARAMS, PM_REMOVE, QS_ALLINPUT, SM_CXSIZEFRAME,
                SM_CYSIZEFRAME, SWP_FRAMECHANGED, SW_SHOW, WM_ACTIVATE, WM_CREATE, WM_DESTROY,
                WM_DPICHANGED, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCCALCSIZE,
                WM_NCHITTEST, WM_QUIT, WM_SETCURSOR, WM_SIZE, WNDCLASSEXW, WNDCLASS_STYLES,
                WS_EX_APPWINDOW, WS_EX_NOREDIRECTIONBITMAP, WS_EX_OVERLAPPEDWINDOW,
                WS_OVERLAPPEDWINDOW,
            },
        },
    },
    UI::{
        Color,
        Composition::{
            CompositionBrush, CompositionDrawingSurface, CompositionEasingFunction,
            CompositionEasingFunctionMode, CompositionEffectBrush,
            CompositionEffectSourceParameter, CompositionPropertySet, CompositionStretch,
            ContainerVisual, Desktop::DesktopWindowTarget, ScalarKeyFrameAnimation, SpriteVisual,
            VisualCollection,
        },
    },
};
use windows_core::{implement, BOOL, HSTRING};
use windows_numerics::{Matrix3x2, Vector2, Vector3};

mod component;
mod composition_element_builder;
mod effect_builder;
mod extra_bindings;
mod hittest;
mod input;
mod source_reader;
mod subsystem;

use crate::hittest::*;
use crate::input::*;

#[macro_export]
macro_rules! scoped_try {
    ($label: tt, $x: expr) => {
        match $x {
            Ok(v) => v,
            Err(e) => break $label Err(e)
        }
    }
}

const fn timespan_ms(ms: u32) -> TimeSpan {
    TimeSpan {
        Duration: (10_000 * ms) as _,
    }
}

const fn pixels_to_dip(pixels: u32, dpi: f32) -> f32 {
    pixels as f32 * 96.0 / dpi
}
const fn signed_pixels_to_dip(pixels: i32, dpi: f32) -> f32 {
    pixels as f32 * 96.0 / dpi
}

const fn dip_to_pixels(dip: f32, dpi: f32) -> f32 {
    dip * dpi / 96.0
}

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

const fn rgb_color_from_hex(hex: u32) -> Color {
    Color {
        R: ((hex >> 16) & 0xff) as _,
        G: ((hex >> 8) & 0xff) as _,
        B: (hex & 0xff) as _,
        A: 255,
    }
}

const fn rgb_color_from_websafe_hex(hex: u16) -> Color {
    const fn e(x: u8) -> u8 {
        x | (x << 4)
    }

    Color {
        R: e(((hex >> 8) & 0x0f) as _),
        G: e(((hex >> 4) & 0x0f) as _),
        B: e((hex & 0x0f) as _),
        A: 255,
    }
}

const fn d2d1_color_f_from_rgb_hex(hex: u32) -> D2D1_COLOR_F {
    let ru = ((hex >> 16) & 0xff) as u8;
    let gu = ((hex >> 8) & 0xff) as u8;
    let bu = (hex & 0xff) as u8;

    D2D1_COLOR_F {
        r: ru as f32 / 255.0,
        g: gu as f32 / 255.0,
        b: bu as f32 / 255.0,
        a: 1.0,
    }
}

const BG_COLOR: Color = rgb_color_from_hex(0x202030);

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
    pub ht: &'r Rc<RefCell<HitTestTreeContext>>,
    pub dpi: f32,
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
    pub grid_size: f32,
}

pub struct AtlasBaseGridView {
    root: SpriteVisual,
    swapchain: IDXGISwapChain2,
    vsh: ID3D11VertexShader,
    psh: ID3D11PixelShader,
    render_params_cb: ID3D11Buffer,
    size_pixels: Cell<(u32, u32)>,
    resize_order: Cell<Option<(u32, u32)>>,
}
impl AtlasBaseGridView {
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
                        BufferCount: 3,
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
                    Some(&D3D11_SUBRESOURCE_DATA {
                        pSysMem: (&AtlasBaseGridRenderParams {
                            pixel_size: [init_width_pixels as _, init_height_pixels as _],
                            grid_size: 64.0,
                        }) as *const _ as _,
                        SysMemPitch: 0,
                        SysMemSlicePitch: 0,
                    }),
                    Some(render_params_cb.as_mut_ptr()),
                )
                .unwrap();
        }
        let render_params_cb = unsafe { render_params_cb.assume_init().unwrap() };

        let bb = unsafe { sc.GetBuffer::<ID3D11Texture2D>(0).unwrap() };
        let mut rtv = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateRenderTargetView(
                    &bb,
                    Some(&D3D11_RENDER_TARGET_VIEW_DESC {
                        ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0 {
                            Texture2D: D3D11_TEX2D_RTV { MipSlice: 0 },
                        },
                    }),
                    Some(rtv.as_mut_ptr()),
                )
                .unwrap()
        };
        let rtv = unsafe { rtv.assume_init().unwrap() };
        unsafe {
            init.subsystem
                .d3d11_imm_context
                .OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            init.subsystem
                .d3d11_imm_context
                .RSSetViewports(Some(&[D3D11_VIEWPORT {
                    TopLeftX: 0.0,
                    TopLeftY: 0.0,
                    Width: init_width_pixels as _,
                    Height: init_height_pixels as _,
                    MinDepth: 0.0,
                    MaxDepth: 1.0,
                }]));
            init.subsystem
                .d3d11_imm_context
                .RSSetScissorRects(Some(&[RECT {
                    left: 0,
                    top: 0,
                    right: init_width_pixels as _,
                    bottom: init_height_pixels as _,
                }]));
            init.subsystem.d3d11_imm_context.VSSetShader(&vsh, None);
            init.subsystem.d3d11_imm_context.PSSetShader(&psh, None);
            init.subsystem
                .d3d11_imm_context
                .PSSetConstantBuffers(0, Some(&[Some(render_params_cb.clone())]));
            init.subsystem
                .d3d11_imm_context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            init.subsystem.d3d11_imm_context.Draw(4, 0);
            init.subsystem.d3d11_imm_context.PSSetShader(None, None);
            init.subsystem.d3d11_imm_context.VSSetShader(None, None);
            init.subsystem.d3d11_imm_context.Flush();
        }

        unsafe {
            sc.Present(0, DXGI_PRESENT(0)).ok().unwrap();
        }

        Self {
            root,
            swapchain: sc,
            vsh,
            psh,
            render_params_cb,
            size_pixels: Cell::new((init_width_pixels, init_height_pixels)),
            resize_order: Cell::new(None),
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

        self.resize_order.set(Some((new_width_px, new_height_px)));
    }

    pub fn update_content(&self, subsystem: &Subsystem) {
        if let Some((req_width_px, req_height_px)) = self.resize_order.replace(None) {
            unsafe {
                self.swapchain
                    .ResizeBuffers(
                        3,
                        req_width_px,
                        req_height_px,
                        DXGI_FORMAT_B8G8R8A8_UNORM,
                        DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
                    )
                    .unwrap();
            }

            self.size_pixels.set((req_width_px, req_height_px));
        }

        let (width_px, height_px) = self.size_pixels.get();

        let mut mapped = core::mem::MaybeUninit::uninit();
        unsafe {
            subsystem
                .d3d11_imm_context
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
                    grid_size: 64.0,
                },
            );
        }
        unsafe {
            subsystem.d3d11_imm_context.Unmap(&self.render_params_cb, 0);
        }

        let bb = unsafe { self.swapchain.GetBuffer::<ID3D11Texture2D>(0).unwrap() };
        let mut rtv = core::mem::MaybeUninit::uninit();
        unsafe {
            subsystem
                .d3d11_device
                .CreateRenderTargetView(
                    &bb,
                    Some(&D3D11_RENDER_TARGET_VIEW_DESC {
                        ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0 {
                            Texture2D: D3D11_TEX2D_RTV { MipSlice: 0 },
                        },
                    }),
                    Some(rtv.as_mut_ptr()),
                )
                .unwrap()
        };
        let rtv = unsafe { rtv.assume_init().unwrap() };
        unsafe {
            subsystem
                .d3d11_imm_context
                .OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            subsystem
                .d3d11_imm_context
                .RSSetViewports(Some(&[D3D11_VIEWPORT {
                    TopLeftX: 0.0,
                    TopLeftY: 0.0,
                    Width: width_px as _,
                    Height: height_px as _,
                    MinDepth: 0.0,
                    MaxDepth: 1.0,
                }]));
            subsystem.d3d11_imm_context.RSSetScissorRects(Some(&[RECT {
                left: 0,
                top: 0,
                right: width_px as _,
                bottom: height_px as _,
            }]));
            subsystem.d3d11_imm_context.VSSetShader(&self.vsh, None);
            subsystem.d3d11_imm_context.PSSetShader(&self.psh, None);
            subsystem
                .d3d11_imm_context
                .PSSetConstantBuffers(0, Some(&[Some(self.render_params_cb.clone())]));
            subsystem
                .d3d11_imm_context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            subsystem.d3d11_imm_context.Draw(4, 0);
            subsystem.d3d11_imm_context.PSSetShader(None, None);
            subsystem.d3d11_imm_context.VSSetShader(None, None);
            subsystem.d3d11_imm_context.Flush();
        }

        unsafe {
            self.swapchain.Present(0, DXGI_PRESENT(0)).ok().unwrap();
        }
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

    pub fn new(init: &mut ViewInitContext) -> Self {
        let button_size_px = init.dip_to_pixels(Self::BUTTON_SIZE);
        let icon_size_px = init.dip_to_pixels(Self::ICON_SIZE);

        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: icon_size_px,
                Height: icon_size_px,
            })
            .unwrap();
        {
            let interop: ICompositionDrawingSurfaceInterop = icon_surface.cast().unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                    dc.SetTransform(&Matrix3x2::translation(
                        init.signed_pixels_to_dip(offset.x),
                        init.signed_pixels_to_dip(offset.y),
                    ));
                }

                let brush = scoped_try!('drawing, unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.9, g: 0.9, b: 0.9, a: 1.0 }, None) });

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

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

        let frame_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: button_size_px,
                Height: button_size_px,
            })
            .unwrap();
        {
            let interop: ICompositionDrawingSurfaceInterop = frame_surface.cast().unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                    dc.SetTransform(&Matrix3x2::translation(
                        init.signed_pixels_to_dip(offset.x),
                        init.signed_pixels_to_dip(offset.y),
                    ));
                }

                let brush = scoped_try!('drawing, unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, None) });

                unsafe {
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
                        &brush,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

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

        let implicit_easing_animation = init
            .subsystem
            .compositor
            .CreateScalarKeyFrameAnimation()
            .unwrap();
        implicit_easing_animation
            .InsertExpressionKeyFrame(0.0, h!("this.StartingValue"))
            .unwrap();
        implicit_easing_animation
            .InsertExpressionKeyFrameWithEasingFunction(
                1.0,
                h!("this.FinalValue"),
                &init
                    .subsystem
                    .compositor
                    .CreateCubicBezierEasingFunction(
                        Vector2 { X: 0.5, Y: 0.0 },
                        Vector2 { X: 0.5, Y: 1.0 },
                    )
                    .unwrap(),
            )
            .unwrap();
        implicit_easing_animation.SetTarget(h!("Opacity")).unwrap();
        implicit_easing_animation
            .SetDuration(timespan_ms(100))
            .unwrap();
        let implicit_move_animation = init
            .subsystem
            .compositor
            .CreateVector3KeyFrameAnimation()
            .unwrap();
        implicit_move_animation
            .InsertExpressionKeyFrame(0.0, h!("this.StartingValue"))
            .unwrap();
        implicit_move_animation
            .InsertExpressionKeyFrameWithEasingFunction(
                1.0,
                h!("this.FinalValue"),
                &init
                    .subsystem
                    .compositor
                    .CreateCubicBezierEasingFunction(
                        Vector2 { X: 0.5, Y: 0.0 },
                        Vector2 { X: 0.5, Y: 1.0 },
                    )
                    .unwrap(),
            )
            .unwrap();
        implicit_move_animation.SetTarget(h!("Offset")).unwrap();
        implicit_move_animation
            .SetDuration(SpriteListPaneView::TRANSITION_DURATION)
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
        ht: &mut HitTestTreeContext,
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

    pub fn transit_hidden(&self, ht: &mut HitTestTreeContext) {
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

    pub fn transit_shown(&self, ht: &mut HitTestTreeContext) {
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
    label: SpriteVisual,
    top: Cell<f32>,
    dpi: Cell<f32>,
}
impl SpriteListCellView {
    const FRAME_TEX_SIZE: f32 = 24.0;
    const CORNER_RADIUS: f32 = 8.0;
    const CELL_HEIGHT: f32 = 20.0;

    fn gen_frame_tex(subsystem: &Subsystem, dpi: f32) -> CompositionDrawingSurface {
        let s = subsystem
            .new_2d_drawing_surface(Size {
                Width: dip_to_pixels(Self::FRAME_TEX_SIZE, dpi),
                Height: dip_to_pixels(Self::FRAME_TEX_SIZE, dpi),
            })
            .unwrap();
        let interop: ICompositionDrawingSurfaceInterop = s.cast().unwrap();
        let mut offs = core::mem::MaybeUninit::uninit();
        let dc: ID2D1DeviceContext = unsafe { interop.BeginDraw(None, offs.as_mut_ptr()).unwrap() };
        let offs = unsafe { offs.assume_init() };
        let r = 'drawing: {
            unsafe {
                dc.SetDpi(dpi, dpi);
            }

            let brush = scoped_try!(
                'drawing,
                unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.875, g: 0.875, b: 0.875, a: 0.25 }, None) }
            );

            let offs_dip = D2D_POINT_2F {
                x: signed_pixels_to_dip(offs.x, dpi),
                y: signed_pixels_to_dip(offs.y, dpi),
            };

            unsafe {
                dc.Clear(None);
                dc.FillRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: offs_dip.x,
                            top: offs_dip.y,
                            right: offs_dip.x + Self::FRAME_TEX_SIZE,
                            bottom: offs_dip.y + Self::FRAME_TEX_SIZE,
                        },
                        radiusX: Self::CORNER_RADIUS,
                        radiusY: Self::CORNER_RADIUS,
                    },
                    &brush,
                );
            }

            Ok(())
        };
        unsafe {
            interop.EndDraw().unwrap();
        }
        r.unwrap();

        s
    }

    pub fn new(init: &mut ViewInitContext, label: &str, init_top: f32) -> Self {
        let frame_tex = Self::gen_frame_tex(init.subsystem, init.dpi);

        let tl = unsafe {
            init.subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &label.encode_utf16().collect::<Vec<_>>(),
                    &init.subsystem.default_ui_format,
                    f32::MAX,
                    f32::MAX,
                )
                .unwrap()
        };
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
        {
            let interop = label_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let brush = scoped_try!(
                    'drawing,
                    unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }, None) }
                );

                unsafe {
                    dc.Clear(None);
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: init.signed_pixels_to_dip(offset.x),
                            y: init.signed_pixels_to_dip(offset.y),
                        },
                        &tl,
                        &brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

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
            &CompositionNineGridBrushParams::new(
                &CompositionSurfaceBrushParams::new(&frame_tex)
                    .stretch(CompositionStretch::Fill)
                    .instantiate(&init.subsystem.compositor)
                    .unwrap(),
            )
            .insets(init.dip_to_pixels(Self::CORNER_RADIUS))
            .instantiate(&init.subsystem.compositor)
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
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&label).unwrap();

        let opacity_transition = init
            .subsystem
            .compositor
            .CreateScalarKeyFrameAnimation()
            .unwrap();
        opacity_transition
            .InsertExpressionKeyFrame(0.0, h!("this.StartingValue"))
            .unwrap();
        opacity_transition
            .InsertExpressionKeyFrameWithEasingFunction(
                1.0,
                h!("this.FinalValue"),
                &init
                    .subsystem
                    .compositor
                    .CreateCubicBezierEasingFunction(
                        Vector2 { X: 0.5, Y: 0.0 },
                        Vector2 { X: 0.5, Y: 1.0 },
                    )
                    .unwrap(),
            )
            .unwrap();
        opacity_transition.SetDuration(timespan_ms(150)).unwrap();
        opacity_transition.SetTarget(h!("Opacity")).unwrap();

        let bg_implicit_animations = init
            .subsystem
            .compositor
            .CreateImplicitAnimationCollection()
            .unwrap();
        bg_implicit_animations
            .Insert(h!("Opacity"), &opacity_transition)
            .unwrap();
        bg.SetImplicitAnimations(&bg_implicit_animations).unwrap();

        Self {
            root,
            bg,
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
        self.bg.SetOpacity(0.5).unwrap();
    }

    pub fn on_leave(&self) {
        self.bg.SetOpacity(0.0).unwrap();
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

        let tl = unsafe {
            subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &name.encode_utf16().collect::<Vec<_>>(),
                    &subsystem.default_ui_format,
                    f32::MAX,
                    f32::MAX,
                )
                .unwrap()
        };
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
        {
            let interop = label_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(dpi, dpi);
                }

                let brush = scoped_try!(
                    'drawing,
                    unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }, None) }
                );

                unsafe {
                    dc.Clear(None);
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: signed_pixels_to_dip(offset.x, dpi),
                            y: signed_pixels_to_dip(offset.y, dpi),
                        },
                        &tl,
                        &brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

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
    const SURFACE_COLOR: windows::UI::Color = windows::UI::Color {
        R: 255,
        G: 255,
        B: 255,
        A: 48,
    };
    const SPACING: f32 = 8.0;
    const ADJUST_AREA_THICKNESS: f32 = 4.0;
    const INIT_WIDTH: f32 = 280.0;
    const HEADER_LABEL_MAIN_COLOR: D2D1_COLOR_F = D2D1_COLOR_F {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    const HEADER_LABEL_SHADOW_COLOR: D2D1_COLOR_F = D2D1_COLOR_F {
        r: 0.1,
        g: 0.1,
        b: 0.1,
        a: 1.0,
    };
    const TRANSITION_DURATION: TimeSpan = timespan_ms(250);
    const CELL_AREA_PADDINGS: RectDIP = RectDIP {
        left: 16.0,
        top: 32.0,
        right: 16.0,
        bottom: 16.0,
    };

    fn gen_frame_tex(subsystem: &Subsystem, dpi: f32) -> CompositionDrawingSurface {
        let s = subsystem
            .new_2d_drawing_surface(Size {
                Width: dip_to_pixels(Self::FRAME_TEX_SIZE, dpi),
                Height: dip_to_pixels(Self::FRAME_TEX_SIZE, dpi),
            })
            .unwrap();
        let interop: ICompositionDrawingSurfaceInterop = s.cast().unwrap();
        let mut offs = core::mem::MaybeUninit::uninit();
        let dc: ID2D1DeviceContext = unsafe { interop.BeginDraw(None, offs.as_mut_ptr()).unwrap() };
        let offs = unsafe { offs.assume_init() };
        let r = 'drawing: {
            unsafe {
                dc.SetDpi(dpi, dpi);
            }

            let brush = scoped_try!('drawing, unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None) });

            let offs_dip = D2D_POINT_2F {
                x: signed_pixels_to_dip(offs.x, dpi),
                y: signed_pixels_to_dip(offs.y, dpi),
            };

            unsafe {
                dc.Clear(None);
                dc.FillRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: offs_dip.x,
                            top: offs_dip.y,
                            right: offs_dip.x + Self::FRAME_TEX_SIZE,
                            bottom: offs_dip.y + Self::FRAME_TEX_SIZE,
                        },
                        radiusX: Self::CORNER_RADIUS,
                        radiusY: Self::CORNER_RADIUS,
                    },
                    &brush,
                );
            }

            Ok(())
        };
        unsafe {
            interop.EndDraw().unwrap();
        }
        r.unwrap();

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
            &CompositeEffectParams {
                sources: &[
                    GaussianBlurEffectParams {
                        source: &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                        blur_amount: Some(Self::BLUR_AMOUNT / 3.0),
                        name: None,
                    }
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
                ],
                mode: None,
            }
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

        let tl = unsafe {
            init.subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &"Sprites".encode_utf16().collect::<Vec<_>>(),
                    &init.subsystem.default_ui_format,
                    f32::MAX,
                    f32::MAX,
                )
                .unwrap()
        };
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
        {
            let interop = header_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let brush = scoped_try!(
                    'drawing,
                    unsafe { dc.CreateSolidColorBrush(&Self::HEADER_LABEL_MAIN_COLOR, None) }
                );

                unsafe {
                    dc.Clear(None);
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: init.signed_pixels_to_dip(offset.x),
                            y: init.signed_pixels_to_dip(offset.y),
                        },
                        &tl,
                        &brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }
        let header_surface_w = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(tm.width + 18.0),
                Height: init.dip_to_pixels(tm.height + 18.0),
            })
            .unwrap();
        {
            let interop = header_surface_w
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let brush = scoped_try!(
                    'drawing,
                    unsafe { dc.CreateSolidColorBrush(&Self::HEADER_LABEL_SHADOW_COLOR, None) }
                );

                unsafe {
                    dc.Clear(None);
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: init.signed_pixels_to_dip(offset.x) + 9.0,
                            y: init.signed_pixels_to_dip(offset.y) + 9.0,
                        },
                        &tl,
                        &brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

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
                &GaussianBlurEffectParams {
                    source: &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                    blur_amount: Some(18.0 / 3.0),
                    name: None,
                }
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
            .Properties()
            .unwrap()
            .InsertScalar(h!("ShownRate"), 1.0)
            .unwrap();
        composition_properties
            .Properties()
            .unwrap()
            .InsertScalar(h!("DPI"), init.dpi)
            .unwrap();
        composition_properties
            .Properties()
            .unwrap()
            .InsertScalar(h!("TopOffset"), 0.0)
            .unwrap();

        let offset_expr = format!("Vector3(-this.Target.Size.X - ({spc} * compositionProperties.DPI / 96.0) + (this.Target.Size.X + ({spc} * 2.0 * compositionProperties.DPI / 96.0)) * compositionProperties.ShownRate, compositionProperties.TopOffset * compositionProperties.DPI / 96.0, 0.0)", spc = Self::SPACING);
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
        ht: &mut HitTestTreeContext,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
        ht.remove_child(self.ht_root);
    }

    pub fn shutdown(&self, ht: &mut HitTestTreeContext) {
        ht.free_rec(self.ht_root);
    }

    pub fn set_top(&self, ht: &mut HitTestTreeContext, top: f32) {
        self.composition_properties
            .Properties()
            .unwrap()
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

    pub fn set_width(&self, ht: &mut HitTestTreeContext, width: f32) {
        self.root
            .SetSize(Vector2 {
                X: dip_to_pixels(width, self.dpi),
                Y: dip_to_pixels(-self.top.get() - Self::SPACING, self.dpi),
            })
            .unwrap();
        ht.get_mut(self.ht_root).width = width;

        self.width.set(width);
    }

    pub fn transit_hidden(&self, ht: &mut HitTestTreeContext) {
        self.composition_properties
            .StartAnimation(h!("ShownRate"), &self.hide_animation)
            .unwrap();
        ht.get_mut(self.ht_root).left = -self.width.get();
    }

    pub fn transit_shown(&self, ht: &mut HitTestTreeContext) {
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
    fn cursor(&self, sender: HitTestTreeRef) -> Option<HCURSOR> {
        if sender == self.view.ht_adjust_area && !self.hidden.get() {
            // TODO: 必要そうならキャッシュする
            return Some(unsafe { LoadCursorW(None, IDC_SIZEWE).unwrap() });
        }

        None
    }

    fn on_pointer_enter(
        &self,
        sender: HitTestTreeRef,
        ht: &mut HitTestTreeContext,
        client_x: f32,
        client_y: f32,
        client_width: f32,
        client_height: f32,
    ) -> EventContinueControl {
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
        _ht: &mut HitTestTreeContext,
        _client_x: f32,
        _client_y: f32,
        _client_width: f32,
        _client_height: f32,
    ) -> EventContinueControl {
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
        _ht: &mut HitTestTreeContext,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
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
        ht: &mut HitTestTreeContext,
        client_x: f32,
        client_y: f32,
        client_width: f32,
        client_height: f32,
    ) -> EventContinueControl {
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
        ht: &mut HitTestTreeContext,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
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
        ht: &mut HitTestTreeContext,
    ) -> EventContinueControl {
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

        EventContinueControl::empty()
    }
}

// TODO: CellViewのプールがすでにArenaになっているので、あとでそれ前提で組み直したい

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

        init.app_state
            .borrow_mut()
            .sprites_update_callbacks
            .push(Box::new({
                let subsystem = Rc::downgrade(init.for_view.subsystem);
                let ht = Rc::downgrade(init.for_view.ht);
                let view = Rc::downgrade(&view);
                let sprite_list_cells = Rc::downgrade(&sprite_list_cells);

                move |sprites| {
                    let Some(subsystem) = subsystem.upgrade() else {
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

                    tracing::info!("sprites updated: {sprites:?}");
                    sprite_list_contents.clear();
                    sprite_list_contents.extend(sprites.iter().map(|x| x.name.clone()));
                    let visible_contents = &sprite_list_contents[..];
                    for (n, c) in visible_contents.iter().enumerate() {
                        if sprite_list_cells.borrow().len() == n {
                            // create new one
                            let new_cell = SpriteListCellView::new(
                                &mut ViewInitContext {
                                    subsystem: &subsystem,
                                    ht: &ht,
                                    dpi: view.dpi,
                                },
                                &c,
                                SpriteListPaneView::CELL_AREA_PADDINGS.top
                                    + n as f32 * SpriteListCellView::CELL_HEIGHT,
                            );
                            new_cell.mount(&view.root.Children().unwrap());
                            sprite_list_cells.borrow_mut().push(new_cell);
                            continue;
                        }

                        sprite_list_cells.borrow()[n].set_name(&c, &subsystem);
                        sprite_list_cells.borrow()[n].set_top(
                            SpriteListPaneView::CELL_AREA_PADDINGS.top
                                + n as f32 * SpriteListCellView::CELL_HEIGHT,
                        );
                    }
                }
            }));

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
        ht: &mut HitTestTreeContext,
        ht_parent: HitTestTreeRef,
    ) {
        self.view.mount(children, ht, ht_parent);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.view.unmount(ht);
    }

    pub fn shutdown(&self, ht: &mut HitTestTreeContext) {
        self.view.shutdown(ht);
    }

    pub fn set_top(&self, ht: &mut HitTestTreeContext, top: f32) {
        self.view.set_top(ht, top);
    }
}

pub struct FileDragAndDropOverlayView {
    root: SpriteVisual,
    composition_params: CompositionPropertySet,
    show_animation: ScalarKeyFrameAnimation,
    hide_animation: ScalarKeyFrameAnimation,
}
impl FileDragAndDropOverlayView {
    pub fn new(init: &mut ViewInitContext) -> Self {
        let composite_effect = CompositeEffectParams {
            sources: &[
                GaussianBlurEffectParams {
                    source: &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                    blur_amount: None,
                    name: Some(h!("Blur")),
                }
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
                ColorSourceEffectParams {
                    color: Some(windows::UI::Color {
                        R: 255,
                        G: 255,
                        B: 255,
                        A: 64,
                    }),
                }
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
            ],
            mode: Some(CanvasComposite::SourceOver),
        }
        .instantiate()
        .unwrap();
        let effect_factory = init
            .subsystem
            .compositor
            .CreateEffectFactoryWithProperties(
                &composite_effect,
                &windows_collections::IIterable::<HSTRING>::from(vec![
                    h!("Blur.BlurAmount").clone()
                ]),
            )
            .unwrap();

        let bg_brush = effect_factory.CreateBrush().unwrap();
        bg_brush
            .SetSourceParameter(
                h!("source"),
                &init.subsystem.compositor.CreateBackdropBrush().unwrap(),
            )
            .unwrap();
        let root = SpriteVisualParams::new(&bg_brush)
            .expand()
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let label_text_utf16 = "ドロップしてファイルを追加"
            .encode_utf16()
            .collect::<Vec<_>>();
        let label = unsafe {
            init.subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &label_text_utf16,
                    &init.subsystem.default_ui_format,
                    f32::MAX,
                    f32::MAX,
                )
                .unwrap()
        };
        unsafe {
            label
                .SetFontSize(
                    96.0,
                    DWRITE_TEXT_RANGE {
                        startPosition: 0,
                        length: label_text_utf16.len() as _,
                    },
                )
                .unwrap();
            label
                .SetFontWeight(
                    DWRITE_FONT_WEIGHT_SEMI_LIGHT,
                    DWRITE_TEXT_RANGE {
                        startPosition: 0,
                        length: label_text_utf16.len() as _,
                    },
                )
                .unwrap();
        }
        let mut metrics = core::mem::MaybeUninit::uninit();
        unsafe {
            label.GetMetrics(metrics.as_mut_ptr()).unwrap();
        }
        let metrics = unsafe { metrics.assume_init() };
        let label_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(metrics.width),
                Height: init.dip_to_pixels(metrics.height),
            })
            .unwrap();
        {
            let interop: ICompositionDrawingSurfaceInterop = label_surface.cast().unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let brush = scoped_try!('drawing, unsafe { dc.CreateSolidColorBrush(&d2d1_color_f_from_rgb_hex(0xcccccc), None) });

                unsafe {
                    dc.Clear(None);
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: init.signed_pixels_to_dip(offset.x),
                            y: init.signed_pixels_to_dip(offset.y),
                        },
                        &label,
                        &brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }
        let label = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&label_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size(Vector2 {
            X: init.dip_to_pixels(metrics.width),
            Y: init.dip_to_pixels(metrics.height),
        })
        .anchor_point(Vector2 { X: 0.5, Y: 0.5 })
        .relative_offset_adjustment_xy(Vector2 { X: 0.5, Y: 0.5 })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        root.Children().unwrap().InsertAtTop(&label).unwrap();

        let composition_params = init.subsystem.compositor.CreatePropertySet().unwrap();
        composition_params.InsertScalar(h!("Rate"), 0.0).unwrap();

        let bg_blur_expression = init
            .subsystem
            .compositor
            .CreateExpressionAnimationWithExpression(h!("cp.Rate * 63.0 / 3.0"))
            .unwrap();
        bg_blur_expression
            .SetExpressionReferenceParameter(h!("cp"), &composition_params)
            .unwrap();
        bg_brush
            .Properties()
            .unwrap()
            .InsertScalar(h!("Blur.BlurAmount"), 0.0)
            .unwrap();
        bg_brush
            .Properties()
            .unwrap()
            .StartAnimation(h!("Blur.BlurAmount"), &bg_blur_expression)
            .unwrap();

        let opacity_expression = init
            .subsystem
            .compositor
            .CreateExpressionAnimationWithExpression(h!("cp.Rate"))
            .unwrap();
        opacity_expression
            .SetExpressionReferenceParameter(h!("cp"), &composition_params)
            .unwrap();
        root.SetOpacity(0.0).unwrap();
        root.StartAnimation(h!("Opacity"), &opacity_expression)
            .unwrap();

        let easing = init
            .subsystem
            .compositor
            .CreateCubicBezierEasingFunction(Vector2 { X: 0.5, Y: 0.0 }, Vector2 { X: 0.5, Y: 1.0 })
            .unwrap();
        let show_animation = SimpleScalarAnimationParams::new(0.0, 1.0, &easing)
            .duration(timespan_ms(200))
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let hide_animation = SimpleScalarAnimationParams::new(1.0, 0.0, &easing)
            .duration(timespan_ms(200))
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        Self {
            root,
            composition_params,
            show_animation,
            hide_animation,
        }
    }

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn show(&self) {
        self.composition_params
            .StartAnimation(h!("Rate"), &self.show_animation)
            .unwrap();
    }

    pub fn hide(&self) {
        self.composition_params
            .StartAnimation(h!("Rate"), &self.hide_animation)
            .unwrap();
    }
}

struct AppWindowStateModel {
    ht: Rc<RefCell<HitTestTreeContext>>,
    ht_root: HitTestTreeRef,
    client_size_pixels: SizePixels,
    dpi: f32,
    dpi_handlers: Vec<std::rc::Weak<dyn DpiHandler>>,
    pointer_input_manager: PointerInputManager,
    composition_target: DesktopWindowTarget,
    composition_root: ContainerVisual,
    header_view: AppHeaderView,
    grid_view: AtlasBaseGridView,
    sprite_list_pane: SpriteListPanePresenter,
    file_dnd_overlay: Rc<FileDragAndDropOverlayView>,
}
impl AppWindowStateModel {
    pub fn new(
        subsystem: &Rc<Subsystem>,
        bound_hwnd: HWND,
        app_state: &Rc<RefCell<AppState>>,
    ) -> Self {
        let mut ht = HitTestTreeContext::new();
        let ht_root = ht.alloc(HitTestTreeData {
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
        println!("init dpi: {dpi}");
        let mut dpi_handlers = Vec::new();
        let pointer_input_manager = PointerInputManager::new();

        let composition_target = unsafe {
            subsystem
                .compositor_desktop_interop
                .CreateDesktopWindowTarget(bound_hwnd, true)
                .unwrap()
        };

        let composition_root = subsystem.compositor.CreateContainerVisual().unwrap();
        composition_root
            .SetRelativeSizeAdjustment(Vector2::one())
            .unwrap();
        composition_target.SetRoot(&composition_root).unwrap();

        let bg = subsystem
            .compositor
            .CreateSpriteVisual()
            .expect("Failed to create bg visual");
        bg.SetBrush(
            &subsystem
                .compositor
                .CreateColorBrushWithColor(BG_COLOR)
                .expect("Failed to create bg brush"),
        )
        .expect("Failed to set bg brush");
        bg.SetRelativeSizeAdjustment(Vector2::one())
            .expect("Failed to set bg size");
        composition_root
            .Children()
            .expect("Failed to get children")
            .InsertAtBottom(&bg)
            .expect("Failed to insert bg");

        let ht = Rc::new(RefCell::new(ht));
        let mut presenter_init_context = PresenterInitContext {
            for_view: ViewInitContext {
                subsystem,
                ht: &ht,
                dpi,
            },
            dpi_handlers: &mut dpi_handlers,
            app_state,
        };

        let grid_view = AtlasBaseGridView::new(&mut presenter_init_context.for_view, 128, 128);
        grid_view.mount(&composition_root.Children().unwrap());
        grid_view.resize(client_size_pixels.width, client_size_pixels.height);

        let sprite_list_pane = SpriteListPanePresenter::new(&mut presenter_init_context);
        sprite_list_pane.mount(
            &composition_root.Children().unwrap(),
            &mut presenter_init_context.for_view.ht.borrow_mut(),
            ht_root,
        );

        let header_view = AppHeaderView::new(
            &mut presenter_init_context.for_view,
            "Peridot SpriteAtlas Visualizer/Editor",
        );
        header_view.mount(&composition_root.Children().unwrap());

        let file_dnd_overlay = Rc::new(FileDragAndDropOverlayView::new(
            &mut presenter_init_context.for_view,
        ));
        file_dnd_overlay.mount(&composition_root.Children().unwrap());

        sprite_list_pane.set_top(&mut ht.borrow_mut(), header_view.height());

        ht.borrow().dump(ht_root);

        Self {
            ht,
            ht_root,
            client_size_pixels,
            dpi,
            dpi_handlers,
            pointer_input_manager,
            composition_target,
            composition_root,
            header_view,
            grid_view,
            sprite_list_pane,
            file_dnd_overlay,
        }
    }

    pub fn shutdown(&mut self) {
        self.sprite_list_pane.shutdown(&mut self.ht.borrow_mut());
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

        if let Some(ht) = self.header_view.nc_hittest(&p, &size) {
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

        self.grid_view.resize(
            self.client_size_pixels.width,
            self.client_size_pixels.height,
        );
    }

    pub fn on_mouse_move(&mut self, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_move(
            &mut self.ht.borrow_mut(),
            self.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );

        // WM_SETCURSORが飛ばないことがあるのでここで設定する
        if let Some(c) = self.pointer_input_manager.cursor(&self.ht.borrow()) {
            unsafe {
                SetCursor(Some(c));
            }
        }
    }

    pub fn on_mouse_left_down(&mut self, hwnd: HWND, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_left_down(
            hwnd,
            &mut self.ht.borrow_mut(),
            self.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );
    }

    pub fn on_mouse_left_up(&mut self, hwnd: HWND, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_left_up(
            hwnd,
            &mut self.ht.borrow_mut(),
            self.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );
    }

    pub fn handle_set_cursor(&mut self) -> bool {
        if let Some(c) = self.pointer_input_manager.cursor(&self.ht.borrow()) {
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

                    sprites.push(SpriteInfo {
                        name: path.file_stem().unwrap().to_str().unwrap().into(),
                        source_path: path.to_path_buf(),
                        width: png_meta.width as _,
                        height: png_meta.height as _,
                        left: 0,
                        top: 0,
                        left_slice: 0,
                        right_slice: 0,
                        top_slice: 0,
                        bottom_slice: 0,
                    });
                }
            } else {
                let mut fs = std::fs::File::open(&path).unwrap();
                let png_meta = source_reader::png::Metadata::try_read(&mut fs).expect("not a png?");

                // strip nul-character
                sprites.push(SpriteInfo {
                    name: path.file_stem().unwrap().to_str().unwrap().into(),
                    source_path: path,
                    width: png_meta.width as _,
                    height: png_meta.height as _,
                    left: 0,
                    top: 0,
                    left_slice: 0,
                    right_slice: 0,
                    top_slice: 0,
                    bottom_slice: 0,
                });
            }
        }
        drop(glock);
        drop(data);

        if let Some(m) = self.app_state.upgrade() {
            m.borrow_mut().add_sprites(sprites);
        }

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
}

pub struct AppState {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub sprites: Vec<SpriteInfo>,
    pub sprites_update_callbacks: Vec<Box<dyn FnMut(&[SpriteInfo])>>,
}
impl AppState {
    pub fn add_sprites(&mut self, sprites: impl IntoIterator<Item = SpriteInfo>) {
        self.sprites.extend(sprites);
        for x in self.sprites_update_callbacks.iter_mut() {
            x(&self.sprites);
        }
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

    let app_state = Rc::new(RefCell::new(AppState {
        atlas_width: 32,
        atlas_height: 32,
        sprites: Vec::new(),
        sprites_update_callbacks: Vec::new(),
    }));

    let mut app_window_state_model = AppWindowStateModel::new(&subsystem, hw, &app_state);
    let dd_helper: IDropTargetHelper =
        unsafe { CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER).unwrap() };
    unsafe {
        RegisterDragDrop(
            hw,
            &IDropTarget::from(DropTargetHandler {
                bound_hwnd: hw,
                overlay_view: app_window_state_model.file_dnd_overlay.clone(),
                dd_helper,
                app_state: Rc::downgrade(&app_state),
            }),
        )
        .unwrap();
    }

    let grid_view_render_waits = unsafe {
        app_window_state_model
            .grid_view
            .swapchain
            .GetFrameLatencyWaitableObject()
    };

    unsafe {
        SetWindowLongPtrW(
            hw,
            GWLP_USERDATA,
            &mut app_window_state_model as *mut _ as _,
        );
    }

    unsafe {
        let _ = ShowWindow(hw, SW_SHOW);
    }

    let mut msg = core::mem::MaybeUninit::uninit();
    'app: loop {
        let r = unsafe {
            MsgWaitForMultipleObjects(
                Some(&[grid_view_render_waits]),
                false,
                INFINITE,
                QS_ALLINPUT,
            )
        };

        if r == WAIT_OBJECT_0 {
            // update grid view
            app_window_state_model.grid_view.update_content(&subsystem);
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
