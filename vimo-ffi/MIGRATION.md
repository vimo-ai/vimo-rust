# vimo-ffi 迁移计划

## 核心设计

vimo-ffi 的职责边界：

| 属于 vimo-ffi | 属于各项目 |
|--------------|-----------|
| catch_unwind 包装 | 日志系统 |
| C 字符串转换 | 日志桥接到宿主 |
| set_error 错误传递 | 日志格式/持久化 |

**关键决策**：日志是各项目的关注点，不是 vimo-ffi 的职责。有特殊日志需求的项目使用 `ffi_boundary_with_log` 自行包装。

## 各项目状态

| 项目 | 状态 | 说明 |
|------|------|------|
| mcp-router-core | ✅ 已完成 | ~35 个 FFI 函数，78 个测试通过 |
| vlaude-ffi | ✅ 已完成 | 14 个 FFI 函数，使用 ffi_boundary_simple |
| claude-session-db | ⚠️ 已有保护 | 17 处 catch_unwind，可选迁移 |
| socket-client-ffi | ⚠️ 已有保护 | 14 处 catch_unwind，可选迁移 |
| sugarloaf-ffi | ❌ 待迁移 | 3/80 函数有保护，需要特殊处理日志桥接 |

## 待办工作

### sugarloaf-ffi 迁移（主要工作）

sugarloaf-ffi 有自己的日志桥接系统（`rust_log_error!` → Swift LogManager），需要本地包装：

```rust
// sugarloaf-ffi/src/ffi/mod.rs
use vimo_ffi::ffi_boundary_with_log;

pub fn ffi_boundary<T, F>(default: T, f: F) -> T
where
    F: FnOnce() -> T,
{
    ffi_boundary_with_log(default, |msg| {
        rust_log_error!("[FFI panic] {}", msg);
    }, f)
}
```

步骤：
1. [ ] Cargo.toml 添加 `vimo-ffi = { git = "..." }`
2. [ ] 修改 ffi/mod.rs 的 ffi_boundary 实现
3. [ ] 给 77 个未保护函数添加 ffi_boundary 包装
   - terminal_pool.rs: 48 个
   - selection.rs: 7 个
   - render_scheduler.rs: 6 个
   - hyperlink.rs: 5 个
   - keyboard.rs: 3 个
   - ime.rs: 2 个
   - logging.rs: 2 个
   - word_boundary.rs: 2 个
   - cursor.rs: 1 个
   - lib.rs: 1 个

### claude-session-db / socket-client-ffi（可选）

已有 catch_unwind 保护，可选择迁移到 vimo-ffi 统一风格。优先级低。

## 相关路径

- vimo-ffi: `/Users/higuaifan/Desktop/vimo/vimo-rust/vimo-ffi`
- sugarloaf-ffi: `ETerm/ETerm/rio/sugarloaf-ffi/src/ffi/`
- sugarloaf 日志系统: `ffi/logging.rs`

## 参考提交

- mcp-router-core: 已合入 main
- vlaude-ffi: `5aae04d` ✨ Add ffi_boundary_simple wrapper for panic safety
- claude-session-db WAL: `0135072` ✨ Enable WAL mode and improve search fallback
