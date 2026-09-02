export { computeMenuPosition } from '../composables/useMenuBounds';
export declare function codeBlockCheckIcon(): unknown;
export interface CodeBlockLanguage {
    id: string;
    label: string;
    aliases: string[];
}
export declare const CODE_BLOCK_LANGUAGES: CodeBlockLanguage[];
export declare function codeBlockLanguages(): CodeBlockLanguage[];
