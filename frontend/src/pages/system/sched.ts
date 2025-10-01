import { succeedStatuses } from "../../lib/options";
import { buildCrudTable } from "../../lib/table";

export { };

const endpoint = "/system/scheds";

const schema = {
  type: "page",
  body: buildCrudTable({
    endpoint,
    filters: [
      {
        type: "input-text",
        name: "key",
        label: "任务标识",
        placeholder: "请输入任务标识",
        clearable: true,
      },
      {
        type: "input-text",
        name: "name",
        label: "任务名称",
        placeholder: "请输入任务名称",
        clearable: true,
      },
      {
        type: "select",
        name: "succeed",
        label: "是否成功",
        placeholder: "请选择是否成功",
        clearable: true,
        options: succeedStatuses,
      },
    ],
    headerToolbar: [],
    bulkActions: [

    ],
    operations: [],
    showCreatedAt: false,
    showUpdatedAt: false,
    columns: [
      {
        name: "key",
        label: "任务标识",
      },
      {
        name: "name",
        label: "任务名称",
      },
      {
        name: "succeed",
        label: "是否成功",
        type: "status",
      },
      {
        name: "schedule",
        label: "运行周期",
      },
      {
        name: "result",
        label: "运行结果",
      },
      {
        name: "run_at",
        label: "运行时间",
        type: "datetime",
      },
      {
        name: "duration_ms",
        label: "运行时长",
        type: "pretty-ms",
      },
    ],
  }),
};
window._j && window._j(schema);