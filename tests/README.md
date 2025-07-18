# MiraScript 黑盒测试

运行黑盒测试以验证 MiraScript 的跨平台兼容性和功能完整性。

## 测试环境

运行测试时，需要提供测试上下文 `t`，该上下文包含一组用于断言和测试的工具函数。

```mira
/** 断言两个值严格相等 */
fn t_eq(a, b);
/** 断言两个值不相等 */
fn t_ne(a, b);
/** 断言一个值为 `true` */
fn t_true(value);
/** 断言一个值为 `false` */
fn t_false(value);
/** 断言函数抛出错误 */
fn t_throws(func);
/** 记录与比对 snapshot */
fn t_snapshot(..values);
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
let v_extern = /* 只需满足 type(v_extern) == "extern" */;
let v_module = /* 只需满足 type(v_module) == "module" */;
```

参考 [packages/mirascript/tests/black-box.ts](../packages/mirascript/tests/black-box.ts) 中的实现。

## 测试快照

测试快照存储在测试文件对应的 `.jsonl` 文件，包含测试中调用 `t_snapshot` 的结果。
