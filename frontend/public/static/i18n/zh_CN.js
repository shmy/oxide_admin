;(function (window) {
    const ftl = `
sign_in = 登录
sign_in_hint = 登录您的账号
sign_in_username = 账号
sign_in_password = 密码
sign_in_captcha = 验证码
app_name = 模版应用
username = 账号
password = 密码
privileged = 特权
enabled = 启用
reset_password = 重置密码
modify_username = 修改账号
modify_password = 修改密码
original_password = 原始密码
new_password = 新密码
confirm_new_password = 确认新密码
sign_out = 退出登录
sign_out_hint = 确定要退出登录吗？
dialog_hint = 提示
cancel = 取消
confirm = 确定
create = 创建
edit = 编辑
delete = 删除
bulk_delete = 批量删除
bulk_delete_hint = 确定要批量删除？
delete_hint = 确定要删除 {$name}？

create_admin = 创建管理员
create_user = 创建用户
edit_admin = 编辑管理员
edit_user = 编辑用户
user = 用户
roles = 角色
created_at = 创建时间
updated_at = 更新时间
operations = 操作
yes = 是
no = 否
all = 全部
datetime_format = YYYY年MM月DD日 A h:mm
create_role = 创建角色
edit_role = 编辑角色
name = 名称
permissions = 权限
language = 语言
switch_language = 切换语言
nickname = 昵称
portrait = 头像
online_devices = 在线设备
title = 标题
category = 分类
create_category = 创建分类
edit_category = 编辑分类
id = ID
create_skill = 创建技能
edit_skill = 编辑技能
description = 描述
unit_price = 单价
unit_price_description = 每小时的价格：人民币 分
reviewed = 已审核
location = 纬度经度
location_description = 格式：纬度,经度<br/>如: 39.908533,116.397394<br/><a target="_blank" href="https://lbs.qq.com/getPoint/">腾讯地图坐标拾取</a>
albums = 图集
province_city_district = 省市区
log = 日志
change_log = 变更日志
operator = 操作人
operated_at = 操作时间
more = 更多
trash = 回收站
`;
    const bundle = new window.FluentBundle.FluentBundle('zh_CN');
    bundle.addResource(new window.FluentBundle.FluentResource(ftl));
    window._t = (key, arg) => {
        const data = bundle.getMessage(key);
        if (!data) {
            return key;
        }
        return bundle.formatPattern(data.value, arg);
    };
})(window);