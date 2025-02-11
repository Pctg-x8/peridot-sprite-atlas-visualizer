use std::os::windows::ffi::OsStringExt;

use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
        System::Registry::{
            RegCloseKey, RegGetValueW, RegOpenKeyExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
            REG_ROUTINE_FLAGS, REG_SAM_FLAGS, RRF_RT_REG_SZ,
        },
    },
};

fn main() {
    let project_root = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("no manifest dir set?"),
    );
    let build_profile = std::env::var_os("PROFILE").expect("no profile set?");
    let target_exe_dir = project_root.join("target").join(&build_profile);
    let out_dir = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("no out_dir set?"));
    let appsdk_root = project_root.join(".nuget/Microsoft.WindowsAppSDK.1.6.250108002");
    let win2d_root = project_root.join(".nuget/Microsoft.Graphics.Win2D.1.3.2");

    // build rc
    let (win10_sdk_installation_folder, win10_sdk_product_version) = find_win10_sdk();
    let rc_exe =
        find_win10_sdk_bin_folder(&win10_sdk_installation_folder, &win10_sdk_product_version)
            .join("rc.exe");
    let include_um =
        find_win10_sdk_include_folder(&win10_sdk_installation_folder, &win10_sdk_product_version)
            .join("um");
    let include_shared =
        find_win10_sdk_include_folder(&win10_sdk_installation_folder, &win10_sdk_product_version)
            .join("shared");
    std::process::Command::new(&rc_exe)
        .arg("/I")
        .arg(include_um)
        .arg("/I")
        .arg(include_shared)
        .args(["/r", "/fo"])
        .arg(out_dir.join("exe.res"))
        .arg(project_root.join("exe.rc"))
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    println!(
        "cargo:rustc-link-search={}",
        appsdk_root.join("lib/win10-x64").display()
    );
    println!("cargo:rustc-link-search={}", out_dir.display());
    // +verbatimで拡張子そのままにLinkerに渡せるらしい
    // https://github.com/rust-lang/rust/issues/81488
    println!("cargo:rustc-link-lib=dylib:+verbatim=exe.res");

    // extra codegen for winrt apis
    let out_path = project_root.join("src/extra_bindings.rs");
    let microsoft_graphics_canvas_winmd_path = project_root
        .join(".nuget/Microsoft.Graphics.Win2D.1.3.2/lib/uap10.0/Microsoft.Graphics.Canvas.winmd");
    windows_bindgen::bindgen([
        "--out",
        out_path.to_str().unwrap(),
        "--reference",
        "windows,skip-root,Windows.Graphics.Effects.IGraphicsEffect",
        "--reference",
        "windows,skip-root,Windows.UI.Color",
        "--reference",
        "windows,skip-root,Windows.Foundation.Collections.IVector",
        "--in",
        "default",
        "--in",
        microsoft_graphics_canvas_winmd_path.to_str().unwrap(),
        "--filter",
        "Microsoft.Graphics.Canvas.Effects.GaussianBlurEffect",
        "--filter",
        "Microsoft.Graphics.Canvas.Effects.ColorSourceEffect",
        "--filter",
        "Microsoft.Graphics.Canvas.Effects.CompositeEffect",
        "--filter",
        "Microsoft.Graphics.Canvas.CanvasComposite",
    ]);

    // ICanvasImage::GetBoundsの定義が何故か二重に出るのでパッチ当てる
    let generated = std::fs::read_to_string(&out_path).unwrap();
    let generated = generated.replace(
        r#"GetBounds: 0,
                        GetBounds: 0,"#,
        "GetBounds: 0,",
    );
    let generated = generated.replace(
        r#"GetBounds: usize,
                GetBounds: usize,"#,
        "GetBounds: usize,",
    );
    std::fs::write(&out_path, &generated).unwrap();

    std::fs::copy(
        appsdk_root.join("runtimes/win-x64/native/Microsoft.WindowsAppRuntime.Bootstrap.dll"),
        target_exe_dir.join("Microsoft.WindowsAppRuntime.Bootstrap.dll"),
    )
    .expect("Failed to copy bootstrap dll");
    std::fs::copy(
        win2d_root.join("runtimes/win-x64/native/Microsoft.Graphics.Canvas.dll"),
        target_exe_dir.join("Microsoft.Graphics.Canvas.dll"),
    )
    .expect("Failed to copy bootstrap dll");
}

