

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
	org_id: i32,
	personal_org_id: i32,
}

pub struct UserPublic {
	id: i32,
	name: String,
	email: String,
	password: String,
	org_id: i32,
	org_name: String,
}

pub struct UserPublicNoOrg {
	id: i32,
	name: String,
	email: String,
	password: String,
}

