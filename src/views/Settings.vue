<script setup lang="ts">
import { ref, onMounted } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  exportArchiveBorrowsCsv,
  exportArchiveBorrowsXlsx,
  exportArchivesCsv,
  exportArchivesXlsx,
  exportInstancesCsv,
  exportInstancesJson,
  exportMemberStatsCsv,
  getAiConfig,
  getDbPath,
  getMobileServerStatus,
  importArchivesFromExcel,
  listAiModels,
  saveFile,
  setAiConfig,
  setDbPath,
  startMobileServer,
  stopMobileServer,
  type AiConfig,
  type ServerStatus,
} from "../api";
import { showError } from "../utils/error";
import * as XLSX from "xlsx";

const dbPath = ref("");
const exportStatus = ref("");
const importStatus = ref("");
const migrateData = ref(true);
const dbStatus = ref("");
const pageSize = ref(10);
const settingsStatus = ref("");

const mobilePort = ref(8421);
const mobileStatus = ref<ServerStatus | null>(null);
const mobileLoading = ref(false);
const mobileMessage = ref("");
const archivesExportFormat = ref<"csv" | "xlsx">("csv");
const borrowsExportFormat = ref<"csv" | "xlsx">("csv");

const aiConfig = ref<AiConfig>({
  enabled: false,
  base_url: "https://api.siliconflow.cn/v1",
  model: "Qwen/Qwen2.5-7B-Instruct",
  api_key: "",
});
const aiStatus = ref("");
const aiModels = ref<string[]>([]);
const aiModelsLoading = ref(false);

onMounted(async () => {
  dbPath.value = await getDbPath();
  const saved = localStorage.getItem("pageSize");
  if (saved) {
    pageSize.value = Math.max(5, Math.min(100, parseInt(saved, 10) || 10));
  }
  const savedPort = localStorage.getItem("mobilePort");
  if (savedPort) {
    mobilePort.value = Math.max(1024, Math.min(65535, parseInt(savedPort, 10) || 8421));
  }
  try {
    mobileStatus.value = await getMobileServerStatus();
  } catch (e) {
    // ignore
  }
  try {
    aiConfig.value = await getAiConfig();
  } catch (e) {
    // ignore
  }
});

const savePageSize = () => {
  const value = Math.max(5, Math.min(100, pageSize.value));
  pageSize.value = value;
  localStorage.setItem("pageSize", String(value));
  settingsStatus.value = "每页条数已保存";
  setTimeout(() => (settingsStatus.value = ""), 3000);
};

const saveAiConfig = async () => {
  try {
    await setAiConfig(aiConfig.value);
    aiStatus.value = "AI 配置已保存";
    setTimeout(() => (aiStatus.value = ""), 3000);
  } catch (e) {
    showError(e);
  }
};

const fetchAiModels = async () => {
  if (!aiConfig.value.api_key.trim()) {
    aiStatus.value = "请先填写 API Key";
    return;
  }
  aiModelsLoading.value = true;
  try {
    aiModels.value = await listAiModels(aiConfig.value);
    aiStatus.value = `已获取 ${aiModels.value.length} 个模型`;
    setTimeout(() => (aiStatus.value = ""), 3000);
  } catch (e) {
    showError(e);
  } finally {
    aiModelsLoading.value = false;
  }
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

const changeDbPath = async () => {
  try {
    const path = await save({
      filters: [{ name: "SQLite", extensions: ["db"] }],
      defaultPath: "archivemanage.db",
    });
    if (!path) return;

    dbStatus.value = "正在切换数据库路径...";
    await setDbPath(path as string, migrateData.value);
    dbStatus.value = "数据库路径已切换，即将自动重启...";
    setTimeout(() => relaunch(), 800);
  } catch (e) {
    showError(e);
    dbStatus.value = "";
  }
};

const importArchives = async () => {
  try {
    const path = await open({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      directory: false,
      multiple: false,
    });
    if (!path) return;

    importStatus.value = "正在导入，请稍候...";
    const [archiveCount, tagCount] = await importArchivesFromExcel(path as string);
    importStatus.value = `导入完成：新增 ${archiveCount} 个档案，${tagCount} 个标签`;
    setTimeout(() => (importStatus.value = ""), 5000);
  } catch (e) {
    showError(e);
    importStatus.value = "";
  }
};

const downloadImportTemplate = async () => {
  try {
    const path = await save({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      defaultPath: "档案导入模板.xlsx",
    });
    if (!path) return;

    const data = [
      ["具体材料", "档案盒名称", "标签"],
      ["1号楼业主资料", "1号楼业主盒", "重要合同"],
      ["消防设备清单", "消防设备盒", "2026年度,设备"],
      ["停车场租赁合同", "合同档案盒", "重要合同、租赁"],
      ["电梯维保记录", "电梯档案盒", ""],
    ];
    const worksheet = XLSX.utils.aoa_to_sheet(data);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "Sheet1");
    const arrayBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
    const uint8Array = new Uint8Array(arrayBuffer);

    await saveFile(path as string, uint8Array);
    importStatus.value = "模板已保存";
    setTimeout(() => (importStatus.value = ""), 3000);
  } catch (e) {
    showError(e);
  }
};

