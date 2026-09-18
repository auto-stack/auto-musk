import { Component, VNode } from 'vue';
import { BlockNode } from '../parser/block-model';
import { WNode } from '../parser/markdown-parser';
import { BlockEditCtx } from './block-component';
import { PanelRenderCtx, PanelRenderer, PanelBodyDecorator } from './panel-registry';
export type BlockWidgetMode = 'view' | 'stream' | 'edit';
export interface BlockWidgetProps {
    mode: BlockWidgetMode;
    /** the block's model — payload shape is the same in every mode */
    node: BlockNode;
    /** stream consumption: false while the segment is still open */
    final?: boolean;
    /** edit consumption (engine / blockId / readonly) */
    ctx?: BlockEditCtx;
}
/** Register one widget as a kind's whole family: the three BlockComponent
 *  slots become thin wrappers that mount the widget with the right mode.
 *  A family registration owns all three slots (it replaces earlier
 *  per-slot registrations for the kind). Family widgets declare the four
 *  family props (mode/node/final/ctx); the non-edit wrappers pass ctx: null
 *  so the generated required-prop checks stay quiet. */
export declare function registerBlockWidget(kind: string, widget: Component): void;
/** Drop a kind's family registration (builtin fallback resumes). */
export declare function unregisterBlockWidget(kind: string): void;
/** Wrap a family widget as a PanelRenderer — the panel face of view mode.
 *  The registry WNode resolves back to its model BlockNode when the editor
 *  bridge produced it; parse-side WNodes (static render, no back-link) get a
 *  fabricated model from the WNode slots (code/language -> inlines/attrs),
 *  the same shape EngineEditor's node-view fallback built (plan 030). */
export declare function panelOf(widget: Component): PanelRenderer;
/** The panel model of a container WNode: the editor bridge's back-link when
 *  present, the fabricated static-render model otherwise. */
export declare function containerPanelModel(w: WNode): BlockNode;
/** Wrap a CONTAINER family widget as a PanelRenderer — panelOf's container
 *  sibling: the children hole gets the renderEmbedded closure (view mode,
 *  final forwarded verbatim; verbed faces like the details marker get their
 *  engine from the editor-side registration's live host window). The body
 *  closure applies the captured panel body decorator (plan 035 T6): the
 *  outer editor decoration pass cannot descend into component props, so
 *  wikilink decoration rides the closure instead. */
export declare function panelOfContainer(widget: Component): PanelRenderer;
/** The list panel adapter: WNode items flattened to the widget's chrome
 *  data ({id, task, checked, cls, children_slot}) — renderListPanel's
 *  reads, item for item (the retired builtin's shape, byte-for-byte; the
 *  item bodies carry the captured panel decorator — see panelOfContainer). */
export declare function listItemsOfPanel(ctx: PanelRenderCtx): unknown[];
/** plan 019: the WNode carries the table header as a 0-or-1 array. */
export declare function tableHeaderCellsOfPanel(ctx: PanelRenderCtx): unknown[];
export declare function tableRowsOfPanel(ctx: PanelRenderCtx): unknown[];
/** Wrap a body closure so its vnodes get the construction-time decorator
 *  applied before they mount (shared by the render-side panel adapters and
 *  EngineEditor's editor-side Details registration). renderEmbedded returns
 *  a SINGLE vnode (the markdown-renderer div) — normalized to the array
 *  decorateWikilinks mutates in place; the rendered DOM is identical. */
export declare function decorateBody(dec: PanelBodyDecorator | null, body: () => VNode | VNode[]): () => VNode[];
