import { ifElementAuthorized } from "../../lib/auth";
import { logoUrl } from "../../lib/constant";
import { enabledStatuses } from "../../lib/options";
import { PERMISSIONS } from "../../lib/permissions";
import { buildCrudTable } from "../../lib/table";

export { };
const endpoint = "/organization/users";
const roleEndpoint = {
  method: "get",
  url: "/options/role",
  cache: 10000,
};

const buildDrawer = (isAdd = true) => {
  const label = isAdd ? _t('create_user') : null;
  const title = isAdd ? _t('create_user') : _t('edit_user');
  const level = isAdd ? "primary" : "link";
  const icon = isAdd ? "fas fa-plus" : "fas fa-edit";
  const tooltip = isAdd ? null : _t('edit_user');

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
      size: "md",
      body: {
        type: "form",
        canAccessSuperData: false,
        api: api,
        initApi: initApi,
        data: {
          enabled: true,
          privileged: false,
          role_ids: [],
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
            label: _t('user_name'),
            required: true,
            disabledOn: "this.privileged",
          },
          {
            type: "input-text",
            name: "account",
            label: _t('user_login_account'),
            required: true,
            disabledOn: "this.privileged",
          },
          {
            type: isAdd ? "input-password" : "hidden",
            name: "password",
            label: _t('user_login_password'),
            required: isAdd,
          },
          {
            type: "input-image",
            name: "portrait",
            label: _t('user_portrait'),
            receiver: "/uploads/image",
            maxLength: 1,
            crop: {
              aspectRatio: 1,
              width: 180,
              height: 180,
            },
          },
          {
            type: "select",
            name: "role_ids",
            label: _t('user_roles'),
            source: roleEndpoint,
            multiple: true,
            joinValues: false,
            extractValue: true,
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

const buildUpdatePasswordDrawer = () => {
  return {
    label: " " + _t('change_password'),
    icon: "fas fa-key",
    type: "button",
    align: "right",
    actionType: "drawer",
    level: "link",
    disabledOn: "this.privileged",
    drawer: {
      title: _t('change_password'),
      size: "lg",
      body: {
        type: "form",
        canAccessSuperData: false,
        api: "put:/system/users/${id}/password",
        body: [
          {
            type: "input-password",
            name: "new_password",
            label: _t('new_password'),
            required: true,
          },
          {
            type: "input-password",
            name: "confirm_new_password",
            label: _t('confirm_new_password'),
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
        label: _t('user_name'),
        placeholder: "",
        clearable: true,
      },
      {
        type: "input-text",
        name: "account",
        label: _t('user_login_account'),
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
        type: "select",
        name: "role_id",
        label: _t('user_roles'),
        placeholder: "",
        joinValues: false,
        extractValue: true,
        searchable: true,
        clearable: true,
        source: roleEndpoint,
      },
    ],
    headerToolbar: [
      ...ifElementAuthorized(PERMISSIONS.SYSTEM.USER.CREATE, buildDrawer()),
    ],
    bulkActions: [
      ...ifElementAuthorized(PERMISSIONS.SYSTEM.USER.ENABLE, {
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
      ...ifElementAuthorized(PERMISSIONS.SYSTEM.USER.DISABLE, {
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
      ...ifElementAuthorized(PERMISSIONS.SYSTEM.USER.UPDATE, buildDrawer(false)),
    ],
    subOperations: [
      ...ifElementAuthorized(PERMISSIONS.SYSTEM.USER.UPDATE_PASSWORD, buildUpdatePasswordDrawer()),
    ],
    deletable: _hasPermission(PERMISSIONS.SYSTEM.USER.DELETE),
    itemDeletableOn: "this.privileged",
    itemCheckableOn: "!this.privileged",
    columns: [
      {
        name: "name",
        label: _t('user_name'),
      },
      {
        name: "portrait",
        label: _t('user_portrait'),
        type: "avatar",
        defaultAvatar: logoUrl,
        shape: "rounded",
        onError: "return true;",
        src: "${portrait}",
      },
      {
        name: "account",
        label: _t('user_login_account'),
      },
      {
        name: "role_names",
        label: _t('user_roles'),
      },
      {
        name: "enabled",
        label: _t('enable'),
        type: "status",
      },
      {
        name: "privileged",
        label: _t('privileged_account'),
        type: "status",
      },
    ],
  }),
};
window._j && window._j(schema);
