<script>
	import WindowControls from '$lib/components/WindowControls.svelte';
	import '../app.css';
	let { children } = $props();
</script>

<section class="relative">
	<div class="absolute w-full">
		<WindowControls />
	</div>

	<main class="bg-black-600 h-screen w-screen rounded-xl">
		{@render children()}
	</main>
</section>
