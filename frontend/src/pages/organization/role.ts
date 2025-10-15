import { ifElementAuthorized } from "../../lib/auth";
import { enabledStatuses } from "../../lib/options";
import { PERMISSIONS } from "../../lib/permissions";
import { buildCrudTable } from "../../lib/table";

export { };
const endpoint = "/organization/roles";
const menuEndpoint = {
  method: "get",
  url: "/options/menu",
  cache: 10000,
};
const permissionEndpoint = {
  method: "get",
  url: "/options/permission",
  cache: 10000,
};

const buildDrawer = (isAdd = true) => {
  const label = isAdd ? _t('create_role') : null;
  const title = isAdd ? _t('create_role') : _t('edit_role');
  const level = isAdd ? "primary" : "link";
  const icon = isAdd ? "fas fa-plus" : "fas fa-edit";
  const tooltip = isAdd ? null : _t('edit_role');

  const api = isAdd ? `post:${endpoint}` : `put:${endpoint}/$id`;
  const initApi = isAdd ? null : `get:${endpoint}/$id`;

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
        data: {
          enabled: true,
          privileged: false,
          menus: [],
          permissions: []
        },
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
            type: "switch",
            name: "enabled",
            label: _t('enable'),
            required: true,
            disabledOn: "this.privileged",
          },
          {
            type: "input-text",
            name: "name",
            label: _t('role_name'),
            required: true,
            disabledOn: "this.privileged",
          },
          {
            type: "tree-select",
            name: "menus",
            label: _t('role_menus'),
            source: menuEndpoint,
            labelField: "label",
            valueField: "key",
            multiple: true,
            autoCheckChildren: false,
            onlyChildren: false,
            joinValues: false,
            extractValue: true,
            disabledOn: "this.privileged",
          },
          {
            name: "permissions",
            type: "checkboxes",
            label: _t('role_permissions'),
            checkAll: true,
            columnsCount: 3,
            inline: false,
            joinValues: false,
            extractValue: true,
            source: permissionEndpoint,
            disabledOn: "this.privileged",
          },
          {
            type: "hidden",
            name: "privileged",
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
    filters: [
      {
        type: "input-text",
        name: "name",
        label: _t('role_name'),
        placeholder: "",
        clearable: true,
      },
      {
        type: "select",
        name: "enabled",
        label: _t('enable'),
        placeholder: "",
        clearable: true,
        options: enabledStatuses,
      },
      {
        type: "tree-select",
        name: "menu",
        label: _t('role_menus'),
        source: menuEndpoint,
        placeholder: "",
        labelField: "label",
        valueField: "key",
      },
    ],
    headerToolbar: [
      ...ifElementAuthorized(PERMISSIONS.ORGANIZATION.ROLE.CREATE, buildDrawer()),
    ],
    bulkActions: [
      ...ifElementAuthorized(PERMISSIONS.ORGANIZATION.ROLE.ENABLE, {
        label: _t('enable'),
        icon: "fas fa-check",
        level: "success",
        tooltip: _t('enable_selected_items'),
        actionType: "ajax",
        api: {
          method: "post",
          url: `${endpoint}/batch/enable`,
          data: {
            ids: "${ids | split}",
          },
        },
        confirmText: _t('are_you_sure_to_batch_enable'),
      }),
      ...ifElementAuthorized(PERMISSIONS.ORGANIZATION.ROLE.DISABLE, {
        label: _t('disable'),
        icon: "fas fa-close",
        level: "warning",
        tooltip: _t('disable_selected_items'),
        actionType: "ajax",
        api: {
          method: "post",
          url: `${endpoint}/batch/disable`,
          data: {
            ids: "${ids | split}",
          },
        },
        confirmText: _t('are_you_sure_to_batch_disable'),
      }),
    ],
    operations: [
      ...ifElementAuthorized(PERMISSIONS.ORGANIZATION.ROLE.UPDATE, buildDrawer(false)),
    ],
    deletable: _hasPermission(PERMISSIONS.ORGANIZATION.ROLE.DELETE),
    itemDeletableOn: "this.privileged",
    itemCheckableOn: "!this.privileged",
    columns: [
      {
        name: "name",
        label: _t('role_name'),
      },
      {
        name: "enabled",
        label: _t('enable'),
        type: "status",
      },
      {
        name: "privileged",
        label: _t('privileged_role'),
        type: "status",
      },
    ],
  }),
};
window._j && window._j(schema);
