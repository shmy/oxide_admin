# Should success when fetch roles given valid token
GET {{base_url}}/api/system/options/role
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" count == 1

# Should success when fetch permissions given valid token
GET {{base_url}}/api/system/options/permission
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" count == 1

# Should success when fetch menus given valid token
GET {{base_url}}/api/system/options/menu
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" count == 1
