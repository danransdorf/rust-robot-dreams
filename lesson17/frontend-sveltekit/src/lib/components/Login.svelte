<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { createEventDispatcher } from 'svelte';
	import Button from './ui/button/button.svelte';
	import Input from './ui/input/input.svelte';

	let username: string;
	let password: string;
	let createAccount = false;

	const dispatch = createEventDispatcher();

	const submit = () => {
		dispatch('login', { username, password, type: createAccount ? 'Register' : 'Login' });
	};

	const loginSubmit = () => {
		createAccount = false;
		submit();
	};
	const registerSubmit = () => {
		createAccount = true;
		submit();
	};
</script>

<Card.Root>
	<Card.Header class="text-xl font-bold">Authentication</Card.Header>
	<Card.Content class="space-y-4">
		<Input
			type="text"
			name="login"
			placeholder="Login"
			aria-label="Login"
			required
			bind:value={username}
		/>
		<Input
			type="password"
			name="password"
			placeholder="Password"
			aria-label="Password"
			required
			bind:value={password}
		/>

		<Card.Footer class="flex gap-4 px-0">
			<Button class="w-full" on:click={registerSubmit} variant="secondary">Register</Button>
			<Button class="w-full" on:click={loginSubmit}>Login</Button>
		</Card.Footer>
	</Card.Content>
</Card.Root>
