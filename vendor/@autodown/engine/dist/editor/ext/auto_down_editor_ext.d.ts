import { VNode } from 'vue';
import { SlashItem } from '../menus/slashItem';
import { getBlockMap } from '../block-map';
export { default as SlashMenu } from '../menus/SlashMenu.vue';
export { default as BubbleMenu } from '../menus/BubbleMenu.vue';
export { default as CodeBlockMenu } from '../menus/CodeBlockMenu.vue';
export declare function editorCheckIcon(): unknown;
export declare function editorXIcon(): unknown;
export declare function normalizeAnchors(md: string): string;
export declare function appendTableIAL(md: string, _editor: unknown): string;
export declare function blockMapOf(editor: any): ReturnType<typeof getBlockMap>;
declare const EngineContentHost: import('vue').DefineComponent<import('vue').ExtractPropTypes<{
    editor: {
        type: ObjectConstructor;
        default: null;
    };
}>, () => VNode<import('vue').RendererNode, import('vue').RendererElement, {
    [key: string]: any;
}>, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<import('vue').ExtractPropTypes<{
    editor: {
        type: ObjectConstructor;
        default: null;
    };
}>> & Readonly<{}>, {
    editor: Record<string, any>;
}, {}, {}, {}, string, import('vue').ComponentProvideOptions, true, {}, any>;
export { EngineContentHost as EditorContent };
export declare function useAutoDownEditorBridge(): {
    items: SlashItem[];
    editor: any;
};
