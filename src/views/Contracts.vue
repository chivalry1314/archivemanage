<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAppStore } from "../stores/app";
import {
  createContract,
  deleteContract,
  deleteContractsBatch,
  exportContractsCsv,
  exportContractsXlsx,
  importContractsFromExcel,
  listContracts,
  saveFile,
  updateContract,
} from "../api";
import { open as openDialog, save } from "@tauri-apps/plugin-dialog";
import Pagination from "../components/Pagination.vue";
import RecordDetailModal from "../components/RecordDetailModal.vue";
import { showError } from "../utils/error";
import { confirm } from "../utils/dialog";
import { useRecordPreview } from "../utils/recordPreview";
import * as XLSX from "xlsx";

const store = useAppStore();
const { previewFields, previewTitle, showPreview, openRecord } = useRecordPreview();

const searchKeyword = ref("");
const showForm = ref(false);
const editing = ref(false);
const currentPage = ref(1);
const total = ref(0);
const perPage = ref(10);
const importStatus = ref("");
const exportStatus = ref("");
const exportFormat = ref<"xlsx" | "csv">("xlsx");
const selectedContractIds = ref<number[]>([]);

const form = ref({
  id: 0,
  contract_no: "",
  contract_name: "",
  party_a: "",
  party_b: "",
  contact_person: "",
  contact_info: "",
  total_amount_with_tax: undefined as number | undefined,
  total_amount_without_tax: undefined as number | undefined,
  tax_amount: undefined as number | undefined,
  payment_cycle: "",
  payment_amount_with_tax: undefined as number | undefined,
  payment_method: "",
  effective_date: "",
  end_date: "",
  sign_date: "",
  handler_party_a: "",
  handler_party_b: "",
  remark: "",
});

const resetForm = () => {
  form.value = {
    id: 0,
    contract_no: "",
    contract_name: "",
    party_a: "",
    party_b: "",
    contact_person: "",
    contact_info: "",
    total_amount_with_tax: undefined,
    total_amount_without_tax: undefined,
    tax_amount: undefined,
    payment_cycle: "",
    payment_amount_with_tax: undefined,
    payment_method: "",
    effective_date: "",
    end_date: "",
    sign_date: "",
    handler_party_a: "",
    handler_party_b: "",
    remark: "",
  };
  editing.value = false;
};

const openForm = () => {
  resetForm();
  showForm.value = true;
};

const getPageSize = () =>
  Math.max(5, Math.min(100, parseInt(localStorage.getItem("pageSize") || "10", 10) || 10));

const load = async () => {
  perPage.value = getPageSize();
  const result = await listContracts(
    searchKeyword.value.trim() || undefined,
    currentPage.value,
    perPage.value
  );
  store.contracts = result.items;
  total.value = result.total;
  if (result.items.length === 0 && currentPage.value > 1) {
    currentPage.value--;
    await load();
  }
};

const submit = async () => {
  if (!form.value.contract_name.trim()) {
    showError("合同名称不能为空");
    return;
  }

  const payload = {
    contract_no: form.value.contract_no.trim() || undefined,
    contract_name: form.value.contract_name.trim(),
    party_a: form.value.party_a.trim() || undefined,
    party_b: form.value.party_b.trim() || undefined,
    contact_person: form.value.contact_person.trim() || undefined,
    contact_info: form.value.contact_info.trim() || undefined,
    total_amount_with_tax: yuanToFen(form.value.total_amount_with_tax),
    total_amount_without_tax: yuanToFen(form.value.total_amount_without_tax),
    tax_amount: yuanToFen(form.value.tax_amount),
    payment_cycle: form.value.payment_cycle.trim() || undefined,
    payment_amount_with_tax: yuanToFen(form.value.payment_amount_with_tax),
    payment_method: form.value.payment_method.trim() || undefined,
    effective_date: form.value.effective_date || undefined,
    end_date: form.value.end_date || undefined,
    sign_date: form.value.sign_date || undefined,
    handler_party_a: form.value.handler_party_a.trim() || undefined,
    handler_party_b: form.value.handler_party_b.trim() || undefined,
    remark: form.value.remark.trim() || undefined,
  };

  try {
    if (editing.value) {
      await updateContract({ id: form.value.id, ...payload });
    } else {
      await createContract(payload);
    }
    resetForm();
    showForm.value = false;
    await load();
  } catch (e) {
    showError(e);
  }
};

