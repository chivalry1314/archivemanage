<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useAppStore } from "../stores/app";
import {
  createArchive,
  createArchiveBorrow,
  deleteArchive,
  deleteArchiveBorrow,
  getAiConfig,
  getArchiveFilePath,
  getArchiveStats,
  listArchiveBorrows,
  listArchiveCategories,
  listArchiveTags,
  listArchives,
  listArchivesByTag,
  listMembers,
  returnArchiveBorrow,
  updateArchive,
  updateArchiveBorrow,
  updateArchiveStatus,
} from "../api";
import ArchiveBoxSelector from "../components/ArchiveBoxSelector.vue";
import ArchiveBoxAiAnalyzer from "../components/ArchiveBoxAiAnalyzer.vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import Pagination from "../components/Pagination.vue";
import RecordDetailModal from "../components/RecordDetailModal.vue";
import { showError } from "../utils/error";
import { confirm } from "../utils/dialog";
import { useRecordPreview } from "../utils/recordPreview";

const store = useAppStore();
const { previewFields, previewTitle, showPreview, openRecord } = useRecordPreview();
const selectedCategory = ref<number | null>(null);
const selectedStatus = ref<string>("");
const searchKeyword = ref("");
const showForm = ref(false);
const editing = ref(false);
const showBorrowForm = ref(false);
const selectedArchive = ref<any>(null);
const showHistory = ref(false);
const historyBorrows = ref<any[]>([]);
const historyPage = ref(1);
const historyTotal = ref(0);
const historyPerPage = ref(10);
const activeTab = ref<"archives" | "borrows">("archives");
const borrowStatusFilter = ref<string>("");
const showBorrowEditForm = ref(false);
const borrowEditForm = ref({
  id: 0,
  archive_id: 0,
  borrower_id: null as number | null,
  purpose: "",
  borrow_date: "",
  due_date: "",
  return_date: "",
  status: "",
  approver_id: null as number | null,
  note: "",
});

const archivePage = ref(1);
const archiveTotal = ref(0);
const archivePerPage = ref(10);

const borrowPage = ref(1);
const borrowTotal = ref(0);
const borrowPerPage = ref(10);

const displayMode = ref<"normal" | "tag-tree">("normal");
const selectedTagId = ref<number | null>(null);
const tagTreePage = ref(1);
const tagTreeTotal = ref(0);
const tagTreePerPage = ref(10);
const tagTreeArchives = ref<any[]>([]);
const selectedArchiveIds = ref<number[]>([]);

const archiveForm = ref({
  id: 0,
  title: "",
  category_id: null as number | null,
  location: "",
  archive_box_id: null as number | null,
  box_name: "",
  file_path: "",
  source_file_path: "",
  keeper_id: null as number | null,
  quantity: 1,
  description: "",
  photos: "",
  tag_ids: [] as number[],
});
const showBoxSelector = ref(false);
const showAiAnalyzer = ref(false);
const aiEnabled = ref(false);

const borrowForm = ref({
  archive_id: 0,
  borrower_id: null as number | null,
  purpose: "",
  borrow_date: "",
  due_date: "",
  approver_id: null as number | null,
  note: "",
});

const statusOptions = [
  { value: "in_stock", label: "在库" },
  { value: "borrowed", label: "借出" },
  { value: "damaged", label: "损坏" },
  { value: "destroyed", label: "销毁" },
];

const resetArchiveForm = () => {
  archiveForm.value = {
    id: 0,
    title: "",
    category_id: null,
    location: "",
    archive_box_id: null,
    box_name: "",
    file_path: "",
    source_file_path: "",
    keeper_id: null,
    quantity: 1,
    description: "",
    photos: "",
    tag_ids: [],
  };
  editing.value = false;
};

const getPageSize = () =>
  Math.max(5, Math.min(100, parseInt(localStorage.getItem("pageSize") || "10", 10) || 10));

const loadCategories = async () => {
  store.archiveCategories = await listArchiveCategories();
};

const loadTags = async () => {
  store.archiveTags = await listArchiveTags();
};

const loadArchives = async () => {
  archivePerPage.value = getPageSize();
  const result = await listArchives(
    selectedCategory.value || undefined,
    selectedStatus.value || undefined,
    searchKeyword.value || undefined,
    archivePage.value,
    archivePerPage.value
  );
  store.archives = result.items;
  archiveTotal.value = result.total;
  if (result.items.length === 0 && archivePage.value > 1) {
    archivePage.value--;
    await loadArchives();
  }
  await loadStats();
};

const loadBorrows = async () => {
  borrowPerPage.value = getPageSize();
  const result = await listArchiveBorrows(
    borrowStatusFilter.value || undefined,
    undefined,
    undefined,
    borrowPage.value,
    borrowPerPage.value
  );
  store.archiveBorrows = result.items;
  borrowTotal.value = result.total;
  if (result.items.length === 0 && borrowPage.value > 1) {
    borrowPage.value--;
    await loadBorrows();
  }
  await loadStats();
};

const loadStats = async () => {
  store.archiveStats = await getArchiveStats();
};

const loadAll = async () => {
  await loadCategories();
  await loadTags();
  await loadArchives();
  await loadBorrows();
  await loadStats();
  store.members = await listMembers();
  try {
    const cfg = await getAiConfig();
    aiEnabled.value = cfg.enabled;
  } catch (e) {
    aiEnabled.value = false;
  }
};

const loadTagTreeArchives = async () => {
  if (!selectedTagId.value) {
    tagTreeArchives.value = [];
    tagTreeTotal.value = 0;
    return;
  }
  tagTreePerPage.value = getPageSize();
  const result = await listArchivesByTag(
    selectedTagId.value,
    tagTreePage.value,
    tagTreePerPage.value
  );
  tagTreeArchives.value = result.items;
  tagTreeTotal.value = result.total;
  if (result.items.length === 0 && tagTreePage.value > 1) {
    tagTreePage.value--;
    await loadTagTreeArchives();
  }
};

const changeArchivePage = (page: number) => {
  archivePage.value = page;
  loadArchives();
};

const changeBorrowPage = (page: number) => {
  borrowPage.value = page;
  loadBorrows();
};

