import { confirm as tauriConfirm, message as tauriMessage } from "@tauri-apps/plugin-dialog";

export const confirm = (message: string) =>
  tauriConfirm(message, { title: "请确认", kind: "warning" });

export const alert = (message: string) =>
  tauriMessage(message, { title: "提示", kind: "info" });
