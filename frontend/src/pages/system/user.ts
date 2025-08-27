import { enabledStatuses } from "../../lib/options";
import { buildCrudTable } from "../../lib/table";

export {};
const endpoint = "/users";

const buildDrawer = (isAdd = true) => {
  const label = isAdd ? "创建用户" : null;
  const title = isAdd ? "创建用户" : "编辑用户";
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
            label: "用户姓名",
            required: true,
            disabledOn: "this.privileged",
          },
          {
            type: "input-text",
            name: "account",
            label: "用户账号",
            required: true,
            disabledOn: "this.privileged",
          },
          {
            type: "input-image",
            name: "portrait",
            label: "头像",
            receiver: "/uploads/image",
            maxLength: 1,
            crop: {
              aspectRatio: 1,
              width: 180,
              height: 180,
            },
          },
          {
            type: isAdd ? "input-password" : "hidden",
            name: "password",
            label: "用户密码",
            required: isAdd,
          },
          {
            type: "select",
            name: "role_ids",
            label: "角色列表",
            source: "/options/roles",
            multiple: true,
            joinValues: false,
            extractValue: true,
            value: "${role_ids || []}",
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

const buildUpdatePasswordDrawer = () => {
  return {
    label: " 修改密码",
    icon: "fas fa-key",
    tooltip: "修改用户密码",
    type: "button",
    align: "right",
    actionType: "drawer",
    level: "link",
    disabledOn: "this.privileged",
    drawer: {
      title: "修改密码",
      size: "lg",
      body: {
        type: "form",
        canAccessSuperData: false,
        api: "put:/users/${id}/password",
        body: [
          {
            type: "input-password",
            name: "new_password",
            label: "新密码",
            required: true,
          },
          {
            type: "input-password",
            name: "confirm_new_password",
            label: "确认新密码",
            required: true,
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
          label: "用户姓名",
          placeholder: "请输入用户姓名",
          clearable: true,
        },
        {
          type: "input-text",
          name: "account",
          label: "用户账号",
          placeholder: "请输入用户账号",
          clearable: true,
        },
        {
          type: "select",
          name: "enabled",
          label: "用户状态",
          placeholder: "请选择用户状态",
          clearable: true,
          options: enabledStatuses,
        },
        {
          type: "select",
          name: "role_id",
          label: "用户角色",
          joinValues: false,
          extractValue: true,
          searchable: true,
          clearable: true,
          placeholder: "请选择角色",
          source: {
            method: "get",
            url: "/options/roles",
            cache: 10000,
          },
        },
        ,
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
    subOperations: [buildUpdatePasswordDrawer()],
    itemDeletableOn: "this.privileged",
    itemCheckableOn: "!this.privileged",
    columns: [
      {
        name: "name",
        label: "用户姓名",
      },
      {
        name: "portrait",
        label: "头像",
        type: "avatar",
        shape: "rounded",
        onError: "return true;",
        src: "${portrait}",
      },
      {
        name: "account",
        label: "用户账号",
      },
      {
        name: "role_names",
        label: "角色列表",
      },
      {
        name: "enabled",
        label: "状态",
        type: "status",
      },
      {
        name: "privileged",
        label: "特权账号",
        type: "status",
      },
    ],
  }),
};
window._j && window._j(schema);
