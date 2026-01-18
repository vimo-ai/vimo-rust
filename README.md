# Vimo Rust Infrastructure

共享的 Rust 基础设施，供所有 Vimo 项目使用。

## Crates

| Crate | 说明 |
|-------|------|
| `vimo-ffi` | FFI 安全工具：panic 捕获、C 字符串转换、错误处理 |

## 使用

```toml
[dependencies]
vimo-ffi = { git = "https://github.com/vimo-ai/vimo-rust" }
```

开发时可在 workspace 中用 `[patch]` 覆盖为本地路径：

```toml
[patch."https://github.com/vimo-ai/vimo-rust"]
vimo-ffi = { path = "../vimo-rust/vimo-ffi" }
```

## vimo-ffi 示例

```rust
use vimo_ffi::{ffi_boundary, cstr_to_str, set_error};
use std::ffi::c_char;

#[no_mangle]
pub extern "C" fn my_function(
    input: *const c_char,
    out_error: *mut *mut c_char,
) -> bool {
    ffi_boundary(out_error, false, || {
        let input_str = unsafe { cstr_to_str(input)? };
        do_something(input_str)?;
        Ok(true)
    })
}
```

## 计划模块

- [ ] `vimo-fs` - 文件操作：atomic write、backup/restore
- [ ] `vimo-config` - 配置文件处理
- [ ] `vimo-async` - 异步运行时工具

## License

MIT
