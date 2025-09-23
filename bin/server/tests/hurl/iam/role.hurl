
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

# Should success when create role given valid token
POST {{base_url}}/api/roles
Authorization: Bearer {{access_token}}
{
  "enabled": true,
  "name": "test1",
  "permission_ids": []
}
HTTP 200
[Captures]
role_id: jsonpath "$.data"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists

# Should success when fetch role given previously created role
GET {{base_url}}/api/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{role_id}}
jsonpath "$.data.name" == "test1"
jsonpath "$.data.enabled" == true

# Should success when update role given previously created role
PUT {{base_url}}/api/roles/{{role_id}}
Authorization: Bearer {{access_token}}
{
  "id": "{{role_id}}",
  "enabled": false,
  "name": "test1"
}
HTTP 200
[Captures]
role_id: jsonpath "$.data"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists

# Should success when enable role given previously created role
POST {{base_url}}/api/roles/batch/enable
Authorization: Bearer {{access_token}}
{
  "ids": ["{{role_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"

# Should success when fetch role given previously created role
GET {{base_url}}/api/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{role_id}}
jsonpath "$.data.enabled" == true

# Should success when disable role given previously created role
POST {{base_url}}/api/roles/batch/disable
Authorization: Bearer {{access_token}}
{
  "ids": ["{{role_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"

# Should success when fetch role given previously created role
GET {{base_url}}/api/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{role_id}}
jsonpath "$.data.enabled" == false

# Should success when delete role given previously created role
POST {{base_url}}/api/roles/batch/delete
Authorization: Bearer {{access_token}}
{
  "ids": ["{{role_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"

# Should success when fetch role given previously created role
GET {{base_url}}/api/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "角色不存在"