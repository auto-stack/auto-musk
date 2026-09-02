import { VNode } from 'vue';
import { PanelSpec } from './palette-map.generated';
export interface RevealBudget {
    /** characters of inline text still revealable (typewriter); Infinity = all */
    remaining: number;
}
export interface PanelRenderCtx {
    node: any;
    final: boolean | undefined;
    budget: RevealBudget | undefined;
    spec: PanelSpec;
    /** nested block content (li body, quote body, table cells) */
    renderEmbedded(children: any[], final: boolean | undefined, budget?: RevealBudget): VNode;
    /** inline children of the panel's text content */
    renderInlineChildren(children: any[] | undefined, final: boolean | undefined, budget?: RevealBudget): VNode[];
}
export type PanelRenderer = (ctx: PanelRenderCtx) => VNode;
/** Register a panel renderer. Overrides the builtin for the same kind
 *  (the extension slots have no builtin to override). */
export declare function registerPanel(kind: string, renderer: PanelRenderer): void;
/** Remove a custom registration, falling back to the builtin renderer. */
export declare function unregisterPanel(kind: string): void;
/** Test/teardown helper: drop every custom registration at once. */
export declare function clearPanelRegistry(): void;
/** Panel spec for a parsed block node. Headings resolve through their
 *  level (H1..H6); everything else maps by block type. */
export declare function specForNode(node: any): PanelSpec;
export declare function resolvePanelRenderer(spec: PanelSpec): PanelRenderer | undefined;
export type { PanelSpec };
export type PanelBodyDecorator = (vnodes: VNode[]) => void;
/** Run fn with the panel body decorator active (window semantics — same
 *  shape as the editor's node-view host window). */
export declare function withPanelDecorator<T>(dec: PanelBodyDecorator, fn: () => T): T;
/** The decorator active at closure-construction time (null outside a
 *  window — static render stays undecorated). */
export declare function currentPanelDecorator(): PanelBodyDecorator | null;
