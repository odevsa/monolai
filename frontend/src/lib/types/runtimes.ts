export interface AccelerationOption {
	id: string;
	label: string;
	is_recommended: boolean;
}

export interface InstallProgress {
	runtime_id: string;
	status: 'idle' | 'downloading' | 'extracting' | 'completed' | 'error' | string;
	percent: number;
	speed_mbps: number;
	downloaded_bytes: number;
	total_bytes: number;
	error_message: string | null;
	message?: string | null;
}

export interface SystemDependency {
	id: string;
	name: string;
	description: string;
	check_library?: string;
	check_binary?: string;
	instructions?: Record<string, string>;
}

export interface SystemDependencyStatus {
	id: string;
	name: string;
	description: string;
	is_satisfied: boolean;
	install_command?: string | null;
}

export interface Runtime {
	id: string;
	name: string;
	version: string;
	icon: string | null;
	website: string | null;
	description: string;
	features: string[];
	is_installed: boolean;
	installed_path: string | null;
	installed_version?: string | null;
	has_update?: boolean;
	active_acceleration: string;
	installed_acceleration: string | null;
	available_accelerations: AccelerationOption[];
	install_progress: InstallProgress | null;
	binary_path?: string | null;
	release_notes_url?: string | null;
	system_dependencies?: SystemDependencyStatus[];
}

export interface ManifestFlag {
	flag: string;
	name: string;
	description: string;
	type: string;
	important?: boolean;
	model?: boolean;
	default_value: string;
	options?: string[];
}

export interface RuntimeManifest {
	id: string;
	name: string;
	description: string;
	binary_name?: string;
	variables?: Record<string, string>;
	features?: string[];
	flags: ManifestFlag[];
	system_dependencies?: SystemDependency[];
}
