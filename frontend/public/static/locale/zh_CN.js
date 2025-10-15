; (function (w) {
    const ftl = `
create_user = 创建用户
edit_user = 编辑用户
create_role = 创建角色
edit_role = 编辑角色
role_name = 角色名称
role_menus = 角色菜单
role_permissions = 角色权限
privileged_role = 特权角色
more_options = 更多选项
delete = 删除
delete_selected_items = 删除选中项
are_you_sure_to_delete = 确定要删除吗？
are_you_sure_to_batch_delete = 确定要批量删除吗？
change_password = 修改密码
created_at = 创建时间
updated_at = 更新时间
query = 查询
reset = 重置
oprations = 操作
refresh = 刷新
enable = 启用
enable_selected_items = 启用选中项
are_you_sure_to_batch_enable = 确定要批量启用吗？
disable = 禁用
disable_selected_items = 禁用选中项
are_you_sure_to_batch_disable = 确定要批量禁用吗？
language = 语言
switch_language = 切换语言
sign_out = 退出登录
are_you_sure_to_sign_out = 确定要退出登录吗？
cancel = 取消
confirm = 确认
user_name = 用户姓名
user_login_account = 用户登录账号
user_login_password = 用户登录密码
user_portrait = 用户头像
user_roles = 用户角色
privileged_account = 特权账号
current_password = 当前密码
new_password = 新密码
confirm_new_password = 确认新密码
`;
    const bundle = new w.FluentBundle.FluentBundle('zh_CN');
    bundle.addResource(new w.FluentBundle.FluentResource(ftl));
    w._t = (key, arg) => {
        const data = bundle.getMessage(key);
        if (!data) {
            return key;
        }
        return bundle.formatPattern(data.value, arg);
    };
})(window);