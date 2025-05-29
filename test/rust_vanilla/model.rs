

pub struct Organization {
	id: i32,
	name: String,
	email: String,
}

pub struct UserPublic {
	org_name: String,
	id: i32,
	name: String,
	email: String,
	password: String,
	org_id: i32,
	personal_org: i32,
}

pub struct User {
	id: i32,
	name: String,
	email: String,
	password: String,
	org_id: i32,
	personal_org: i32,
}

