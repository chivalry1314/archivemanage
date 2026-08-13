import { ref } from "vue";

export interface RecordField {
  label: string;
  value: string;
}

export const useRecordPreview = () => {
  const previewFields = ref<RecordField[]>([]);
  const previewTitle = ref("详情");
  const showPreview = ref(false);

  const openRecord = (title: string, fields: RecordField[]) => {
    previewFields.value = fields;
    previewTitle.value = title;
    showPreview.value = true;
  };

  return {
    previewFields,
    previewTitle,
    showPreview,
    openRecord,
  };
};
