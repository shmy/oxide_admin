
# Should error when fetch roles without token
GET {{base_url}}/api/organization/roles
HTTP 401
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "Access token is required"
jsonpath "$.data" not exists

# Should success when fetch roles given valid token
GET {{base_url}}/api/organization/roles
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.items" count == 1
jsonpath "$.data.total" == 1

# Should error when fetch role without token
GET {{base_url}}/api/organization/roles/fake_role
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "Role not found"
jsonpath "$.data" not exists

# Should error when create role given invalid json body
POST {{base_url}}/api/organization/roles
Authorization: Bearer {{access_token}}
{
  "enabled": true,
  "menus": [],
  "permissions": []
}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" contains "Failed to deserialize the JSON body"

# Should success when create role given valid token
POST {{base_url}}/api/organization/roles
Authorization: Bearer {{access_token}}
{
  "enabled": true,
  "name": "test1",
  "menus": [],
  "permissions": []
}
HTTP 200
[Captures]
role_id: jsonpath "$.data"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" matches ".{24}"

# Should success when fetch role given previously created role
GET {{base_url}}/api/organization/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{role_id}}
jsonpath "$.data.name" == "test1"
jsonpath "$.data.enabled" == true

# Should success when update role given previously created role
PUT {{base_url}}/api/organization/roles/{{role_id}}
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
jsonpath "$.data" matches ".{24}"

# Should success when enable role given previously created role
POST {{base_url}}/api/organization/roles/batch/enable
Authorization: Bearer {{access_token}}
{
  "ids": ["{{role_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null

# Should success when fetch role given previously created role
GET {{base_url}}/api/organization/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{role_id}}
jsonpath "$.data.enabled" == true

# Should success when disable role given previously created role
POST {{base_url}}/api/organization/roles/batch/disable
Authorization: Bearer {{access_token}}
{
  "ids": ["{{role_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null

# Should success when fetch role given previously created role
GET {{base_url}}/api/organization/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{role_id}}
jsonpath "$.data.enabled" == false

# Should success when delete role given previously created role
POST {{base_url}}/api/organization/roles/batch/delete
Authorization: Bearer {{access_token}}
{
  "ids": ["{{role_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null

# Should success when fetch role given previously created role
GET {{base_url}}/api/organization/roles/{{role_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "Role not found"
jsonpath "$.data" not exists