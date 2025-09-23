# Should success when fetch roles given valid token
GET {{base_url}}/api/options/roles
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists
jsonpath "$.data[0]" exists

# Should success when fetch permissions given valid token
GET {{base_url}}/api/options/permissions
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists
jsonpath "$.data[0]" exists
