<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { ArchiveBox } from "../stores/app";
import { listArchiveBoxesPaged } from "../api";
import Pagination from "./Pagination.vue";
import { showError } from "../utils/error";

const props = defineProps<{
  show: boolean;
  selectedId?: number | null;
}>();

const emit = defineEmits<{
  (e: "select", box: ArchiveBox): void;
  (e: "close"): void;
}>();

const boxes = ref<ArchiveBox[]>([]);
const searchKeyword = ref("");
const currentPage = ref(1);
const total = ref(0);
const perPage = ref(10);
const loading = ref(false);

const getPageSize = () =>
  Math.max(5, Math.min(100, parseInt(localStorage.getItem("pageSize") || "10", 10) || 10));

const load = async () => {
  if (!props.show) return;
  loading.value = true;
  try {
    perPage.value = getPageSize();
    const result = await listArchiveBoxesPaged(
      currentPage.value,
      perPage.value,
      searchKeyword.value.trim() || undefined
    );
    boxes.value = result.items;
    total.value = result.total;
    if (result.items.length === 0 && currentPage.value > 1) {
      currentPage.value--;
      await load();
    }
  } catch (e) {
    showError(e);
  } finally {
    loading.value = false;
  }
};

const search = () => {
  currentPage.value = 1;
  load();
};

const changePage = (page: number) => {
  currentPage.value = page;
  load();
};

const selectBox = (box: ArchiveBox) => {
  emit("select", box);
};

watch(() => props.show, (visible) => {
  if (visible) {
    searchKeyword.value = "";
    currentPage.value = 1;
    load();
  }
});

onMounted(() => {
  if (props.show) load();
});
</script>

<template>
  <div
    v-if="show"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    @click.self="emit('close')"
  >
    <div class="bg-white rounded-xl shadow-xl w-full max-w-3xl p-6 max-h-[90vh] flex flex-col">
      <h3 class="text-lg font-semibold text-slate-800 mb-4">选择档案盒</h3>

      <div class="flex flex-wrap gap-3 items-center mb-4">
        <input
          v-model="searchKeyword"
          @input="search"
          placeholder="搜索档案盒名称 / 存放位置 / 备注"
          class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 w-72"
        />
        <div class="flex-1"></div>
        <button
          @click="emit('close')"
          class="px-4 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition text-sm"
        >
          取消
        </button>
      </div>

      <div class="bg-white rounded-xl shadow-sm border overflow-hidden flex-1 overflow-auto">
        <div class="table-scroll-wrapper">
          <table class="w-full text-left text-sm table-min-content">
            <thead class="bg-slate-50">
              <tr>
                <th class="px-6 py-3 font-medium text-slate-600 w-16">选择</th>
                <th class="px-6 py-3 font-medium text-slate-600">档案盒名称</th>
                <th class="px-6 py-3 font-medium text-slate-600">存放位置</th>
                <th class="px-6 py-3 font-medium text-slate-600">备注</th>
              </tr>
            </thead>
            <tbody class="divide-y">
              <tr
                v-for="b in boxes"
                :key="b.id"
                class="hover:bg-slate-50 cursor-pointer"
                @click="selectBox(b)"
              >
                <td class="px-6 py-4">
                  <input
                    type="radio"
                    :checked="selectedId === b.id"
                    class="w-4 h-4"
                    @click.stop="selectBox(b)"
                  />
                </td>
                <td class="px-6 py-4 font-medium text-slate-800 truncate-cell">{{ b.name }}</td>
                <td class="px-6 py-4 text-slate-600">{{ b.location || "-" }}</td>
                <td class="px-6 py-4 text-slate-500 truncate-cell">{{ b.note || "-" }}</td>
              </tr>
              <tr v-if="boxes.length === 0">
                <td colspan="4" class="px-6 py-8 text-center text-slate-400">
                  {{ loading ? "加载中..." : "暂无档案盒" }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="mt-4">
        <Pagination
          :page="currentPage"
          :per-page="perPage"
          :total="total"
          @change="changePage"
        />
      </div>
    </div>
  </div>
</template>
