use super::DetectedEditor;
use core_foundation::array::{CFArrayGetCount, CFArrayGetValueAtIndex};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation::url::{kCFURLPOSIXPathStyle, CFURL};
use std::path::Path;

type LSRolesMask = u32;
const K_LS_ROLES_ALL: LSRolesMask = 0xFFFFFFFF;

extern "C" {
    fn LSCopyApplicationURLsForURL(
        url: core_foundation::url::CFURLRef,
        roles: LSRolesMask,
    ) -> core_foundation::array::CFArrayRef;

    fn LSCopyDefaultApplicationURLForURL(
        url: core_foundation::url::CFURLRef,
        roles: LSRolesMask,
        out_error: *mut core_foundation::error::CFErrorRef,
    ) -> core_foundation::url::CFURLRef;
}

fn app_name_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

fn cfurl_to_path(url_ref: core_foundation::url::CFURLRef) -> Option<String> {
    if url_ref.is_null() {
        return None;
    }
    let url: CFURL = unsafe { TCFType::wrap_under_get_rule(url_ref) };
    let cf_path = url.get_file_system_path(kCFURLPOSIXPathStyle);
    Some(cf_path.to_string())
}

pub fn detect(file_path: &Path) -> Vec<DetectedEditor> {
    let path_str = file_path.to_str().unwrap_or("");
    let cf_str = CFString::new(path_str);
    let file_url = CFURL::from_file_system_path(cf_str, kCFURLPOSIXPathStyle, false);

    let default_path = unsafe {
        let raw = LSCopyDefaultApplicationURLForURL(
            file_url.as_concrete_TypeRef(),
            K_LS_ROLES_ALL,
            std::ptr::null_mut(),
        );
        if raw.is_null() {
            None
        } else {
            let url = CFURL::wrap_under_create_rule(raw);
            let p = url.get_file_system_path(kCFURLPOSIXPathStyle);
            Some(p.to_string())
        }
    };

    let raw_array = unsafe {
        LSCopyApplicationURLsForURL(file_url.as_concrete_TypeRef(), K_LS_ROLES_ALL)
    };
    if raw_array.is_null() {
        return vec![];
    }

    let count = unsafe { CFArrayGetCount(raw_array) };
    let mut editors = Vec::new();

    for i in 0..count {
        let raw_url =
            unsafe { CFArrayGetValueAtIndex(raw_array, i) as core_foundation::url::CFURLRef };
        if let Some(path) = cfurl_to_path(raw_url) {
            let is_default = default_path.as_ref() == Some(&path);
            editors.push(DetectedEditor {
                name: app_name_from_path(&path),
                path,
                is_default,
            });
        }
    }

    unsafe { core_foundation::base::CFRelease(raw_array as *const _) };

    editors.sort_by(|a, b| b.is_default.cmp(&a.is_default));
    editors
}
