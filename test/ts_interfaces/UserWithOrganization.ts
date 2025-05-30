import type { UserPublic } from './UserPublic';
import type { Organization } from './Organization';
export interface UserWithOrganization {
	user: UserPublic;
	organization: Organization;
}
