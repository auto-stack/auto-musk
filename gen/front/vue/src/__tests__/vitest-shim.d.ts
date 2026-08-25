// vitest-shim.d.ts — vitest 类型环境声明(PLAN-041 债务收口⑥附)。
//
// gen/package.json 为 auto build 再生产物(devDep 不可持存,vitest 每次被
// 抹除);测试运行经 `npx -y vitest@2.1.9 run`(版本锁定,不装 devDep)。
// 本 shim 仅满足 vue-tsc 的模块解析——注意:若将来把 vitest 装回 devDep,
// 需删除本文件(真类型与 ambient 声明会冲突)。
declare module 'vitest' {
  export const describe: any
  export const it: any
  export const expect: any
  export const beforeEach: any
  export const vi: any
}
