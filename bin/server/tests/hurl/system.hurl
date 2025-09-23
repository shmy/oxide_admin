# Should error when fetch system info without token
GET {{base_url}}/api/systems/info
HTTP 401
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "请提供 Token"
jsonpath "$.data" not exists

# Should success when fetch system info given valid token
GET {{base_url}}/api/systems/info
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists
jsonpath "$.data.process" exists
jsonpath "$.data.system" exists
