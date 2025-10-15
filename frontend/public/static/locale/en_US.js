; (function (w) {
    const ftl = `
create_user = Create User
edit_user = Edit User
create_role = Create Role
edit_role = Edit Role
role_name = Role Name
role_menus = Role Menus
role_permissions = Role Permissions
privileged_role = Privileged Role
more_options = More Options
delete = Delete
delete_selected_items = Delete Selected items
are_you_sure_to_delete = Are you sure to delete?
are_you_sure_to_batch_delete = Are you sure to batch delete?
change_password = Change Password
created_at = Created At
updated_at = Updated At
query = Query
reset = Reset
oprations = Operations
refresh = Refresh
enable = Enable
enable_selected_items = Enable Selected items
are_you_sure_to_batch_enable = Are you sure to batch enable?
disable = Disable
disable_selected_items = Disable Selected items
are_you_sure_to_batch_disable = Are you sure to batch disable?
language = Language
switch_language = Switch Language
sign_out = Sign Out
are_you_sure_to_sign_out = Are you sure to sign out?
cancel = Cancel
confirm = Confirm
user_name = User Name
user_login_account = User Login Account
user_login_password = User Login Password
user_portrait = User Portrait
user_roles = User Roles
privileged_account = Privileged Account
current_password = Current Password
new_password = New Password
confirm_new_password = Confirm New Password
`;
    const bundle = new w.FluentBundle.FluentBundle('en_US');
    bundle.addResource(new w.FluentBundle.FluentResource(ftl));
    w._t = (key, arg) => {
        const data = bundle.getMessage(key);
        if (!data) {
            return key;
        }
        return bundle.formatPattern(data.value, arg);
    };
})(window);