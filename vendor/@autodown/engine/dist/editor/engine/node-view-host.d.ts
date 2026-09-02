import { BlockNode } from '../../parser/block-model';
import { EditorEngine } from './editor-engine';
import { EditorAdapter } from './tiptap-adapter';
export interface NodeViewHostScope {
    engine: EditorEngine;
    adapter?: EditorAdapter;
}
/** Open a synchronous render window for `host`'s engine. */
export declare function pushNodeViewHost(host: NodeViewHostScope): void;
/** Close the innermost render window. */
export declare function popNodeViewHost(): void;
/** The editor currently rendering (undefined outside a render window). */
export declare function currentNodeViewHost(): NodeViewHostScope | undefined;
export interface NodeViewProps {
    node: {
        attrs: Record<string, unknown>;
        textContent: string;
        id: string;
    };
    updateAttributes: (patch: Record<string, unknown>) => void;
    deleteNode: () => void;
    getPos: () => number;
    selected: boolean;
    editor: EditorAdapter | null;
    extension: {
        options: Record<string, unknown>;
    };
    decorations: unknown[];
}
/** Wrap a NodeView widget mount with the NodeViewContent body source: the
 *  widget templates render the hole bare, the provider injects the block's
 *  embedded VNodes (plan 026 P1T2). */
export declare const NodeViewContentProvider: import('vue').DefineComponent<import('vue').ExtractPropTypes<{
    content: {
        type: FunctionConstructor;
        required: true;
    };
}>, () => import('vue').VNode<import('vue').RendererNode, import('vue').RendererElement, {
    [key: string]: any;
}>[] | undefined, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<import('vue').ExtractPropTypes<{
    content: {
        type: FunctionConstructor;
        required: true;
    };
}>> & Readonly<{}>, {}, {}, {}, {}, string, import('vue').ComponentProvideOptions, true, {}, any>;
/** Mount a NodeView widget with fabricated props, its NodeViewContent hole
 *  fed by `content` (embedded body VNodes). */
export declare function mountNodeView(view: unknown, props: NodeViewProps, content: () => unknown[]): import('vue').VNode<import('vue').RendererNode, import('vue').RendererElement, {
    [key: string]: any;
}>;
/** Fabricate the tiptap-shaped widget props for a model block. `engine`
 *  optional: without it the widget renders but never writes back. */
export declare function nodeViewProps(node: BlockNode, engine?: EditorEngine, selected?: boolean, adapter?: EditorAdapter | null): NodeViewProps;
