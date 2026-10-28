export interface Project {
  id: number;
  task_count?: number;
  name: string;
  description: string | null;
  color: string | null;
  archived_at: string | null;
  owner_id: number;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectRequest {
  name: string;
  description?: string | null;
  color?: string | null;
}
