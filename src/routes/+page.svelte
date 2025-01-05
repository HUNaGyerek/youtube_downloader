<script>
	import { onMount } from 'svelte';

	let progress = $state(0);

	let isDownloaded = $state(true);
	function toggleDownloaded() {
		if (downloadFolder.value === '' || link.value === '') return;

		isDownloaded = !isDownloaded;
	}
	onMount(() => {
		setInterval(() => {
			if (progress === 100) {
				return;
			}
			progress += 1;
		}, 100);
	});
</script>

<section class="flex flex-col items-center text-white">
	<div class="flex w-full flex-col items-center rounded-t-xl bg-black-500">
		<h1 class="mt-20 text-3xl font-bold">Music Downloader</h1>
		<div class="mt-10 flex flex-col items-center">
			<button class="text-xl font-semibold decoration-blue-500 underline-offset-2 hover:underline"
				>Browsing</button
			>
			<p id="downloadFolder">D:\asd\asd</p>
		</div>
		<div class="mt-5 flex h-10 w-full flex-row justify-center gap-3">
			<input
				type="text"
				id="link"
				autocomplete="off"
				class="w-96 rounded-md bg-black-400 px-3 outline-2 outline-gray-400 focus:outline"
			/>
			<select
				name="extensions"
				id="ext"
				class="w-20 rounded-md bg-black-400 outline-2 outline-gray-400 focus:outline"
			>
				<option value="mp3">mp3</option>
				<option value="mp4">mp4</option>
				<option value="wav">wav</option>
			</select>
		</div>
		<div class="mt-2 flex w-full flex-row items-center justify-center gap-3">
			<progress
				id="progress-bar"
				class="h-6 w-96 {progress === 100 ? '[&::-webkit-progress-value]:rounded-lg' : ''}"
				max="100"
				value={+progress}
			></progress>
			<p class="w-20">{progress}%</p>
		</div>
		<div class="mb-10 mt-10">
			<button
				onclick={toggleDownloaded}
				class="text-xl font-semibold decoration-blue-500 underline-offset-2 hover:underline"
				>Download</button
			>
		</div>
	</div>
	{#if isDownloaded}
		<div class="mt-5 flex w-full flex-col items-center gap-3">
			<div class="flex flex-col items-center">
				<h2 class="font-semibold">Name</h2>
				<p>Name of song</p>
			</div>
			<div class="flex flex-col items-center">
				<h2 class="font-semibold">Lenght</h2>
				<p>00:00:00</p>
			</div>
			<div class="flex flex-col items-center">
				<h2 class="font-semibold">Author</h2>
				<p>Don't know</p>
			</div>
		</div>
	{/if}
</section>

<style lang="postcss">
	#progress-bar::-webkit-progress-bar {
		@apply rounded-md bg-black-400;
	}

	#progress-bar::-webkit-progress-value {
		@apply rounded-s-md bg-blue-500;
	}
</style>
