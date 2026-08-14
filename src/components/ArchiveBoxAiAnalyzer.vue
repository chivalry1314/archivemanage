<script setup lang="ts">
import { ref, watch } from "vue";
import type { ArchiveBox, ArchiveBoxSuggestion } from "../stores/app";
import {
  analyzeArchiveBox,
  createArchiveBox,
  listArchiveBoxes,
} from "../api";
import ArchiveBoxSelector from "./ArchiveBoxSelector.vue";
import { showError } from "../utils/error";

const props = defineProps<{
  show: boolean;
  title: string;
  categoryId?: number | null;
}>();

const emit = defineEmits<{
  (e: "select", box: ArchiveBox): void;
  (e: "close"): void;
}>();

const loading = ref(false);
const errorMsg = ref("");
const suggestion = ref<ArchiveBoxSuggestion | null>(null);
const existingBoxes = ref<ArchiveBox[]>([]);
const showBoxSelector = ref(false);
const showCreateForm = ref(false);
const newBoxForm = ref({
  name: "",
  location: "",
  note: "",
});

const reset = () => {
  loading.value = false;
  errorMsg.value = "";
  suggestion.value = null;
  existingBoxes.value = [];
  showBoxSelector.value = false;
  showCreateForm.value = false;
  newBoxForm.value = { name: "", location: "", note: "" };
};

const analyze = async () => {
  if (!props.title.trim()) {
    errorMsg.value = "请先填写档案名称";
    return;
  }
  loading.value = true;
  errorMsg.value = "";
  suggestion.value = null;
  try {
    existingBoxes.value = await listArchiveBoxes();
    suggestion.value = await analyzeArchiveBox(
      { title: props.title.trim(), category_id: props.categoryId || undefined },
      existingBoxes.value
    );
    if (!suggestion.value.is_existing) {
      newBoxForm.value.name = suggestion.value.box_name;
    }
  } catch (e: any) {
    errorMsg.value = e?.message || String(e);
  } finally {
    loading.value = false;
  }
};

const adopt = () => {
  if (!suggestion.value) return;
  if (suggestion.value.is_existing && suggestion.value.matched_box_id) {
    const box = existingBoxes.value.find(
      (b) => b.id === suggestion.value?.matched_box_id
    );
    if (box) {
      emit("select", box);
      return;
    }
  }
  // New box suggested
  newBoxForm.value.name = suggestion.value.box_name;
  showCreateForm.value = true;
};

const createAndAdopt = async () => {
  if (!newBoxForm.value.name.trim()) return;
  try {
    const box = await createArchiveBox({
      name: newBoxForm.value.name.trim(),
      location: newBoxForm.value.location.trim() || undefined,
      note: newBoxForm.value.note.trim() || undefined,
    });
    emit("select", box);
  } catch (e) {
    showError(e);
  }
};

const onSelectOther = (box: ArchiveBox) => {
  emit("select", box);
};

watch(
  () => props.show,
  (visible) => {
    if (visible) {
      reset();
      analyze();
    }
  }
);
</script>

<template>
  <div
    v-if="show"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    @click.self="emit('close')"
  >
    <div class="bg-white rounded-xl shadow-xl w-full max-w-lg p-6 max-h-[90vh] overflow-auto">
      <h3 class="text-lg font-semibold text-slate-800 mb-4">AI 档案盒识别</h3>

      <div class="mb-4 text-sm text-slate-600">
        档案名称：<span class="font-medium text-slate-800">{{ title }}</span>
      </div>

      <div v-if="loading" class="py-8 text-center text-slate-500">
        <div class="inline-block w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin mb-2"></div>
        <p>AI 正在分析中...</p>
      </div>

      <div v-else-if="errorMsg" class="bg-red-50 text-red-700 px-4 py-3 rounded-lg text-sm mb-4">
        {{ errorMsg }}
      </div>

      <div v-else-if="suggestion" class="space-y-4">
        <div class="bg-blue-50 rounded-lg p-4">
          <div class="text-sm text-slate-500 mb-1">AI 推荐档案盒</div>
          <div class="text-lg font-semibold text-slate-800">
            {{ suggestion.box_name }}
            <span
              v-if="suggestion.is_existing"
              class="ml-2 px-2 py-0.5 text-xs bg-green-100 text-green-700 rounded-full"
            >
              已存在
            </span>
            <span
              v-else
              class="ml-2 px-2 py-0.5 text-xs bg-amber-100 text-amber-700 rounded-full"
            >
              建议新建
            </span>
          </div>
          <div v-if="suggestion.reason" class="text-sm text-slate-600 mt-2">
            {{ suggestion.reason }}
          </div>
        </div>

        <div v-if="showCreateForm" class="space-y-3 border rounded-lg p-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">新档案盒名称 *</label>
            <input
              v-model="newBoxForm.name"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">存放位置</label>
            <input
              v-model="newBoxForm.location"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">备注</label>
            <input
              v-model="newBoxForm.note"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div class="flex justify-end gap-2">
            <button
              @click="showCreateForm = false"
              class="px-4 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition text-sm"
            >
              取消
            </button>
            <button
              @click="createAndAdopt"
              class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition text-sm"
            >
              创建并采纳
            </button>
          </div>
        </div>

        <div v-else class="flex flex-wrap gap-3">
          <button
            @click="adopt"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
          >
            {{ suggestion.is_existing ? "采纳推荐" : "创建并采纳" }}
          </button>
          <button
            @click="showBoxSelector = true"
            class="px-5 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition"
          >
            选择其他
          </button>
          <button
            @click="emit('close')"
            class="px-5 py-2 bg-white border text-slate-700 rounded-lg hover:bg-slate-50 transition"
          >
            取消
          </button>
        </div>
      </div>

      <div v-else class="py-8 text-center text-slate-400">
        无法获取 AI 分析结果
      </div>
    </div>

    <ArchiveBoxSelector
      :show="showBoxSelector"
      :selected-id="undefined"
      @select="onSelectOther"
      @close="showBoxSelector = false"
    />
  </div>
</template>
