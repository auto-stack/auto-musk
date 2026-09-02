import { Ref } from 'vue';
import { StreamingSegment } from './streaming.generated';
export type { MarkdownSegment, ComponentSegment, StreamingSegment, } from './streaming.generated';
export declare function useStreamingDocument(rawText: Ref<string>): {
    segments: import('vue').ComputedRef<StreamingSegment[]>;
};
