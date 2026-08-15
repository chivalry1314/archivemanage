<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAppStore } from "../stores/app";
import {
  createTask,
  deleteTask,
  listTasks,
  updateTask,
  listTaskInstances,
  completeInstance,
  uncompleteInstance,
  exportInstancesCsv,
  exportInstancesJson,
  exportMemberStatsCsv,
} from "../api";
import Pagination from "../components/Pagination.vue";
import RecordDetailModal from "../components/RecordDetailModal.vue";
import { showError } from "../utils/error";
import { confirm } from "../utils/dialog";
import { useRecordPreview } from "../utils/recordPreview";

const store = useAppStore();
const { previewFields, previewTitle, showPreview, openRecord } = useRecordPreview();
const showForm = ref(false);

const openTaskDetail = (item: any) => {
  openRecord("任务详情", [
    { label: "任务名称", value: item.task.title },
    { label: "任务描述", value: item.task.description || "-" },
    { label: "周期", value: cycleLabel(item.task.cycle_type) },
    { label: "提醒日期", value: `${item.task.cycle_day}号` },
    { label: "开始日期", value: item.task.start_date },
    { label: "结束日期", value: item.task.end_date || "无" },
    { label: "被指派人", value: item.assignees.map((m: any) => m.name).join("、") || "（未指派）" },
    { label: "提醒时间", value: `提前 ${item.task.reminder_minutes} 分钟` },
    { label: "声音提醒", value: item.task.sound_enabled ? "开启" : "关闭" },
  ]);
};
const editing = ref(false);
const form = ref({
  id: 0,
  title: "",
  description: "",
  cycle_type: "monthly",
  cycle_day: 1,
  start_date: "",
  end_date: "",
  reminder_minutes: 15,
  sound_enabled: true,
  assignee_ids: [] as number[],
});

const instances = ref<any[]>([]);
const selectedTaskId = ref<number | null>(null);

const currentPage = ref(1);
const total = ref(0);
const perPage = ref(10);

const instancePage = ref(1);
const instanceTotal = ref(0);
const instancePerPage = ref(10);
const taskExportStatus = ref("");

const cycleOptions = [
  { value: "monthly", label: "每月" },
  { value: "quarterly", label: "每季度" },
  { value: "halfyearly", label: "每半年" },
  { value: "yearly", label: "每年" },
];

const getPageSize = () =>
  Math.max(5, Math.min(100, parseInt(localStorage.getItem("pageSize") || "10", 10) || 10));

const resetForm = () => {
  form.value = {
    id: 0,
    title: "",
    description: "",
    cycle_type: "monthly",
    cycle_day: 1,
    start_date: new Date().toISOString().split("T")[0],
    end_date: "",
    reminder_minutes: 15,
    sound_enabled: true,
    assignee_ids: [],
  };
  editing.value = false;
};

const load = async () => {
  perPage.value = getPageSize();
  const result = await listTasks(currentPage.value, perPage.value);
  store.tasks = result.items;
  total.value = result.total;
  if (result.items.length === 0 && currentPage.value > 1) {
    currentPage.value--;
    await load();
  }
};

const submit = async () => {
  if (!form.value.title.trim() || !form.value.start_date) return;

  const payload = {
    title: form.value.title.trim(),
    description: form.value.description.trim() || undefined,
    cycle_type: form.value.cycle_type,
    cycle_day: form.value.cycle_day,
    start_date: form.value.start_date,
    end_date: form.value.end_date || undefined,
    reminder_minutes: form.value.reminder_minutes,
    sound_enabled: form.value.sound_enabled,
    assignee_ids: form.value.assignee_ids,
  };

  try {
    if (editing.value) {
      await updateTask({ id: form.value.id, ...payload } as any);
    } else {
      await createTask(payload as any);
    }

    resetForm();
    showForm.value = false;
    currentPage.value = 1;
    await load();
  } catch (e) {
    showError(e);
  }
};

const editTask = (task: any) => {
  form.value = {
    id: task.task.id,
    title: task.task.title,
    description: task.task.description || "",
    cycle_type: task.task.cycle_type,
    cycle_day: task.task.cycle_day,
    start_date: task.task.start_date,
    end_date: task.task.end_date || "",
    reminder_minutes: task.task.reminder_minutes,
    sound_enabled: task.task.sound_enabled,
    assignee_ids: task.assignees.map((m: any) => m.id),
  };
  editing.value = true;
  showForm.value = true;
};

const removeTask = async (id: number) => {
  if (!(await confirm("确定要删除该任务吗？所有历史实例也会被删除。"))) return;
  try {
    await deleteTask(id);
    await load();
  } catch (e) {
    showError(e);
  }
};

const viewInstances = async (taskId: number) => {
  selectedTaskId.value = taskId;
  instancePage.value = 1;
  await loadInstances();
};

const loadInstances = async () => {
  if (selectedTaskId.value === null) return;
  instancePerPage.value = getPageSize();
  const result = await listTaskInstances(
    selectedTaskId.value,
    instancePage.value,
    instancePerPage.value
  );
  instances.value = result.items;
  instanceTotal.value = result.total;
  if (result.items.length === 0 && instancePage.value > 1) {
    instancePage.value--;
    await loadInstances();
  }
};

