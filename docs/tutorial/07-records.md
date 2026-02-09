# 记录

记录（record）是 MiraScript 中的键值对集合，类似于其他语言中的对象或字典。和数组一样，记录也是不可变的。

## 创建记录

使用圆括号 `()` 创建记录，键值对用逗号分隔：

```mira
let person = (name: "小明", age: 20, city: "北京");
debug_print(person);
```

### 命名键与未命名键

记录的键可以命名，也可以不命名。未命名的键会自动编号为 `0`、`1`、`2`……

```mira
let named = (x: 10, y: 20);           // 命名键
let unnamed = ("hello", 42, true);    // 未命名键，等价于 (0: "hello", 1: 42, 2: true)
let empty = ();                       // 空记录

debug_print("命名:", named);
debug_print("未命名:", unnamed);
debug_print("空记录:", empty);
```

:::info 单元素记录
创建只有一个未命名元素的记录时，需要加尾逗号以区别于括号分组：

```mira
let group = (42);    // 这是分组表达式，值为 42
let record = (42, ); // 这才是单元素记录
debug_print(type(group));   // "number"
debug_print(type(record));  // "record"
```

:::

### 键名省略

当键名与变量名相同时，可以使用 `:变量名` 简写：

```mira
let name = "小红";
let age = 19;
let student = (:name, :age);  // 等价于 (name: name, age: age)
debug_print(student);
```

### JSON 语法

也可以使用 JSON 对象语法来创建记录，此时键名必须加引号：

```mira
let data = { "name": "小华", "score": 95 };
debug_print(data);
debug_print(data.name);
```

## 访问属性

使用 `.` 或 `[]` 访问记录的属性：

```mira
let person = (name: "小明", age: 20, city: "北京");

// 点号访问
debug_print("姓名:", person.name);
debug_print("年龄:", person.age);

// 方括号访问（支持动态键名）
debug_print("城市:", person["city"]);

// 访问不存在的属性返回 nil
debug_print("电话:", person.phone);  // nil
```

对于未命名键，使用序数访问：

```mira
let point = (3, 4);
debug_print("x:", point.0);
debug_print("y:", point[1]);
```

## 展开与更新

使用 `..` 展开一个记录到另一个记录中，后面的键会覆盖前面的：

```mira
let base = (name: "小明", age: 20, city: "北京");

// 添加新字段
let with_email = (..base, email: "xiaoming@example.com");
debug_print("添加字段:", with_email);

// 更新字段（后面的覆盖前面的）
let older = (..base, age: 21);
debug_print("原记录:", base);     // age 仍为 20
debug_print("更新后:", older);    // age 为 21
```

## 省略 `nil` 值键

使用 `?:` 可以在值为 `nil` 时自动省略该键：

```mira
let name = "小明";
let phone = nil;
let email = "test@example.com";

let contact = (name?: name, phone?: phone, email?: email);
debug_print(contact); // phone 字段不会出现
```

## 检查键是否存在

使用 `in` 运算符检查记录是否包含某个键：

```mira
let config = (host: "localhost", port: 8080);
debug_print("有 host:", "host" in config);     // true
debug_print("有 timeout:", "timeout" in config); // false
```

## 遍历记录

### 获取键、值、键值对

```mira
let person = (name: "小明", age: 20, city: "北京");

debug_print("所有键:", keys(person));
debug_print("所有值:", values(person));
debug_print("键值对:", entries(person));
```

`keys()` 返回键名数组，`values()` 返回值数组，`entries()` 返回 `(key, value)` 记录的数组。

## 嵌套记录

记录可以嵌套，用于表示更复杂的数据结构：

```mira
let student = (
  name: "小明",
  scores: (math: 92, physics: 88, english: 76),
  address: (city: "北京", district: "海淀区"),
);

debug_print("数学成绩:", student.scores.math);
debug_print("所在区:", student.address.district);

// 安全地访问深层数据
debug_print("不存在的属性:", student.contact.phone); // nil，不会报错
```

## 实际应用

```mira
// 二维向量运算
let v1 = (3, 4);
let v2 = (1, 2);

// 向量加法
let v_add = (v1.0 + v2.0, v1.1 + v2.1);
debug_print("$v1 + $v2 =", v_add);

// 向量模
let magnitude = sqrt(v1.0^2 + v1.1^2);
debug_print("|$v1| =", magnitude);

// 使用记录模拟简单的数据表
let products = [
  (name: "笔记本", price: 5800, stock: 15),
  (name: "鼠标", price: 129, stock: 50),
  (name: "键盘", price: 299, stock: 30),
];

// 计算总库存价值
let mut total = 0;
for p in products {
  total += p.price * p.stock;
}
debug_print("总库存价值: $total 元");
```

## 小结

- 使用 `(key: value)` 创建记录，也支持 JSON 语法 `{ "key": value }`
- `:variable` 简写省略键名，`?:` 自动省略 `nil` 值
- `.key` 或 `["key"]` 访问属性，不存在返回 `nil`
- `..` 展开记录，用于添加或覆盖字段
- 记录**不可变**，"修改"操作实际上是创建新记录
- `keys()`、`values()`、`entries()` 函数用于获取记录信息
