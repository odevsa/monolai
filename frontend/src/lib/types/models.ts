export interface ModelPreset {
	id: string;
	model_id: string;
	name: string;
	runtime_id: string;
	flags: Record<string, string> | string;
	is_default: boolean;
	created_at?: string;
}

export interface ModelRecord {
	id: string;
	name: string;
	filename: string;
	relative_path: string;
	file_path?: string;
	size_bytes: number;
	format: string;
	modality: string;
	architecture?: string;
	source_repo?: string;
	is_auxiliary?: boolean;
	auxiliary_type?: string | null;
	created_at?: string;
	presets: ModelPreset[];
	file_exists?: boolean;
	runtime?: string;
	flags?: string;
}

export interface ModelFileItem {
	name?: string;
	filename?: string;
	path?: string;
	relative_path?: string;
	absolute_path?: string;
	size?: number;
	size_bytes?: number;
	format?: string;
	modality?: string;
	architecture?: string;
	is_auxiliary?: boolean;
	auxiliary_type?: string | null;
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

export interface CreatePresetPayload {
	name: string;
	runtime_id: string;
	flags: Record<string, string> | string;
	is_default?: boolean;
}

export interface UpdatePresetPayload {
	name?: string;
	runtime_id?: string;
	flags?: Record<string, string> | string;
	is_default?: boolean;
}

export interface RepositoryRecord {
	id: string;
	name: string;
	repo_type: 'huggingface' | 'custom' | string;
	base_url: string;
	mirror_url?: string | null;
	api_token?: string | null;
	enabled: boolean;
	is_enabled?: boolean;
	is_builtin: boolean;
	config_json?: string;
	created_at?: string;
}

export interface CreateRepositoryPayload {
	id: string;
	name: string;
	repo_type: string;
	base_url: string;
	mirror_url?: string | null;
	api_token?: string | null;
	enabled?: boolean;
	is_enabled?: boolean;
}

export interface UpdateRepositoryPayload {
	name?: string;
	base_url?: string;
	mirror_url?: string | null;
	api_token?: string | null;
	enabled?: boolean;
	is_enabled?: boolean;
}

export interface DownloadRecord {
	id: string;
	model_name: string;
	url: string;
	filename?: string;
	file_path: string;
	total_bytes: number;
	downloaded_bytes: number;
	status: 'queued' | 'downloading' | 'paused' | 'completed' | 'failed' | 'cancelled';
	error_message?: string;
	created_at: string;
	updated_at: string;
	speed_bytes_sec: number;
}

export interface EnqueueDownloadPayload {
	url: string;
	model_name: string;
	filename: string;
	modality?: string;
	repo_id?: string;
	download_url?: string;
	size_bytes?: number;
	repository_id?: string;
}

export interface HubModelSibling {
	rfilename: string;
	size?: number;
	quant?: string;
	download_url?: string;
}

export interface HubModelItem {
	id: string;
	name?: string;
	author?: string;
	downloads?: number;
	likes?: number;
	pipeline_tag?: string;
	modality: string;
	siblings: HubModelSibling[];
	is_gated?: boolean;
	requires_auth?: boolean;
}

export interface HubModelReadme {
	content: string;
}
