import { Ref } from 'vue';
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
export declare function buildSegments(text: string): StreamingSegment[];
export declare function useStreamingDocument(rawText: Ref<string>): {
    segments: import('vue').ComputedRef<StreamingSegment[]>;
};
