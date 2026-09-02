export type TaskMarker = 'TODO' | 'DOING' | 'DONE' | 'NOW' | 'LATER';
export type TaskWorkflow = 'todo' | 'now';
export declare const TASK_MARKERS: TaskMarker[];
export declare const TASK_MARKER_RE: RegExp;
export declare const PRIORITY_RE: RegExp;
export interface ScheduledInfo {
    keyword: 'SCHEDULED' | 'DEADLINE';
    date: Date;
    rawDate: string;
    repeater?: string;
}
export declare function parseScheduled(line: string): ScheduledInfo | null;
export declare function cycleTaskMarker(lines: string[], lineIdx: number, workflow?: TaskWorkflow): string[];
export declare function setPriority(line: string, priority: 'A' | 'B' | 'C'): string;
export declare function removePriority(line: string): string;
