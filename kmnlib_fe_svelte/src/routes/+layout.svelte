<script lang="ts">
	import "../app.pcss";
	import { ModeWatcher } from "mode-watcher";
	import Header from "./Header.svelte";
</script>

<ModeWatcher />
<Header />
<slot />
