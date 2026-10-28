export interface Tag {
  id: number;
  task_count?: number;
  name: string;
  color: string | null;
  owner_id: number;
  created_at: string;
  updated_at: string;
}

export interface CreateTagRequest {
  name: string;
  color?: string | null;
}
