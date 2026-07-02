//! 渲染组卡推荐结果（deck_result golden 回归 oracle）。
//!
//! 用法（dev 容器内，一律 `--offline`）:
//!   render-deck <fixture.json> <output.png>
//!
//! - 输入：`DeckResultCard` JSON（见 `tests/golden/deck_result/fixtures/*.json`）
//! - 走 v2 IR：`DeckResultCard::to_widget_document()` → `render_document()`
//! - 空 `AssetStore` + 无 masterdata：卡面缩略图画占位，与 `render-widget-doc`
//!   同 ctx；对回归 oracle 足够（重渲染 baseline 与自身逐字节相同）
//! - 输出格式：JSON fixture 的 `output_quality` 是 JPEG 质量，本 bin **改用 PNG**
//!   （golden 对拍要求逐字节确定；skia JPEG 有 encoder 抖动风险）
//! - 与 `render-widget-doc`/`render-profile` 一起构成 W3 三链回归 oracle：
//!     game_card 链 = 游戏卡自由变换承重墙（elements/draw_element_on_canvas）
//!     widget_doc 链 = IR 通用节点（render_document + widgets/*）
//!     deck_result 链 = IR 拼装范本（60 处 WidgetNode 构造 + baseline 换算）

use std::path::PathBuf;

use allium_renderer::assets::AssetStore;
use allium_renderer::context::RenderContext;
use allium_renderer::deck_result::DeckResultCard;
use allium_renderer::render_document::render_document;
use allium_renderer::widget_node::OutputFormat;
use allium_renderer::widgets::theme::Theme;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(input) = args.first().map(PathBuf::from) else {
        eprintln!("用法: render-deck <fixture.json> <output.png>");
        std::process::exit(2);
    };
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("deck_result.png"));

    let raw = std::fs::read_to_string(&input).unwrap_or_else(|err| {
        eprintln!("读取 fixture 失败 {}: {err}", input.display());
        std::process::exit(1);
    });
    let mut card: DeckResultCard = serde_json::from_str(&raw).unwrap_or_else(|err| {
        eprintln!("解析 DeckResultCard 失败: {err}");
        std::process::exit(1);
    });

    // golden 对拍必须逐字节确定 → 覆盖为 PNG 输出，忽略 fixture 里的 JPEG 质量。
    let mut doc = card.to_widget_document();
    doc.output = OutputFormat::Png;
    card.output_quality = None;

    let assets = AssetStore::new(64);
    let theme = Theme::default();
    let ctx = RenderContext::new(&assets, &theme);

    let result = render_document(&doc, &ctx);

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).unwrap_or_else(|err| {
                eprintln!("创建输出目录失败 {}: {err}", parent.display());
                std::process::exit(1);
            });
        }
    }
    std::fs::write(&output, &result.image).unwrap_or_else(|err| {
        eprintln!("写入 PNG 失败 {}: {err}", output.display());
        std::process::exit(1);
    });
    println!(
        "完成: {} ({}x{}, {} bytes)",
        output.display(),
        result.width,
        result.height,
        result.image.len()
    );
}
