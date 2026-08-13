# MiraScript 测试说明

本目录存放 MiraScript 黑盒测试。
每个 `.mira` 文件会在独立上下文中执行，因此测试文件应当：

- 只覆盖一个清晰主题，优先按语言构造或标准库函数拆分；
- 不依赖其他测试文件声明的变量、函数或模块；
- 使用行为命名，避免把同一断言复制到“综合”测试和专项测试中；
- 对压力测试使用 `_huge.mira` 后缀。此类文件有意保持为大文件，以验证大源码、深层结构或大数据量场景。

目录职责：

- `feature/`：声明、模块、闭包、插值等语言特性；
- `logic/`：分支、循环、返回和模式匹配；
- `types/`：各值类型的构造与访问行为；
- `operators/`：运算符、优先级和求值语义；
- `lib/`：全局标准库与内置模块；
- `e2e/`：跨多个语言特性的完整示例；
- `harness/`：验证黑盒测试注入的上下文；
- 文件名包含 `_huge`：大源码或大数据量压力测试。

## 黑盒测试上下文

```mirascript-doc
/** 断言两个值严格相等 */
fn t_eq(a, b, message);
/** 断言两个值不相等 */
fn t_ne(a, b, message);
/** 断言一个值为 `true` */
fn t_true(value, message);
/** 断言一个值为 `false` */
fn t_false(value, message);
/** 断言函数抛出错误 */
fn t_throws(func, message);
/** 断言函数在指定时间内未执行完毕 */
fn t_timeout(func, message);
/** 断言测试失败 */
fn t_never(message);

/** 用于测试的数据 */
let v_array = [];
let v_record = ();
let v_nil = nil;
let v_true = true;
let v_false = false;
let v_number = 42;
let v_string = "Hello, Mira!";
let v_fn = fn() { return "I am a function"; };
let v_fn_another = fn() { return "I am another function"; };
let v_module = /* 名为 v_module 的模块 */;
let v_module_another = /* 名为 v_module_another 的模块 */;

/** 可选测试数据 */
let has_extern = true; /* v_extern 和 v_extern_another 存在 */
let v_extern = /* 空外部对象 */;
let v_extern_another = /* 另一个外部对象 */;
```

## 参考实现

测试上下文的注入逻辑可参考：

- `packages/mirascript/tests/black-box/_run.ts`
