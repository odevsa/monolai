export interface CpuInfo {
	usage: number;
	cores: number;
	brand: string;
	frequency_mhz: number;
	temperature_c?: number | null;
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

export interface GpuDeviceInfo {
	index: number;
	name: string;
	vendor: string;
	memory_total_bytes?: number | null;
	memory_used_bytes?: number | null;
	memory_free_bytes?: number | null;
	driver_version?: string | null;
	is_dedicated: boolean;
	usage?: number | null;
	temperature_c?: number | null;
	clock_mhz?: number | null;
}

export interface DiskItemInfo {
	name: string;
	mount_point: string;
	total_bytes: number;
	used_bytes: number;
	free_bytes: number;
	percentage: number;
}

export interface StorageInfo {
	total_bytes: number;
	used_bytes: number;
	free_bytes: number;
	percentage: number;
	disks: DiskItemInfo[];
}

export interface SysInfo {
	cpu: CpuInfo;
	ram: RamInfo;
	os: OsInfo;
	storage: StorageInfo;
	gpu: GpuInfo | null;
	vram?: VramInfo | null;
	gpus: GpuDeviceInfo[];
	timestamp: number;
}

export interface HostMetricsTick {
	cpu_usage: number;
	cpu_temp_c?: number | null;
	cpu_frequency_mhz?: number | null;
	ram_used_bytes: number;
	ram_total_bytes: number;
	ram_free_bytes: number;
	ram_percentage: number;
	storage_used_bytes: number;
	storage_total_bytes: number;
	storage_percentage: number;
	disk_read_bytes_sec: number;
	disk_write_bytes_sec: number;
	net_rx_bytes_sec: number;
	net_tx_bytes_sec: number;
	gpu_usage: number | null;
	vram_used_bytes?: number | null;
	vram_total_bytes?: number | null;
	vram_free_bytes?: number | null;
	vram_percentage?: number | null;
	gpus: GpuDeviceInfo[];
	timestamp: number;
}