const editContract = (c: any) => {
  form.value = {
    id: c.id,
    contract_no: c.contract_no || "",
    contract_name: c.contract_name,
    party_a: c.party_a || "",
    party_b: c.party_b || "",
    contact_person: c.contact_person || "",
    contact_info: c.contact_info || "",
    total_amount_with_tax: fenToYuan(c.total_amount_with_tax),
    total_amount_without_tax: fenToYuan(c.total_amount_without_tax),
    tax_amount: fenToYuan(c.tax_amount),
    payment_cycle: c.payment_cycle || "",
    payment_amount_with_tax: fenToYuan(c.payment_amount_with_tax),
    payment_method: c.payment_method || "",
    effective_date: c.effective_date || "",
    end_date: c.end_date || "",
    sign_date: c.sign_date || "",
    handler_party_a: c.handler_party_a || "",
    handler_party_b: c.handler_party_b || "",
    remark: c.remark || "",
  };
  editing.value = true;
  showForm.value = true;
};

const removeContract = async (id: number) => {
  if (!(await confirm("确定删除该合同记录？"))) return;
  try {
    await deleteContract(id);
    selectedContractIds.value = selectedContractIds.value.filter((i) => i !== id);
    await load();
  } catch (e) {
    showError(e);
  }
};

const toggleSelectAllContracts = () => {
  const currentIds = store.contracts.map((c) => c.id);
  const allSelected = currentIds.every((id) => selectedContractIds.value.includes(id));
  if (allSelected) {
    selectedContractIds.value = selectedContractIds.value.filter((id) => !currentIds.includes(id));
  } else {
    const merged = new Set([...selectedContractIds.value, ...currentIds]);
    selectedContractIds.value = Array.from(merged);
  }
};

