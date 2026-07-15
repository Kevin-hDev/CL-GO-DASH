use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub(super) struct CefLibrary;

impl CefLibrary {
    pub(super) fn load(framework: &Path) -> Result<Self, ()> {
        let path = CString::new(framework.as_os_str().as_bytes()).map_err(|_| ())?;
        let path = unsafe { &*path.as_c_str().as_ptr() };
        if cef::load_library(Some(path)) != 1 {
            return Err(());
        }
        Ok(Self)
    }
}

impl Drop for CefLibrary {
    fn drop(&mut self) {
        let _ = cef::unload_library();
    }
}
