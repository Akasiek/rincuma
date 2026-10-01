export interface Project {
  id: number;
  name: string;
  description: string | null;
  color: string | null;
  archived_at: string | null;
  owner_id: number;
  created_at: string;
  updated_at: string;
}
