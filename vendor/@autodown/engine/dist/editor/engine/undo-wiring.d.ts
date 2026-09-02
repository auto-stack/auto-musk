import { BlockHostController } from './host-controller';
import { EditorEngine } from './editor-engine';
export type HistoryAction = 'undo' | 'redo' | null;
/** The history half of a keydown: Ctrl/Cmd+Z (Shift variant → redo) and
 *  Ctrl/Cmd+Y → redo. Everything else (including the mark shortcuts and
 *  Ctrl+End navigation) returns null and keeps bubbling untouched. */
export declare function historyActionOf(e: Pick<KeyboardEvent, 'ctrlKey' | 'metaKey' | 'shiftKey' | 'key'>): HistoryAction;
/** Run one history hop and realign EVERY cached host's knownText with the
 *  restored tree — undo can revert any block, and a stale knownText would
 *  baseline the host's next diffToOp against ghost text. Returns false
 *  (touching nothing) when the stack is empty. */
export declare function runHistory(engine: EditorEngine, hosts: Iterable<BlockHostController>, action: Exclude<HistoryAction, null>): boolean;
