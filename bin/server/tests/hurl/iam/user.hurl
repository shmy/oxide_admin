
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
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.items" count == 1
jsonpath "$.data.total" == 1

# Should error when fetch user without token
GET {{base_url}}/api/users/fake_user
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "用户不存在"
jsonpath "$.data" not exists

# Should error when create user given invalid json body
POST {{base_url}}/api/users
Authorization: Bearer {{access_token}}
{
 
}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" contains "Failed to deserialize the JSON body"

# Should success when create user given valid token
POST {{base_url}}/api/users
Authorization: Bearer {{access_token}}
{
  "account": "test",
  "enabled": true,
  "name": "Test",
  "password": "123123",
  "portrait": null,
  "role_ids": []
}
HTTP 200
[Captures]
user_id: jsonpath "$.data"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" matches ".{24}"

# Should success when fetch user given previously created user
GET {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{user_id}}
jsonpath "$.data.account" == "test"
jsonpath "$.data.enabled" == true
jsonpath "$.data.portrait" == null

# Should success when update user password given previously created user
PUT {{base_url}}/api/users/{{user_id}}/password
Authorization: Bearer {{access_token}}
{
  "confirm_new_password": "123456",
  "new_password": "123456"
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null

# Should success when update user given previously created user
PUT {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
{
  "id": "{{user_id}}",
  "enabled": false,
  "name": "Test1",
  "portrait": "/static/images/avatar.webp"
}
HTTP 200
[Captures]
user_id: jsonpath "$.data"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" matches ".{24}"

# Should success when update portrait given previously created user
PUT {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
{
  "id": "{{user_id}}",
  "portrait": "/static/images/avatar2.webp"
}
HTTP 200
[Captures]
user_id: jsonpath "$.data"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" matches ".{24}"

# Should success when remove portrait given previously created user
PUT {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
{
  "id": "{{user_id}}",
  "portrait": null
}
HTTP 200
[Captures]
user_id: jsonpath "$.data"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" matches ".{24}"

# Should success when enable user given previously created user
POST {{base_url}}/api/users/batch/enable
Authorization: Bearer {{access_token}}
{
  "ids": ["{{user_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null

# Should success when fetch user given previously created user
GET {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{user_id}}
jsonpath "$.data.enabled" == true

# Should success when disable user given previously created user
POST {{base_url}}/api/users/batch/disable
Authorization: Bearer {{access_token}}
{
  "ids": ["{{user_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null

# Should success when fetch user given previously created user
GET {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.id" == {{user_id}}
jsonpath "$.data.enabled" == false

# Should success when delete user given previously created user
POST {{base_url}}/api/users/batch/delete
Authorization: Bearer {{access_token}}
{
  "ids": ["{{user_id}}"]
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null

# Should success when fetch user given previously created user
GET {{base_url}}/api/users/{{user_id}}
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "用户不存在"
jsonpath "$.data" not exists