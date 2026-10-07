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
	active_acceleration: string;
	installed_acceleration: string | null;
	available_accelerations: AccelerationOption[];
	install_progress: InstallProgress | null;
	binary_path?: string | null;
}

export interface ManifestFlag {
	flag: string;
	name: string;
	description: string;
	type: string;
	important: boolean;
	default_value: string;
	options?: string[];
}

export interface RuntimeManifest {
	id: string;
	name: string;
	description: string;
	features?: string[];
	flags: ManifestFlag[];
}
