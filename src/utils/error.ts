import { alert } from "./dialog";

export const showError = (err: any) => {
  const message = err?.message || String(err) || "未知错误";

  const text =
    typeof message === "string" && message.toLowerCase().includes("foreign key")
      ? "该记录已被其他数据引用，无法删除。请先解除关联后再试。"
      : `操作失败：${message}`;

  alert(text).catch(() => {});
};
