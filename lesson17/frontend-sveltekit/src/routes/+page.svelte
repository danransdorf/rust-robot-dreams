<script lang="ts">
	import Chat from '$lib/components/Chat.svelte';
	import Connect from '$lib/components/Connect.svelte';
	import Login from '$lib/components/Login.svelte';
	import { processMessage } from '$lib/utils';
	import { connectWebsocket } from '$lib/utils/socket';
	import {
		isMessageServerResponse,
		type ProcessedMessage,
		type ServerResponse,
		type StreamRequest
	} from '$lib/utils/types';
	import Cookies from 'js-cookie';

	let connected = false;
	let authToken = /* Cookies.get('authToken') ??  */ '';

	let messages: ProcessedMessage[] = [];

	let ws: WebSocket | null = null;

	const connect = (address: string) => {
		ws = connectWebsocket(
			address,
			() => {
				connected = true;
			},
			(event) => {
				const serverResponse: ServerResponse = JSON.parse(event.data);
				console.log(serverResponse);

				if (isMessageServerResponse(serverResponse)) {
					messages = [...messages, processMessage(serverResponse.Message)];
				} else {
					authToken = serverResponse.Auth.token;
					Cookies.set('authToken', authToken, { expires: 1 });

					ws?.send(JSON.stringify({ ReadRequest: { jwt: authToken, amount: 10, offset: 0 } }));
				}
			},
			() => {
				connected = false;
			}
		);
	};

	const getAuth = (detail: { username: string; password: string; type: 'Register' | 'Login' }) => {
		const streamRequest: StreamRequest = {
			AuthRequest: {
				username: detail.username,
				password: detail.password,
				kind: detail.type
			}
		};

		ws?.send(JSON.stringify(streamRequest));
	};
</script>

<main>
	<h1 class="text-4xl">Chat Client</h1>
	{#if connected}
		<p>Connected</p>

		{#if authToken}
			<p>Logged in</p>
			<Chat {messages} />
		{:else}
			<Login on:login={(e) => getAuth(e.detail)} />
		{/if}
	{:else}
		<Connect on:connect={(e) => connect(e.detail.address)} />
	{/if}
</main>
