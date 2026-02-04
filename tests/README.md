# MiraScript 黑盒测试

运行黑盒测试以验证 MiraScript 的跨平台兼容性和功能完整性。

## 测试环境

运行测试时，需要提供以下测试上下文，该上下文包含一组用于断言和测试的工具函数。

```mira
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

参考 [packages/mirascript/tests/black-box/\_run.ts](../packages/mirascript/tests/black-box/_run.ts) 中的实现。
