import { htmlText } from './code_block_widget_ext';
export { htmlText };
export declare function commitTableCell(controller: any, e: any): void;
/** The dyn root's tag: the view face IS the table (tablePanel's root), the
 *  other two faces are divs. */
export declare function rootTag(mode: string): string;
/** The dyn root's single class chain per face (see header note 2). */
export declare function rootClass(mode: string, final: boolean, readonly: boolean): string;
/** The view root table's aria-busy (tablePanel pinned "false"); absent on
 *  the edit/stream roots. */
export declare function rootAriaBusy(mode: string): string | undefined;
/** The edit root's data-block-id; absent on the view/stream roots. */
export declare function rootBlockId(mode: string, blockId: string): string | undefined;
/** The edit root's data-node-type; absent on the view/stream roots. */
export declare function rootNodeType(mode: string): string | undefined;
/** The stream face's normalized header list, pre-shaped for the template
 *  loop: {col, html} pairs — html is the escaped header text because the
 *  DSL's `text` emits a <span>{{}}</span> wrapper while the SFC template
 *  pinned bare <th>col</th> children. */
export declare function streamHeader(columns: unknown): Array<{
    col: string;
    html: string;
}>;
/** The stream face's normalized body: rows × columns of {col, html} cells —
 *  `row[col] ?? ''` per cell (the SFC's missing-key fallback), keys carried
 *  for the template loops (col per cell, index per row — the SFC's keys). */
export declare function streamBody(columns: unknown, rows: unknown): Array<Array<{
    col: string;
    html: string;
}>>;
/** The loading row's colspan: Math.max(1, columns.length) — never a 0 span. */
export declare function streamColspan(columns: unknown): number;
