# Should success when change password given valid token
POST {{base_url}}/api/profile/password
Authorization: Bearer {{access_token}}
{
    "new_password": "123123",
    "confirm_new_password": "123123",
    "password": "123456"
}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null

# Should success when refresh token given valid refresh token
POST {{base_url}}/api/auth/token
{
    "token": "{{refresh_token}}"
}
HTTP 200
[Captures]
access_token: jsonpath "$.data.access_token"
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" exists

# Should success when sign out given valid token
POST {{base_url}}/api/profile/sign_out
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data" == null