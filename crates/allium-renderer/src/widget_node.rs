//! WidgetDocument 与 WidgetNode 合约定义（schema v2）。
//!
//! 相较 v1（每个变体内嵌 `id`、`position`/`visible` 挂在 `ChildEntry`）：
//! - `WidgetNode` 提层为具名 struct，`id`/`position`/`visible` 上移到外层。
//! - 变体数据收进 `NodeKind`，`#[serde(tag = "type")]` 标签化。
//! - `Container.children` 直接为 `Vec<WidgetNode>`，`ChildEntry` 删除。
//!
//! JSON 形状示例：
//! ```json
//! {
//!   "id": "root",
//!   "position": { "x": 0, "y": 0, "rotation": 0, "scale": [1, 1] },
//!   "visible": true,
//!   "kind": { "type": "container", "layout": "absolute", "children": [ ... ] }
//! }
//! ```
//! 消费端一次性迁移到 v2，serde 不再兼容读 v1（历史 PG/前端 draft 由离线迁移程序转换）。

use serde::{Deserialize, Serialize};

use crate::widgets::image::AssetImageFit;
use crate::widgets::theme::Color;

/// 当前 schema 版本。历史 v1 数据须离线迁移到 v2；serde 不做兼容读。
pub const WIDGET_DOCUMENT_SCHEMA_VERSION: u32 = 2;

/// Widget 文档根。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetDocument {
    /// Schema 版本号（v2 起 = [`WIDGET_DOCUMENT_SCHEMA_VERSION`]）。
    pub version: u32,
    /// 画布规格。
    pub canvas: CanvasSpec,
    /// 根节点。
    pub root: WidgetNode,
    /// 输出格式。
    pub output: OutputFormat,
}

/// 画布规格。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSpec {
    /// 画布宽度。
    pub width: u32,
    /// 画布高度。
    pub height: u32,
    /// 画布背景色。
    pub background: Color,
}

/// 输出格式定义。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// JPEG 输出，参数为质量。
    Jpeg(u8),
    /// PNG 输出。
    Png,
    /// WebP 输出，参数为质量。
    Webp(u8),
}

/// Widget 场景树节点（v2）。
///
/// `id`/`position`/`visible` 是所有节点共有的定位/可见性字段，
/// 具体形状与参数放到 `kind`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetNode {
    /// 节点稳定 ID。
    pub id: String,
    /// 节点在父容器坐标系下的定位。缺省视为 [`Position::default`]（左上原点、无旋转/缩放）。
    #[serde(default)]
    pub position: Position,
    /// 是否可见。缺省 `true`；`false` 跳过渲染。
    #[serde(default = "default_visible")]
    pub visible: bool,
    /// 节点具体形状与参数。
    pub kind: NodeKind,
}

impl WidgetNode {
    /// 返回节点类型名。
    pub fn type_name(&self) -> &'static str {
        self.kind.type_name()
    }
}

