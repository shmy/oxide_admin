import { chunk } from "es-toolkit";

const buildFilter = (items?: any[]) => {
  if (!items || items.length === 0) {
    return null;
  }
  let chunks = chunk(items, 4);
  return {
    title: "",
    body: chunks.map((body) => {
      return {
        type: "group",
        mode: "horizontal",
        horizontal: {
          left: 3,
          right: 9,
        },
        body: body.map((item) => {
          return {
            ...item,
            columnRatio: 3,
          };
        }),
      };
    }),
    actions: [
      {
        type: "submit",
        icon: "fas fa-search",
        level: "primary",
        label: "查询",
      },
      {
        icon: "fas fa-broom",
        type: "reset",
        label: "重置",
      },
    ],
  };
};

type buildCrudTableProps = {
  endpoint: string;
  deleteEndpoint?: string;
  componentId?: string;
  filter?: any[];
  headerToolbar?: any[];
  bulkActions?: any[];
  operations?: any[];
  subOperations?: any[];
  columns: any[];
  deletable?: boolean;
  itemDeletableOn?: string;
  itemCheckableOn?: string;
};

export const buildCrudTable = (props: buildCrudTableProps) => {
  const deletable = props.deletable ?? true;
  const bulkActions = [];
  const deleteEndpoint = props.deleteEndpoint ?? props.endpoint;
  if (deletable) {
    bulkActions.push({
      label: "删除",
      icon: "fas fa-trash",
      level: "danger",
      tooltip: "删除所选中项",
      actionType: "ajax",
      api: {
        method: "post",
        url: `${deleteEndpoint}/batch/delete`,
        data: {
          ids: "${ids | split}",
        },
      },
      confirmText: "确定要批量删除?",
    });
  }
  bulkActions.push(...(props.bulkActions || []));
  const operationButtons = props.operations || [];
  if (deletable) {
    const buttons = props.subOperations || [];
    buttons.push({
      type: "button",
      level: "link",
      icon: "fas fa-trash",
      label: " 删除",
      size: "lg",
      disabledOn: props.itemDeletableOn,
      confirmText: "确定要删除吗？",
      actionType: "ajax",
      className: "text-danger",
      api: {
        method: "post",
        url: `${deleteEndpoint}/batch/delete`,
        data: {
          ids: ["${id}"],
        },
      },
    });
    operationButtons.push({
      type: "dropdown-button",
      level: "link",
      icon: "fa fa-ellipsis-h",
      hideCaret: true,
      tooltip: "更多选项",
      buttons: buttons,
    });
  }
  const data = {
    type: "crud",
    id: props.componentId,
    api: props.endpoint,
    pageField: "page",
    perPageField: "page_size",
    perPage: 20,
    syncLocation: true,
    autoGenerateFilter: false,
    rowClassNameExpr: "${index % 2 ? 'bg-gray-50' : null}",

    filter: buildFilter(props.filter),
    headerToolbar: [
      "bulkActions",
      {
        label: "",
        icon: "fas fa-repeat",
        type: "button",
        actionType: "reload",
        tooltip: "刷新数据",
      },
      ...(props.headerToolbar || []),
    ],
    bulkActions: bulkActions,
    itemCheckableOn: props.itemCheckableOn,
    columns: [
      ...props.columns,
      {
        name: "created_at",
        label: "创建时间",
        // "sortable": sortable,
        type: "datetime",
      },
      {
        name: "updated_at",
        label: "更新时间",
        // "sortable": sortable,
        type: "datetime",
      },
    ],
  };
  if (operationButtons.length > 0) {
    data.columns.push({
      label: "操作",
      type: "operation",
      width: `${operationButtons.length * 50}px`,
      fixed: "right",
      align: "center",
      buttons: operationButtons,
    });
  }
  return data;
};
