export interface Todo {
	id: number;
	done: boolean;
	title: string;
	description: string;
	creator_user_id: number;
	assigned_user_id?: number;
	list_id?: number;
	org_id: number;
}
