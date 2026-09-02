export interface RenderedKatex {
    html: string;
    error: string;
}
export declare function renderKatexPreview(source: string, displayMode: boolean): RenderedKatex;
export interface RenderedMermaid {
    svg: string;
    error: string;
}
export declare function renderMermaidPreview(source: string): Promise<RenderedMermaid>;
export type ArtifactKind = 'html' | 'svg';
export interface RenderedArtifact {
    /** the body's display form — drives the VM-side displayer */
    kind: ArtifactKind;
    /** the rendered body ("" when error != "") */
    body: string;
    /** "" on success; the render error message otherwise */
    error: string;
}
export type ArtifactBlockKind = 'MathBlock' | 'Mermaid';
/** The single put choke point (plan 031 D6/T8): a SUCCESSFUL final render
 *  lands in the host-injected store under the single-source artifactHash
 *  key. No store registered -> no-op (pre-031 behavior, byte for byte).
 *  Repeated puts of the same (kind, source) rewrite the same key — the
 *  "exactly once" semantics come from final-renders-only + idempotent
 *  keys, not from call counting. Exported for the node-view bridge's
 *  synchronous katex path (the bridge family in src/editor/ext/). */
export declare function recordArtifact(blockKind: ArtifactBlockKind, source: string, artifact: RenderedArtifact): void;
/** Produce the persistable artifact for a final render: math -> katex HTML
 *  (display mode, same face as the node view), mermaid -> SVG. Errors are
 *  data, not exceptions (the preview-bridge idiom). A successful result is
 *  recorded into the artifact store when one is registered. */
export declare function artifactFor(blockKind: ArtifactBlockKind, source: string): Promise<RenderedArtifact>;
