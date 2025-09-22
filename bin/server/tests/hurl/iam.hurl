# Roles
GET {{base_url}}/api/roles
HTTP 401
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "请提供 Token"
jsonpath "$.data" not exists

GET {{base_url}}/api/roles
Authorization: Bearer {{access_token}}
HTTP 200
[Captures]
role_id: jsonpath "$.data.items[0].id"

[Asserts]
jsonpath "$.data.items[0].name" == "admin"
jsonpath "$.data.items[1]" not exists
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.total" exists

GET {{base_url}}/api/roles/fake_role
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "角色不存在"
jsonpath "$.data" not exists

GET {{base_url}}/api/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists


# Users
GET {{base_url}}/api/users
HTTP 401
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "请提供 Token"
jsonpath "$.data" not exists

GET {{base_url}}/api/users
Authorization: Bearer {{access_token}}
HTTP 200
[Captures]
user_id: jsonpath "$.data.items[0].id"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists
jsonpath "$.data.total" exists

GET {{base_url}}/api/users/fake_user
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "用户不存在"
jsonpath "$.data" not exists

GET {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists
