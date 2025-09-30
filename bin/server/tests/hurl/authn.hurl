# Should error when fetch current user without token
GET {{base_url}}/api/profile/current
HTTP 401
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" == "Access token is required"
jsonpath "$.data" not exists

# Should success when fetch current user given valid token
GET {{base_url}}/api/profile/current
Authorization: Bearer {{access_token}}
HTTP 200
[Asserts]
jsonpath "$.status" == 0
jsonpath "$.msg" == "OK"
jsonpath "$.data.user.account" == "admin"

# Should error when sign in with invalid account
POST {{base_url}}/api/authn/sign_in
{
    "account": "fake_account",
    "password": "123456"
}
HTTP 200
[Asserts]
jsonpath "$.status" == 1
jsonpath "$.msg" contains "Too Many Requests"
jsonpath "$.data" not exists
