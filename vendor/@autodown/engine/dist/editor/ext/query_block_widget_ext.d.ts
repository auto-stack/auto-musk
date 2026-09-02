import { BlockNode } from '../../parser/block-model';
import { RunQueryFn } from '../engine/data-loaders';
/** The family node prop's query text (attrs.query — '' when absent). */
export declare function queryText(node: BlockNode | undefined): string;
/** The registered query runner (null = the placeholder state — the
 *  EngineEditor runQuery prop never arrived). */
export declare function queryRunner(): RunQueryFn | null;
export declare function normalizeQueryResults(res: any): any[];
export declare function errorMessage(e: unknown): string;
