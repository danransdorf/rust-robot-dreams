<script lang="ts">
	import { connect_ws } from '$lib/socket';

	let connected = false;

	let address: string;

	const connect = () => {
		connect_ws(
			address,
			() => {
				connected = true;
			},
			(event) => {
				console.log(event.data);
			},
			() => {
				connected = false;
			}
		);
	};
</script>

<main>
	<h1 class="text-4xl">Chat Client</h1>
	{#if connected}
		<p>Connected</p>
	{:else}
		<label for="address">Address</label>
		<input name="address" bind:value={address} placeholder="ws://localhost:3000" />
	{/if}
	<button on:click={connect}>Connect</button>
</main>