/// 节点具体形状与参数（v2）。
///
/// `#[serde(tag = "type")]` 用外部标签化：JSON 形如
/// `{ "type": "container", ... }`，与 v1 的枚举 tag 命名一致以便迁移工具复用。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    /// 容器节点。
    Container {
        /// 容器布局。
        layout: Layout,
        /// 子节点列表。
        children: Vec<WidgetNode>,
    },
    /// 卡面缩略图节点。
    CardThumbnail {
        /// 缩略图尺寸。
        size: f32,
        /// 卡面素材 key。
        card_image_key: String,
        /// 稀有度类型。
        rarity: String,
        /// 属性类型。
        attr: String,
        /// 突破等级。
        master_rank: i32,
        /// 是否为特训后。
        trained: bool,
        /// 是否显示信息层。
        show_info: bool,
        /// 等级文本。
        level_text: String,
    },
    /// 玻璃面板节点。
    GlassPanel {
        /// 宽度。
        width: f32,
        /// 高度。
        height: f32,
        /// 裁切扰动强度。
        clip_variance: f32,
    },
    /// 任意颜色圆角面板节点。
    Panel {
        /// 宽度。
        width: f32,
        /// 高度。
        height: f32,
        /// 圆角半径。
        radius: f32,
        /// 填充色。
        fill: Color,
        /// 边框色。
        border: Option<Color>,
        /// 边框宽度。
        border_width: f32,
    },
    /// 素材图片节点。
    AssetImage {
        /// 素材 key。
        asset_key: String,
        /// 宽度。
        width: f32,
        /// 高度。
        height: f32,
        /// 填充方式。
        fit: AssetImageFit,
        /// 圆角半径。
        radius: f32,
    },
    /// text-in-box 文本节点。
    ///
    /// 显式 `width × height` 盒子 + `padding` + `(align, v_align)` + `line_height`
    /// 定位文字，与前端 WidgetPreview 像素对齐。
    SimpleText {
        /// 文本内容。
        content: String,
        /// 字号（px）。
        font_size: f32,
        /// 文本颜色。
        color: Color,
        /// 盒子宽度（px）。
        #[serde(default = "default_simple_text_width")]
        width: f32,
        /// 盒子高度（px）。
        #[serde(default = "default_simple_text_height")]
        height: f32,
        /// 水平对齐（沿用旧字段名，避免破坏 JSON schema）。
        #[serde(default)]
        align: TextAlignValue,
        /// 垂直对齐。
        #[serde(default)]
        v_align: VAlignValue,
        /// 内边距（px，统一四边）。
        #[serde(default = "default_simple_text_padding")]
        padding: f32,
        /// 行高倍数（对应 CSS line-height）。
        #[serde(default = "default_simple_text_line_height")]
        line_height: f32,
        /// 是否启用霓虹发光。
        glow: bool,
    },
    /// 指标标签节点。
    StatsBadge {
        /// 指标名称。
        label: String,
        /// 指标值。
        value: String,
        /// 指标颜色。
        color: Color,
        /// 是否高亮。
        is_highlight: bool,
    },
    /// 圆角文本标签节点。
    TextBadge {
        /// 标签文本。
        text: String,
        /// 背景色。
        bg_color: Color,
        /// 文本颜色。
        text_color: Color,
    },
    /// 平台个人资料的游戏 general 面板。
    ProfileGeneral {
        /// 游戏自定义名片 general 类型。
        general_type: i32,
    },
}

impl NodeKind {
    /// 返回节点类型名。
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Container { .. } => "container",
            Self::CardThumbnail { .. } => "card_thumbnail",
            Self::GlassPanel { .. } => "glass_panel",
            Self::Panel { .. } => "panel",
            Self::AssetImage { .. } => "asset_image",
            Self::SimpleText { .. } => "simple_text",
            Self::StatsBadge { .. } => "stats_badge",
            Self::TextBadge { .. } => "text_badge",
            Self::ProfileGeneral { .. } => "profile_general",
        }
    }
}

/// 节点定位信息。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// 局部 X 坐标。
    pub x: f32,
    /// 局部 Y 坐标。
    pub y: f32,
    /// 旋转角度。
    pub rotation: f32,
    /// 缩放因子。
    pub scale: (f32, f32),
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale: (1.0, 1.0),
        }
    }
}

impl Position {
    /// 是否为默认值——H/V 流式布局中 position 应保持默认，否则报 `PositionIgnored`。
    pub fn is_default(&self) -> bool {
        self == &Position::default()
    }
}

/// 容器布局模式。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// 绝对定位布局。
    Absolute,
    /// 水平流式布局。
    Horizontal {
        /// 子节点间距。
        gap: f32,
    },
    /// 垂直流式布局。
    Vertical {
        /// 子节点间距。
        gap: f32,
    },
}

/// SimpleText 盒内水平对齐（沿用 SimpleText.align 字段名）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextAlignValue {
    /// 靠左。
    Left,
    /// 水平居中。
    Center,
    /// 靠右。
    Right,
}

impl Default for TextAlignValue {
    fn default() -> Self {
        Self::Left
    }
}

/// SimpleText 盒内垂直对齐。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VAlignValue {
    /// 顶部对齐。
    Top,
    /// 垂直居中。
    Middle,
    /// 底部对齐。
    Bottom,
}

impl Default for VAlignValue {
    fn default() -> Self {
        Self::Top
    }
}

/// SimpleText 默认值常量——前后端共享，避免硬编码不一致。
pub const SIMPLE_TEXT_DEFAULT_WIDTH: f32 = 260.0;
pub const SIMPLE_TEXT_DEFAULT_HEIGHT: f32 = 72.0;
pub const SIMPLE_TEXT_DEFAULT_PADDING: f32 = 4.0;
pub const SIMPLE_TEXT_DEFAULT_LINE_HEIGHT: f32 = 1.2;

