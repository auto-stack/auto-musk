declare const BUBBLE_ICONS: {
    readonly bold: import('vue').FunctionalComponent<import('lucide-vue-next').LucideProps, {}, any, {}>;
    readonly italic: import('vue').FunctionalComponent<import('lucide-vue-next').LucideProps, {}, any, {}>;
    readonly underline: import('vue').FunctionalComponent<import('lucide-vue-next').LucideProps, {}, any, {}>;
    readonly strike: import('vue').FunctionalComponent<import('lucide-vue-next').LucideProps, {}, any, {}>;
    readonly code: import('vue').FunctionalComponent<import('lucide-vue-next').LucideProps, {}, any, {}>;
    readonly link: import('vue').FunctionalComponent<import('lucide-vue-next').LucideProps, {}, any, {}>;
};
export declare function bubbleIcon(name: keyof typeof BUBBLE_ICONS): unknown;
export declare function bubbleShouldShow({ editor, state, }: {
    editor: any;
    state: {
        selection: {
            empty: boolean;
        };
    };
}): boolean;
export declare function runBubbleLink(editor: any, prompt: string | null | undefined): void;
declare const EngineBubbleMenu: import('vue').DefineComponent<import('vue').ExtractPropTypes<{
    editor: {
        type: ObjectConstructor;
        default: null;
    };
    options: {
        type: ObjectConstructor;
        default: null;
    };
    shouldShow: {
        type: FunctionConstructor;
        default: null;
    };
}>, () => import('vue').VNode<import('vue').RendererNode, import('vue').RendererElement, {
    [key: string]: any;
}> | null, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<import('vue').ExtractPropTypes<{
    editor: {
        type: ObjectConstructor;
        default: null;
    };
    options: {
        type: ObjectConstructor;
        default: null;
    };
    shouldShow: {
        type: FunctionConstructor;
        default: null;
    };
}>> & Readonly<{}>, {
    editor: Record<string, any>;
    options: Record<string, any>;
    shouldShow: Function;
}, {}, {}, {}, string, import('vue').ComponentProvideOptions, true, {}, any>;
export { EngineBubbleMenu as TiptapBubbleMenu };
