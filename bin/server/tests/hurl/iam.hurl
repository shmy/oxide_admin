# Should error when fetch roles without token
GET {{base_url}}/api/roles
HTTP 401
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "请提供 Token"
jsonpath "$.data" not exists

# Should success when fetch roles given valid token
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

# Should error when fetch role without token
GET {{base_url}}/api/roles/fake_role
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "角色不存在"
jsonpath "$.data" not exists

# Should success when fetch role given valid token
GET {{base_url}}/api/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists

# Should error when fetch users without token
GET {{base_url}}/api/users
HTTP 401
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "请提供 Token"
jsonpath "$.data" not exists

# Should success when fetch users given valid token
GET {{base_url}}/api/users
Authorization: Bearer {{access_token}}
HTTP 200
[Captures]
user_id: jsonpath "$.data.items[0].id"
[Asserts]
jsonpath "$.data.items[0].name" == "Admin"
jsonpath "$.data.items[1]" not exists
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.total" exists

# Should error when fetch user without token
GET {{base_url}}/api/users/fake_user
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "用户不存在"
jsonpath "$.data" not exists

# Should success when fetch user given valid token
GET {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists
