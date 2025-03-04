use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{CreateEventW, ResetEvent, SetEvent},
};
use windows_core::PCWSTR;

#[repr(transparent)]
pub struct NativeEvent(HANDLE);
unsafe impl Sync for NativeEvent {}
unsafe impl Send for NativeEvent {}
impl Drop for NativeEvent {
    #[inline(always)]
    fn drop(&mut self) {
        if let Err(e) = unsafe { CloseHandle(self.0) } {
            tracing::warn!({?e}, "CloseHandle failed");
        }
    }
}
impl NativeEvent {
    #[inline(always)]
    pub fn new(
        manual_reset: bool,
        name: impl windows_core::Param<PCWSTR>,
    ) -> windows_core::Result<Self> {
        let h = unsafe { CreateEventW(None, manual_reset, false, name)? };
        Ok(Self(h))
    }

    #[inline(always)]
    pub fn signal(&self) {
        unsafe {
            SetEvent(self.0).unwrap();
        }
    }

    #[inline(always)]
    pub fn reset(&self) {
        unsafe {
            ResetEvent(self.0).unwrap();
        }
    }

    pub const fn handle(&self) -> HANDLE {
        self.0
    }
}
