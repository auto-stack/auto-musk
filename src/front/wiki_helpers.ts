// wiki_helpers.ts — WikiNav 树过滤逻辑逃生舱（Plan 023 队列 B6）
//
// 对标 src/front/components/WikiNav.vue（逃生舱，已删除）的 filteredWikiTree/
// filteredRawTree。.at 无法表达 filter+toLowerCase+includes，放逃生舱 fn。

/** 按 query 过滤树节点（name 大小写不敏感 contains；空 query 返回全量）。 */
export function wikiFilterTree(nodes: any[], query: string): any[] {
  if (!query) return nodes || []
  const q = query.toLowerCase()
  return (nodes || []).filter((n) => n.name.toLowerCase().includes(q))
}
