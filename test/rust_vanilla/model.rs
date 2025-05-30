

pub struct OrgModel {
	user: User,
}

pub struct User {
	id: i32,
	name: String,
	email: String,
	password: String,
	org_id: i32,
	personal_org_id: i32,
}