const changeTagTreePage = (page: number) => {
  tagTreePage.value = page;
  loadTagTreeArchives();
};

const selectTag = (tagId: number) => {
  selectedTagId.value = tagId;
  tagTreePage.value = 1;
  loadTagTreeArchives();
};

const submitArchive = async () => {
  if (!archiveForm.value.title.trim() || !archiveForm.value.category_id) return;

  if (
    archiveForm.value.source_file_path.trim() &&
    !archiveForm.value.archive_box_id
  ) {
    showError(new Error("上传电子文件前必须先选择档案盒。"));
    return;
  }

  const payload = {
    title: archiveForm.value.title.trim(),
    category_id: archiveForm.value.category_id,
    location: archiveForm.value.location.trim() || undefined,
    archive_box_id: archiveForm.value.archive_box_id || undefined,
    box_name: archiveForm.value.box_name.trim() || undefined,
    file_path: archiveForm.value.file_path.trim() || undefined,
    source_file_path: archiveForm.value.source_file_path.trim() || undefined,
    keeper_id: archiveForm.value.keeper_id || undefined,
    quantity: archiveForm.value.quantity || 1,
    description: archiveForm.value.description.trim() || undefined,
    photos: archiveForm.value.photos.trim() || undefined,
    tag_ids: archiveForm.value.tag_ids,
  };

  try {
    if (editing.value) {
      await updateArchive({ id: archiveForm.value.id, ...payload } as any);
    } else {
      await createArchive(payload as any);
    }

    showForm.value = false;
    resetArchiveForm();
    archivePage.value = 1;
    await loadArchives();
    if (selectedTagId.value) {
      await loadTagTreeArchives();
    }
  } catch (e) {
    showError(e);
  }
};

const searchArchives = () => {
  archivePage.value = 1;
  loadArchives();
};

const filterArchives = () => {
  archivePage.value = 1;
  loadArchives();
};

const editArchive = (item: any) => {
  archiveForm.value = {
    id: item.archive.id,
    title: item.archive.title,
    category_id: item.archive.category_id,
    location: item.archive.location || "",
    archive_box_id: item.archive.archive_box_id || null,
    box_name: item.archive.box_name || "",
    file_path: item.archive.file_path || "",
    source_file_path: "",
    keeper_id: item.archive.keeper_id || null,
    quantity: item.archive.quantity,
    description: item.archive.description || "",
    photos: item.archive.photos || "",
    tag_ids: (item.tags || []).map((t: any) => t.id),
  };
  editing.value = true;
  showForm.value = true;
};

const onSelectBox = (box: any) => {
  archiveForm.value.archive_box_id = box.id;
  archiveForm.value.box_name = box.name;
  // 存放位置由档案盒自动带出：优先位置，未配置则用档案盒名称
  archiveForm.value.location = box.location || box.name || "";
  showBoxSelector.value = false;
};

const clearSelectedBox = () => {
  archiveForm.value.archive_box_id = null;
  archiveForm.value.box_name = "";
  archiveForm.value.location = "";
};

const onAiSelectBox = (box: any) => {
  onSelectBox(box);
  showAiAnalyzer.value = false;
};

const removeArchive = async (id: number) => {
  if (!(await confirm("确定要删除该档案吗？"))) return;
  try {
    await deleteArchive(id);
    await loadArchives();
    if (selectedTagId.value) await loadTagTreeArchives();
  } catch (e) {
    showError(e);
  }
};

const toggleSelectAllArchives = () => {
  const currentIds = displayMode.value === "normal"
    ? filteredArchives.value.map((a) => a.archive.id)
    : tagTreeArchives.value.map((a) => a.archive.id);
  const allSelected = currentIds.every((id) => selectedArchiveIds.value.includes(id));
  if (allSelected) {
    selectedArchiveIds.value = selectedArchiveIds.value.filter((id) => !currentIds.includes(id));
  } else {
    const merged = new Set([...selectedArchiveIds.value, ...currentIds]);
    selectedArchiveIds.value = Array.from(merged);
  }
};

const removeSelectedArchives = async () => {
  if (selectedArchiveIds.value.length === 0) return;
  if (!(await confirm(`确定要删除选中的 ${selectedArchiveIds.value.length} 个档案吗？`))) return;

  let success = 0;
  let failed = 0;
  for (const id of selectedArchiveIds.value) {
    try {
      await deleteArchive(id);
      success++;
    } catch (e) {
      failed++;
    }
  }

  selectedArchiveIds.value = [];
  await loadArchives();
  if (selectedTagId.value) await loadTagTreeArchives();

  if (failed > 0) {
    await showError(new Error(`删除完成：成功 ${success} 个，失败 ${failed} 个。`));
  }
};

const buildTagTree = (tags: any[]) => {
  const map = new Map<number, any>();
  tags.forEach((t) => map.set(t.id, { ...t, children: [] }));
  const roots: any[] = [];
  tags.forEach((t) => {
    const node = map.get(t.id)!;
    if (t.parent_id && map.has(t.parent_id)) {
      map.get(t.parent_id)!.children.push(node);
    } else {
      roots.push(node);
    }
  });
  return roots;
};

const flattenTagTree = (nodes: any[], depth = 0): any[] => {
  const result: any[] = [];
  nodes.forEach((n) => {
    result.push({ ...n, depth });
    if (n.children?.length) result.push(...flattenTagTree(n.children, depth + 1));
  });
  return result;
};

const tagTreeFlat = computed(() => flattenTagTree(buildTagTree(store.archiveTags)));

const toggleTag = (tagId: number) => {
  const idx = archiveForm.value.tag_ids.indexOf(tagId);
  if (idx >= 0) {
    archiveForm.value.tag_ids.splice(idx, 1);
  } else {
    archiveForm.value.tag_ids.push(tagId);
  }
};

const tagNames = (tags: any[]) => {
  return (tags || []).map((t) => t.name).join("、") || "-";
};

