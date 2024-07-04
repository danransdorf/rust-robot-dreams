<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import Button from './ui/button/button.svelte';
	import Input from './ui/input/input.svelte';
	import Label from './ui/label/label.svelte';

	let address = '';

	const dispatch = createEventDispatcher();
	const connect = () => {
		dispatch('connect', address);
	};
</script>

<div class="space-y-2">
	<Label for="address"
		>Address

		<Input name="address" bind:value={address} placeholder="ws://localhost:11111" />
	</Label>
	<Button on:click={connect}>Connect</Button>
</div>
