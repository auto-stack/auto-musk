<!--
  DeleteConfirmDialog.vue — 删除确认域 alert-dialog 消费适配器（PLAN-058 三期）
  //
  // VM 轨真源 = ports/delete_confirm.vm.at（内联确认行降级——alert-dialog
  // schema backends iced:none,VM 轨不渲染,实机快照实证整树丢弃）。
  // 本适配器只做标准件组合（ui/alert-dialog,shadcn-vue/reka-ui,脚手架源自
  // auto-man assets）:受控 open + target 分流文案;reka-ui 关闭路径（ESC/
  // 遮罩/Cancel 钮）统一折算为 cancel 事件,父侧状态不残留。i18n 文案全部
  // 由父侧 props 注入（与 VM 轨同契约,本件不持 t()）。
-->
<script setup lang="ts">
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"

// props 键用蛇形——.at codegen 按源侧 prop 名原样传参（session_title 等）,
// Vue 运行时对下划线键不做 camel 归一,类型与运行时须同名。
const props = defineProps<{
  open: boolean
  target: string
  session_title: string
  all_title: string
  confirm_text: string
  cancel_text: string
}>()

const emit = defineEmits<{
  confirm: [target: string]
  cancel: []
}>()

function onConfirm() {
  emit("confirm", props.target)
}
</script>

<template>
  <AlertDialog :open="props.open" @update:open="(v: boolean) => { if (!v) emit('cancel') }">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>
          {{ props.target === '__all__' ? props.all_title : props.session_title }}
        </AlertDialogTitle>
        <AlertDialogDescription>
          {{ props.target === '__all__' ? props.all_title : props.session_title }}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>{{ props.cancel_text }}</AlertDialogCancel>
        <AlertDialogAction @click="onConfirm">{{ props.confirm_text }}</AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