const removeSelectedContracts = async () => {
  if (selectedContractIds.value.length === 0) return;
  if (!(await confirm(`确定要删除选中的 ${selectedContractIds.value.length} 条合同记录吗？`))) return;

  try {
    await deleteContractsBatch(selectedContractIds.value);
    selectedContractIds.value = [];
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

const openDetail = (c: any) => {
  openRecord("合同详情", [
    { label: "合同编号", value: c.contract_no || "-" },
    { label: "合同名称", value: c.contract_name },
    { label: "合同甲方", value: c.party_a || "-" },
    { label: "合同乙方", value: c.party_b || "-" },
    { label: "对方联系人", value: c.contact_person || "-" },
    { label: "联系方式", value: c.contact_info || "-" },
    { label: "合同总金额（含税）", value: formatMoney(c.total_amount_with_tax) },
    { label: "合同总金额（不含税）", value: formatMoney(c.total_amount_without_tax) },
    { label: "税额", value: formatMoney(c.tax_amount) },
    { label: "付款周期", value: c.payment_cycle || "-" },
    { label: "每次支付金额（含税）", value: formatMoney(c.payment_amount_with_tax) },
    { label: "付款方式", value: c.payment_method || "-" },
    { label: "合同生效日期", value: c.effective_date || "-" },
    { label: "合同终止日期", value: c.end_date || "-" },
    { label: "合同签订日期", value: c.sign_date || "-" },
    { label: "甲方经办人", value: c.handler_party_a || "-" },
    { label: "乙方经办人", value: c.handler_party_b || "-" },
    { label: "备注", value: c.remark || "-" },
  ]);
};

const downloadTemplate = async () => {
  try {
    const path = await save({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      defaultPath: "合同导入模板.xlsx",
    });
    if (!path) return;

    const headersRow1 = [
      "序号",
      "合同编号",
      "合同名称",
      "合同当事人",
      null,
      "对方联系人",
      "联系方式",
      "合同总金额（含税）",
      "合同总金额（不含税）",
      "税额",
      "付款周期",
      "每次支付金额（含税）",
      "付款方式",
      "合同生效日期",
      "合同终止日期",
      "合同签订日期",
      "经办人",
      null,
      "备注",
    ];
    const headersRow2 = [
      null,
      null,
      null,
      "合同甲方",
      "合同乙方",
      null,
      null,
      null,
      null,
      null,
      null,
      null,
      null,
      null,
      null,
      null,
      "甲方经办人",
      "乙方经办人",
      null,
    ];
    const sample = [
      "1",
      "HT-2026-001",
      "保洁服务合同",
      "甲方示例公司",
      "乙方示例公司",
      "示例联系人",
      "13800138000",
      120000,
      113207.55,
      6792.45,
      "季度",
      30000,
      "银行转账",
      "2026-01-01",
      "2026-12-31",
      "2026-01-01",
      "示例经办人甲",
      "示例经办人乙",
      "半年租金15300，押金2550",
    ];

    const worksheet = XLSX.utils.aoa_to_sheet([headersRow1, headersRow2, sample]);
    worksheet["!merges"] = [
      { s: { r: 0, c: 0 }, e: { r: 1, c: 0 } },
      { s: { r: 0, c: 1 }, e: { r: 1, c: 1 } },
      { s: { r: 0, c: 2 }, e: { r: 1, c: 2 } },
      { s: { r: 0, c: 3 }, e: { r: 0, c: 4 } },
      { s: { r: 0, c: 5 }, e: { r: 1, c: 5 } },
      { s: { r: 0, c: 6 }, e: { r: 1, c: 6 } },
      { s: { r: 0, c: 7 }, e: { r: 1, c: 7 } },
      { s: { r: 0, c: 8 }, e: { r: 1, c: 8 } },
      { s: { r: 0, c: 9 }, e: { r: 1, c: 9 } },
      { s: { r: 0, c: 10 }, e: { r: 1, c: 10 } },
      { s: { r: 0, c: 11 }, e: { r: 1, c: 11 } },
      { s: { r: 0, c: 12 }, e: { r: 1, c: 12 } },
      { s: { r: 0, c: 13 }, e: { r: 1, c: 13 } },
      { s: { r: 0, c: 14 }, e: { r: 1, c: 14 } },
      { s: { r: 0, c: 15 }, e: { r: 1, c: 15 } },
      { s: { r: 0, c: 16 }, e: { r: 0, c: 17 } },
      { s: { r: 0, c: 18 }, e: { r: 1, c: 18 } },
    ];

    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "Sheet1");
    const arrayBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });
    await saveFile(path as string, new Uint8Array(arrayBuffer));
    importStatus.value = "模板已保存";
    setTimeout(() => (importStatus.value = ""), 3000);
  } catch (e) {
    showError(e);
  }
};

const importExcel = async () => {
  try {
    const path = await openDialog({
      filters: [{ name: "Excel", extensions: ["xlsx"] }],
      directory: false,
      multiple: false,
    });
    if (!path) return;

    importStatus.value = "正在导入，请稍候...";
    const count = await importContractsFromExcel(path as string);
    importStatus.value = `导入完成：新增 ${count} 条合同记录`;
    setTimeout(() => (importStatus.value = ""), 5000);
    await load();
  } catch (e) {
    showError(e);
    importStatus.value = "";
  }
};

const exportExcel = async () => {
  try {
    const dateStr = new Date().toISOString().split("T")[0];
    if (exportFormat.value === "xlsx") {
      const path = await save({
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
        defaultPath: `合同台账_${dateStr}.xlsx`,
      });
      if (!path) return;
      const bytes = await exportContractsXlsx();
      await saveFile(path as string, new Uint8Array(bytes));
    } else {
      const csv = await exportContractsCsv();
      downloadFile(csv, `合同台账_${dateStr}.csv`, "text/csv;charset=utf-8;");
    }
    exportStatus.value = "合同台账导出成功";
    setTimeout(() => (exportStatus.value = ""), 3000);
  } catch (e) {
    showError(e);
  }
};

const yuanToFen = (yuan: number | undefined): number | undefined => {
  if (yuan === undefined || isNaN(yuan)) return undefined;
  return Math.round(yuan * 100);
};

const fenToYuan = (fen: number | undefined): number | undefined => {
  if (fen === undefined) return undefined;
  return fen / 100;
};

