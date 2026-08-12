import { useEffect, useRef } from 'react';
import type { CSSProperties, JSX, RefObject } from 'react';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import type { VmAny } from '@mirascript/mirascript';
import Layout from '@theme/Layout';
import Highlight from '../components/Mira/index';
import MonacoHighlight from '../components/Mira/highlight';
import styles from './index.module.css';

const heroCode = `let future = ideas
  ::map(build)
  ::run();`;

const features = [
    {
        token: 'if',
        eyebrow: 'Everything returns',
        title: '表达式优先',
        desc: 'if、match、loop 都有返回值。让控制流直接参与计算，少写中间变量，多表达真实意图。',
        example: 'let level = if score > 90 { "A" } else { "B" };',
    },
    {
        token: 'let',
        eyebrow: 'Safe by default',
        title: '不可变数据',
        desc: '数据默认不可变，需要改变时用 mut 明确标记。状态变化在代码里清晰可见。',
        example: 'let point = (x: 12, y: 24);',
    },
    {
        token: '??',
        eyebrow: 'Nil, handled',
        title: '空安全',
        desc: '安全访问、空合并与非空断言协同工作，缺失值不再打断整个执行流程。',
        example: 'user.profile.name ?? "访客"',
    },
    {
        token: '::',
        eyebrow: 'Data flows',
        title: '扩展调用',
        desc: '用 :: 把数据与函数连成自然的处理管线，复杂转换仍然能从左到右阅读。',
        example: 'items::map(parse)::filter(valid)',
    },
    {
        token: '=>',
        eyebrow: 'One syntax, many shapes',
        title: '模式匹配',
        desc: '字面量、范围、解构与守卫集中在一个 match 中，覆盖复杂分支而不牺牲可读性。',
        example: 'case 90..100 { "excellent" }',
    },
    {
        token: '$()',
        eyebrow: 'Strings with context',
        title: '字符串插值',
        desc: '直接在字符串中引用名称或表达式，日志、模板和用户提示都更接近最终内容。',
        example: '"Hello, $name — $(items::len()) items"',
    },
];

const codeExamples = [
    {
        title: '表达式 & 模式匹配',
        code: `// 控制流也是值
let status = if score > 60 { "pass" } else { "fail" };

let message = match score {
  case 90..100 { "优秀" }
  case 60..<90 { "及格" }
  case _       { "继续加油" }
};

(status, message)`,
        context: { score: 85 },
    },
    {
        title: '函数 & 扩展调用',
        code: `fn double { it * 2 }

[5, 2, 8, 1, 9, 3, 7, 4, 6]
  ::map(double)
  ::filter(fn { it > 3 })
  ::sort()`,
        context: {},
    },
    {
        title: '字符串插值',
        code: `let name = "MiraScript";
let items = [1, 2, 3];

debug_print("Hello, $name!");
debug_print("count: $(items::len())");`,
        context: {},
    },
    {
        title: '空安全',
        code: `let user = (name: "Alice");

// 安全访问不存在的路径
debug_print(user.address.city); // nil

let city = user.address.city ?? "未知";
debug_print(user.name!, city);`,
        context: {},
    },
];

const runtimeSteps = [
    {
        index: '01',
        title: '写一次',
        desc: '用紧凑、可组合的语法描述规则与数据处理逻辑。',
    },
    {
        index: '02',
        title: '嵌进去',
        desc: '通过 Node.js、Python 或 WebAssembly 把 MiraScript 带进你的产品。',
    },
    {
        index: '03',
        title: '到处运行',
        desc: '同一套语言语义覆盖浏览器与服务端，交付一致的执行体验。',
    },
];

