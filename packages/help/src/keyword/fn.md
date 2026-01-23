---
token: 'fn'
order: 24
---

使用 `fn` 关键字定义一个函数或函数表达式。

- 定义一个简单的函数

  ```mira
  fn add(a, b) {
    return a + b;
  }
  ```

- 定义一个变量，并使用函数表达式初始化

  ```mira
  let multiply = fn(x, y) {
    return x * y;
  };
  ```

- 当函数只有一个参数时，可以省略括号，并用 `it` 引用该参数
  ```mira
  fn square {
    return it * it;
  }
  ```
