<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useAppStore } from "../stores/app";
import {
  createArchiveTag,
  deleteArchiveTag,
  listArchiveTags,
  listArchiveTagsPaged,
  updateArchiveTag,
} from "../api";
import Pagination from "../components/Pagination.vue";
import RecordDetailModal from "../components/RecordDetailModal.vue";
import { showError } from "../utils/error";
import { confirm } from "../utils/dialog";
import { useRecordPreview } from "../utils/recordPreview";

const store = useAppStore();
const { previewFields, previewTitle, showPreview, openRecord } = useRecordPreview();
const editing = ref(false);

const openTagDetail = (tag: any) => {
  openRecord("标签详情", [
    { label: "标签名称", value: tag.name },
    { label: "父标签", value: parentName(tag.parent_id) },
    { label: "备注", value: tag.note || "-" },
    { label: "创建时间", value: tag.created_at },
  ]);
};
const showForm = ref(false);
const searchKeyword = ref("");
const form = ref({
  id: 0,
  name: "",
  parent_id: null as number | null,
  note: "",
});

const currentPage = ref(1);
const total = ref(0);
const perPage = ref(10);

const getPageSize = () =>
  Math.max(5, Math.min(100, parseInt(localStorage.getItem("pageSize") || "10", 10) || 10));

const resetForm = () => {
  form.value = { id: 0, name: "", parent_id: null, note: "" };
  editing.value = false;
};

const openForm = () => {
  resetForm();
  showForm.value = true;
};

const loadAllTags = async () => {
  store.archiveTags = await listArchiveTags();
};

const load = async () => {
  perPage.value = getPageSize();
  const result = await listArchiveTagsPaged(
    currentPage.value,
    perPage.value,
    searchKeyword.value.trim() || undefined
  );
  store.archiveTags = result.items;
  total.value = result.total;
  if (result.items.length === 0 && currentPage.value > 1) {
    currentPage.value--;
    await load();
  }
};

const submit = async () => {
  if (!form.value.name.trim()) return;

  const payload = {
    name: form.value.name.trim(),
    parent_id: form.value.parent_id || undefined,
    note: form.value.note.trim() || undefined,
  };

  try {
    if (editing.value) {
      await updateArchiveTag({ id: form.value.id, ...payload });
    } else {
      await createArchiveTag(payload);
    }

    resetForm();
    showForm.value = false;
    searchKeyword.value = "";
    currentPage.value = 1;
    await loadAllTags();
    await load();
  } catch (e) {
    showError(e);
  }
};

const editTag = (tag: any) => {
  form.value = {
    id: tag.id,
    name: tag.name,
    parent_id: tag.parent_id || null,
    note: tag.note || "",
  };
  editing.value = true;
  showForm.value = true;
};

const removeTag = async (id: number) => {
  if (!(await confirm("删除标签会同时删除其下级标签，确定删除？"))) return;
  try {
    await deleteArchiveTag(id);
    await loadAllTags();
    await load();
  } catch (e) {
    showError(e);
  }
};

const selectedTagIds = ref<number[]>([]);

const toggleSelectAllTags = () => {
  const currentIds = store.archiveTags.map((t) => t.id);
  const allSelected = currentIds.every((id) => selectedTagIds.value.includes(id));
  if (allSelected) {
    selectedTagIds.value = [];
  } else {
    selectedTagIds.value = [...currentIds];
  }
};

