<script lang="ts">
	let connected = false;
	let ws;

	let address: string;

	const connect = () => {
		ws = new WebSocket(address || 'ws://localhost:3000');
		ws.onopen = () => {
			connected = true;
		};
		ws.onmessage = (event) => {
			console.log(event.data);
		};
		ws.onclose = () => {
			connected = false;
		};
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
