/**
 * @autodown/engine — rendered-artifact cache key (render layer).
 *
 * GENERATED FILE — do not edit by hand.
 * Source: auto/artifact_hash.at (Auto language). Regenerate with: pnpm gen:render
 * (see auto/README.md for the pipeline and the applied post-fixes)
 */
export declare function fnvOffsetBasis(): number;
export declare function fnvPrime(): number;
export declare function u32Modulus(): number;
export declare function bitAt(x: number, p: number): number;
export declare function xor32(a: number, b: number): number;
export declare function mulMod32(a: number, b: number): number;
export declare function fnvStep(h: number, u: number): number;
export declare function hexDigit(d: number): string;
export declare function hex32(v: number): string;
export declare function artifactKeyOf(kind: string, source_len: number, units: number[]): string;
