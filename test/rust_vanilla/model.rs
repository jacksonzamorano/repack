use chrono::NaiveDateTime;

pub struct Organization {
	id: i32,
	name: String,
	email: String,
}

pub struct UserWithOrganization {
	user: UserPublic,
	organization: Organization,
}

pub struct User {
	id: i32,
	name: String,
	email: String,
	password: String,
	login_count: i64,
	last_login: chrono::NaiveDateTime,
	total_cost: f64,
	org_id: i32,
	personal_org_id: i32,
}

pub struct List {
	id: i32,
	title: String,
	description: String,
	creator_user_id: i32,
	org_id: i32,
}

pub struct UserPublic {
	id: i32,
	name: String,
	email: String,
	login_count: i64,
	last_login: chrono::NaiveDateTime,
	total_cost: f64,
	org_id: i32,
	personal_org_id: i32,
}

pub struct Todo {
	id: i32,
	done: bool,
	title: String,
	description: String,
	creator_user_id: i32,
	assigned_user_id: Option<i32>,
	list_id: Option<i32>,
	org_id: i32,
}

