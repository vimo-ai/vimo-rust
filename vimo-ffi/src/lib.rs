//! Vimo FFI Safety Utilities
//!
//! 提供 Rust FFI 边界的安全工具，包括：
//! - Panic 捕获，防止跨 FFI 边界传播
//! - C 字符串转换工具
//! - 统一的错误处理模式
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use vimo_ffi::{ffi_boundary, set_error};
//! use std::ffi::c_char;
//!
//! #[no_mangle]
//! pub extern "C" fn my_ffi_function(
//!     input: *const c_char,
//!     out_error: *mut *mut c_char,
//! ) -> bool {
//!     ffi_boundary(out_error, false, || {
//!         // 你的逻辑，可以安全地 panic 或返回 Result
//!         let input_str = unsafe { cstr_to_str(input)? };
//!         do_something(input_str)?;
//!         Ok(true)
//!     })
//! }
//! ```

mod panic;
mod string;
mod error;

pub use panic::*;
pub use string::*;
pub use error::*;