struct RegistryKey(HKEY);
impl Drop for RegistryKey {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0).ok().unwrap();
        }
    }
}
impl RegistryKey {
    #[inline]
    pub fn open(
        root: HKEY,
        subkey: impl windows::core::Param<PCWSTR>,
        options: Option<u32>,
        sam_desired: REG_SAM_FLAGS,
    ) -> windows::core::Result<Self> {
        let mut h = core::mem::MaybeUninit::uninit();
        unsafe {
            RegOpenKeyExW(root, subkey, options, sam_desired, h.as_mut_ptr()).ok()?;
        }

        Ok(Self(unsafe { h.assume_init() }))
    }

    pub fn string_value(
        &self,
        value: impl windows::core::Param<PCWSTR> + Copy,
        flags: REG_ROUTINE_FLAGS,
    ) -> windows::core::Result<std::ffi::OsString> {
        let mut tempbuffer = [0u16; 256];
        let mut len = 0;
        let r = unsafe {
            RegGetValueW(
                self.0,
                None,
                value,
                flags | RRF_RT_REG_SZ,
                None,
                Some(tempbuffer.as_mut_ptr() as _),
                Some(&mut len),
            )
        };
        if r == ERROR_SUCCESS {
            return Ok(std::ffi::OsString::from_wide(
                &tempbuffer[..(len / 2) as usize - 1],
            ));
        }
        if r == ERROR_MORE_DATA {
            // retry
            let mut buffer = Vec::with_capacity((len / 2) as _);
            unsafe {
                buffer.set_len(buffer.capacity());
            }
            unsafe {
                RegGetValueW(
                    self.0,
                    None,
                    value,
                    flags | RRF_RT_REG_SZ,
                    None,
                    Some(buffer.as_mut_ptr() as _),
                    Some(&mut len),
                )
                .ok()?;
            }

            return Ok(std::ffi::OsString::from_wide(
                &buffer[..(len / 2) as usize - 1],
            ));
        }

        Err(windows::core::Error::from(r))
    }
}

fn find_win10_sdk() -> (std::path::PathBuf, std::ffi::OsString) {
    // レジストリの中にあるらしい
    // https://stackoverflow.com/questions/35119223/how-to-programmatically-detect-and-locate-the-windows-10-sdk

    let key = RegistryKey::open(
        HKEY_LOCAL_MACHINE,
        w!("SOFTWARE\\WOW6432Node\\Microsoft\\Microsoft SDKs\\Windows\\v10.0"),
        None,
        KEY_READ,
    )
    .expect("Failed to open registry");

    let installation_folder = key
        .string_value(w!("InstallationFolder"), REG_ROUTINE_FLAGS(0))
        .expect("Failed to get InstallationFolder value");
    let mut product_version = key
        .string_value(w!("ProductVersion"), REG_ROUTINE_FLAGS(0))
        .expect("Failed to get ProductVersion value");
    product_version.push(".0");

    (
        std::path::PathBuf::from(installation_folder),
        product_version,
    )
}

fn find_win10_sdk_include_folder(
    installation_folder: &std::path::PathBuf,
    product_version: &std::ffi::OsString,
) -> std::path::PathBuf {
    installation_folder.join("Include").join(product_version)
}

fn find_win10_sdk_bin_folder(
    installation_folder: &std::path::PathBuf,
    product_version: &std::ffi::OsString,
) -> std::path::PathBuf {
    let bits_str = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        unimplemented!();
    };

    installation_folder
        .join("bin")
        .join(product_version)
        .join(bits_str)
}