const openBorrowDetail = (item: any) => {
  openRecord("借还详情", [
    { label: "档案编号", value: item.archive.archive.code },
    { label: "档案名称", value: item.archive.archive.title },
    { label: "借阅人", value: item.borrower.name },
    { label: "借阅用途", value: item.borrow.purpose || "-" },
    { label: "借阅日期", value: item.borrow.borrow_date },
    { label: "应还日期", value: item.borrow.due_date },
    { label: "归还日期", value: item.borrow.return_date || "-" },
    { label: "状态", value: borrowStatusLabel(item.borrow.status) },
    { label: "审批人", value: item.approver?.name || "-" },
    { label: "备注", value: item.borrow.note || "-" },
  ]);
};

const openArchiveDetail = (item: any) => {
  openRecord("档案详情", [
    { label: "档案编号", value: item.archive.code },
    { label: "档案名称", value: item.archive.title },
    { label: "档案盒名称", value: item.archive.box_name || "-" },
    { label: "分类", value: categoryName(item.archive.category_id) },
    { label: "标签", value: tagNames(item.tags) },
    { label: "存放位置", value: item.archive.location || "-" },
    { label: "保管人", value: memberName(item.archive.keeper_id) },
    { label: "状态", value: statusLabel(item.archive.status) },
    { label: "数量", value: String(item.archive.quantity) },
    { label: "电子文件", value: fileNameFromPath(item.archive.file_path) || "-" },
    { label: "描述", value: item.archive.description || "-" },
  ]);
};

const openBorrowForm = (item: any) => {
  selectedArchive.value = item;
  borrowForm.value = {
    archive_id: item.archive.id,
    borrower_id: null,
    purpose: "",
    borrow_date: new Date().toISOString().split("T")[0],
    due_date: "",
    approver_id: null,
    note: "",
  };
  showBorrowForm.value = true;
};

const submitBorrow = async () => {
  if (!borrowForm.value.borrower_id || !borrowForm.value.due_date) return;

  try {
    await createArchiveBorrow({
      archive_id: borrowForm.value.archive_id,
      borrower_id: borrowForm.value.borrower_id,
      purpose: borrowForm.value.purpose.trim() || undefined,
      borrow_date: borrowForm.value.borrow_date,
      due_date: borrowForm.value.due_date,
      approver_id: borrowForm.value.approver_id || undefined,
      note: borrowForm.value.note.trim() || undefined,
    });

    showBorrowForm.value = false;
    await loadAll();
  } catch (e) {
    showError(e);
  }
};

const returnBorrow = async (borrow: any) => {
  try {
    const returnDate = new Date().toISOString().split("T")[0];
    await returnArchiveBorrow(borrow.borrow.id, returnDate);
    await loadAll();
  } catch (e) {
    showError(e);
  }
};

const openBorrowEditForm = (item: any) => {
  borrowEditForm.value = {
    id: item.borrow.id,
    archive_id: item.borrow.archive_id,
    borrower_id: item.borrow.borrower_id,
    purpose: item.borrow.purpose || "",
    borrow_date: item.borrow.borrow_date,
    due_date: item.borrow.due_date,
    return_date: item.borrow.return_date || "",
    status: item.borrow.status,
    approver_id: item.borrow.approver_id || null,
    note: item.borrow.note || "",
  };
  showBorrowEditForm.value = true;
};

const submitBorrowEdit = async () => {
  if (!borrowEditForm.value.borrower_id || !borrowEditForm.value.borrow_date || !borrowEditForm.value.due_date) {
    return;
  }

  try {
    await updateArchiveBorrow({
      id: borrowEditForm.value.id,
      borrower_id: borrowEditForm.value.borrower_id,
      purpose: borrowEditForm.value.purpose.trim() || undefined,
      borrow_date: borrowEditForm.value.borrow_date,
      due_date: borrowEditForm.value.due_date,
      return_date: borrowEditForm.value.return_date || undefined,
      status: borrowEditForm.value.status,
      approver_id: borrowEditForm.value.approver_id || undefined,
      note: borrowEditForm.value.note.trim() || undefined,
    } as any);

    showBorrowEditForm.value = false;
    await loadAll();
  } catch (e) {
    showError(e);
  }
};

const removeBorrow = async (item: any) => {
  if (!(await confirm("确定要删除这条借还记录吗？"))) return;
  try {
    await deleteArchiveBorrow(item.borrow.id);
    await loadAll();
  } catch (e) {
    showError(e);
  }
};

const changeStatus = async (item: any, status: string) => {
  try {
    await updateArchiveStatus(item.archive.id, status);
    await loadArchives();
    if (selectedTagId.value) await loadTagTreeArchives();
  } catch (e) {
    showError(e);
  }
};

const loadHistory = async () => {
  if (!selectedArchive.value) return;
  historyPerPage.value = getPageSize();
  const result = await listArchiveBorrows(
    undefined,
    selectedArchive.value.archive.id,
    undefined,
    historyPage.value,
    historyPerPage.value
  );
  historyBorrows.value = result.items;
  historyTotal.value = result.total;
  if (result.items.length === 0 && historyPage.value > 1) {
    historyPage.value--;
    await loadHistory();
  }
};

const viewHistory = async (item: any) => {
  selectedArchive.value = item;
  historyPage.value = 1;
  await loadHistory();
  showHistory.value = true;
};

const changeHistoryPage = (page: number) => {
  historyPage.value = page;
  loadHistory();
};

const categoryName = (id: number) => {
  return store.archiveCategories.find((c) => c.id === id)?.name || "-";
};

const memberName = (id?: number) => {
  if (!id) return "-";
  return store.members.find((m) => m.id === id)?.name || "-";
};

const statusLabel = (status: string) => {
  const map: Record<string, string> = {
    in_stock: "在库",
    borrowed: "借出",
    damaged: "损坏",
    destroyed: "销毁",
  };
  return map[status] || status;
};

const statusClass = (status: string) => {
  const map: Record<string, string> = {
    in_stock: "bg-green-100 text-green-700",
    borrowed: "bg-amber-100 text-amber-700",
    damaged: "bg-orange-100 text-orange-700",
    destroyed: "bg-gray-100 text-gray-700",
  };
  return map[status] || "bg-gray-100 text-gray-700";
};

