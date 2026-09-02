import { Ref } from 'vue';
export interface MenuPosition {
    top: number;
    left: number;
}
export interface ContainerRect {
    width: number;
    height: number;
}
export interface TriggerRect {
    top: number;
    left: number;
    bottom: number;
    right: number;
    width: number;
    height: number;
}
export type MenuPlacement = 'bottom' | 'top' | 'bottom-start' | 'bottom-end' | 'top-start' | 'top-end';
export interface MenuBoundsOptions {
    placement?: MenuPlacement;
    gap?: number;
    align?: 'left' | 'right';
}
export declare function computeMenuPosition(trigger: TriggerRect, menuWidth: number, menuHeight: number, container: ContainerRect, placement?: MenuPlacement, gap?: number, align?: 'left' | 'right'): MenuPosition;
export declare function useMenuBounds(menuRef: Ref<HTMLElement | undefined>): {
    positionStyle: Ref<Record<string, string>, Record<string, string>>;
    applyPosition: (trigger: TriggerRect, container: ContainerRect, options?: MenuBoundsOptions) => void;
};
