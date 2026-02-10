import type { JSX } from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Highlight from '../components/Mira/index';
import styles from './index.module.css';
import type { VmAny } from '@mirascript/mirascript';

/* ---------- Feature 数据 ---------- */
const features = [
    {
        icon: '🧮',
        title: '表达式优先',
        desc: '几乎所有语法结构都是表达式，都有返回值。if、match、loop 均可直接赋值，编写更简洁自然。',
    },
    {
        icon: '🔒',
        title: '不可变数据',
        desc: '默认数据不可变，消除意外副作用。需要可变时使用 mut 显式声明，安全且清晰。',
    },
    {
        icon: '🛡️',
        title: '空安全',
        desc: '访问不存在的属性返回 nil 而非报错，配合 ?? 空合并和 ! 断言，轻松处理缺失值。',
    },
    {
        icon: '🔗',
        title: '扩展调用',
        desc: '使用 :: 语法链式组合函数，value::fn(arg) 等价于 fn(value, arg)，数据流一目了然。',
    },
    {
        icon: '🎯',
        title: '模式匹配',
        desc: '强大的 match 表达式支持字面量、范围、解构、守卫等多种模式，替代冗长的 if-else 链。',
    },
    {
        icon: '📐',
        title: '字符串插值',
        desc: '"hello, $name" 和 "$(expr)" 语法让字符串拼接直观优雅，还支持模板模式。',
    },
];

/* ---------- 代码示例 ---------- */
const codeExamples = [
    {
        title: '表达式 & 模式匹配',
        code: `// 表达式优先：if/match 都有返回值
let status = if score > 60 { "pass" } else { "fail" };
// 模式匹配
let msg = match score {
  case 90..100 { "优秀" }
  case 60..<90 { "及格" }
  case _       { "不及格" }
};

(status, msg)`,
        context: {
            score: 85,
        },
    },
    {
        title: '函数 & 扩展调用',
        code: `// 函数声明 & 单参数简写
fn double { it * 2 }
fn add(x, y) { x + y }

// 扩展调用链
let result = [1, 2, 3]
  ::map(double)
  ::filter(fn { it > 3 });
  
result`,
        context: {},
    },
    {
        title: '字符串插值',
        code: `// 字符串插值
let name = "MiraScript";
debug_print("Hello, $name!");

// 表达式插值
let items = [1, 2, 3];
debug_print("count: $(items::len())");`,
        context: {},
    },
    {
        title: '空安全',
        code: `// 空安全访问
let user = (name: "Alice");
debug_print(user.address.city);  // => nil

// 空合并与默认值
let city = user.address.city ?? "未知";
debug_print(user.name!);         // 断言非空`,
        context: {},
    },
];

/** 代码片段卡片 */
function CodeSnippet({ title, code, context }: { title: string; code: string; context: Record<string, VmAny> }): JSX.Element {
    return (
        <div className={styles['codeExample']}>
            <div className={styles['codeHeader']}>
                <span className={styles['codeDot']} />
                {title}
            </div>
            <Highlight value={code} mode="Script" context={context} readOnly />
        </div>
    );
}

/* ---------- 主页组件 ---------- */

/**
 * Home
 */
export default function Home(): JSX.Element {
    const { siteConfig } = useDocusaurusContext();
    return (
        <Layout title={siteConfig.title} description={siteConfig.tagline}>
            {/* Hero */}
            <header className={styles['hero']}>
                <div className={styles['heroContent']}>
                    <h1 className={styles['heroTitle']}>MiraScript</h1>
                    <p className={styles['heroTagline']}>
                        表达式优先、不可变数据为核心的现代脚本语言。
                        <br />
                        简洁、安全、易于嵌入。
                    </p>
                    <div className={styles['heroButtons']}>
                        <Link className={styles['btnPrimary']} to="/tutorial/introduction/">
                            快速开始 →
                        </Link>
                        <Link className={styles['btnSecondary']} to="/cheatsheet/">
                            速查表
                        </Link>
                    </div>
                </div>
            </header>

            <main>
                {/* Features */}
                <section className={styles['features']}>
                    <div className={styles['featuresInner']}>
                        <h2 className={styles['sectionTitle']}>语言特性</h2>
                        <p className={styles['sectionSubtitle']}>专为嵌入式场景打造的表达式语言</p>
                        <div className={styles['featureGrid']}>
                            {features.map((f) => (
                                <div key={f.title} className={styles['featureCard']}>
                                    <span className={styles['featureIcon']}>{f.icon}</span>
                                    <h3 className={styles['featureTitle']}>{f.title}</h3>
                                    <p className={styles['featureDesc']}>{f.desc}</p>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                {/* Code Showcase */}
                <section className={styles['codeShowcase']}>
                    <div className={styles['codeShowcaseInner']}>
                        <h2 className={styles['sectionTitle']}>代码示例</h2>
                        <p className={styles['sectionSubtitle']}>直观感受 MiraScript 的表达力</p>
                        <div className={styles['codeExamples']}>
                            {codeExamples.map((ex) => (
                                <CodeSnippet key={ex.title} {...ex} />
                            ))}
                        </div>
                    </div>
                </section>

                {/* Why MiraScript */}
                <section className={styles['comparison']}>
                    <div className={styles['comparisonInner']}>
                        <h2 className={styles['sectionTitle']}>为什么选择 MiraScript</h2>
                        <p className={styles['sectionSubtitle']}>与传统脚本语言相比的优势</p>
                        <ul className={styles['comparisonList']}>
                            <li className={styles['comparisonItem']}>
                                <span className={styles['comparisonIcon']}>⚡</span>
                                <div className={styles['comparisonText']}>
                                    <strong>高性能</strong>
                                    <p>Rust 编写的解释器内核，支持编译到 WebAssembly，在浏览器和服务端均可高效运行。</p>
                                </div>
                            </li>
                            <li className={styles['comparisonItem']}>
                                <span className={styles['comparisonIcon']}>🧩</span>
                                <div className={styles['comparisonText']}>
                                    <strong>易于嵌入</strong>
                                    <p>提供 Node.js、Python、WASM 等多平台绑定，轻松集成到你的应用中。</p>
                                </div>
                            </li>
                            <li className={styles['comparisonItem']}>
                                <span className={styles['comparisonIcon']}>🎨</span>
                                <div className={styles['comparisonText']}>
                                    <strong>开发者体验</strong>
                                    <p>内建 VS Code 扩展、Monaco 编辑器集成，提供语法高亮和智能提示。</p>
                                </div>
                            </li>
                            <li className={styles['comparisonItem']}>
                                <span className={styles['comparisonIcon']}>📖</span>
                                <div className={styles['comparisonText']}>
                                    <strong>完善文档</strong>
                                    <p>交互式教程、可运行的代码示例、完整的语言参考，快速上手不迷路。</p>
                                </div>
                            </li>
                        </ul>
                    </div>
                </section>

                {/* CTA */}
                <section className={styles['cta']}>
                    <h2 className={styles['ctaTitle']}>准备好开始了吗？</h2>
                    <p className={styles['ctaDesc']}>跟随交互式教程，几分钟内上手 MiraScript。</p>
                    <div className={styles['ctaButtons']}>
                        <Link className={styles['btnPrimary']} to="/tutorial/introduction/">
                            阅读教程 →
                        </Link>
                        <Link className={styles['btnSecondary']} to="/cheatsheet/">
                            查看速查表
                        </Link>
                    </div>
                </section>
            </main>
        </Layout>
    );
}
