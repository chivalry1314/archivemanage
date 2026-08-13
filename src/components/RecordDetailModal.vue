<script setup lang="ts">
import type { RecordField } from "../utils/recordPreview";

defineProps<{
  show: boolean;
  title: string;
  fields: RecordField[];
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <div
    v-if="show"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]"
    @click.self="emit('close')"
  >
    <div class="bg-white rounded-xl shadow-xl w-full max-w-lg p-6 max-h-[80vh] flex flex-col">
      <h3 class="text-lg font-semibold text-slate-800 mb-4">{{ title }}</h3>
      <div class="flex-1 overflow-auto">
        <div
          v-for="(field, index) in fields"
          :key="index"
          class="py-3 border-b last:border-b-0"
        >
          <div class="text-xs text-slate-500 mb-1">{{ field.label }}</div>
          <div class="text-sm text-slate-800 whitespace-pre-wrap break-all">{{ field.value || "-" }}</div>
        </div>
      </div>
      <div class="mt-6 flex justify-end">
        <button
          @click="emit('close')"
          class="px-5 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition"
        >
          关闭
        </button>
      </div>
    </div>
  </div>
</template>
