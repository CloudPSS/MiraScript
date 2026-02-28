# 变量

在 MiraScript 中，变量用于给值起一个名字，方便后续引用。

## 不可变变量 `let`

使用 `let` 声明一个不可变变量。变量一旦创建，就不能再修改：

```mira
let name = "小明";
let age = 20;
let pi = 3.14159;

debug_print("$name 今年 $age 岁");
debug_print("π ≈ $pi");
```

尝试给不可变变量重新赋值会产生错误：

```mira
let x = 10;
// x = 20; // 错误！不可变变量不能重新赋值
debug_print(x);
```

## 可变变量 `let mut`

如果你需要一个可以修改的变量，使用 `let mut` 声明：

```mira
let mut count = 0;
debug_print("初始值:", count);

count = 10;
debug_print("赋值后:", count);
```

### 复合赋值运算符

可变变量支持复合赋值，这是一种简写形式：

```mira
let mut x = 10;
x += 5;   // 等价于 x = x + 5
debug_print("x += 5:", x);

x -= 3;   // 等价于 x = x - 3
debug_print("x -= 3:", x);

x *= 2;   // 等价于 x = x * 2
debug_print("x *= 2:", x);

x /= 4;   // 等价于 x = x / 4
debug_print("x /= 4:", x);

x %= 3;   // 等价于 x = x % 3
debug_print("x %= 3:", x);
```

## 常量 `const`

使用 `const` 声明常量，常量名必须以 `@` 开头：

```mira
const @MAX_SIZE = 100;
const @GREETING = "你好";

debug_print(@MAX_SIZE);
debug_print(@GREETING);
```

常量与 `let` 类似也不能修改，但 `@` 前缀让它在代码中一目了然，表明这是一个不会改变的配置值。

## 块作用域

花括号 `{ }` 创建一个新的作用域，在其中声明的变量对外不可见：

```mira
let x = "外层";
{
  let y = "内层";
  debug_print(x); // 可以访问外层变量
  debug_print(y); // 可以访问内层变量
}
// debug_print(y); // 错误！y 在块外不可见
debug_print(x);
```

内层作用域可以声明与外层同名的变量，称为**遮蔽**（shadowing）：

```mira
let value = 10;
debug_print("外层:", value);   // 10
{
  let value = 20;              // 遮蔽外层的 value
  debug_print("内层:", value); // 20
}
debug_print("外层:", value);   // 10，不受内层影响
```

## 查看类型

使用 `type()` 关键字可以查看任何值的类型：

```mira
debug_print(type(42));         // "number"
debug_print(type("hello"));    // "string"
debug_print(type(true));       // "boolean"
debug_print(type(nil));        // "nil"
debug_print(type([1, 2]));     // "array"
debug_print(type((a: 1)));     // "record"
debug_print(type(fn { }));     // "function"
```

## 类型转换

MiraScript 提供了显式类型转换函数：

```mira
// 转为数字
debug_print(to_number("42"));         // 42
debug_print(to_number(true));         // 1
debug_print(to_number(false));        // 0

// 转为字符串
debug_print(to_string(42));           // "42"
debug_print(to_string(true));         // "true"

// 带默认值的转换（转换失败时返回默认值而非报错）
debug_print(to_number("abc", 0));     // 0
```

## 小结

- `let` 声明不可变绑定，`let mut` 声明可变绑定
- `const @NAME` 声明常量，以 `@` 开头
- `{ }` 创建块作用域，内部变量对外不可见
- `type()` 查看类型，`to_number()`、`to_string()` 进行显式类型转换
