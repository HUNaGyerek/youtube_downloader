<script>
	let { children } = $props();
</script>

<section class="flex flex-col text-white">
	<div class="h-screen w-full rounded-xl bg-black-500">
		<div class="mt-14 flex w-full flex-row">
			<div class="w-28">
				<h1 class="ms-3 text-xl font-bold">Settings</h1>
				<div class="ms-5 flex flex-col">
					<a href="settings">Themes</a>
					<a href="#">asd</a>
				</div>
                <div></div>
			</div>
			<main class="w-auto">
				{@render children()}
			</main>
		</div>
	</div>
</section>