fn default_visible() -> bool {
    true
}
fn default_simple_text_width() -> f32 {
    SIMPLE_TEXT_DEFAULT_WIDTH
}
fn default_simple_text_height() -> f32 {
    SIMPLE_TEXT_DEFAULT_HEIGHT
}
fn default_simple_text_padding() -> f32 {
    SIMPLE_TEXT_DEFAULT_PADDING
}
fn default_simple_text_line_height() -> f32 {
    SIMPLE_TEXT_DEFAULT_LINE_HEIGHT
}

#[cfg(test)]
mod tests {
    use super::{
        CanvasSpec, Layout, NodeKind, OutputFormat, Position, TextAlignValue, VAlignValue,
        WidgetDocument, WidgetNode, SIMPLE_TEXT_DEFAULT_HEIGHT, SIMPLE_TEXT_DEFAULT_LINE_HEIGHT,
        SIMPLE_TEXT_DEFAULT_PADDING, SIMPLE_TEXT_DEFAULT_WIDTH, WIDGET_DOCUMENT_SCHEMA_VERSION,
    };
    use crate::widgets::theme::Color;

    fn sample_color() -> Color {
        Color::new(1.0, 0.5, 0.0, 1.0)
    }

    fn node(id: &str, kind: NodeKind) -> WidgetNode {
        WidgetNode {
            id: id.to_string(),
            position: Position::default(),
            visible: true,
            kind,
        }
    }

    #[test]
    fn widget_node_round_trip_container() {
        let n = WidgetNode {
            id: "root".to_string(),
            position: Position::default(),
            visible: true,
            kind: NodeKind::Container {
                layout: Layout::Horizontal { gap: 12.0 },
                children: vec![node(
                    "glass",
                    NodeKind::GlassPanel {
                        width: 120.0,
                        height: 64.0,
                        clip_variance: 0.2,
                    },
                )],
            },
        };

        let json = serde_json::to_string(&n).expect("序列化 container 失败");
        let decoded: WidgetNode = serde_json::from_str(&json).expect("反序列化 container 失败");
        assert_eq!(decoded, n);
    }

    #[test]
    fn widget_node_round_trip_card_thumbnail() {
        let n = node(
            "card",
            NodeKind::CardThumbnail {
                size: 156.0,
                card_image_key: "cards/1".to_string(),
                rarity: "rarity_4".to_string(),
                attr: "cool".to_string(),
                master_rank: 3,
                trained: true,
                show_info: true,
                level_text: "Lv.60".to_string(),
            },
        );

        let json = serde_json::to_string(&n).expect("序列化 card_thumbnail 失败");
        let decoded: WidgetNode =
            serde_json::from_str(&json).expect("反序列化 card_thumbnail 失败");
        assert_eq!(decoded, n);
    }

    #[test]
    fn widget_node_round_trip_simple_text() {
        let n = node(
            "text",
            NodeKind::SimpleText {
                content: "hello".to_string(),
                font_size: 18.0,
                color: sample_color(),
                width: 200.0,
                height: 60.0,
                align: TextAlignValue::Center,
                v_align: VAlignValue::Middle,
                padding: 4.0,
                line_height: 1.2,
                glow: true,
            },
        );

        let json = serde_json::to_string(&n).expect("序列化 simple_text 失败");
        let decoded: WidgetNode = serde_json::from_str(&json).expect("反序列化 simple_text 失败");
        assert_eq!(decoded, n);
    }

    #[test]
    fn widget_node_position_and_visible_default_when_absent() {
        // v2 允许省略 position/visible；反序列化时取默认值。
        let json = r#"{
            "id": "text",
            "kind": {
                "type": "simple_text",
                "content": "legacy",
                "font_size": 16.0,
                "color": { "r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0 },
                "align": "left",
                "glow": false
            }
        }"#;
        let decoded: WidgetNode = serde_json::from_str(json).expect("反序列化 v2 缺省字段失败");
        assert_eq!(decoded.position, Position::default());
        assert!(decoded.visible);
        match decoded.kind {
            NodeKind::SimpleText {
                width,
                height,
                v_align,
                padding,
                line_height,
                ..
            } => {
                assert_eq!(width, SIMPLE_TEXT_DEFAULT_WIDTH);
                assert_eq!(height, SIMPLE_TEXT_DEFAULT_HEIGHT);
                assert_eq!(v_align, VAlignValue::Top);
                assert_eq!(padding, SIMPLE_TEXT_DEFAULT_PADDING);
                assert_eq!(line_height, SIMPLE_TEXT_DEFAULT_LINE_HEIGHT);
            }
            _ => panic!("不应为非 SimpleText 节点"),
        }
    }

