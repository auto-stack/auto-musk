import { renderKatexPreview } from '../../render/preview';
export { renderKatexPreview };
export { focusCodeArea, nodeText, ctxReadonly, ctxBlockId, codeController, editOnlyAttr, viewMarker } from './code_block_widget_ext';
export declare function renderMathBlockPreview(source: string): {
    html: string;
    error: string;
};
/** rows attr for the source textarea: draft line count + 1 breathing line,
 *  clamped to [4, 24] — past the cap CSS takes over (max-height +
 *  overflow). */
export declare function textareaRows(source: string): string;
