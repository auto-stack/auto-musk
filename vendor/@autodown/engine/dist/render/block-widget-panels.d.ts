import { VNode } from 'vue';
/** The ```json table segment's stream face (plan 042 T3): the retired
 *  StreamingRenderer-local StreamingTableFace, hoisted to a shared export
 *  so both registration layers use ONE implementation — StreamingRenderer
 *  registers it for pure-render consumers (its module scope is the old
 *  face's home, and the only consumer of stream slots), EngineEditor's
 *  table family registration reuses it (superseding with the same closure).
 *  NOTE: this module must not CALL registerBlockComponent at top level —
 *  the render-node side-effect cycle would hit block-component's TDZ. */
export declare function tableStreamFace(node: any, final: boolean): VNode;