const toggleInstance = async (item: any) => {
  try {
    if (item.instance.status === "completed") {
      await uncompleteInstance(item.instance.id);
    } else {
      await completeInstance(item.instance.id);
    }
    await loadInstances();
  } catch (e) {
    showError(e);
  }
};

const changePage = (page: number) => {
  currentPage.value = page;
  load();
};

const changeInstancePage = (page: number) => {
  instancePage.value = page;
  loadInstances();
};

const downloadFile = (content: string, filename: string, type: string) => {
  const blob = new Blob([content], { type });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

const exportInstances = async () => {
  try {
    const csv = await exportInstancesCsv();
    downloadFile(csv, `任务实例_${new Date().toISOString().split("T")[0]}.csv`, "text/csv;charset=utf-8;");
    taskExportStatus.value = "任务实例导出成功";
    setTimeout(() => (taskExportStatus.value = ""), 3000);
  } catch (e) {
    showError(e);
  }
};

const exportStats = async () => {
  try {
    const csv = await exportMemberStatsCsv();
    downloadFile(csv, `人员统计_${new Date().toISOString().split("T")[0]}.csv`, "text/csv;charset=utf-8;");
    taskExportStatus.value = "人员统计导出成功";
    setTimeout(() => (taskExportStatus.value = ""), 3000);
  } catch (e) {
    showError(e);
  }
};

const exportJson = async () => {
  try {
    const json = await exportInstancesJson();
    downloadFile(json, `任务数据_${new Date().toISOString().split("T")[0]}.json`, "application/json");
    taskExportStatus.value = "JSON 导出成功";
    setTimeout(() => (taskExportStatus.value = ""), 3000);
  } catch (e) {
    showError(e);
  }
};

const cycleLabel = (type: string) => {
  const map: Record<string, string> = {
    monthly: "每月",
    quarterly: "每季度",
    halfyearly: "每半年",
    yearly: "每年",
  };
  return map[type] || type;
};

const statusLabel = (status: string) => {
  const map: Record<string, string> = {
    pending: "待办",
    completed: "已完成",
    overdue: "逾期",
  };
  return map[status] || status;
};

const statusClass = (status: string) => {
  const map: Record<string, string> = {
    pending: "bg-amber-100 text-amber-700",
    completed: "bg-green-100 text-green-700",
    overdue: "bg-red-100 text-red-700",
  };
  return map[status] || "bg-gray-100 text-gray-700";
};

onMounted(() => {
  resetForm();
  load();
});
</script>

<template>
  <div class="space-y-6">
    <div class="flex flex-wrap justify-between items-center gap-3">
      <div v-if="taskExportStatus" class="text-sm text-blue-600">
        {{ taskExportStatus }}
      </div>
      <div class="flex-1"></div>
      <div class="flex flex-wrap gap-2">
        <button
          @click="exportInstances"
          class="px-4 py-2.5 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition text-sm"
        >
          导出任务实例
        </button>
        <button
          @click="exportStats"
          class="px-4 py-2.5 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition text-sm"
        >
          导出人员统计
        </button>
        <button
          @click="exportJson"
          class="px-4 py-2.5 bg-slate-600 text-white rounded-lg hover:bg-slate-700 transition text-sm"
        >
          导出 JSON
        </button>
        <button
          @click="showForm = true; resetForm();"
          class="px-5 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
        >
          + 新建任务
        </button>
      </div>
    </div>

    <!-- Form Modal -->
    <div
      v-if="showForm"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showForm = false"
    >
      <div class="bg-white rounded-xl shadow-xl w-full max-w-2xl p-6 max-h-[90vh] overflow-auto">
        <h3 class="text-lg font-semibold text-slate-800 mb-4">
          {{ editing ? "编辑任务" : "新建任务" }}
        </h3>

        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">任务名称 *</label>
            <input
              v-model="form.title"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="例如：月度考核表提交"
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-slate-700 mb-1">任务说明</label>
            <textarea
              v-model="form.description"
              rows="2"
              class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            ></textarea>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">周期 *</label>
              <select
                v-model="form.cycle_type"
                class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option v-for="opt in cycleOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">每月几号 *</label>
              <input
                v-model.number="form.cycle_day"
                type="number"
                min="1"
                max="31"
                class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">开始日期 *</label>
              <input
                v-model="form.start_date"
                type="date"
                class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">结束日期（可选）</label>
              <input
                v-model="form.end_date"
                type="date"
                class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-slate-700 mb-1">提前提醒（分钟）</label>
              <input
                v-model.number="form.reminder_minutes"
                type="number"
                min="0"
                class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div class="flex items-center gap-2 pt-7">
              <input
                id="sound"
                v-model="form.sound_enabled"
                type="checkbox"
                class="w-4 h-4"
              />
              <label for="sound" class="text-sm text-slate-700">提醒时播放声音</label>
            </div>
          </div>

          <div>
            <label class="block text-sm font-medium text-slate-700 mb-2">指派人员（可选）</label>
            <div class="flex flex-wrap gap-2">
              <label
                v-for="m in store.members"
                :key="m.id"
                class="flex items-center gap-1 px-3 py-1.5 border rounded-lg cursor-pointer hover:bg-slate-50"
              >
                <input
                  type="checkbox"
                  :value="m.id"
                  v-model="form.assignee_ids"
                  class="w-4 h-4"
                />
                <span class="text-sm">{{ m.name }}</span>
              </label>
              <span v-if="store.members.length === 0" class="text-sm text-slate-400">
                暂无人员，请先到“人员管理”添加
              </span>
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
            @click="submit"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
          >
            保存
          </button>
        </div>
      </div>
    </div>

    <!-- Task List -->
    <div class="bg-white rounded-xl shadow-sm border overflow-hidden">
      <div class="table-scroll-wrapper">
      <table class="w-full text-left text-sm table-min-content">
        <thead class="bg-slate-50">
          <tr>
            <th class="px-6 py-3 font-medium text-slate-600">任务名称</th>
            <th class="px-6 py-3 font-medium text-slate-600">周期</th>
            <th class="px-6 py-3 font-medium text-slate-600">日期</th>
            <th class="px-6 py-3 font-medium text-slate-600">被指派人</th>
            <th class="px-6 py-3 font-medium text-slate-600">提醒</th>
            <th class="px-6 py-3 font-medium text-slate-600">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y">
          <tr v-for="item in store.tasks" :key="item.task.id" class="hover:bg-slate-50">
            <td class="px-6 py-4">
              <div
                class="font-medium text-slate-800 truncate-cell"
                @dblclick="openTaskDetail(item)"
              >
                {{ item.task.title }}
              </div>
              <div
                v-if="item.task.description"
                class="text-xs text-slate-400 mt-1 truncate-cell"
                @dblclick="openTaskDetail(item)"
              >
                {{ item.task.description }}
              </div>
            </td>
            <td class="px-6 py-4 text-slate-600">{{ cycleLabel(item.task.cycle_type) }}</td>
            <td class="px-6 py-4 text-slate-600">{{ item.task.cycle_day }}号</td>
            <td
              class="px-6 py-4 text-slate-600 truncate-cell"
              @dblclick="openTaskDetail(item)"
            >
              {{ item.assignees.map((m) => m.name).join("、") || "（未指派）" }}
            </td>
            <td class="px-6 py-4 text-slate-600">
              提前 {{ item.task.reminder_minutes }} 分钟
              {{ item.task.sound_enabled ? "· 声音" : "" }}
            </td>
            <td class="px-6 py-4 flex gap-2">
              <button
                @click="editTask(item)"
                class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
              >
                编辑
              </button>
              <button
                @click="viewInstances(item.task.id)"
                class="px-3 py-1 text-xs bg-blue-50 text-blue-600 hover:bg-blue-100 rounded transition"
              >
                实例
              </button>
              <button
                @click="removeTask(item.task.id)"
                class="px-3 py-1 text-xs bg-red-50 text-red-600 hover:bg-red-100 rounded transition"
              >
                删除
              </button>
            </td>
          </tr>
          <tr v-if="store.tasks.length === 0">
            <td colspan="6" class="px-6 py-8 text-center text-slate-400">暂无任务</td>
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

    <!-- Instances Modal -->
    <div
      v-if="selectedTaskId !== null"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="selectedTaskId = null"
    >
      <div class="bg-white rounded-xl shadow-xl w-full max-w-3xl p-6 max-h-[80vh] overflow-auto">
        <h3 class="text-lg font-semibold text-slate-800 mb-4">任务实例</h3>
        <div class="divide-y border rounded-lg">
          <div
            v-for="item in instances"
            :key="item.instance.id"
            class="px-4 py-3 flex items-center justify-between hover:bg-slate-50"
          >
            <div>
              <span class="text-sm text-slate-600">{{ item.instance.due_date }}</span>
              <span
                :class="['ml-3 px-2 py-0.5 text-xs rounded-full', statusClass(item.instance.status)]"
              >
                {{ statusLabel(item.instance.status) }}
              </span>
            </div>
            <button
              v-if="item.instance.status !== 'completed'"
              @click="toggleInstance(item)"
              class="px-3 py-1 text-xs bg-green-600 text-white rounded hover:bg-green-700 transition"
            >
              完成
            </button>
            <button
              v-else
              @click="toggleInstance(item)"
              class="px-3 py-1 text-xs bg-slate-100 text-slate-600 rounded hover:bg-slate-200 transition"
            >
              撤销
            </button>
          </div>
          <div v-if="instances.length === 0" class="px-4 py-6 text-center text-slate-400">
            暂无实例
          </div>
        </div>
        <Pagination
          :page="instancePage"
          :per-page="instancePerPage"
          :total="instanceTotal"
          @change="changeInstancePage"
        />
        <div class="mt-4 flex justify-end">
          <button
            @click="selectedTaskId = null"
            class="px-5 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition"
          >
            关闭
          </button>
        </div>
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