const formatMoney = (fen: number | undefined): string => {
  if (fen === undefined) return "-";
  return (fen / 100).toFixed(2);
};

const downloadFile = (content: string, filename: string, mimeType: string) => {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

onMounted(load);
</script>

<template>
  <div class="space-y-6">
    <div class="bg-white p-4 rounded-xl shadow-sm border flex flex-wrap gap-3 items-center">
      <input
        v-model="searchKeyword"
        @input="search"
        placeholder="搜索合同编号 / 名称 / 甲方 / 乙方 / 经办人"
        class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 w-80"
      />
      <div class="flex-1"></div>
      <select
        v-model="exportFormat"
        class="px-3 py-2 border rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="xlsx">Excel</option>
        <option value="csv">CSV</option>
      </select>
      <button
        @click="exportExcel"
        class="px-4 py-2.5 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition text-sm"
      >
        导出
      </button>
      <button
        @click="downloadTemplate"
        class="px-4 py-2.5 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition text-sm"
      >
        下载模板
      </button>
      <button
        @click="importExcel"
        class="px-4 py-2.5 bg-slate-100 text-slate-700 rounded-lg hover:bg-slate-200 transition text-sm"
      >
        导入
      </button>
      <button
        v-if="selectedContractIds.length > 0"
        @click="removeSelectedContracts"
        class="px-5 py-2.5 bg-red-600 text-white rounded-lg hover:bg-red-700 transition"
      >
        批量删除（{{ selectedContractIds.length }}）
      </button>
      <button
        @click="openForm"
        class="px-5 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition"
      >
        + 添加合同
      </button>
    </div>

    <div v-if="importStatus || exportStatus" class="text-sm text-blue-600">
      {{ importStatus || exportStatus }}
    </div>

    <!-- Form Modal -->
    <div
      v-if="showForm"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showForm = false"
    >
      <div class="bg-white rounded-xl shadow-xl w-full max-w-4xl p-6 max-h-[90vh] overflow-auto">
        <h3 class="text-lg font-semibold text-slate-800 mb-4">
          {{ editing ? "编辑合同" : "添加合同" }}
        </h3>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <input
            v-model="form.contract_no"
            placeholder="合同编号"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.contract_name"
            placeholder="合同名称 *"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.party_a"
            placeholder="合同甲方"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.party_b"
            placeholder="合同乙方"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.contact_person"
            placeholder="对方联系人"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.contact_info"
            placeholder="联系方式"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model.number="form.total_amount_with_tax"
            type="number"
            step="0.01"
            placeholder="合同总金额（含税）"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model.number="form.total_amount_without_tax"
            type="number"
            step="0.01"
            placeholder="合同总金额（不含税）"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model.number="form.tax_amount"
            type="number"
            step="0.01"
            placeholder="税额"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.payment_cycle"
            placeholder="付款周期"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model.number="form.payment_amount_with_tax"
            type="number"
            step="0.01"
            placeholder="每次支付金额（含税）"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.payment_method"
            placeholder="付款方式"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.effective_date"
            type="date"
            placeholder="合同生效日期"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.end_date"
            type="date"
            placeholder="合同终止日期"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.sign_date"
            type="date"
            placeholder="合同签订日期"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.handler_party_a"
            placeholder="甲方经办人"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            v-model="form.handler_party_b"
            placeholder="乙方经办人"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <textarea
            v-model="form.remark"
            placeholder="备注"
            rows="3"
            class="px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 md:col-span-2"
          ></textarea>
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
                  :checked="store.contracts.length > 0 && store.contracts.every((c) => selectedContractIds.includes(c.id))"
                  @change="toggleSelectAllContracts"
                  class="w-4 h-4"
                />
              </th>
              <th class="px-4 py-3 font-medium text-slate-600">合同编号</th>
              <th class="px-4 py-3 font-medium text-slate-600">合同名称</th>
              <th class="px-4 py-3 font-medium text-slate-600">甲方</th>
              <th class="px-4 py-3 font-medium text-slate-600">乙方</th>
              <th class="px-4 py-3 font-medium text-slate-600">联系人</th>
              <th class="px-4 py-3 font-medium text-slate-600">联系方式</th>
              <th class="px-4 py-3 font-medium text-slate-600">总金额（含税）</th>
              <th class="px-4 py-3 font-medium text-slate-600">总金额（不含税）</th>
              <th class="px-4 py-3 font-medium text-slate-600">税额</th>
              <th class="px-4 py-3 font-medium text-slate-600">付款周期</th>
              <th class="px-4 py-3 font-medium text-slate-600">每次支付</th>
              <th class="px-4 py-3 font-medium text-slate-600">付款方式</th>
              <th class="px-4 py-3 font-medium text-slate-600">生效日期</th>
              <th class="px-4 py-3 font-medium text-slate-600">终止日期</th>
              <th class="px-4 py-3 font-medium text-slate-600">签订日期</th>
              <th class="px-4 py-3 font-medium text-slate-600">甲方经办人</th>
              <th class="px-4 py-3 font-medium text-slate-600">乙方经办人</th>
              <th class="px-4 py-3 font-medium text-slate-600">备注</th>
              <th class="px-4 py-3 font-medium text-slate-600">操作</th>
            </tr>
          </thead>
          <tbody class="divide-y">
            <tr
              v-for="c in store.contracts"
              :key="c.id"
              class="hover:bg-slate-50"
            >
              <td class="px-4 py-3">
                <input
                  type="checkbox"
                  :value="c.id"
                  v-model="selectedContractIds"
                  class="w-4 h-4"
                />
              </td>
              <td
                class="px-4 py-3 text-slate-800 truncate-cell"
                @dblclick="openDetail(c)"
              >
                {{ c.contract_no || "-" }}
              </td>
              <td
                class="px-4 py-3 font-medium text-slate-800 truncate-cell"
                @dblclick="openDetail(c)"
              >
                {{ c.contract_name }}
              </td>
              <td class="px-4 py-3 text-slate-600 truncate-cell">{{ c.party_a || "-" }}</td>
              <td class="px-4 py-3 text-slate-600 truncate-cell">{{ c.party_b || "-" }}</td>
              <td class="px-4 py-3 text-slate-600 truncate-cell">{{ c.contact_person || "-" }}</td>
              <td class="px-4 py-3 text-slate-600 truncate-cell">{{ c.contact_info || "-" }}</td>
              <td class="px-4 py-3 text-slate-600">{{ formatMoney(c.total_amount_with_tax) }}</td>
              <td class="px-4 py-3 text-slate-600">{{ formatMoney(c.total_amount_without_tax) }}</td>
              <td class="px-4 py-3 text-slate-600">{{ formatMoney(c.tax_amount) }}</td>
              <td class="px-4 py-3 text-slate-600">{{ c.payment_cycle || "-" }}</td>
              <td class="px-4 py-3 text-slate-600">{{ formatMoney(c.payment_amount_with_tax) }}</td>
              <td class="px-4 py-3 text-slate-600">{{ c.payment_method || "-" }}</td>
              <td class="px-4 py-3 text-slate-600">{{ c.effective_date || "-" }}</td>
              <td class="px-4 py-3 text-slate-600">{{ c.end_date || "-" }}</td>
              <td class="px-4 py-3 text-slate-600">{{ c.sign_date || "-" }}</td>
              <td class="px-4 py-3 text-slate-600">{{ c.handler_party_a || "-" }}</td>
              <td class="px-4 py-3 text-slate-600">{{ c.handler_party_b || "-" }}</td>
              <td class="px-4 py-3 text-slate-600 truncate-cell" :title="c.remark || ''">{{ c.remark || "-" }}</td>
              <td class="px-4 py-3 flex gap-2">
                <button
                  @click="editContract(c)"
                  class="px-3 py-1 text-xs bg-slate-100 hover:bg-slate-200 rounded transition"
                >
                  编辑
                </button>
                <button
                  @click="removeContract(c.id)"
                  class="px-3 py-1 text-xs bg-red-50 text-red-600 hover:bg-red-100 rounded transition"
                >
                  删除
                </button>
              </td>
            </tr>
            <tr v-if="store.contracts.length === 0">
              <td colspan="20" class="px-6 py-8 text-center text-slate-400">暂无合同记录</td>
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
