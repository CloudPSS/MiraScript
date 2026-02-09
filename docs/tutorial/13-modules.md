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

MiraScript 提供了 `matrix` 模块用于矩阵运算。矩阵用二维数组表示：

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
debug_print("3×3 单位矩阵:", matrix.identity(3, 3));
debug_print("2×3 零矩阵:", matrix.zeros(2, 3));
```

## 小结

- `mod name { }` 定义模块，`pub` 标记公开成员
- 通过 `.` 访问模块成员
- 可变成员只能通过模块内部函数修改
- 模块可以嵌套，也可以遍历和解构
- `matrix` 模块提供矩阵运算功能
