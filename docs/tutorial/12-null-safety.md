# 空安全与错误处理

在处理真实数据时，经常会遇到缺失值或无效数据的情况。MiraScript 通过 `nil` 和空安全机制优雅地处理这些问题。

## `nil` —— 空值

`nil` 表示"没有值"，在很多场景下自然出现：

```mira
let arr = [1, 2, 3];
debug_print(arr[10]);       // nil，索引越界

let record = (name: "小明");
debug_print(record.age);    // nil，属性不存在

fn find_even(arr) {
  for n in arr {
    if n % 2 == 0 { return n; }
  }
}
debug_print(find_even([1, 3, 5])); // nil，没找到偶数
```

## 空安全访问

MiraScript 默认的属性访问是**空安全**的。对 `nil` 取属性不会报错，而是返回 `nil`：

```mira
let user = (name: "小明", profile: (age: 20));
let empty = nil;

debug_print(user.name);             // "小明"
debug_print(user.profile.age);      // 20
debug_print(empty.name);            // nil，不会报错
debug_print(empty.profile.age);     // nil，连续访问也安全
debug_print(user.address.street);   // nil，中间属性不存在也安全
```

这意味着你不需要在每一层访问前检查是否为 `nil`，代码更加简洁。

## 空合并运算符 `??`

`??` 运算符在左侧为 `nil` 时返回右侧的值，常用于提供默认值：

```mira
let config = (host: "localhost");

let host = config.host ?? "0.0.0.0";
let port = config.port ?? 8080;
let timeout = config.timeout ?? 30;

debug_print("host:", host);       // "localhost"（使用实际值）
debug_print("port:", port);       // 8080（使用默认值）
debug_print("timeout:", timeout); // 30（使用默认值）
```

### 链式默认值

```mira
// 依次尝试多个来源
let user_name = nil;
let env_name = nil;
let name = user_name ?? env_name ?? "anonymous";
debug_print("用户名:", name);
```

## 非空断言 `!`

`!` 操作符断言值不为 `nil`。如果值确实是 `nil`，会抛出异常：

```mira
let value = 42;
debug_print(value!);  // 42，没问题

// let empty = nil;
// debug_print(empty!);  // 会抛出 NilError 异常！
```

:::warning
谨慎使用 `!`。只在你确定值不为 `nil` 的时候使用，否则程序会因异常而中断。大多数情况下，`??` 是更安全的选择。
:::

## 带默认值的类型转换

内置的类型转换函数支持第二个参数作为转换失败时的默认值：

```mira
// 正常转换
debug_print(to_number("42"));            // 42

// 转换失败时使用默认值
debug_print(to_number("abc", 0));        // 0
debug_print(to_number(nil, -1));         // -1
debug_print(to_number("not a number", 0)); // 0
```

## 使用模式匹配处理错误

结合 `match` 和模式匹配，可以根据不同情况分别处理：

### 安全除法

```mira
fn safe_divide(a, b) {
  if b == 0 { nil } else { a / b }
}

let result = safe_divide(10, 3);
match result {
  case nil { debug_print("除法失败") }
  case r { debug_print("结果: $(r:.2)") }
};

// 或者使用 ?? 更简洁
let display = safe_divide(10, 0) ?? "无法计算";
debug_print(display);
```

### 类型分支处理

```mira
fn process(input) {
  match type(input) {
    case "number" { "数字 $input 的平方是 $(input^2)" }
    case "string" { "字符串 '$input' 的长度是 ${ len(input::chars()) }" }
    case "array" { "数组有 ${ len(input) } 个元素" }
    case "nil" { "空值" }
    case t { "不支持的类型: $t" }
  }
}

debug_print(process(5));
debug_print(process("hello"));
debug_print(process([1, 2, 3]));
debug_print(process(nil));
```

## 使用 `is` 检查 `nil`

```mira
let value = nil;

if value is nil {
  debug_print("值为空");
}

if value is not nil {
  debug_print("值不为空:", value);
} else {
  debug_print("值为空，使用默认值");
}
```

## 实际应用

### 安全的数据提取

```mira
// 从可能不完整的数据中安全地提取信息
fn create_user(input) {
  (
    name: input.name ?? "匿名用户",
    age: to_number(input.age, 0),
    email: input.email ?? "未设置",
    role: input.role ?? "普通用户",
  )
}

let partial_data = (name: "小明", age: "20");
let user = create_user(partial_data);
debug_print("用户:", user);
```

### 链式空安全处理

```mira
// 处理深层嵌套数据
let response = (
  data: (
    users: [
      (name: "Alice", contact: (email: "alice@example.com")),
      (name: "Bob", contact: nil),
    ]
  )
);

// 安全地获取每个用户的邮箱
for user in response.data.users {
  let email = user.contact.email ?? "无邮箱";
  debug_print("$(user.name): $email");
}
```

## 小结

- 属性访问默认空安全，`nil.anything` 返回 `nil` 而非报错
- `??` 空合并运算符提供默认值
- `!` 非空断言在值为 `nil` 时抛出异常，谨慎使用
- 类型转换函数支持 fallback 参数：`to_number(x, default)`
- 结合 `match`、`is`、`??` 实现健壮的错误处理