const exportInstances = async () => {
  const csv = await exportInstancesCsv();
  downloadFile(csv, `任务实例_${new Date().toISOString().split("T")[0]}.csv`, "text/csv;charset=utf-8;");
  exportStatus.value = "任务实例导出成功";
  setTimeout(() => (exportStatus.value = ""), 3000);
};

const exportJson = async () => {
  const json = await exportInstancesJson();
  downloadFile(json, `任务数据_${new Date().toISOString().split("T")[0]}.json`, "application/json");
  exportStatus.value = "JSON 导出成功";
  setTimeout(() => (exportStatus.value = ""), 3000);
};

const exportStats = async () => {
  const csv = await exportMemberStatsCsv();
  downloadFile(csv, `人员统计_${new Date().toISOString().split("T")[0]}.csv`, "text/csv;charset=utf-8;");
  exportStatus.value = "人员统计导出成功";
  setTimeout(() => (exportStatus.value = ""), 3000);
};

const exportArchives = async () => {
  const dateStr = new Date().toISOString().split("T")[0];
  if (archivesExportFormat.value === "xlsx") {
    const path = await save({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      defaultPath: `档案台账_${dateStr}.xlsx`,
    });
    if (!path) return;
    const bytes = await exportArchivesXlsx();
    await saveFile(path as string, new Uint8Array(bytes));
  } else {
    const csv = await exportArchivesCsv();
    downloadFile(csv, `档案台账_${dateStr}.csv`, "text/csv;charset=utf-8;");
  }
  exportStatus.value = "档案台账导出成功";
  setTimeout(() => (exportStatus.value = ""), 3000);
};

const exportArchiveBorrows = async () => {
  const dateStr = new Date().toISOString().split("T")[0];
  if (borrowsExportFormat.value === "xlsx") {
    const path = await save({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      defaultPath: `档案借还记录_${dateStr}.xlsx`,
    });
    if (!path) return;
    const bytes = await exportArchiveBorrowsXlsx();
    await saveFile(path as string, new Uint8Array(bytes));
  } else {
    const csv = await exportArchiveBorrowsCsv();
    downloadFile(csv, `档案借还记录_${dateStr}.csv`, "text/csv;charset=utf-8;");
  }
  exportStatus.value = "档案借还记录导出成功";
  setTimeout(() => (exportStatus.value = ""), 3000);
};

const toggleMobileServer = async () => {
  mobileLoading.value = true;
  mobileMessage.value = "";
  try {
    if (mobileStatus.value?.running) {
      mobileStatus.value = await stopMobileServer();
      mobileMessage.value = "手机搜索服务已停止";
    } else {
      const port = Math.max(1024, Math.min(65535, mobilePort.value));
      mobilePort.value = port;
      localStorage.setItem("mobilePort", String(port));
      mobileStatus.value = await startMobileServer(port);
      if (mobileStatus.value.error) {
        mobileMessage.value = mobileStatus.value.error;
      } else {
        mobileMessage.value = `服务已开启，请用手机浏览器访问：${mobileStatus.value.url}`;
      }
    }
  } catch (e) {
    showError(e);
  } finally {
    mobileLoading.value = false;
  }
};

const copyMobileUrl = async () => {
  if (!mobileStatus.value?.url) return;
  try {
    await navigator.clipboard.writeText(mobileStatus.value.url);
    mobileMessage.value = "链接已复制到剪贴板";
    setTimeout(() => (mobileMessage.value = ""), 3000);
  } catch (e) {
    showError(e);
  }
};
</script>

