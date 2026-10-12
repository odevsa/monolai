export interface ConfigStatus {
	is_valid: boolean;
	has_models: boolean;
	has_runtimes: boolean;
	created_auto_file: boolean;
	is_docker?: boolean;
	loaded_path: string | null;
	expected_path: string;
	models_dir: string | null;
	runtimes_dir?: string | null;
	host?: string;
	port?: number;
	error_message: string | null;
	example_yaml: string;
	cli_command_example?: string;
}

export interface HardwareReport {
	os: string;
	arch: string;
	available_accelerations: string[];
	recommended_acceleration: string;
	detected_gpus: string[];
}

export interface SetupConfigPayload {
	models: string;
	runtimes: string;
	hardware?: string;
	host: string;
	port: number;
}
