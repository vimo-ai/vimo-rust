//! C 字符串转换工具

use std::ffi::{c_char, CStr, CString};

use crate::FfiError;

/// 将 C 字符串指针转换为 Rust &str
///
/// # Safety
/// 调用者必须确保指针有效且指向以 null 结尾的 UTF-8 字符串
///
/// # 示例
///
/// ```rust,ignore
/// let rust_str = unsafe { cstr_to_str(c_ptr)? };
/// ```
pub unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Result<&'a str, FfiError> {
    if ptr.is_null() {
        return Err(FfiError::NullPointer);
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map_err(|_| FfiError::InvalidUtf8)
}

/// 将 C 字符串指针转换为 Rust String
///
/// # Safety
/// 调用者必须确保指针有效且指向以 null 结尾的 UTF-8 字符串
pub unsafe fn cstr_to_string(ptr: *const c_char) -> Result<String, FfiError> {
    cstr_to_str(ptr).map(|s| s.to_string())
}

/// 将 Rust 字符串转换为 C 字符串（堆分配）
///
/// 返回的指针必须由调用者释放（使用 `free_cstring`）
pub fn str_to_cstring(s: &str) -> Result<*mut c_char, FfiError> {
    CString::new(s)
        .map(|cs| cs.into_raw())
        .map_err(|_| FfiError::StringContainsNull)
}

/// 释放由本库分配的 C 字符串
///
/// # Safety
/// 指针必须是由 `str_to_cstring` 或类似函数返回的
#[no_mangle]
pub unsafe extern "C" fn vimo_ffi_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// 可选的 C 字符串转换 - null 返回 None
///
/// # Safety
/// 如果指针非 null，必须指向有效的 UTF-8 字符串
pub unsafe fn cstr_to_option_str<'a>(ptr: *const c_char) -> Result<Option<&'a str>, FfiError> {
    if ptr.is_null() {
        Ok(None)
    } else {
        cstr_to_str(ptr).map(Some)
    }
}

/// 可选的 C 字符串转换 - null 返回默认值
///
/// # Safety
/// 如果指针非 null，必须指向有效的 UTF-8 字符串
pub unsafe fn cstr_to_str_or<'a>(ptr: *const c_char, default: &'a str) -> &'a str {
    if ptr.is_null() {
        default
    } else {
        CStr::from_ptr(ptr).to_str().unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr_to_str() {
        let cs = CString::new("hello").unwrap();
        let result = unsafe { cstr_to_str(cs.as_ptr()) };
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_cstr_to_str_null() {
        let result = unsafe { cstr_to_str(std::ptr::null()) };
        assert!(matches!(result, Err(FfiError::NullPointer)));
    }

    #[test]
    fn test_str_to_cstring() {
        let ptr = str_to_cstring("hello").unwrap();
        let back = unsafe { CString::from_raw(ptr) };
        assert_eq!(back.to_str().unwrap(), "hello");
    }

    #[test]
    fn test_cstr_to_option_str() {
        let cs = CString::new("hello").unwrap();
        let result = unsafe { cstr_to_option_str(cs.as_ptr()) };
        assert_eq!(result.unwrap(), Some("hello"));

        let result = unsafe { cstr_to_option_str(std::ptr::null()) };
        assert_eq!(result.unwrap(), None);
    }
}
