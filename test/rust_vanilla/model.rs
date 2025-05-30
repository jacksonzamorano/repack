use chrono::NaiveDateTime;

pub struct Organization {
	id: i32,
	name: String,
	email: String,
}

pub struct OrgModel {
	user: UserPublic,
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

pub struct UserPublic {
	id: i32,
	name: String,
	email: String,
	password: String,
	login_count: i64,
	last_login: chrono::NaiveDateTime,
	total_cost: f64,
	org_id: i32,
	org_name: String,
}

pub struct UserPublicNoOrg {
	id: i32,
	name: String,
	email: String,
	password: String,
	login_count: i64,
	last_login: chrono::NaiveDateTime,
	total_cost: f64,
}

