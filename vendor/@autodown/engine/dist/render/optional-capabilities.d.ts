import { HighlightFn } from './highlight';
import { RenderedArtifact } from './preview';
export type { HighlightFn } from './highlight';
export type NodeRendererFactory = () => unknown;
/** Host-injected artifact persistence (plan 031 D6): the engine never
 *  touches disk — a host (demo in-memory, VM disk cache + resvg later)
 *  registers a store and successful FINAL renders land in it keyed by
 *  artifactHash (single-source, TS/rust byte-identical). `get` is for
 *  tests and VM consumption demos; the web live render never reads it. */
export interface ArtifactStore {
    get(key: string): RenderedArtifact | undefined;
    put(key: string, artifact: RenderedArtifact): void;
}
/** Register (or clear) the katex renderer. Calling without a factory marks
 *  the capability enabled with the library default (when it grows in). */
export declare function enableKatex(factory?: NodeRendererFactory): void;
/** Register (or clear) the mermaid renderer. */
export declare function enableMermaid(factory?: NodeRendererFactory): void;
/** Register (or clear) the syntax highlighter. The optional argument is the
 *  platform implementation (see highlight.ts for the contract): a VM backend
 *  supplies its own bridge, the Vue layer calls enableHighlight() with no
 *  argument and the lowlight default is resolved at the call site. */
export declare function enableHighlight(impl?: HighlightFn): void;
export declare function isCapabilityEnabled(name: string): boolean;
/** Register the host artifact store. Registering twice replaces (the
 *  latest host wins — demo/jade remount scenarios). */
export declare function enableArtifactStore(store: ArtifactStore): void;
/** The registered store, or null (the render paths' no-op guard). Exported
 *  for preview.ts's put choke point; hosts read through their own handle. */
export declare function getArtifactStore(): ArtifactStore | null;
/** All capabilities absent -> the renderer still works (degraded path). */
export declare function clearOptionalCapabilities(): void;
