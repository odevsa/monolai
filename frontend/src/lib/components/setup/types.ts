export interface HardwareOption {
	id: string;
	titleKey: string;
	descKey: string;
}

export function getHardwareName(id?: string | null): string {
	switch (id?.toLowerCase()) {
		case 'cuda':
			return 'NVIDIA CUDA';
		case 'rocm':
			return 'AMD ROCm';
		case 'vulkan':
			return 'Vulkan';
		case 'cpu':
			return 'CPU (AVX2)';
		case 'auto':
			return 'Auto Detect';
		default:
			return id ? id.toUpperCase() : 'Unknown';
	}
}

export const HARDWARE_OPTIONS: HardwareOption[] = [
	{
		id: 'auto',
		titleKey: 'setup.hardwareAutoTitle',
		descKey: 'setup.hardwareAutoDesc'
	},
	{
		id: 'cpu',
		titleKey: 'setup.hardwareCpuTitle',
		descKey: 'setup.hardwareCpuDesc'
	},
	{
		id: 'cuda',
		titleKey: 'setup.hardwareCudaTitle',
		descKey: 'setup.hardwareCudaDesc'
	},
	{
		id: 'vulkan',
		titleKey: 'setup.hardwareVulkanTitle',
		descKey: 'setup.hardwareVulkanDesc'
	},
	{
		id: 'rocm',
		titleKey: 'setup.hardwareRocmTitle',
		descKey: 'setup.hardwareRocmDesc'
	}
];