/** 驱动主页的滚动进度、视差与进入视口动画。 */
function useHomeMotion(): RefObject<HTMLDivElement | null> {
    const pageRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const page = pageRef.current;
        if (!page) return;

        const revealItems = Array.from(page.querySelectorAll<HTMLElement>('[data-reveal]'));
        const reduceMotion = globalThis.matchMedia('(prefers-reduced-motion: reduce)').matches;
        page.classList.add(styles['is-motion-ready']!);

        if (reduceMotion) {
            for (const item of revealItems) item.classList.add(styles['is-visible']!);
            page.style.setProperty('--scroll-progress', '1');
            return;
        }

        const observer = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (!entry.isIntersecting) continue;
                    entry.target.classList.add(styles['is-visible']!);
                    observer.unobserve(entry.target);
                }
            },
            { rootMargin: '0px 0px -12% 0px', threshold: 0.12 },
        );
        for (const item of revealItems) observer.observe(item);

        let animationFrame = 0;
        const updateScrollEffects = () => {
            animationFrame = 0;
            const pageTop = page.offsetTop;
            const distance = Math.max(page.scrollHeight - globalThis.innerHeight, 1);
            const progress = Math.min(Math.max((globalThis.scrollY - pageTop) / distance, 0), 1);
            const heroProgress = Math.min(Math.max((globalThis.scrollY - pageTop) / globalThis.innerHeight, 0), 1);
            page.style.setProperty('--scroll-progress', progress.toFixed(4));
            page.style.setProperty('--hero-shift', `${(heroProgress * 72).toFixed(2)}px`);
        };
        const handleScroll = () => {
            if (!animationFrame) animationFrame = globalThis.requestAnimationFrame(updateScrollEffects);
        };

        updateScrollEffects();
        globalThis.addEventListener('scroll', handleScroll, { passive: true });
        globalThis.addEventListener('resize', handleScroll);

        return () => {
            observer.disconnect();
            globalThis.removeEventListener('scroll', handleScroll);
            globalThis.removeEventListener('resize', handleScroll);
            if (animationFrame) globalThis.cancelAnimationFrame(animationFrame);
        };
    }, []);

    return pageRef;
}

/** 主页复用的主要行动按钮。 */
function CtaButtons(): JSX.Element {
    return (
        <div className={styles['cta-buttons']}>
            <Link className={styles['btn-primary']} to="/playground/">
                在线运行
                <span aria-hidden="true">↗</span>
            </Link>
            <Link className={styles['btn-secondary']} to="/tutorial/introduction/">
                开始学习
                <span aria-hidden="true">→</span>
            </Link>
        </div>
    );
}

/** 可直接执行的 MiraScript 代码卡片。 */
function CodeSnippet({ title, code, context, delay }: { title: string; code: string; context: Record<string, VmAny>; delay: number }): JSX.Element {
    return (
        <article className={`${styles['code-example']} ${styles['reveal']}`} data-reveal style={{ '--reveal-delay': `${delay}ms` } as CSSProperties}>
            <div className={styles['code-header']}>
                <span className={styles['window-dots']} aria-hidden="true">
                    <i />
                    <i />
                    <i />
                </span>
                <span>{title}</span>
                <span className={styles['code-status']}>LIVE</span>
            </div>
            <Highlight value={code} mode="Script" context={context} autoRun />
        </article>
    );
}

/** 首屏的纯 CSS 语法运行时装置。 */
function SyntaxCore(): JSX.Element {
    return (
        <div className={styles['syntax-stage']} aria-hidden="true">
            <div className={styles['orbit-ring']} />
            <div className={`${styles['orbit-chip']} ${styles['orbit-chip-if']}`}>if</div>
            <div className={`${styles['orbit-chip']} ${styles['orbit-chip-match']}`}>match</div>
            <div className={`${styles['orbit-chip']} ${styles['orbit-chip-pipe']}`}>::</div>
            <div className={styles['syntax-core']}>
                <span className={styles['core-glow']} />
                <img className={styles['core-logo']} src="/favicon.svg" alt="" />
                <span className={styles['core-label']}>runtime ready</span>
            </div>
            <div className={styles['floating-code']}>
                <div className={styles['floating-code-top']}>
                    <span>hello.mira</span>
                    <span>●</span>
                </div>
                <code className={styles['floating-code-content']}>
                    <MonacoHighlight code={heroCode} />
                </code>
            </div>
        </div>
    );
}

