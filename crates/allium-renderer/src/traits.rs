//! 渲染输出类型。

use crate::render_document::RenderTiming;

/// 渲染输出：编码后的图片字节
pub struct RenderOutput {
    /// 编码后的图片字节（JPEG/WebP）
    pub data: Vec<u8>,
    /// MIME 类型（如 "image/jpeg"）
    pub content_type: String,
    /// 图片宽度
    pub width: u32,
    /// 图片高度
    pub height: u32,
    /// 文档渲染分段耗时；旧图元路径暂不填充。
    pub timing: Option<RenderTiming>,
}
