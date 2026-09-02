/**
 * @autodown/vue — streaming document segmentation.
 *
 * GENERATED FILE — do not edit by hand.
 * Source: auto/streaming.at (Auto language). Regenerate with: pnpm gen
 * (see auto/README.md for the pipeline and the applied post-fixes)
 */
export interface MarkdownSegment {
    type: 'markdown';
    text: string;
}
export interface ComponentSegment {
    type: 'component';
    componentType: string;
    props: Record<string, any>;
    final: boolean;
}
export type StreamingSegment = MarkdownSegment | ComponentSegment;
export interface JSONBlock {
    start: number;
    end: number;
    content: string;
    closed: boolean;
}
export declare function parsePartialJSON(text: string): any;
export declare function findJSONBlocks(text: string): JSONBlock[];
export declare function isComponentJSON(value: any): boolean;
export declare function detectComponentType(raw: string): string | null;
export declare function buildSegments(text: string): StreamingSegment[];
