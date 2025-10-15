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
        label: _t('query'),
      },
      {
        icon: "fas fa-broom",
        type: "reset",
        label: _t('reset'),
      },
    ],
  };
};

type buildCrudTableProps = {
  endpoint: string;
  deleteEndpoint?: string;
  componentId?: string;
  filters?: any[];
  headerToolbar?: any[];
  bulkActions?: any[];
  operations?: any[];
  subOperations?: any[];
  columns: any[];
  deletable?: boolean;
  itemDeletableOn?: string;
  itemCheckableOn?: string;
  showCreatedAt?: boolean;
  showUpdatedAt?: boolean;
};

export const buildCrudTable = (props: buildCrudTableProps) => {
  const deletable = props.deletable ?? true;
  const bulkActions = [];
  const deleteEndpoint = props.deleteEndpoint ?? props.endpoint;
  if (deletable) {
    bulkActions.push({
      label: _t('delete'),
      icon: "fas fa-trash",
      level: "danger",
      tooltip: _t('delete_selected_items'),
      actionType: "ajax",
      api: {
        method: "post",
        url: `${deleteEndpoint}/batch/delete`,
        data: {
          ids: "${ids | split}",
        },
      },
      confirmText: _t('are_you_sure_to_batch_delete'),
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
      label: " " + _t('delete'),
      size: "lg",
      disabledOn: props.itemDeletableOn,
      confirmText: _t('are_you_sure_to_delete'),
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
      tooltip: _t("more_options"),
      buttons: buttons,
    });
  }

  const columns = [...props.columns];
  if (props.showCreatedAt ?? true) {
    columns.push({
      name: "created_at",
      label: _t('created_at'),
      type: "datetime",
    });
  }
  if (props.showUpdatedAt ?? true) {
    columns.push({
      name: "updated_at",
      label: _t('updated_at'),
      type: "datetime",
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

    filter: buildFilter(props.filters),
    headerToolbar: [
      "bulkActions",
      {
        label: "",
        icon: "fas fa-repeat",
        type: "button",
        actionType: "reload",
        tooltip: _t('refresh'),
      },
      ...(props.headerToolbar || []),
    ],
    bulkActions: bulkActions,
    itemCheckableOn: props.itemCheckableOn,
    columns,
  };
  if (operationButtons.length > 0) {
    data.columns.push({
      label: _t('oprations'),
      type: "operation",
      width: `${operationButtons.length * 50}px`,
      fixed: "right",
      align: "center",
      buttons: operationButtons,
    });
  }
  return data;
};
