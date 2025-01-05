<script>
	import { Minus, Settings, X } from 'lucide-svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	function handleSettings() {
		invoke('create_settings');
	}
	async function handleTray() {
		await getCurrentWindow().minimize();
	}
	async function handleExit() {
		let window = await getCurrentWindow();
		if (window.label === 'settings') {
			window.hide();
			return;
		}
		window.close();
	}
</script>

<div
	data-tauri-drag-region
	class="flex w-full justify-end gap-2 rounded-t-xl bg-black-600 p-2 text-white"
>
	{#await getCurrentWindow() then tauri_window}
		{#if tauri_window.label !== 'settings'}
			<button onclick={handleSettings}><Settings /></button>
		{/if}
	{/await}
	<button onclick={handleTray}><Minus /></button>
	<button onclick={handleExit}><X /></button>
</div>
