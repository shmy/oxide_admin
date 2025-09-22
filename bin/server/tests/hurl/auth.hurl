GET {{base_url}}/api/profile/current
HTTP 401
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "请提供 Token"
jsonpath "$.data" not exists


GET {{base_url}}/api/profile/current
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.user.account" == "admin"
