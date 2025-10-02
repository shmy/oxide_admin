import { buildCrudTable } from "../../lib/table";

export { };

const endpoint = "/system/scheds";

const schema = {
  type: "page",
  body: buildCrudTable({
    endpoint,
    filters: [

    ],
    headerToolbar: [],
    bulkActions: [

    ],
    operations: [],
    showCreatedAt: false,
    showUpdatedAt: false,
    deletable: false,
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
        name: "expr",
        label: "运行周期",
      },
      {
        name: "last_succeed",
        label: "上次运行状态",
        type: "status",
      },
      {
        name: "last_result",
        label: "上次运行结果",
      },
      {
        name: "last_run_at",
        label: "上次运行时间",
        type: "datetime",
      },
      {
        name: "last_duration_ms",
        label: "上次运行时长",
        type: "pretty-ms",
      },
      {
        name: "next_run_at",
        label: "下次运行时间",
        type: "datetime",
      },
    ],
  }),
};
window._j && window._j(schema);