const fileNameFromPath = (path?: string) => {
  if (!path) return "";
  return path.replace(/^.*[\\/]/, "");
};

const selectElectronicFile = async () => {
  try {
    const path = await openDialog({
      multiple: false,
      directory: false,
    });
    if (path && typeof path === "string") {
      archiveForm.value.source_file_path = path;
    }
  } catch (e) {
    showError(e);
  }
};

const openArchiveFile = async (item: any) => {
  try {
    const path = await getArchiveFilePath(item.archive.id);
    await openPath(path);
  } catch (e) {
    showError(e);
  }
};

const borrowStatusLabel = (status: string) => {
  const map: Record<string, string> = {
    borrowed: "借阅中",
    returned: "已归还",
    overdue: "逾期",
  };
  return map[status] || status;
};

const borrowStatusClass = (status: string) => {
  const map: Record<string, string> = {
    borrowed: "bg-blue-100 text-blue-700",
    returned: "bg-green-100 text-green-700",
    overdue: "bg-red-100 text-red-700",
  };
  return map[status] || "bg-gray-100 text-gray-700";
};

const activeBorrows = computed(() => {
  return (store.archiveBorrows || []).filter((b) => b?.borrow?.status !== "returned");
});

const archiveCount = computed(() => store.archiveStats.total_count);
const inStockCount = computed(() => store.archiveStats.in_stock_count);
const borrowedCount = computed(() => store.archiveStats.borrowed_count);
const damagedCount = computed(() => store.archiveStats.damaged_count);
const destroyedCount = computed(() => store.archiveStats.destroyed_count);
const overdueCount = computed(() => store.archiveStats.overdue_count);

const filteredArchives = computed(() =>
  (store.archives || []).filter((a) => a && a.archive)
);

onMounted(loadAll);
</script>

