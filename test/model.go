package main;
import "time"
import "database/sql"
import "github.com/google/uuid"


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
	EmailId string `json:"email_id"`
}
func UserByEmail(db *sql.DB, _email string) (*User, error) {
	rows, err := db.Query("SELECT users.id AS id, users.created_date AS created_date, users.last_login AS last_login, users.name AS name, users.email AS email, users.user_type AS user_type, users.subscription_id AS subscription_id, LOWER(name) || '_' || LOWER(email) AS email_id FROM users WHERE users.email = $1;", _email)
	if err != nil {
		return nil, err		
	}
	defer rows.Close()
	
	return nil, nil
}
func UsersByType(db *sql.DB, _typ UserType) ([]User, error) {
	values := make([]User, 0)
	rows, err := db.Query("SELECT users.id AS id, users.created_date AS created_date, users.last_login AS last_login, users.name AS name, users.email AS email, users.user_type AS user_type, users.subscription_id AS subscription_id, LOWER(name) || '_' || LOWER(email) AS email_id FROM users WHERE users.user_type = $1;", _typ)
	if err != nil {
		return values, err		
	}
	defer rows.Close()
	
	return values, nil
}
func DeleteUserById(db *sql.DB, _id uuid.UUID) error {
	rows, err := db.Query("DELETE FROM users WHERE users.id = $1;", _id)
	if err != nil {
		return err		
	}
	defer rows.Close()
	return nil
}
func CreateUser(db *sql.DB, __id uuid.UUID, __name string, __email string, __user_type UserType) (*User, error) {
	rows, err := db.Query("WITH users AS (INSERT INTO users (id, name, email, user_type) VALUES ($1, $2, $3, $4) RETURNING *) AS users SELECT users.id AS id, users.created_date AS created_date, users.last_login AS last_login, users.name AS name, users.email AS email, users.user_type AS user_type, users.subscription_id AS subscription_id, LOWER(name) || '_' || LOWER(email) AS email_id FROM users;", __id, __name, __email, __user_type)
	if err != nil {
		return nil, err		
	}
	defer rows.Close()
	
	return nil, nil
}
func UpdateUserEmail(db *sql.DB, _id uuid.UUID, _email string) error {
	rows, err := db.Query("WITH users AS (UPDATE users SET email = $1 WHERE id = $2 RETURNING *) SELECT users.id AS id, users.created_date AS created_date, users.last_login AS last_login, users.name AS name, users.email AS email, users.user_type AS user_type, users.subscription_id AS subscription_id, LOWER(name) || '_' || LOWER(email) AS email_id FROM users;", _id, _email)
	if err != nil {
		return err		
	}
	defer rows.Close()
	return nil
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
func UserToken(db *sql.DB, _id uuid.UUID) ([]UserWithToken, error) {
	values := make([]UserWithToken, 0)
	rows, err := db.Query("SELECT users.id AS user_id, t.token_value AS token_value FROM users INNER JOIN tokens t ON users.id = t.user_id WHERE users.id = $1;", _id)
	if err != nil {
		return values, err		
	}
	defer rows.Close()
	
	return values, nil
}
