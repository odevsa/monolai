export interface ModelRecord {
	id: string;
	runtime: string;
	flags: string;
	created_at?: string;
	file_exists?: boolean;
}

export interface ModelFileItem {
	name?: string;
	filename?: string;
	path?: string;
	relative_path?: string;
	absolute_path?: string;
	size?: number;
	size_bytes?: number;
}

export interface RunningModelStatus {
	model_id: string;
	runtime_id: string;
	pid: number;
	port: number;
	state: { status: string; message?: string } | string;
	idle_seconds: number;
}

export type RunningModelStateType = 'ready' | 'loading' | 'error' | 'idle';

export interface CreateModelPayload {
	id: string;
	runtime: string;
	flags: string;
}

export interface UpdateModelPayload {
	runtime?: string;
	flags?: string;
}
