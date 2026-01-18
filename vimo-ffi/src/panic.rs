//! Panic 捕获工具
//!
//! Rust 的 panic 跨 FFI 边界是未定义行为，必须在边界处捕获。

use std::any::Any;
use std::ffi::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::set_error;

/// FFI 边界防护 - 捕获 panic 并转换为错误
///
/// 这是最常用的 FFI 包装函数，适用于返回 bool 且有 out_error 参数的场景。
///
/// # 参数
/// - `out_error`: 错误输出指针，panic 或错误时写入错误信息
/// - `default`: panic 或错误时返回的默认值
/// - `f`: 要执行的闭包，返回 `Result<T, E>`
///
/// # 示例
///
/// ```rust,ignore
/// #[no_mangle]
/// pub extern "C" fn do_something(out_error: *mut *mut c_char) -> bool {
///     ffi_boundary(out_error, false, || {
///         might_fail()?;
///         Ok(true)
///     })
/// }
/// ```
pub fn ffi_boundary<T, E, F>(out_error: *mut *mut c_char, default: T, f: F) -> T
where
    E: std::fmt::Display,
    F: FnOnce() -> Result<T, E>,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            unsafe { set_error(out_error, &e.to_string()) };
            default
        }
        Err(panic) => {
            let msg = extract_panic_message(&panic);
            unsafe { set_error(out_error, &format!("internal panic: {}", msg)) };
            default
        }
    }
}

/// FFI 边界防护 - 简化版，不处理 Result
///
/// 适用于不会返回错误的场景，只捕获 panic。
///
/// # 参数
/// - `default`: panic 时返回的默认值
/// - `f`: 要执行的闭包
///
/// # 示例
///
/// ```rust,ignore
/// #[no_mangle]
/// pub extern "C" fn get_count() -> i32 {
///     ffi_boundary_simple(-1, || {
///         compute_count()
///     })
/// }
/// ```
pub fn ffi_boundary_simple<T, F>(default: T, f: F) -> T
where
    F: FnOnce() -> T,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic) => {
            let msg = extract_panic_message(&panic);
            eprintln!("[vimo-ffi] panic caught: {}", msg);
            default
        }
    }
}

/// FFI 边界防护 - 带日志回调
///
/// 允许自定义 panic 日志处理。
///
/// # 参数
/// - `default`: panic 时返回的默认值
/// - `on_panic`: panic 时的回调，接收 panic 消息
/// - `f`: 要执行的闭包
pub fn ffi_boundary_with_log<T, F, L>(default: T, on_panic: L, f: F) -> T
where
    F: FnOnce() -> T,
    L: FnOnce(&str),
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic) => {
            let msg = extract_panic_message(&panic);
            on_panic(&msg);
            default
        }
    }
}

/// 从 panic 信息中提取可读消息
fn extract_panic_message(panic: &Box<dyn Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_ffi_boundary_success() {
        let result: bool = ffi_boundary(ptr::null_mut(), false, || Ok::<_, String>(true));
        assert!(result);
    }

    #[test]
    fn test_ffi_boundary_error() {
        let mut error_ptr: *mut c_char = ptr::null_mut();
        let result: bool = ffi_boundary(&mut error_ptr, false, || {
            Err::<bool, _>("something failed")
        });
        assert!(!result);
        assert!(!error_ptr.is_null());
        // 清理
        unsafe {
            if !error_ptr.is_null() {
                let _ = std::ffi::CString::from_raw(error_ptr);
            }
        }
    }

    #[test]
    fn test_ffi_boundary_panic() {
        let mut error_ptr: *mut c_char = ptr::null_mut();
        let result: bool = ffi_boundary(&mut error_ptr, false, || {
            panic!("test panic");
            #[allow(unreachable_code)]
            Ok::<bool, String>(true)
        });
        assert!(!result);
        assert!(!error_ptr.is_null());
        // 清理
        unsafe {
            if !error_ptr.is_null() {
                let _ = std::ffi::CString::from_raw(error_ptr);
            }
        }
    }

    #[test]
    fn test_ffi_boundary_simple_success() {
        let result = ffi_boundary_simple(-1, || 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_ffi_boundary_simple_panic() {
        let result = ffi_boundary_simple(-1, || {
            panic!("test panic");
            #[allow(unreachable_code)]
            42
        });
        assert_eq!(result, -1);
    }
}
