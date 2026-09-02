/**
 * AutoDown Core — Shared types and IAL (Inline Attribute List) utilities.
 *
 * GENERATED FILE — do not edit by hand.
 * Source: auto/ial.at (Auto language). Regenerate with: pnpm gen
 * (see auto/README.md for the pipeline and the applied post-fixes)
 */
export declare class TableAttr {
    cols: (number | null)[];
    rows: (number | null)[];
    constructor(cols: (number | null)[], rows: (number | null)[]);
}
export declare class PreDoc {
    md: string;
    tableAttrs: TableAttr[];
    constructor(md: string, tableAttrs: TableAttr[]);
}
export declare function startsWithStr(s: string, prefix: string): boolean;
export declare function startsWithAt(s: string, prefix: string, at: number): boolean;
export declare function endsWithStr(s: string, suffix: string): boolean;
export declare function trimStartStr(s: string): string;
export declare function trimEndStr(s: string): string;
export declare function hasChar(s: string, code: number): boolean;
export declare function findStr(s: string, needle: string): number;
export declare function findStrFrom(s: string, needle: string, from: number): number;
export declare function rfindChar(s: string, code: number): number;
export declare function scanIntPrefix(s: string): number | null;
export declare function stripQuotes(s: string): string;
export declare function parseValue(s: string): number | null;
export declare function parseArray(s: string): (number | null)[];
export declare function parseRows(s: string | null): (number | null)[];
export declare function formatValue(v: number | null): string;
export declare function formatArray(arr: (number | null)[]): string;
export declare function hasAnyValue(arr: (number | null)[]): boolean;
export declare function isPipeRow(line: string): boolean;
export declare function isDelimRow(line: string): boolean;
export declare function parseIalLine(line: string): TableAttr | null;
export declare function preprocessMarkdown(md: string): PreDoc;
export declare function buildIAL(colwidth: (number | null)[], rowheight: (number | null)[]): string | null;
