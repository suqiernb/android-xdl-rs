mod internals;
mod wrapper;

use syn::{DeriveInput, parse_macro_input};

/**
## 派生属性
### `#[native(implicit(rename = "snake_case"))]`
- 隐式生成的符号会使用指定的命名规则对字段名进行转换
- 可用选项: `"lowercase"` `"UPPERCASE"` `"PascalCase"` `"camelCase"`
  `"snake_case"` `"SCREAMING_SNAKE_CASE"`
- 默认值: `"snake_case"`

### `#[native(implicit(debug))]`
- 隐式生成的符号将从调试符号表中加载
- 默认值: `false`

### `#[native(symbol(prefix = "il2cpp_", suffix = ""))]`
为所有符号添加前后缀

### `#[native(logger)]`
生成日志相关代码: 成功加载记录 `trace` 级别日志, 失败记录 `warn` 级别日志
- 默认值: `false`

## 字段属性
### `#[native(implicit)]`
参考[派生属性](#)

### `#[native(logger)]`
参考[派生属性](#)

### `#[native(symbol = ["puts"])]`
指定要加载的符号名称
- 使用 `c"puts"` 表示不需要添加前后缀
- 使用 `"puts"debug` 表示从调试符号表中加载
- 如果没有指定, 会根据字段名隐式生成默认的符号
- 如果存在多个符号名称, 将会按顺序优先级加载, 直至加载成功为止
 */
#[proc_macro_derive(NativeBridge, attributes(native))]
pub fn derive_native_bridge(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match wrapper::expand_derive(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