const removeSelectedTags = async () => {
  if (selectedTagIds.value.length === 0) return;
  if (!(await confirm(`确定要删除选中的 ${selectedTagIds.value.length} 个标签吗？其下级标签也会一并删除。`))) return;

  let success = 0;
  let failed = 0;
  for (const id of selectedTagIds.value) {
    try {
      await deleteArchiveTag(id);
      success++;
    } catch (e) {
      failed++;
    }
  }

  selectedTagIds.value = [];
  await loadAllTags();
  await load();

  if (failed > 0) {
    await showError(new Error(`删除完成：成功 ${success} 个，失败 ${failed} 个。`));
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

// Build a flat option list with indentation for the parent select.
const tagById = computed(() => {
  const map = new Map<number, any>();
  store.archiveTags.forEach((t) => map.set(t.id, t));
  return map;
});

const tagDepth = (tag: any, visited = new Set<number>()): number => {
  if (!tag.parent_id || visited.has(tag.id)) return 0;
  visited.add(tag.id);
  const parent = tagById.value.get(tag.parent_id);
  if (!parent) return 0;
  return 1 + tagDepth(parent, visited);
};

const parentOptions = computed(() => {
  // Exclude the current tag and its descendants to avoid cycles.
  const invalid = new Set<number>();
  if (editing.value && form.value.id) {
    invalid.add(form.value.id);
    const collect = (parentId: number) => {
      store.archiveTags
        .filter((t) => t.parent_id === parentId)
        .forEach((t) => {
          invalid.add(t.id);
          collect(t.id);
        });
    };
    collect(form.value.id);
  }

  return store.archiveTags
    .filter((t) => !invalid.has(t.id))
    .map((t) => ({
      ...t,
      depth: tagDepth(t),
    }))
    .sort((a, b) => a.name.localeCompare(b.name, "zh-CN"));
});

const parentName = (parentId?: number) => {
  if (!parentId) return "-";
  return store.archiveTags.find((t) => t.id === parentId)?.name || "-";
};

onMounted(async () => {
  await loadAllTags();
  await load();
});
</script>

<template>
  <div class="space-y-6">
    <div class="bg-white p-4 rounded-xl shadow-sm border flex flex-wrap gap-3 items-center">
      <input
        v-model="searchKeyword"
        @input="search"
        placeholder="搜索标签名称 / 备注"
        class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 w-72"
      />
      <div class="flex-1"></div>
      <button
        v-if="selectedTagIds.length > 0"
        @click="removeSelectedTags"
        class="px-5 py-2.5 bg-red-600 text-white rounded-lg hover:bg-red-700 transition"
      >
        批量删除（{{ selectedTagIds.length }}）
      </button>
      <button
        @click="openForm"
        class="px-5 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
      >
        + 添加标签
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
          {{ editing ? "编辑标签" : "添加标签" }}
        </h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">标签名称 *</label>
            <input
              v-model="form.name"
              placeholder="如：重要合同"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">父标签</label>
            <select
              v-model="form.parent_id"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option :value="null">无（顶层标签）</option>
              <option v-for="t in parentOptions" :key="t.id" :value="t.id">
                {{ "　".repeat(t.depth) }}{{ t.depth > 0 ? "└ " : "" }}{{ t.name }}
              </option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">备注</label>
            <input
              v-model="form.note"
              placeholder="备注"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
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
            <th class="px-4 py-3 font-medium text-slate-600 w-10">
              <input
                type="checkbox"
                :checked="store.archiveTags.length > 0 && store.archiveTags.every((t) => selectedTagIds.includes(t.id))"
                @change="toggleSelectAllTags"
                class="w-4 h-4"
              />
            </th>
            <th class="px-6 py-3 font-medium text-slate-600">标签名称</th>
            <th class="px-6 py-3 font-medium text-slate-600">父标签</th>
            <th class="px-6 py-3 font-medium text-slate-600">备注</th>
            <th class="px-6 py-3 font-medium text-slate-600">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          <tr v-for="tag in store.archiveTags" :key="tag.id" class="hover:bg-slate-50">
            <td class="px-4 py-4">
              <input
                type="checkbox"
                :value="tag.id"
                v-model="selectedTagIds"
                class="w-4 h-4"
              />
            </td>
            <td
              class="px-6 py-4 font-medium text-slate-800 truncate-cell"
              @dblclick="openTagDetail(tag)"
            >
              {{ tag.name }}
            </td>
            <td class="px-6 py-4 text-slate-500">{{ parentName(tag.parent_id) }}</td>
            <td
              class="px-6 py-4 text-slate-500 truncate-cell"
              @dblclick="openTagDetail(tag)"
            >
              {{ tag.note || "-" }}
            </td>
            <td class="px-6 py-4 flex gap-2">
              <button
                @click="editTag(tag)"
                class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
              >
                编辑
              </button>
              <button
                @click="removeTag(tag.id)"
                class="px-3 py-1 text-xs bg-red-50 text-red-600 hover:bg-red-100 rounded transition"
              >
                删除
              </button>
            </td>
          </tr>
          <tr v-if="store.archiveTags.length === 0">
            <td colspan="5" class="px-6 py-8 text-center text-slate-400">暂无标签</td>
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
