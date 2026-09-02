export interface SafeParseResult {
    ok: boolean;
    value: any;
}
export declare function safeJsonParse(s: string): SafeParseResult;
export declare function typeOf(v: any): string;
export declare function isTruthy(v: any): boolean;
