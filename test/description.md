# Enums
## UserType
* **Admin: `Admin`
* **User: `User`
* **Guest: `Guest`

# Objects
## User
**Record**: *This object is a record. A table (called `users`) will store the data.*
- **id**: UUID v4
	- Primary key
- **created_date**: Timestamp
	- Defaults to `NOW()`
- **last_login**: Timestamp
- **name**: String
- **user_type**: UserType
- **subscription_id**: String

## ContactInfo
**Record**: *This object is a record. A table (called `contacts`) will store the data.*
- **id**: UUID v4
	- Primary key
- **created_date**: Timestamp
	- Defaults to `NOW()`
- **email**: String
- **user_id**: UUID v4
	- References `User.id`

### Joins
**These joins will be added to your ContactInfo queries to fully load all of the requested items.**
- `j_user_id`: `self.user_id = users.id`
	- References `User`.

## FullUser
**Synthetic**: *This object cannot be stored in databases. It will be created as a view.*
- **id**: UUID v4
	- Primary key
- **created_date**: Timestamp
	- Defaults to `NOW()`
- **email**: String
- **user_id**: UUID v4
	- References `User.id`
- **name**: String

### Joins
**These joins will be added to your FullUser queries to fully load all of the requested items.**
- `j_user_id`: `self.user_id = users.id`
	- References `User`.

## UserList
**Struct**: *This object is meant for internal use only and will not be saved in a database.*
- **users**: User

