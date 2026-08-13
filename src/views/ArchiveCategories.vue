<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAppStore } from "../stores/app";
import {
  createArchiveCategory,
  deleteArchiveCategory,
  listArchiveCategoriesPaged,
  updateArchiveCategory,
} from "../api";
import Pagination from "../components/Pagination.vue";
import RecordDetailModal from "../components/RecordDetailModal.vue";
import { showError } from "../utils/error";
import { confirm } from "../utils/dialog";
import { useRecordPreview } from "../utils/recordPreview";

const store = useAppStore();
const { previewFields, previewTitle, showPreview, openRecord } = useRecordPreview();
const editing = ref(false);

const openCategoryDetail = (c: any) => {
  openRecord("分类详情", [
    { label: "分类名称", value: c.name },
    { label: "编号前缀", value: c.code_prefix },
    { label: "备注", value: c.note || "-" },
    { label: "创建时间", value: c.created_at },
  ]);
};
const showForm = ref(false);
const searchKeyword = ref("");
const form = ref({
  id: 0,
  name: "",
  code_prefix: "",
  note: "",
});

const currentPage = ref(1);
const total = ref(0);
const perPage = ref(10);

const getPageSize = () =>
  Math.max(5, Math.min(100, parseInt(localStorage.getItem("pageSize") || "10", 10) || 10));

const resetForm = () => {
  form.value = { id: 0, name: "", code_prefix: "", note: "" };
  editing.value = false;
};

const openForm = () => {
  resetForm();
  showForm.value = true;
};

const load = async () => {
  perPage.value = getPageSize();
  const result = await listArchiveCategoriesPaged(
    currentPage.value,
    perPage.value,
    searchKeyword.value.trim() || undefined
  );
  store.archiveCategories = result.items;
  total.value = result.total;
  if (result.items.length === 0 && currentPage.value > 1) {
    currentPage.value--;
    await load();
  }
};

const submit = async () => {
  if (!form.value.name.trim() || !form.value.code_prefix.trim()) return;

  const payload = {
    name: form.value.name.trim(),
    code_prefix: form.value.code_prefix.trim().toUpperCase(),
    note: form.value.note.trim() || undefined,
  };

  try {
    if (editing.value) {
      await updateArchiveCategory({ id: form.value.id, ...payload });
    } else {
      await createArchiveCategory(payload);
    }

    resetForm();
    showForm.value = false;
    searchKeyword.value = "";
    currentPage.value = 1;
    await load();
  } catch (e) {
    showError(e);
  }
};

const editCategory = (c: any) => {
  form.value = {
    id: c.id,
    name: c.name,
    code_prefix: c.code_prefix,
    note: c.note || "",
  };
  editing.value = true;
  showForm.value = true;
};

const removeCategory = async (id: number) => {
  if (!(await confirm("删除分类后，该分类下的档案将失去分类关联，确定删除？"))) return;
  try {
    await deleteArchiveCategory(id);
    await load();
  } catch (e) {
    showError(e);
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

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <div class="bg-white p-4 rounded-xl shadow-sm border flex flex-wrap gap-3 items-center">
      <input
        v-model="searchKeyword"
        @input="search"
        placeholder="搜索分类名称 / 编号前缀 / 备注"
        class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 w-72"
      />
      <div class="flex-1"></div>
      <button
        @click="openForm"
        class="px-5 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
      >
        + 添加分类
      </button>
    </div>

    <!-- Form Modal -->
    <div
      v-if="showForm"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showForm = false"
    >
      <div class="bg-white rounded-xl shadow-xl w-full max-w-lg p-6 max-h-[90vh] overflow-auto">
        <h3 class="text-lg font-semibold text-slate-800 mb-4">
          {{ editing ? "编辑分类" : "添加分类" }}
        </h3>
        <div class="grid grid-cols-1 gap-4">
          <input
            v-model="form.name"
            placeholder="分类名称 *"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.code_prefix"
            placeholder="编号前缀 *，如 YZ、SB、HT"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.note"
            placeholder="备注"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div class="mt-6 flex justify-end gap-3">
          <button
            @click="showForm = false"
            class="px-5 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition"
          >
            取消
          </button>
          <button
            @click="submit"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
          >
            {{ editing ? "保存" : "添加" }}
          </button>
        </div>
      </div>
    </div>

    <div class="bg-white rounded-xl shadow-sm border overflow-hidden">
      <div class="table-scroll-wrapper">
      <table class="w-full text-left text-sm table-min-content">
        <thead class="bg-slate-50">
          <tr>
            <th class="px-6 py-3 font-medium text-slate-600">分类名称</th>
            <th class="px-6 py-3 font-medium text-slate-600">编号前缀</th>
            <th class="px-6 py-3 font-medium text-slate-600">备注</th>
            <th class="px-6 py-3 font-medium text-slate-600">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          <tr v-for="c in store.archiveCategories" :key="c.id" class="hover:bg-slate-50">
            <td
              class="px-6 py-4 font-medium text-slate-800 truncate-cell"
              @dblclick="openCategoryDetail(c)"
            >
              {{ c.name }}
            </td>
            <td class="px-6 py-4 font-mono text-slate-600">{{ c.code_prefix }}</td>
            <td
              class="px-6 py-4 text-slate-500 truncate-cell"
              @dblclick="openCategoryDetail(c)"
            >
              {{ c.note || "-" }}
            </td>
            <td class="px-6 py-4 flex gap-2">
              <button
                @click="editCategory(c)"
                class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
              >
                编辑
              </button>
              <button
                @click="removeCategory(c.id)"
                class="px-3 py-1 text-xs bg-red-50 text-red-600 hover:bg-red-100 rounded transition"
              >
                删除
              </button>
            </td>
          </tr>
          <tr v-if="store.archiveCategories.length === 0">
            <td colspan="4" class="px-6 py-8 text-center text-slate-400">暂无分类</td>
          </tr>
        </tbody>
      </table>
      </div>
      <Pagination
        :page="currentPage"
        :per-page="perPage"
        :total="total"
        @change="changePage"
      />
    </div>
  </div>

  <RecordDetailModal
    :show="showPreview"
    :title="previewTitle"
    :fields="previewFields"
    @close="showPreview = false"
  />
</template>
