import { VNode } from 'vue';
/** Injection key for the NodeViewContent hole's body (plan 026 P1T2): the
 *  mounting bridge provides the block's embedded VNodes; the widget templates
 *  use NodeViewContent bare (no own children), so the hole renders what the
 *  assembly injected. Nearest provider wins — nested node-views resolve to
 *  their own wrapper. */
export declare const NODE_VIEW_CONTENT_KEY = "autodown-node-view-content";
export declare const NodeViewWrapper: import('vue').DefineComponent<import('vue').ExtractPropTypes<{
    as: {
        type: (StringConstructor | ObjectConstructor)[];
        default: string;
    };
}>, () => VNode<import('vue').RendererNode, import('vue').RendererElement, {
    [key: string]: any;
}>, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<import('vue').ExtractPropTypes<{
    as: {
        type: (StringConstructor | ObjectConstructor)[];
        default: string;
    };
}>> & Readonly<{}>, {
    as: string | Record<string, any>;
}, {}, {}, {}, string, import('vue').ComponentProvideOptions, true, {}, any>;
export declare const NodeViewContent: import('vue').DefineComponent<import('vue').ExtractPropTypes<{
    as: {
        type: (StringConstructor | ObjectConstructor)[];
        default: string;
    };
}>, () => VNode<import('vue').RendererNode, import('vue').RendererElement, {
    [key: string]: any;
}>, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<import('vue').ExtractPropTypes<{
    as: {
        type: (StringConstructor | ObjectConstructor)[];
        default: string;
    };
}>> & Readonly<{}>, {
    as: string | Record<string, any>;
}, {}, {}, {}, string, import('vue').ComponentProvideOptions, true, {}, any>;
