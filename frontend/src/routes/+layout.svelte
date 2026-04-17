<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { onMount } from 'svelte';
	import { restoreSession } from '$lib/stores/auth';

	let { children } = $props();

	onMount(() => {
		void restoreSession();
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
{@render children()}
