import { BlockNode } from '../parser/block-model';
import { WNode } from '../parser/markdown-parser';
/** The model block a converted WNode came from (undefined for parse-side
 *  WNodes — static render, no writeback). */
export declare function blockOfWNode(w: WNode): BlockNode | undefined;
export declare function blockNodesToWNodes(nodes: BlockNode[]): WNode[];
export declare function blockNodeToWNode(node: BlockNode): WNode;