/** MiraScript 产品主页。 */
export default function Home(): JSX.Element {
    const { siteConfig } = useDocusaurusContext();
    const pageRef = useHomeMotion();

    return (
        <Layout title={siteConfig.title} description={siteConfig.tagline}>
            <div ref={pageRef} className={styles['home']}>
                <div className={styles['scroll-progress']} aria-hidden="true" />

                <header className={styles['hero']}>
                    <div className={styles['hero-grid']} aria-hidden="true" />
                    <div className={styles['hero-inner']}>
                        <div className={styles['hero-copy']}>
                            <p className={styles['eyebrow']}>
                                <span />
                                Expression-first scripting language
                            </p>
                            <h1 className={styles['hero-title']}>
                                让嵌入式逻辑
                                <span>写得像思考一样自然</span>
                            </h1>
                            <p className={styles['hero-tagline']}>
                                MiraScript 是一门表达式优先、默认不可变的现代脚本语言。
                                <br className={styles['desktop-break']} /> 为产品规则而生，为多端运行而设计。
                            </p>
                            <CtaButtons />
                            <div className={styles['runtime-badges']} aria-label="支持的运行环境">
                                <span>Rust core</span>
                                <span>WebAssembly</span>
                                <span>Node.js</span>
                                <span>Python</span>
                            </div>
                        </div>
                        <SyntaxCore />
                    </div>
                    <div className={styles['scroll-cue']} aria-hidden="true">
                        <span>SCROLL TO EXPLORE</span>
                        <i />
                    </div>
                </header>

                <main>
                    <section className={styles['features']}>
                        <div className={styles['section-inner']}>
                            <div className={`${styles['section-heading']} ${styles['reveal']}`} data-reveal>
                                <p className={styles['section-kicker']}>LANGUAGE / 01</p>
                                <h2>复杂能力，简单表达</h2>
                                <p>把安全、组合与表达力放进语言本身，而不是留给使用者反复补救。</p>
                            </div>
                            <div className={styles['feature-grid']}>
                                {features.map((feature, index) => (
                                    <article
                                        key={feature.title}
                                        className={`${styles['feature-card']} ${styles['reveal']}`}
                                        data-reveal
                                        style={{ '--reveal-delay': `${(index % 3) * 80}ms` } as CSSProperties}
                                    >
                                        <div className={styles['feature-meta']}>
                                            <span className={styles['feature-token']}>{feature.token}</span>
                                            <span>{feature.eyebrow}</span>
                                        </div>
                                        <h3>{feature.title}</h3>
                                        <p>{feature.desc}</p>
                                        <code className={styles['feature-code']}>
                                            <MonacoHighlight code={feature.example} />
                                        </code>
                                    </article>
                                ))}
                            </div>
                        </div>
                    </section>

                    <section className={styles['flow-section']}>
                        <div className={styles['flow-aura']} aria-hidden="true" />
                        <div className={styles['flow-inner']}>
                            <div className={`${styles['flow-intro']} ${styles['reveal']}`} data-reveal>
                                <p className={styles['section-kicker']}>RUNTIME / 02</p>
                                <h2>从一段逻辑，到每个运行环境</h2>
                                <p>MiraScript 把语言与宿主解耦。业务逻辑保持纯粹，接入方式交给轻量运行时。</p>
                                <Link to="/tutorial/introduction/" className={styles['text-link']}>
                                    了解语言设计 <span aria-hidden="true">→</span>
                                </Link>
                            </div>
                            <ol className={styles['runtime-steps']}>
                                {runtimeSteps.map((step, index) => (
                                    <li
                                        key={step.index}
                                        className={`${styles['runtime-step']} ${styles['reveal']}`}
                                        data-reveal
                                        style={{ '--reveal-delay': `${index * 100}ms` } as CSSProperties}
                                    >
                                        <span>{step.index}</span>
                                        <div>
                                            <h3>{step.title}</h3>
                                            <p>{step.desc}</p>
                                        </div>
                                    </li>
                                ))}
                            </ol>
                        </div>
                    </section>

                    <section className={styles['code-showcase']}>
                        <div className={styles['section-inner']}>
                            <div className={`${styles['section-heading']} ${styles['reveal']}`} data-reveal>
                                <p className={styles['section-kicker']}>PLAYGROUND / 03</p>
                                <h2>别只看语法，直接运行</h2>
                                <p>下面不是截图。修改输入上下文，立即查看 MiraScript 的真实执行结果。</p>
                            </div>
                            <div className={styles['code-examples']}>
                                {codeExamples.map((example, index) => (
                                    <CodeSnippet key={example.title} {...example} delay={(index % 2) * 100} />
                                ))}
                            </div>
                        </div>
                    </section>

                    <section className={styles['closing']}>
                        <div className={`${styles['closing-card']} ${styles['reveal']}`} data-reveal>
                            <span className={styles['closing-signal']} aria-hidden="true">
                                READY
                            </span>
                            <p className={styles['section-kicker']}>YOUR TURN</p>
                            <h2>
                                把下一段想法，
                                <br />
                                写成可运行的表达式。
                            </h2>
                            <p>从交互式教程开始，几分钟内掌握 MiraScript 的核心语法。</p>
                            <CtaButtons />
                        </div>
                    </section>
                </main>
            </div>
        </Layout>
    );
}
