export { focusCodeArea, nodeText, ctxReadonly, ctxBlockId, codeController, editOnlyAttr, viewMarker } from './code_block_widget_ext';
export { textareaRows } from './math_block_widget_ext';
export declare function renderMermaidPreview(source: string): Promise<{
    svg: string;
    error: string;
}>;
export interface MermaidRenderState {
    svg: string;
    error: string;
    loading: boolean;
}
type RenderCallback = (state: MermaidRenderState) => void;
export declare function scheduleMermaidRender(source: string, cb: RenderCallback): void;
