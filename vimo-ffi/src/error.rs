//! FFI 错误处理工具

use std::ffi::{c_char, CString};

use thiserror::Error;

/// FFI 通用错误类型
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FfiError {
    #[error("null pointer")]
    NullPointer,

    #[error("invalid UTF-8 string")]
    InvalidUtf8,

    #[error("string contains null byte")]
    StringContainsNull,

    #[error("{0}")]
    Custom(String),
}

impl FfiError {
    /// 创建自定义错误
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

/// 设置 FFI 错误输出指针
///
/// # Safety
/// `out_error` 必须是有效的可写指针，或者 null（会被忽略）
///
/// # 示例
///
/// ```rust,ignore
/// unsafe { set_error(out_error, "something went wrong") };
/// ```
pub unsafe fn set_error(out_error: *mut *mut c_char, msg: &str) {
    if out_error.is_null() {
        return;
    }
    if let Ok(c_string) = CString::new(msg) {
        *out_error = c_string.into_raw();
    }
}

/// 设置 FFI 错误输出指针（从 Error trait）
///
/// # Safety
/// 同 `set_error`
pub unsafe fn set_error_from<E: std::fmt::Display>(out_error: *mut *mut c_char, err: &E) {
    set_error(out_error, &err.to_string());
}

/// 检查指针非空，否则返回错误
///
/// # 示例
///
/// ```rust,ignore
/// fn my_ffi(ptr: *const c_char, out_error: *mut *mut c_char) -> bool {
///     if let Err(e) = check_not_null(ptr) {
///         unsafe { set_error(out_error, &e.to_string()) };
///         return false;
///     }
///     // ...
/// }
/// ```
pub fn check_not_null<T>(ptr: *const T) -> Result<(), FfiError> {
    if ptr.is_null() {
        Err(FfiError::NullPointer)
    } else {
        Ok(())
    }
}

/// 检查多个指针非空
///
/// # 示例
///
/// ```rust,ignore
/// check_all_not_null(&[ptr1 as *const _, ptr2 as *const _])?;
/// ```
pub fn check_all_not_null(ptrs: &[*const std::ffi::c_void]) -> Result<(), FfiError> {
    for ptr in ptrs {
        if ptr.is_null() {
            return Err(FfiError::NullPointer);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_set_error() {
        let mut error_ptr: *mut c_char = ptr::null_mut();
        unsafe { set_error(&mut error_ptr, "test error") };
        assert!(!error_ptr.is_null());

        let error_str = unsafe { CString::from_raw(error_ptr) };
        assert_eq!(error_str.to_str().unwrap(), "test error");
    }

    #[test]
    fn test_set_error_null_out() {
        // 不应该 panic
        unsafe { set_error(ptr::null_mut(), "test error") };
    }

    #[test]
    fn test_check_not_null() {
        let val = 42i32;
        assert!(check_not_null(&val as *const i32).is_ok());
        assert!(check_not_null(ptr::null::<i32>()).is_err());
    }

    #[test]
    fn test_ffi_error_display() {
        assert_eq!(FfiError::NullPointer.to_string(), "null pointer");
        assert_eq!(FfiError::InvalidUtf8.to_string(), "invalid UTF-8 string");
        assert_eq!(
            FfiError::custom("my error").to_string(),
            "my error"
        );
    }
}
