package main;
import "github.com/google/uuid"
import "time"
import "database/sql"


type UserType string

const (
	UserTypeAdmin UserType = "Admin"
	UserTypeUser UserType = "User"
	UserTypeGuest UserType = "Guest"
)

type User struct {
	Id uuid.UUID `json:"id"`
	CreatedDate time.Time `json:"created_date"`
	LastLogin *time.Time `json:"last_login"`
	Name string `json:"name"`
	Email string `json:"email"`
	UserType UserType `json:"user_type"`
	SubscriptionId *string `json:"subscription_id"`
}
func UserByEmail(db *sql.DB, _email string) {
	db.Query("SELECT users.id AS id, users.created_date AS created_date, users.last_login AS last_login, users.name AS name, users.email AS email, users.user_type AS user_type, users.subscription_id AS subscription_id FROM users WHERE users.email = $1;", _email)
}
func UsersByType(db *sql.DB, _typ UserType) {
	db.Query("SELECT users.id AS id, users.created_date AS created_date, users.last_login AS last_login, users.name AS name, users.email AS email, users.user_type AS user_type, users.subscription_id AS subscription_id FROM users WHERE users.user_type = $1;", _typ)
}
func DeleteUserById(db *sql.DB, _id uuid.UUID) {
	db.Query("DELETE FROM users WHERE users.id = $1;", _id)
}
type Token struct {
	Id uuid.UUID `json:"id"`
	CreatedDate time.Time `json:"created_date"`
	UserId uuid.UUID `json:"user_id"`
	TokenValue uuid.UUID `json:"token_value"`
}
type UserWithToken struct {
	UserId uuid.UUID `json:"user_id"`
	TokenValue uuid.UUID `json:"token_value"`
}
func UserToken(db *sql.DB, _id uuid.UUID) {
	db.Query("SELECT users.id AS user_id, t.token_value AS token_value FROM users INNER JOIN tokens t ON users.id = t.user_id WHERE users.id = $1;", _id)
}
