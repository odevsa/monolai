export interface CpuInfo {
	usage: number;
	cores: number;
	brand: string;
	frequency_mhz: number;
}

export interface RamInfo {
	total_bytes: number;
	used_bytes: number;
	free_bytes: number;
	percentage: number;
}

export interface OsInfo {
	name: string;
	kernel_version: string;
	os_version: string;
	hostname: string;
	uptime_seconds: number;
}

export interface VramInfo {
	total_bytes: number;
	used_bytes: number;
	free_bytes: number;
	percentage: number;
}

export interface GpuInfo {
	name: string;
	vendor: string;
	memory_total_bytes?: number | null;
	driver_version?: string | null;
	is_dedicated: boolean;
}

export interface SysInfo {
	cpu: CpuInfo;
	ram: RamInfo;
	os: OsInfo;
	gpu: GpuInfo | null;
	vram?: VramInfo | null;
	timestamp: number;
}

export interface HostMetricsTick {
	cpu_usage: number;
	ram_used_bytes: number;
	ram_total_bytes: number;
	ram_free_bytes: number;
	ram_percentage: number;
	gpu_usage: number | null;
	vram_used_bytes?: number | null;
	vram_total_bytes?: number | null;
	vram_free_bytes?: number | null;
	vram_percentage?: number | null;
	timestamp: number;
}
