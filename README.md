# allium-renderer

基于 Skia 的 CPU 图片渲染层，将结构化数据渲染为 JPEG/WebP 字节。

## 渲染路径

包含两条渲染路径：

- **图元组合**：场景实现 `Renderable::compose()` 返回 `SceneTree`（纯数据，不触碰 Skia），再驱动 Skia Canvas 绘制并编码。`compose` 阶段无图形依赖，可单独测试。
- **场景图解释**：从 JSON 反序列化为 `WidgetDocument` / `WidgetNode`（serde 标签化枚举），按 layer 展开排序后逐元素绘制。

## 推荐用法

渲染入口是 `render_document::render_document()`——一个**纯同步**函数，输入 `WidgetDocument` + `RenderContext`，输出编码后的图片字节。它本身不做任何调度，由调用方决定在哪个线程、以何种并发策略执行。这是刻意的：渲染层只负责"怎么画"，并发与调度属于上层职责。

如何把同步渲染接入异步服务，本 crate 不做规定，但需要自己解决以下几点：

- **线程隔离**：CPU 密集的光栅化不能跑在 Tokio 的异步 I/O 线程上，否则阻塞整个运行时。最简单的做法是丢进一个独立的 rayon 线程池，再用 oneshot channel 桥接回 async——`executor::RenderExecutor` 就是这个最基础形态的示例。
- **但线程池隔离只是最基础的一层。** 生产环境通常还需要：请求优先级（交互式请求插队批量预热）、队列上限与背压（拒绝而非堆积）、单飞/去重、超时与取消传播、worker panic 重生、以及指标埋点。这些都不在渲染层内——上层调度器（在本 crate 之外）才是承载它们的地方，`RenderExecutor` 并不覆盖这些。

## 两级线程模型

渲染并发分两层，职责不同：

- **渲染线程（外层，串行）**：单个渲染任务从头到尾在一个线程上跑。一次只画一张图，方便控制内存峰值与调度公平性。这一层由调用方提供（如上文的上层调度器），不在本 crate 内。
- **光栅化线程（内层，并行）**：`sdf` 模块把文字轮廓光栅化成像素位图时，需要对窗口内每个设备像素做反变换 + 距离场采样 + 超采样累加。这是 CPU 软件光栅化（区别于 GPU 硬件管线），纯 CPU 算术。各像素行相互独立、只读共享状态，因此按行切分到一个**专用 rayon 线程池**并行执行（见 `sdf::rasterize`，池内线程命名 `raster-*`）。

该专用池**不复用 rayon 全局池**，线程数由环境变量 `SCAPUS_RASTER_THREADS` 控制（默认 2）。因为外层保证任意时刻至多一个渲染在跑，所以这个内层池进程内全局共享即可。设为 1 时并行迭代自动退化为串行，无需单独代码路径。

> 当前生产配置为 **1 个渲染线程 + 2 个光栅化线程**，按物理核数选取。逐像素数学与串行版逐字节一致，输出不随线程数变化。

## 模块地图

| 模块 | 职责 |
| --- | --- |
| `traits` | `Renderable` trait 与 `RenderOutput` 输出定义 |
| `primitives` | 图元（`SceneTree`）定义 |
| `render_document` | **推荐入口**：`render_document()` 同步渲染函数 |
| `executor` | 最基础的线程池隔离示例（已弃用，仅供参考） |
| `widget_node` | 前后端文档合约（`WidgetDocument` / `WidgetNode`） |
| `widgets` | 控件实现（面板、文本、徽章、缩略图等）与主题 |
| `elements` | 名片各区块的组合逻辑 |
| `text` | 字体解析、文本测量、富文本解析 |
| `sdf` | SDF 文字轮廓的逐像素并行光栅化（专用线程池，需 `skia` feature） |
| `assets` | 素材内存缓存（LRU） |
| `masterdata` | 渲染所需的游戏数据解析接口 |
| `init` | 启动初始化（字体安装等） |

## Features

| Feature | 说明 |
| --- | --- |
| `skia` | 启用 Skia 后端，提供实际光栅化能力 |
| `dev` | 在 `skia` 基础上额外启用 `tracing-subscriber`，供 `tools/` 下的诊断 bin 使用 |

不启用任何 feature 时，`compose` 等纯数据路径可独立编译与测试。

## 构建

启用 `skia` feature 会编译 `skia-safe`，并依赖系统 freetype（通过 `freetype-rs` / `pkg-config`）。这套原生依赖在部分平台上需要额外配置，建议在带有 freetype、pkg-config 的 Linux 环境（或容器）中构建。

```bash
cargo build -p allium-renderer --features skia
```

## 许可证

[AGPL-3.0-only](./LICENSE)。Copyright (C) allium / emptysekai。
