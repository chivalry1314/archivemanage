<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  page: number;
  perPage: number;
  total: number;
}>();

const emit = defineEmits<{
  (e: "change", page: number): void;
}>();

const totalPages = computed(() => Math.max(1, Math.ceil(props.total / props.perPage)));

const visiblePages = computed(() => {
  const pages: number[] = [];
  const start = Math.max(1, props.page - 2);
  const end = Math.min(totalPages.value, props.page + 2);
  for (let i = start; i <= end; i++) {
    pages.push(i);
  }
  return pages;
});

const go = (p: number) => {
  if (p >= 1 && p <= totalPages.value && p !== props.page) {
    emit("change", p);
  }
};
</script>

<template>
  <div class="flex items-center justify-between px-6 py-3 bg-slate-50 border-t">
    <div class="text-sm text-slate-500">
      共 {{ total }} 条，每页 {{ perPage }} 条
    </div>
    <div class="flex items-center gap-1">
      <button
        @click="go(1)"
        :disabled="page === 1"
        class="px-3 py-1.5 text-sm rounded-lg border bg-white text-slate-600 hover:bg-slate-100 disabled:opacity-40 disabled:cursor-not-allowed transition"
      >
        首页
      </button>
      <button
        @click="go(page - 1)"
        :disabled="page === 1"
        class="px-3 py-1.5 text-sm rounded-lg border bg-white text-slate-600 hover:bg-slate-100 disabled:opacity-40 disabled:cursor-not-allowed transition"
      >
        上一页
      </button>
      <button
        v-for="p in visiblePages"
        :key="p"
        @click="go(p)"
        :class="[
          'px-3 py-1.5 text-sm rounded-lg border transition',
          page === p
            ? 'bg-blue-600 text-white border-blue-600'
            : 'bg-white text-slate-600 hover:bg-slate-100',
        ]"
      >
        {{ p }}
      </button>
      <button
        @click="go(page + 1)"
        :disabled="page === totalPages"
        class="px-3 py-1.5 text-sm rounded-lg border bg-white text-slate-600 hover:bg-slate-100 disabled:opacity-40 disabled:cursor-not-allowed transition"
      >
        下一页
      </button>
      <button
        @click="go(totalPages)"
        :disabled="page === totalPages"
        class="px-3 py-1.5 text-sm rounded-lg border bg-white text-slate-600 hover:bg-slate-100 disabled:opacity-40 disabled:cursor-not-allowed transition"
      >
        末页
      </button>
    </div>
  </div>
</template>