    #[test]
    fn widget_node_round_trip_panel() {
        let n = node(
            "panel",
            NodeKind::Panel {
                width: 240.0,
                height: 120.0,
                radius: 8.0,
                fill: sample_color(),
                border: Some(Color::new(0.0, 0.0, 0.0, 1.0)),
                border_width: 1.5,
            },
        );

        let json = serde_json::to_string(&n).expect("序列化 panel 失败");
        let decoded: WidgetNode = serde_json::from_str(&json).expect("反序列化 panel 失败");
        assert_eq!(decoded, n);
    }

    #[test]
    fn widget_node_round_trip_asset_image() {
        let n = node(
            "img",
            NodeKind::AssetImage {
                asset_key: "presets/mysekai/item_1.png".to_string(),
                width: 128.0,
                height: 64.0,
                fit: crate::widgets::image::AssetImageFit::Contain,
                radius: 4.0,
            },
        );

        let json = serde_json::to_string(&n).expect("序列化 asset_image 失败");
        let decoded: WidgetNode = serde_json::from_str(&json).expect("反序列化 asset_image 失败");
        assert_eq!(decoded, n);
    }

    #[test]
    fn widget_node_round_trip_stats_badge() {
        let n = node(
            "stats",
            NodeKind::StatsBadge {
                label: "Score".to_string(),
                value: "123".to_string(),
                color: sample_color(),
                is_highlight: true,
            },
        );

        let json = serde_json::to_string(&n).expect("序列化 stats_badge 失败");
        let decoded: WidgetNode = serde_json::from_str(&json).expect("反序列化 stats_badge 失败");
        assert_eq!(decoded, n);
    }

    #[test]
    fn widget_node_round_trip_text_badge() {
        let n = node(
            "badge",
            NodeKind::TextBadge {
                text: "EVENT".to_string(),
                bg_color: sample_color(),
                text_color: Color::new(1.0, 1.0, 1.0, 1.0),
            },
        );

        let json = serde_json::to_string(&n).expect("序列化 text_badge 失败");
        let decoded: WidgetNode = serde_json::from_str(&json).expect("反序列化 text_badge 失败");
        assert_eq!(decoded, n);
    }

    #[test]
    fn widget_node_round_trip_profile_general() {
        let n = node(
            "profile_name",
            NodeKind::ProfileGeneral { general_type: 13 },
        );

        let json = serde_json::to_string(&n).expect("序列化 profile_general 失败");
        let decoded: WidgetNode =
            serde_json::from_str(&json).expect("反序列化 profile_general 失败");
        assert_eq!(decoded, n);
    }

    #[test]
    fn widget_document_round_trip_with_nested_container() {
        let doc = WidgetDocument {
            version: WIDGET_DOCUMENT_SCHEMA_VERSION,
            canvas: CanvasSpec {
                width: 1080,
                height: 1920,
                background: Color::new(0.1, 0.1, 0.1, 1.0),
            },
            root: WidgetNode {
                id: "root".to_string(),
                position: Position::default(),
                visible: true,
                kind: NodeKind::Container {
                    layout: Layout::Absolute,
                    children: vec![WidgetNode {
                        id: "inner".to_string(),
                        position: Position {
                            x: 32.0,
                            y: 48.0,
                            rotation: 0.0,
                            scale: (1.0, 1.0),
                        },
                        visible: true,
                        kind: NodeKind::Container {
                            layout: Layout::Vertical { gap: 8.0 },
                            children: vec![node(
                                "panel",
                                NodeKind::GlassPanel {
                                    width: 200.0,
                                    height: 80.0,
                                    clip_variance: 0.0,
                                },
                            )],
                        },
                    }],
                },
            },
            output: OutputFormat::Png,
        };

        let json = serde_json::to_string(&doc).expect("序列化 document 失败");
        let decoded: WidgetDocument = serde_json::from_str(&json).expect("反序列化 document 失败");
        assert_eq!(decoded, doc);
    }
}
