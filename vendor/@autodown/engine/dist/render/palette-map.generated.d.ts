/**
 * @autodown/engine — palette map (render layer).
 *
 * GENERATED FILE — do not edit by hand.
 * Source: auto/palette_map.at (Auto language). Regenerate with: pnpm gen:render
 * (see auto/README.md for the pipeline and the applied post-fixes)
 */
export interface PanelSpec {
    kind: string;
    tag: string;
    class_token: string;
    registry: string;
    extension: boolean;
}
export declare function panelHeading(level: number): PanelSpec;
export declare function panelOfBlock(blockType: string): PanelSpec;
export declare function isExtensionPanel(kind: string): boolean;
export declare function builtinPanelKinds(): string[];
export declare function extensionPanelKinds(): string[];
