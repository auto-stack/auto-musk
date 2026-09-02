import { Component } from 'vue';
/** Slash item contract (plan 018: tiptap-free — `editor` is the engine
 *  chain adapter, `range` the Suggestion-compatible char range). */
export interface SlashItem {
    title: string;
    description: string;
    icon: Component;
    searchTerms: string[];
    command: (ctx: {
        editor: any;
        range: {
            from: number;
            to: number;
        };
    }) => void;
}
