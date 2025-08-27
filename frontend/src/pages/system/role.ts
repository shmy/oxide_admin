import { enabledStatuses } from "../../lib/options";
import { buildCrudTable } from "../../lib/table";

export {};
const endpoint = "/roles";

const buildDrawer = (isAdd = true) => {
  const label = isAdd ? "创建角色" : null;
  const title = isAdd ? "创建角色" : "编辑角色";
  const level = isAdd ? "primary" : "link";
  const icon = isAdd ? "fas fa-plus" : "fas fa-edit";
  const tooltip = isAdd ? null : "编辑";

  const api = isAdd ? "post:" + endpoint : "put:" + endpoint + "/$id";
  const initApi = isAdd ? null : "get:" + endpoint + "/$id";

  return {
    label: label,
    icon: icon,
    tooltip: tooltip,
    type: "button",
    align: "right",
    actionType: "drawer",
    level: level,
    disabledOn: "this.privileged",
    drawer: {
      title: title,
      size: "lg",
      body: {
        type: "form",
        canAccessSuperData: false,
        api: api,
        initApi: initApi,
        body: [
          {
            type: "static-text",
            name: "id",
            label: "ID",
            copyable: true,
            disabled: true,
            visible: !isAdd,
          },
          {
            type: "input-text",
            name: "name",
            label: "角色名称",
            required: true,
            disabledOn: "this.privileged",
          },
          {
            type: "tree-select",
            name: "permission_ids",
            label: "权限列表",
            source: "/options/permissions",
            labelField: "label",
            valueField: "key",
            multiple: true,
            autoCheckChildren: false,
            onlyChildren: false,
            joinValues: false,
            extractValue: true,
            value: "${permission_ids || []}",
            disabledOn: "this.privileged",
          },
          {
            type: "hidden",
            name: "privileged",
            value: false,
            required: true,
          },
          {
            type: "switch",
            name: "enabled",
            label: "状态",
            value: true,
            required: true,
            disabledOn: "this.privileged",
          },
        ],
      },
    },
  };
};

const schema = {
  type: "page",
  body: buildCrudTable({
    endpoint,
    filter: [
      [
        {
          type: "input-text",
          name: "name",
          label: "角色名称",
          placeholder: "请输入角色名称",
          clearable: true,
        },
        {
          type: "select",
          name: "enabled",
          label: "角色状态",
          placeholder: "请选择角色状态",
          clearable: true,
          options: enabledStatuses,
        },
        {
          type: "tree-select",
          name: "permission_id",
          label: "角色权限",
          source: {
            method: "get",
            url: "/options/permissions",
            cache: 10000,
          },
          placeholder: "请选择权限",
          labelField: "label",
          valueField: "key",
        },
        {
          type: "hidden",
        },
      ],
    ],
    headerToolbar: [buildDrawer()],
    bulkActions: [
      {
        label: "启用",
        icon: "fas fa-check",
        level: "success",
        tooltip: "将所选中项的状态设为启用",
        actionType: "ajax",
        api: {
          method: "post",
          url: `${endpoint}/batch/enable`,
          data: {
            ids: "${ids | split}",
          },
        },
        confirmText: "确定要批量将状态设为启用?",
      },
      {
        label: "禁用",
        icon: "fas fa-close",
        level: "warning",
        tooltip: "将所选中项的状态设为禁用",
        actionType: "ajax",
        api: {
          method: "post",
          url: `${endpoint}/batch/disable`,
          data: {
            ids: "${ids | split}",
          },
        },
        confirmText: "确定要批量将状态设为禁用?",
      },
    ],
    operations: [buildDrawer(false)],
    itemDeletableOn: "this.privileged",
    itemCheckableOn: "!this.privileged",
    columns: [
      {
        name: "name",
        label: "角色名称",
      },
      {
        type: "tree-select",
        name: "permission_ids",
        label: "权限列表",
        static: true,
        source: {
          method: "get",
          url: "/options/permissions",
          cache: 10000,
        },
        labelField: "label",
        valueField: "key",
      },
      {
        name: "enabled",
        label: "状态",
        type: "status",
      },
      {
        name: "privileged",
        label: "特权角色",
        type: "status",
      },
    ],
  }),
};
window._j && window._j(schema);
