# 模块

模块用于将相关的变量和函数组织在一起，避免命名冲突，使代码结构更清晰。

## 定义模块

使用 `mod` 关键字定义模块，用 `pub` 关键字标记需要对外公开的成员：

```mira
mod math_utils {
  // pub 标记的成员可以从外部访问
  pub fn square { it^2 }
  pub fn cube { it^3 }
  pub const @PI_APPROX = 3.14;

  // 没有 pub 的成员是私有的，只能在模块内部使用
  fn helper { it * 2 }

  // 内部可以使用私有成员
  pub fn double_square { helper(it)::square() }
}

debug_print("square(5) =", math_utils.square(5));
debug_print("cube(3) =", math_utils.cube(3));
debug_print("PI ≈", math_utils.@PI_APPROX);
debug_print("double_square(3) =", math_utils.double_square(3));
```

## 访问模块成员

使用 `.` 访问模块的公开成员：

```mira
mod config {
  pub let host = "localhost";
  pub let port = 8080;
  pub let version = "1.0.0";
}

debug_print("服务器: $(config.host):$(config.port)");
debug_print("版本:", config.version);
```

## 模块中的可变状态

模块可以包含可变成员，但只能通过模块内部的函数来修改：

```mira
mod counter {
  pub let mut value = 0;

  pub fn increment() {
    value += 1;
  }

  pub fn reset() {
    value = 0;
  }
}

debug_print("初始值:", counter.value);
counter.increment();
counter.increment();
counter.increment();
debug_print("加三次:", counter.value);
counter.reset();
debug_print("重置后:", counter.value);
```

:::info
注意：不能从外部直接给模块成员赋值（如 `counter.value = 10` 会报错），必须通过模块提供的函数来修改。
:::

## 嵌套模块

模块可以嵌套：

```mira
mod app {
  pub mod database {
    pub let connection = "mysql://localhost:3306";
    pub fn query(sql) { "执行查询: $sql" }
  }

  pub mod cache {
    pub let mut size = 0;
    pub fn add() { size += 1; }
  }
}

debug_print(app.database.connection);
debug_print(app.database.query("SELECT * FROM users"));

app.cache.add();
debug_print("缓存大小:", app.cache.size);
```

## 遍历模块

可以使用 `for` 遍历模块的公开成员名：

```mira
mod settings {
  pub let theme = "dark";
  pub let language = "zh-CN";
  pub let font_size = 14;
}

for key in settings {
  debug_print("$key:", settings[key]);
}
```

## 解构模块

可以从模块中提取函数：

```mira
mod utils {
  pub fn add(x, y) { x + y }
  pub fn mul(x, y) { x * y }
}

let (:add, :mul) = utils;
debug_print("add(3, 4) =", add(3, 4));
debug_print("mul(3, 4) =", mul(3, 4));
```

## 内置模块：`matrix`

MiraScript 提供了 [`matrix` 模块](../lib/20-matrix.md) 用于矩阵运算。矩阵用二维数组表示：

```mira
let m = [[1, 2], [3, 4]];

debug_print("大小:", matrix.size(m));
debug_print("转置:", matrix.transpose(m));
```

```mira
let a = [[1, 2], [3, 4]];
let b = [[5, 6], [7, 8]];

debug_print("矩阵加法:", matrix.add(a, b));
debug_print("矩阵乘法:", matrix.multiply(a, b));
```

```mira
// 创建特殊矩阵
debug_print("3×3 单位矩阵:", matrix.identity(3));
debug_print("2×3 零矩阵:", matrix.zeros(2, 3));
```

## 内置模块：`complex`

MiraScript 提供了 [`complex` 模块](../lib/10-complex.md) 用于复数运算。复数使用普通 record `(实部, 虚部)` 表示，`complex.I` 是虚数单位 `(0, 1)`：

```mira
let z = (3, 4);

debug_print("实部:", complex.real(z)); // 3
debug_print("虚部:", complex.imag(z)); // 4
debug_print("模:", complex.abs(z));    // 5
debug_print("共轭:", complex.conj(z)); // (3, -4)
```

使用 `complex.from` 可以显式转换输入。其他复数函数的参数也使用相同规则：record 同时包含 `0`、`1` 字段时，分别按 `to_number` 语义转换，忽略其他字段；其他输入整体按数字转换，虚部补 `0`。无法转换的值会报错，数组 `[3, 4]` 不会被当作复数。

```mira
debug_print(complex.from(3));                        // (3, 0)
debug_print(complex.from(('3', '4', name: '示例'))); // (3, 4)
debug_print(complex.add(1, 2));                      // (3, 0)
debug_print(complex.multiply((1, 2), (3, 4)));       // (-5, 10)
debug_print(complex.multiply(complex.I, complex.I)); // (-1, 0)
```

复数运算通过模块函数完成，不改变 `+`、`*` 等运算符的含义。运算结果始终是复数 record，只有 `real`、`imag`、`abs`、`arg` 等结果必定为实数的函数返回 number。

模块还提供幂根、指数对数、三角及双曲函数，以及对应的反函数。多值函数返回主值，角度单位为弧度；`polar(r, theta)` 用模和辐角构造复数：

```mira
debug_print(complex.sqrt(-4));        // (0, 2)
debug_print(complex.log(-1));         // (0, PI)
debug_print(complex.polar(2, 0));     // (2, 0)
debug_print(complex.arg(complex.I));  // PI / 2
```

## 小结

- `mod name { }` 定义模块，`pub` 标记公开成员
- 通过 `.` 访问模块成员
- 可变成员只能通过模块内部函数修改
- 模块可以嵌套，也可以遍历和解构
- `matrix` 模块提供矩阵运算功能
- `complex` 模块使用 `(实部, 虚部)` record 提供复数运算功能
