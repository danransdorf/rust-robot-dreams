<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	let username: string;
	let password: string;
	let createAccount = false;

	const dispatch = createEventDispatcher();

	const submit = () => {
		dispatch('login', { username, password, type: createAccount ? 'Register' : 'Login' });
	};
</script>

<main>
	<h1>Sign in</h1>
	<input
		type="text"
		name="login"
		placeholder="Login"
		aria-label="Login"
		autoComplete="username"
		required
		bind:value={username}
	/>
	<input
		type="password"
		name="password"
		placeholder="Password"
		aria-label="Password"
		autoComplete="current-password"
		required
		bind:value={password}
	/>
	<fieldset>
		<label for="remember">
			<input
				type="checkbox"
				role="switch"
				id="remember"
				name="remember"
				bind:checked={createAccount}
			/>
			Create an account
		</label>
	</fieldset>
	<button on:click={submit}> Login </button>
</main>
