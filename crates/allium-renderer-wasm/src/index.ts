/**
 * @empty-sekai/renderer-wasm — 浏览器名片渲染。
 *
 * 入口导出核心封装与类型。Worker 调度见子路径 `@empty-sekai/renderer-wasm/worker`。
 *
 * 最小用法（主线程，注意 skia 光栅化同步阻塞）：
 * ```ts
 * import createAlliumRenderer from "@empty-sekai/renderer-wasm/allium_renderer_wasm.js";
 * import { AlliumRenderer, ImageFormat } from "@empty-sekai/renderer-wasm";
 *
 * const r = await AlliumRenderer.create(createAlliumRenderer);
 * r.registerFont("FZLanTingHei-DB-GBK", await fetchBytes("/fonts/lanting.ttf"));
 * for (const [name, json] of tables) r.loadMasterData(name, json);
 * r.init();
 * for (const key of r.collectAssetKeys(cardJson)) {
 *   r.putAsset(key, await fetchBytes(assetUrl(key)));
 * }
 * const jpeg = r.render(cardJson, ImageFormat.Jpeg);
 * ```
 */

export { AlliumRenderer, ImageFormat, AlliumRenderError } from "./renderer.js";
export type { CroppedLayerOutput, LayerCrop } from "./renderer.js";
export type { EmscriptenModule, EmscriptenModuleFactory } from "./emscripten.js";
