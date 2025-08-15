# User

## Queries
### UserByEmail
#### Query:
`SELECT users.id AS id, users.created_date AS created_date, users.last_login AS last_login, users.name AS name, users.email AS email, users.user_type AS user_type, users.subscription_id AS subscription_id, LOWER(name) || '_' || LOWER(email) AS email_id FROM users WHERE users.email = $1;`
#### Arguments:
- _email: String
### UsersByType
#### Query:
`SELECT users.id AS id, users.created_date AS created_date, users.last_login AS last_login, users.name AS name, users.email AS email, users.user_type AS user_type, users.subscription_id AS subscription_id, LOWER(name) || '_' || LOWER(email) AS email_id FROM users WHERE users.user_type = $1;`
#### Arguments:
- _typ: UserType
### DeleteUserById
#### Query:
`DELETE FROM users WHERE users.id = $1;`
#### Arguments:
- _id: UUID v4

# Token

## Queries

# UserWithToken

## Queries
### UserToken
#### Query:
`SELECT users.id AS user_id, t.token_value AS token_value FROM users INNER JOIN tokens t ON users.id = t.user_id WHERE users.id = $1;`
#### Arguments:
- _id: UUID v4

