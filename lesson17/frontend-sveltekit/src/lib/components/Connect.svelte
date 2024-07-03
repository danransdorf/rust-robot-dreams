<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	let address = '';

	const dispatch = createEventDispatcher();
	const connect = () => {
		dispatch('connect', address);
	};
</script>

<label for="address">Address</label>
<input name="address" bind:value={address} placeholder="ws://localhost:11111" />
<button on:click={connect}>Connect</button>
