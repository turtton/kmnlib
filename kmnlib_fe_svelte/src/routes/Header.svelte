<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import Sun from "lucide-svelte/icons/sun";
	import Moon from "lucide-svelte/icons/moon";

	import { toggleMode } from "mode-watcher";
	import { page } from "$app/stores";

	const pages = [
		{
			name: "KMN Lib",
			url: "/"
		},
		{
			name: "Book",
			url: "/book"
		}
	];
</script>

<header>
	<div class="container flex items-center space-x-2 px-4 py-2">
		<nav class="place-self-center">
			<ul class="flex space-x-4">
				{#each pages as content}
					<li aria-current={$page.url.pathname === content.url ? "page" : undefined}>
						<a href={content.url}>{content.name}</a>
					</li>
				{/each}
			</ul>
		</nav>
		<div class="flex flex-1 justify-end">
			<Button on:click={toggleMode} variant="outline" size="icon">
				<Sun
					class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
				/>
				<Moon
					class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
				/>
				<span class="sr-only">Toggle theme</span>
			</Button>
		</div>
	</div>
</header>

<style>
	header {
		border-bottom-width: 1px;
	}
	li[aria-current="page"] a {
		text-decoration: underline;
	}
</style>
