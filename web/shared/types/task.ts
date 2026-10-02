export type TaskPriority = 'none' | 'low' | 'medium' | 'high' | 'urgent';

export interface Task {
	id: number;
	name: string;
	description: string | null;
	due_at: string | null;
	completed_at: string | null;
	priority: TaskPriority;
	owner_id: number;
	project_id: number | null;
	parent_id: number | null;
	tag_ids: number[];
	created_at: string;
	updated_at: string;
}

export interface SaveTaskRequest {
	name: string;
	description?: string | null;
	due_at?: string | null;
	priority?: TaskPriority | null;
	project_id?: number | null;
	parent_id?: number | null;
	tag_ids?: number[];
}