<template>
  <div class="max-w-3xl space-y-6">
    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">任务数据导出</h3>
      <p class="text-sm text-slate-500 mb-4">
        将任务数据导出为 CSV 或 JSON 文件，方便备份和统计分析。
      </p>
      <div class="flex flex-wrap gap-3">
        <button
          @click="exportInstances"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
        >
          导出任务实例 (CSV)
        </button>
        <button
          @click="exportStats"
          class="px-5 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition"
        >
          导出人员统计 (CSV)
        </button>
        <button
          @click="exportJson"
          class="px-5 py-2 bg-slate-600 text-white rounded-lg hover:bg-slate-700 transition"
        >
          导出 JSON
        </button>
      </div>
    </div>

    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">档案数据导出</h3>
      <p class="text-sm text-slate-500 mb-4">
        将档案台账和借还记录导出为 CSV 或 Excel 文件。
      </p>
      <div class="space-y-4">
        <div class="flex flex-wrap items-center gap-3">
          <span class="text-sm text-slate-600 w-24">档案台账</span>
          <div class="flex items-center bg-slate-100 rounded-lg p-1">
            <button
              @click="archivesExportFormat = 'csv'"
              :class="[
                'px-3 py-1.5 text-sm rounded-md transition',
                archivesExportFormat === 'csv'
                  ? 'bg-white text-slate-800 shadow-sm'
                  : 'text-slate-600 hover:bg-slate-200',
              ]"
            >
              CSV
            </button>
            <button
              @click="archivesExportFormat = 'xlsx'"
              :class="[
                'px-3 py-1.5 text-sm rounded-md transition',
                archivesExportFormat === 'xlsx'
                  ? 'bg-white text-slate-800 shadow-sm'
                  : 'text-slate-600 hover:bg-slate-200',
              ]"
            >
              Excel
            </button>
          </div>
          <button
            @click="exportArchives"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
          >
            导出档案台账
          </button>
        </div>
        <div class="flex flex-wrap items-center gap-3">
          <span class="text-sm text-slate-600 w-24">借还记录</span>
          <div class="flex items-center bg-slate-100 rounded-lg p-1">
            <button
              @click="borrowsExportFormat = 'csv'"
              :class="[
                'px-3 py-1.5 text-sm rounded-md transition',
                borrowsExportFormat === 'csv'
                  ? 'bg-white text-slate-800 shadow-sm'
                  : 'text-slate-600 hover:bg-slate-200',
              ]"
            >
              CSV
            </button>
            <button
              @click="borrowsExportFormat = 'xlsx'"
              :class="[
                'px-3 py-1.5 text-sm rounded-md transition',
                borrowsExportFormat === 'xlsx'
                  ? 'bg-white text-slate-800 shadow-sm'
                  : 'text-slate-600 hover:bg-slate-200',
              ]"
            >
              Excel
            </button>
          </div>
          <button
            @click="exportArchiveBorrows"
            class="px-5 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition"
          >
            导出借还记录
          </button>
        </div>
      </div>
    </div>

    <div v-if="exportStatus" class="bg-green-50 text-green-700 px-6 py-3 rounded-lg text-sm">
      {{ exportStatus }}
    </div>

    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">档案导入</h3>
      <p class="text-sm text-slate-500 mb-4">
        从 Excel 导入档案台账。要求 Sheet1 第一行为表头（具体材料、档案盒名称、标签），第二行起为数据。
        未指定材料的标签也会创建；未指定分类的档案归入“未知分类”，保管人默认为“未知保管人”。
      </p>
      <div class="flex flex-wrap gap-3">
        <button
          @click="importArchives"
          class="px-5 py-2 bg-amber-600 text-white rounded-lg hover:bg-amber-700 transition"
        >
          选择 Excel 文件导入
        </button>
        <button
          @click="downloadImportTemplate"
          class="px-5 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition"
        >
          下载导入模板
        </button>
      </div>
      <div v-if="importStatus" class="mt-3 text-sm text-green-600">
        {{ importStatus }}
      </div>
    </div>

    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">列表分页设置</h3>
      <p class="text-sm text-slate-500 mb-4">
        设置各列表页面每页显示的条数，范围 5~100。
      </p>
      <div class="flex items-center gap-3">
        <input
          v-model.number="pageSize"
          type="number"
          min="5"
          max="100"
          class="w-24 px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          @click="savePageSize"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
        >
          保存
        </button>
      </div>
      <div v-if="settingsStatus" class="mt-3 text-sm text-green-600">
        {{ settingsStatus }}
      </div>
    </div>

    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">档案盒 AI 识别</h3>
      <p class="text-sm text-slate-500 mb-4">
        开启后，登记档案时可以使用 AI 根据档案标题和分类自动推荐最合适的档案盒。支持硅基流动等 OpenAI 兼容 API。
      </p>
      <div class="space-y-4">
        <label class="flex items-center gap-2 text-sm text-slate-700">
          <input
            type="checkbox"
            v-model="aiConfig.enabled"
            class="w-4 h-4"
          />
          启用档案盒 AI 识别
        </label>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-1">API 地址</label>
          <input
            v-model="aiConfig.base_url"
            placeholder="https://api.siliconflow.cn/v1"
            class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-1">API Key</label>
          <input
            v-model="aiConfig.api_key"
            type="password"
            placeholder="sk-..."
            class="w-full px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <p class="text-xs text-slate-400 mt-1">API Key 仅保存在本地 config.json 中，请妥善保管。</p>
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-700 mb-1">模型名</label>
          <div class="flex items-center gap-3">
            <select
              v-model="aiConfig.model"
              class="flex-1 px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="" disabled>请选择或手动获取模型</option>
              <option
                v-if="aiConfig.model && !aiModels.includes(aiConfig.model)"
                :value="aiConfig.model"
              >
                {{ aiConfig.model }}
              </option>
              <option v-for="m in aiModels" :key="m" :value="m">{{ m }}</option>
            </select>
            <button
              type="button"
              @click="fetchAiModels"
              :disabled="aiModelsLoading"
              class="px-4 py-2 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition text-sm disabled:opacity-50"
            >
              {{ aiModelsLoading ? "获取中..." : "获取模型" }}
            </button>
          </div>
          <p class="text-xs text-slate-400 mt-1">填写 API Key 和 API 地址后，点击“获取模型”即可从服务商拉取可用模型列表。</p>
        </div>
        <button
          @click="saveAiConfig"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
        >
          保存 AI 配置
        </button>
      </div>
      <div v-if="aiStatus" class="mt-3 text-sm text-green-600">
        {{ aiStatus }}
      </div>
    </div>

    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">数据备份与存储位置</h3>
      <p class="text-sm text-slate-500 mb-4">
        数据库文件当前路径如下。你可以直接复制该文件进行备份，也可以修改存储位置。
        修改路径时可以选择是否迁移现有数据；若不迁移，则在新位置创建一个空数据库。
      </p>
      <div class="bg-slate-100 rounded-lg px-4 py-3 text-sm text-slate-700 break-all font-mono mb-4">
        {{ dbPath }}
      </div>
      <div class="flex flex-wrap items-center gap-4">
        <label class="flex items-center gap-2 text-sm text-slate-700">
          <input
            type="checkbox"
            v-model="migrateData"
            class="w-4 h-4"
          />
          迁移现有数据到新路径
        </label>
        <button
          @click="changeDbPath"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
        >
          修改数据库路径
        </button>
      </div>
      <div v-if="dbStatus" class="mt-3 text-sm text-green-600">
        {{ dbStatus }}
      </div>
    </div>

    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">手机搜索服务</h3>
      <p class="text-sm text-slate-500 mb-4">
        开启后，手机和电脑在同一 WiFi 下，手机浏览器访问下方地址即可搜索档案。
        服务只在电脑运行时可用，首次开启可能需要放行 Windows 防火墙。
      </p>
      <div class="flex flex-wrap items-center gap-4 mb-4">
        <div class="flex items-center gap-2">
          <label class="text-sm text-slate-700">端口</label>
          <input
            v-model.number="mobilePort"
            type="number"
            min="1024"
            max="65535"
            :disabled="mobileStatus?.running"
            class="w-24 px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-slate-100"
          />
        </div>
        <button
          @click="toggleMobileServer"
          :disabled="mobileLoading"
          :class="[
            'px-5 py-2 rounded-lg text-white transition disabled:opacity-70',
            mobileStatus?.running
              ? 'bg-red-600 hover:bg-red-700'
              : 'bg-blue-600 hover:bg-blue-700',
          ]"
        >
          {{ mobileLoading ? "处理中..." : mobileStatus?.running ? "停止服务" : "开启服务" }}
        </button>
      </div>
      <div v-if="mobileStatus?.running && mobileStatus?.url" class="bg-slate-100 rounded-lg px-4 py-3 text-sm text-slate-700 break-all font-mono mb-3 flex items-center justify-between gap-3">
        <span>{{ mobileStatus.url }}</span>
        <button
          @click="copyMobileUrl"
          class="px-3 py-1 bg-white border rounded hover:bg-slate-50 text-xs shrink-0"
        >
          复制链接
        </button>
      </div>
      <div
        v-if="mobileMessage"
        :class="[
          'mt-3 text-sm',
          mobileStatus?.error ? 'text-red-600' : 'text-green-600',
        ]"
      >
        {{ mobileMessage }}
      </div>
    </div>

    <div class="bg-white rounded-xl shadow-sm border p-6">
      <h3 class="font-semibold text-slate-800 mb-4">使用说明</h3>
      <ul class="list-disc list-inside text-sm text-slate-600 space-y-2">
        <li>在“人员管理”中添加员工。</li>
        <li>在“任务管理”中创建周期性任务，并指派人员。</li>
        <li>在“档案分类”中维护物业档案分类，如业主档案、设备档案、合同档案。</li>
        <li>在“档案管理”中登记纸质档案，档案编号会自动生成。</li>
        <li>档案借出时选择员工借阅人，设置应还日期，到期前会自动提醒。</li>
        <li>软件会在截止日期前自动弹出桌面提醒。</li>
        <li>关闭主窗口后，软件会驻留在系统托盘继续运行提醒。</li>
      </ul>
    </div>
  </div>
</template>
