export interface LogEntry {
	id: number;
	timestamp: string;
	level: string; // 'INFO' | 'WARN' | 'ERROR' | 'DEBUG'
	service: string;
	message: string;
}

export interface LogFilterParams {
	service?: string;
	level?: string;
	search?: string;
	limit?: number;
}