<template>
  <div class="space-y-6">
    <!-- Stats Row -->
    <div class="grid grid-cols-6 gap-4">
      <div class="bg-white p-4 rounded-xl shadow-sm border text-center">
        <div class="text-sm text-slate-500">总数</div>
        <div class="text-2xl font-bold text-slate-800">{{ archiveCount }}</div>
      </div>
      <div class="bg-white p-4 rounded-xl shadow-sm border text-center">
        <div class="text-sm text-slate-500">在库</div>
        <div class="text-2xl font-bold text-green-600">{{ inStockCount }}</div>
      </div>
      <div class="bg-white p-4 rounded-xl shadow-sm border text-center">
        <div class="text-sm text-slate-500">借出</div>
        <div class="text-2xl font-bold text-amber-600">{{ borrowedCount }}</div>
      </div>
      <div class="bg-white p-4 rounded-xl shadow-sm border text-center">
        <div class="text-sm text-slate-500">损坏</div>
        <div class="text-2xl font-bold text-orange-600">{{ damagedCount }}</div>
      </div>
      <div class="bg-white p-4 rounded-xl shadow-sm border text-center">
        <div class="text-sm text-slate-500">销毁</div>
        <div class="text-2xl font-bold text-slate-600">{{ destroyedCount }}</div>
      </div>
      <div class="bg-white p-4 rounded-xl shadow-sm border text-center">
        <div class="text-sm text-slate-500">逾期未还</div>
        <div class="text-2xl font-bold text-red-600">{{ overdueCount }}</div>
      </div>
    </div>

    <!-- Tabs -->
    <div class="bg-white p-2 rounded-xl shadow-sm border flex gap-2">
      <button
        @click="activeTab = 'archives'"
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition',
          activeTab === 'archives'
            ? 'bg-blue-600 text-white'
            : 'text-slate-600 hover:bg-slate-100',
        ]"
      >
        档案列表
      </button>
      <button
        @click="activeTab = 'borrows'"
        :class="[
          'px-4 py-2 rounded-lg text-sm font-medium transition',
          activeTab === 'borrows'
            ? 'bg-blue-600 text-white'
            : 'text-slate-600 hover:bg-slate-100',
        ]"
      >
        借还记录
        <span
          v-if="activeBorrows.length > 0"
          class="ml-1.5 px-1.5 py-0.5 text-xs bg-red-100 text-red-600 rounded-full"
        >
          {{ activeBorrows.length }}
        </span>
      </button>
    </div>

    <!-- Toolbar -->
    <div v-if="activeTab === 'archives'" class="bg-white p-4 rounded-xl shadow-sm border flex flex-wrap gap-3 items-center">
      <div class="flex items-center bg-slate-100 rounded-lg p-1">
        <button
          @click="displayMode = 'normal'"
          :class="[
            'px-3 py-1.5 text-sm rounded-md transition',
            displayMode === 'normal'
              ? 'bg-white text-slate-800 shadow-sm'
              : 'text-slate-600 hover:bg-slate-200',
          ]"
        >
          普通
        </button>
        <button
          @click="displayMode = 'tag-tree'"
          :class="[
            'px-3 py-1.5 text-sm rounded-md transition',
            displayMode === 'tag-tree'
              ? 'bg-white text-slate-800 shadow-sm'
              : 'text-slate-600 hover:bg-slate-200',
          ]"
        >
          标签树
        </button>
      </div>
      <template v-if="displayMode === 'normal'">
        <input
          v-model="searchKeyword"
          @input="searchArchives"
          placeholder="搜索编号/名称/位置"
          class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 w-56"
        />
        <select
          v-model="selectedCategory"
          @change="filterArchives"
          class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option :value="null">全部分类</option>
          <option v-for="c in store.archiveCategories" :key="c.id" :value="c.id">
            {{ c.name }}
          </option>
        </select>
        <select
          v-model="selectedStatus"
          @change="filterArchives"
          class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="">全部状态</option>
          <option v-for="s in statusOptions" :key="s.value" :value="s.value">
            {{ s.label }}
          </option>
        </select>
      </template>
      <div class="flex-1"></div>
      <button
        v-if="selectedArchiveIds.length > 0"
        @click="removeSelectedArchives"
        class="px-5 py-2.5 bg-red-600 text-white rounded-lg hover:bg-red-700 transition"
      >
        批量删除（{{ selectedArchiveIds.length }}）
      </button>
      <button
        @click="showForm = true; resetArchiveForm();"
        class="px-5 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
      >
        + 登记档案
      </button>
    </div>

    <!-- Archive Form Modal -->
    <div
      v-if="showForm"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showForm = false"
    >
      <div class="bg-white rounded-xl shadow-xl w-full max-w-xl p-6 max-h-[90vh] overflow-auto">
        <h3 class="text-lg font-semibold text-slate-800 mb-4">
          {{ editing ? "编辑档案" : "登记档案" }}
        </h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">档案名称 *</label>
            <input
              v-model="archiveForm.title"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">分类 *</label>
            <select
              v-model="archiveForm.category_id"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option :value="null">请选择</option>
              <option v-for="c in store.archiveCategories" :key="c.id" :value="c.id">
                {{ c.name }}（{{ c.code_prefix }}）
              </option>
            </select>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">存放位置</label>
              <input
                :value="archiveForm.location || '未选择档案盒'"
                readonly
                class="w-full px-4 py-2 border rounded-lg bg-slate-50 text-slate-700 focus:outline-none"
              />
              <p class="text-xs text-slate-400 mt-1">存放位置由档案盒自动带出，不能手动填写</p>
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">数量</label>
              <input
                v-model.number="archiveForm.quantity"
                type="number"
                min="1"
                class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">档案盒 *</label>
            <div class="flex items-center gap-3">
              <input
                :value="archiveForm.box_name || '未选择档案盒'"
                readonly
                placeholder="点击右侧按钮选择档案盒"
                class="flex-1 px-4 py-2 border rounded-lg bg-slate-50 text-slate-700 focus:outline-none"
              />
              <button
                type="button"
                @click="showBoxSelector = true"
                class="px-4 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition text-sm"
              >
                选择
              </button>
              <button
                v-if="aiEnabled"
                type="button"
                @click="showAiAnalyzer = true"
                :disabled="!archiveForm.title.trim()"
                :title="!archiveForm.title.trim() ? '请先填写档案名称' : 'AI 识别档案盒'"
                class="px-4 py-2 bg-purple-100 text-purple-700 rounded-lg hover:bg-purple-200 transition text-sm disabled:opacity-50 disabled:cursor-not-allowed"
              >
                AI 识别
              </button>
              <button
                v-if="archiveForm.archive_box_id"
                type="button"
                @click="clearSelectedBox"
                class="px-3 py-2 text-slate-400 hover:text-red-600 transition"
                title="清除选择"
              >
                ✕
              </button>
            </div>
            <p class="text-xs text-slate-400 mt-1">请到“档案盒维护”页面管理档案盒；电子文件将按档案盒名称分文件夹存放</p>
          </div>
          <div class="space-y-2">
            <label class="block text-sm font-medium text-slate-700">电子文件</label>
            <div class="flex items-center gap-3">
              <button
                type="button"
                @click="selectElectronicFile"
                :disabled="!archiveForm.archive_box_id"
                :title="!archiveForm.archive_box_id ? '请先选择档案盒' : ''"
                class="px-4 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition text-sm disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {{ editing ? "更换文件" : "选择文件" }}
              </button>
              <span class="text-sm text-slate-600 truncate max-w-xs">
                {{ fileNameFromPath(archiveForm.source_file_path) || fileNameFromPath(archiveForm.file_path) || "未选择文件" }}
              </span>
            </div>
            <p class="text-xs text-slate-400">文件将保存到数据库同目录下，按档案盒名称分文件夹存放；选择档案盒后才可选择文件</p>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">保管人</label>
            <select
              v-model="archiveForm.keeper_id"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option :value="null">请选择</option>
              <option v-for="m in store.members" :key="m.id" :value="m.id">{{ m.name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">备注</label>
            <textarea
              v-model="archiveForm.description"
              rows="3"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            ></textarea>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">标签</label>
            <div class="border rounded-lg p-3 max-h-40 overflow-y-auto space-y-1">
              <label
                v-for="tag in tagTreeFlat"
                :key="tag.id"
                class="flex items-center gap-2 text-sm text-slate-700 hover:bg-slate-50 px-2 py-1 rounded cursor-pointer"
                :style="{ paddingLeft: `${tag.depth * 16 + 8}px` }"
              >
                <input
                  type="checkbox"
                  :value="tag.id"
                  :checked="archiveForm.tag_ids.includes(tag.id)"
                  @change="toggleTag(tag.id)"
                  class="w-4 h-4"
                />
                {{ tag.name }}
              </label>
              <div v-if="tagTreeFlat.length === 0" class="text-sm text-slate-400 px-2">
                暂无标签，请先到“档案标签”页面添加
              </div>
            </div>
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
            @click="submitArchive"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
          >
            保存
          </button>
        </div>
      </div>
    </div>

    <ArchiveBoxSelector
      :show="showBoxSelector"
      :selected-id="archiveForm.archive_box_id"
      @select="onSelectBox"
      @close="showBoxSelector = false"
    />

    <ArchiveBoxAiAnalyzer
      :show="showAiAnalyzer"
      :title="archiveForm.title"
      :category-id="archiveForm.category_id"
      @select="onAiSelectBox"
      @close="showAiAnalyzer = false"
    />

    <!-- Borrow Form Modal -->
    <div
      v-if="showBorrowForm"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showBorrowForm = false"
    >
      <div class="bg-white rounded-xl shadow-xl w-full max-w-lg p-6">
        <h3 class="text-lg font-semibold text-slate-800 mb-4">
          借出档案：{{ selectedArchive?.archive?.title }}
        </h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">借阅人 *</label>
            <select
              v-model="borrowForm.borrower_id"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option :value="null">请选择</option>
              <option v-for="m in store.members" :key="m.id" :value="m.id">{{ m.name }}</option>
            </select>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">借阅日期 *</label>
              <input
                v-model="borrowForm.borrow_date"
                type="date"
                class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">应还日期 *</label>
              <input
                v-model="borrowForm.due_date"
                type="date"
                class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">借阅用途</label>
            <input
              v-model="borrowForm.purpose"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">审批人</label>
            <select
              v-model="borrowForm.approver_id"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option :value="null">请选择</option>
              <option v-for="m in store.members" :key="m.id" :value="m.id">{{ m.name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">备注</label>
            <textarea
              v-model="borrowForm.note"
              rows="2"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            ></textarea>
          </div>
        </div>
        <div class="mt-6 flex justify-end gap-3">
          <button
            @click="showBorrowForm = false"
            class="px-5 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition"
          >
            取消
          </button>
          <button
            @click="submitBorrow"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
          >
            确认借出
          </button>
        </div>
      </div>
    </div>

    <!-- History Modal -->
    <div
      v-if="showHistory"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showHistory = false"
    >
      <div class="bg-white rounded-xl shadow-xl w-full max-w-3xl p-6 max-h-[80vh] overflow-auto">
        <h3 class="text-lg font-semibold text-slate-800 mb-4">
          借阅历史：{{ selectedArchive?.archive?.title }}
        </h3>
        <div class="divide-y border rounded-lg">
          <div
            v-for="item in historyBorrows"
            :key="item.borrow.id"
            class="px-4 py-3 flex items-center justify-between"
          >
            <div>
              <div class="text-sm font-medium">{{ item.borrower.name }}</div>
              <div class="text-xs text-slate-500">
                {{ item.borrow.borrow_date }} 借出，应还 {{ item.borrow.due_date }}
                <span v-if="item.borrow.return_date">，归还 {{ item.borrow.return_date }}</span>
              </div>
            </div>
            <span :class="['px-2 py-0.5 text-xs rounded-full', borrowStatusClass(item.borrow.status)]">
              {{ borrowStatusLabel(item.borrow.status) }}
            </span>
          </div>
          <div v-if="historyBorrows.length === 0" class="px-4 py-6 text-center text-slate-400">
            无借阅记录
          </div>
        </div>
        <Pagination
          :page="historyPage"
          :per-page="historyPerPage"
          :total="historyTotal"
          @change="changeHistoryPage"
        />
        <div class="mt-4 flex justify-end">
          <button
            @click="showHistory = false"
            class="px-5 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition"
          >
            关闭
          </button>
        </div>
      </div>
    </div>

    <!-- Archive Table -->
    <div v-if="activeTab === 'archives'" class="bg-white rounded-xl shadow-sm border overflow-hidden">
      <template v-if="displayMode === 'normal'">
        <div class="table-scroll-wrapper">
        <table class="w-full text-left text-sm table-min-content">
          <thead class="bg-slate-50">
            <tr>
              <th class="px-4 py-3 font-medium text-slate-600 w-10">
                <input
                  type="checkbox"
                  :checked="filteredArchives.length > 0 && filteredArchives.every((a) => selectedArchiveIds.includes(a.archive.id))"
                  @change="toggleSelectAllArchives"
                  class="w-4 h-4"
                />
              </th>
              <th class="px-6 py-3 font-medium text-slate-600">档案编号</th>
              <th class="px-6 py-3 font-medium text-slate-600">名称</th>
              <th class="px-6 py-3 font-medium text-slate-600">档案盒</th>
              <th class="px-6 py-3 font-medium text-slate-600">分类</th>
              <th class="px-6 py-3 font-medium text-slate-600">标签</th>
              <th class="px-6 py-3 font-medium text-slate-600">位置</th>
              <th class="px-6 py-3 font-medium text-slate-600">保管人</th>
              <th class="px-6 py-3 font-medium text-slate-600">状态</th>
              <th class="px-6 py-3 font-medium text-slate-600">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y">
            <tr v-for="item in filteredArchives" :key="item.archive.id" class="hover:bg-slate-50">
              <td class="px-4 py-4">
                <input
                  type="checkbox"
                  :value="item.archive.id"
                  v-model="selectedArchiveIds"
                  class="w-4 h-4"
                />
              </td>
              <td class="px-6 py-4 font-mono text-slate-700">{{ item.archive.code }}</td>
              <td
                class="px-6 py-4 font-medium text-slate-800 truncate-cell"
                @dblclick="openArchiveDetail(item)"
              >
                {{ item.archive.title }}
              </td>
              <td
                class="px-6 py-4 text-slate-600 truncate-cell"
                @dblclick="openArchiveDetail(item)"
              >
                {{ item.archive.box_name || "-" }}
              </td>
              <td class="px-6 py-4 text-slate-600">{{ categoryName(item.archive.category_id) }}</td>
              <td
                class="px-6 py-4 text-slate-600 truncate-cell"
                @dblclick="openArchiveDetail(item)"
              >
                {{ tagNames(item.tags) }}
              </td>
              <td
                class="px-6 py-4 text-slate-600 truncate-cell"
                @dblclick="openArchiveDetail(item)"
              >
                {{ item.archive.location || "-" }}
              </td>
              <td class="px-6 py-4 text-slate-600">{{ memberName(item.archive.keeper_id) }}</td>
              <td class="px-6 py-4">
                <span :class="['px-2 py-1 text-xs rounded-full', statusClass(item.archive.status)]">
                  {{ statusLabel(item.archive.status) }}
                </span>
              </td>
              <td class="px-6 py-4 flex gap-2 flex-wrap">
                <button
                  @click="editArchive(item)"
                  class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
                >
                  编辑
                </button>
                <button
                  v-if="item.archive.status === 'in_stock'"
                  @click="openBorrowForm(item)"
                  class="px-3 py-1 text-xs bg-blue-50 text-blue-600 hover:bg-blue-100 rounded transition"
                >
                  借出
                </button>
                <button
                  @click="viewHistory(item)"
                  class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
                >
                  历史
                </button>
                <button
                  v-if="item.archive.file_path"
                  @click="openArchiveFile(item)"
                  class="px-3 py-1 text-xs bg-purple-50 text-purple-600 hover:bg-purple-100 rounded transition"
                >
                  打开文件
                </button>
                <select
                  @change="(e: any) => changeStatus(item, e.target.value)"
                  class="px-2 py-1 text-xs border rounded"
                >
                  <option value="" disabled selected>变更状态</option>
                  <option value="in_stock">在库</option>
                  <option value="damaged">损坏</option>
                  <option value="destroyed">销毁</option>
                </select>
                <button
                  @click="removeArchive(item.archive.id)"
                  class="px-3 py-1 text-xs bg-red-50 text-red-600 hover:bg-red-100 rounded transition"
                >
                  删除
                </button>
              </td>
            </tr>
            <tr v-if="filteredArchives.length === 0">
              <td colspan="10" class="px-6 py-8 text-center text-slate-400">暂无档案</td>
            </tr>
          </tbody>
        </table>
        </div>
        <Pagination
          :page="archivePage"
          :per-page="archivePerPage"
          :total="archiveTotal"
          @change="changeArchivePage"
        />
      </template>

      <template v-else>
        <div class="flex h-[calc(100vh-220px)]">
          <div class="w-64 border-r bg-slate-50 overflow-y-auto p-4">
            <h4 class="text-sm font-semibold text-slate-700 mb-3">标签</h4>
            <div class="space-y-1">
              <button
                v-for="tag in tagTreeFlat"
                :key="tag.id"
                @click="selectTag(tag.id)"
                :class="[
                  'w-full text-left text-sm px-3 py-2 rounded-lg transition',
                  selectedTagId === tag.id
                    ? 'bg-blue-600 text-white'
                    : 'text-slate-700 hover:bg-slate-200',
                ]"
                :style="{ paddingLeft: `${tag.depth * 16 + 12}px` }"
              >
                {{ tag.name }}
              </button>
              <div v-if="tagTreeFlat.length === 0" class="text-sm text-slate-400 px-3">
                暂无标签
              </div>
            </div>
          </div>
          <div class="flex-1 flex flex-col overflow-hidden">
            <div class="px-6 py-3 border-b bg-white text-sm text-slate-600">
              已选标签：<span class="font-medium text-slate-800">{{
                store.archiveTags.find((t) => t.id === selectedTagId)?.name || "未选择"
              }}</span>
            </div>
            <div class="flex-1 overflow-auto">
              <table class="w-full text-left text-sm table-min-content">
                <thead class="bg-slate-50">
                  <tr>
                    <th class="px-4 py-3 font-medium text-slate-600 w-10">
                      <input
                        type="checkbox"
                        :checked="tagTreeArchives.length > 0 && tagTreeArchives.every((a) => selectedArchiveIds.includes(a.archive.id))"
                        @change="toggleSelectAllArchives"
                        class="w-4 h-4"
                      />
                    </th>
                    <th class="px-6 py-3 font-medium text-slate-600">档案编号</th>
                    <th class="px-6 py-3 font-medium text-slate-600">名称</th>
                    <th class="px-6 py-3 font-medium text-slate-600">档案盒</th>
                    <th class="px-6 py-3 font-medium text-slate-600">分类</th>
                    <th class="px-6 py-3 font-medium text-slate-600">标签</th>
                    <th class="px-6 py-3 font-medium text-slate-600">位置</th>
                    <th class="px-6 py-3 font-medium text-slate-600">保管人</th>
                    <th class="px-6 py-3 font-medium text-slate-600">状态</th>
                    <th class="px-6 py-3 font-medium text-slate-600">操作</th>
                  </tr>
                </thead>
                <tbody class="divide-y">
                  <tr v-for="item in tagTreeArchives" :key="item.archive.id" class="hover:bg-slate-50">
                    <td class="px-4 py-4">
                      <input
                        type="checkbox"
                        :value="item.archive.id"
                        v-model="selectedArchiveIds"
                        class="w-4 h-4"
                      />
                    </td>
                    <td class="px-6 py-4 font-mono text-slate-700">{{ item.archive.code }}</td>
                    <td
                      class="px-6 py-4 font-medium text-slate-800 truncate-cell"
                      @dblclick="openArchiveDetail(item)"
                    >
                      {{ item.archive.title }}
                    </td>
                    <td
                      class="px-6 py-4 text-slate-600 truncate-cell"
                      @dblclick="openArchiveDetail(item)"
                    >
                      {{ item.archive.box_name || "-" }}
                    </td>
                    <td class="px-6 py-4 text-slate-600">{{ categoryName(item.archive.category_id) }}</td>
                    <td
                      class="px-6 py-4 text-slate-600 truncate-cell"
                      @dblclick="openArchiveDetail(item)"
                    >
                      {{ tagNames(item.tags) }}
                    </td>
                    <td
                      class="px-6 py-4 text-slate-600 truncate-cell"
                      @dblclick="openArchiveDetail(item)"
                    >
                      {{ item.archive.location || "-" }}
                    </td>
                    <td class="px-6 py-4 text-slate-600">{{ memberName(item.archive.keeper_id) }}</td>
                    <td class="px-6 py-4">
                      <span :class="['px-2 py-1 text-xs rounded-full', statusClass(item.archive.status)]">
                        {{ statusLabel(item.archive.status) }}
                      </span>
                    </td>
                    <td class="px-6 py-4 flex gap-2 flex-wrap">
                      <button
                        @click="editArchive(item)"
                        class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
                      >
                        编辑
                      </button>
                      <button
                        v-if="item.archive.status === 'in_stock'"
                        @click="openBorrowForm(item)"
                        class="px-3 py-1 text-xs bg-blue-50 text-blue-600 hover:bg-blue-100 rounded transition"
                      >
                        借出
                      </button>
                      <button
                        @click="viewHistory(item)"
                        class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
                      >
                        历史
                      </button>
                      <button
                        v-if="item.archive.file_path"
                        @click="openArchiveFile(item)"
                        class="px-3 py-1 text-xs bg-purple-50 text-purple-600 hover:bg-purple-100 rounded transition"
                      >
                        打开文件
                      </button>
                      <select
                        @change="(e: any) => changeStatus(item, e.target.value)"
                        class="px-2 py-1 text-xs border rounded"
                      >
                        <option value="" disabled selected>变更状态</option>
                        <option value="in_stock">在库</option>
                        <option value="damaged">损坏</option>
                        <option value="destroyed">销毁</option>
                      </select>
                      <button
                        @click="removeArchive(item.archive.id)"
                        class="px-3 py-1 text-xs bg-red-50 text-red-600 hover:bg-red-100 rounded transition"
                      >
                        删除
                      </button>
                    </td>
                  </tr>
                  <tr v-if="tagTreeArchives.length === 0">
                    <td colspan="10" class="px-6 py-8 text-center text-slate-400">
                      {{ selectedTagId ? "该标签下暂无档案" : "请选择一个标签" }}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <Pagination
              :page="tagTreePage"
              :per-page="tagTreePerPage"
              :total="tagTreeTotal"
              @change="changeTagTreePage"
            />
          </div>
        </div>
      </template>
    </div>

    <!-- Borrow Records -->
    <div v-if="activeTab === 'borrows'" class="bg-white rounded-xl shadow-sm border overflow-hidden">
      <div class="px-6 py-4 border-b bg-slate-50 flex flex-wrap justify-between items-center gap-3">
        <h3 class="font-semibold text-slate-800">借还记录</h3>
        <select
          v-model="borrowStatusFilter"
          @change="borrowPage = 1; loadBorrows()"
          class="px-3 py-1.5 text-sm border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="">全部记录</option>
          <option value="borrowed">借阅中</option>
          <option value="returned">已归还</option>
          <option value="overdue">逾期</option>
        </select>
      </div>
      <div class="table-scroll-wrapper">
      <table class="w-full text-left text-sm table-min-content">
        <thead class="bg-slate-50">
          <tr>
            <th class="px-6 py-3 font-medium text-slate-600">档案编号</th>
            <th class="px-6 py-3 font-medium text-slate-600">档案名称</th>
            <th class="px-6 py-3 font-medium text-slate-600">借阅人</th>
            <th class="px-6 py-3 font-medium text-slate-600">借阅日期</th>
            <th class="px-6 py-3 font-medium text-slate-600">应还日期</th>
            <th class="px-6 py-3 font-medium text-slate-600">归还日期</th>
            <th class="px-6 py-3 font-medium text-slate-600">状态</th>
            <th class="px-6 py-3 font-medium text-slate-600">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          <tr v-for="item in store.archiveBorrows" :key="item.borrow.id" class="hover:bg-slate-50">
            <td class="px-6 py-4 font-mono text-slate-700">{{ item.archive.archive.code }}</td>
            <td
              class="px-6 py-4 font-medium text-slate-800 truncate-cell"
              @dblclick="openBorrowDetail(item)"
            >
              {{ item.archive.archive.title }}
            </td>
            <td class="px-6 py-4 text-slate-600">{{ item.borrower.name }}</td>
            <td class="px-6 py-4 text-slate-600">{{ item.borrow.borrow_date }}</td>
            <td class="px-6 py-4 text-slate-600">{{ item.borrow.due_date }}</td>
            <td class="px-6 py-4 text-slate-600">{{ item.borrow.return_date || "-" }}</td>
            <td class="px-6 py-4">
              <span :class="['px-2 py-1 text-xs rounded-full', borrowStatusClass(item.borrow.status)]">
                {{ borrowStatusLabel(item.borrow.status) }}
              </span>
            </td>
            <td class="px-6 py-4 flex gap-2 flex-wrap">
              <button
                v-if="item.borrow.status !== 'returned'"
                @click="returnBorrow(item)"
                class="px-3 py-1 text-xs bg-green-600 text-white hover:bg-green-700 rounded transition"
              >
                归还
              </button>
              <button
                @click="openBorrowEditForm(item)"
                class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
              >
                编辑
              </button>
              <button
                @click="removeBorrow(item)"
                class="px-3 py-1 text-xs bg-red-50 text-red-600 hover:bg-red-100 rounded transition"
              >
                删除
              </button>
            </td>
          </tr>
          <tr v-if="store.archiveBorrows.length === 0">
            <td colspan="8" class="px-6 py-8 text-center text-slate-400">当前没有借还记录</td>
          </tr>
        </tbody>
      </table>
      </div>
      <Pagination
        :page="borrowPage"
        :per-page="borrowPerPage"
        :total="borrowTotal"
        @change="changeBorrowPage"
      />
    </div>
  </div>

  <!-- Borrow Edit Modal -->
  <div
    v-if="showBorrowEditForm"
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    @click.self="showBorrowEditForm = false"
  >
    <div class="bg-white rounded-xl shadow-xl w-full max-w-lg p-6 max-h-[90vh] overflow-auto">
      <h3 class="text-lg font-semibold text-slate-800 mb-4">编辑借还记录</h3>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-1">借阅人 *</label>
          <select
            v-model="borrowEditForm.borrower_id"
            class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option :value="null">请选择</option>
            <option v-for="m in store.members" :key="m.id" :value="m.id">{{ m.name }}</option>
          </select>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">借阅日期 *</label>
            <input
              v-model="borrowEditForm.borrow_date"
              type="date"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">应还日期 *</label>
            <input
              v-model="borrowEditForm.due_date"
              type="date"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">归还日期</label>
            <input
              v-model="borrowEditForm.return_date"
              type="date"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">状态 *</label>
            <select
              v-model="borrowEditForm.status"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="borrowed">借阅中</option>
              <option value="returned">已归还</option>
              <option value="overdue">逾期</option>
            </select>
          </div>
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-1">借阅用途</label>
          <input
            v-model="borrowEditForm.purpose"
            class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-1">审批人</label>
          <select
            v-model="borrowEditForm.approver_id"
            class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option :value="null">请选择</option>
            <option v-for="m in store.members" :key="m.id" :value="m.id">{{ m.name }}</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-1">备注</label>
          <textarea
            v-model="borrowEditForm.note"
            rows="2"
            class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          ></textarea>
        </div>
      </div>
      <div class="mt-6 flex justify-end gap-3">
        <button
          @click="showBorrowEditForm = false"
          class="px-5 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition"
        >
          取消
        </button>
        <button
          @click="submitBorrowEdit"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
        >
          保存
        </button>
      </div>
    </div>
  </div>

  <RecordDetailModal
    :show="showPreview"
    :title="previewTitle"
    :fields="previewFields"
    @close="showPreview = false"
  />
</template>
