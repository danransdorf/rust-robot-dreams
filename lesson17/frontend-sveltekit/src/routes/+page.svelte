<script lang="ts">
	import Chat from '$lib/components/Chat.svelte';
	import Connect from '$lib/components/Connect.svelte';
	import Login from '$lib/components/Login.svelte';
	import { connectWebsocket } from '$lib/utils/socket';
	import {
		isFileVariant,
		isImageVariant,
		isMessageServerResponse,
		isTextVariant,
		type MessageResponse,
		type ServerResponse,
		type StreamRequest
	} from '$lib/utils/types';
	import Cookies from 'js-cookie';

	let connected = false;
	let authToken = Cookies.get('authToken') ?? '';

	let messages: MessageResponse[] = [];

	let ws: WebSocket | null = null;

	const processMessage = (message: MessageResponse) => {
		if (isImageVariant(message.content)) {
			const base64 = btoa(
				new Uint8Array(message.content.Image).reduce(
					(data, byte) => data + String.fromCharCode(byte),
					''
				)
			);

			return {
				id: message.id,
				user: message.user,
				content: {
					kind: 'Image',
					Image: base64
				}
			};
		} else if (isFileVariant(message.content)) {
			const base64 = btoa(
				new Uint8Array(message.content.File[1]).reduce(
					(data, byte) => data + String.fromCharCode(byte),
					''
				)
			);

			return {
				id: message.id,
				user: message.user,
				content: {
					kind: 'File',
					File: [message.content.File[0], base64] as const
				}
			};
		} else if (isTextVariant(message.content)) {
			return {
				id: message.id,
				user: message.user,
				content: {
					kind: 'Text',
					Text: message.content.Text
				}
			};
		} else {
			console.log('unknown message type', message);
			return {
				id: message.id,
				user: message.user,
				content: {
					kind: 'Text',
					Text: 'Unknown message type'
				}
			};
		}
	};

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
					messages = [...messages, serverResponse.Message];
				} else {
					authToken = serverResponse.AuthToken;
					Cookies.set('authToken', authToken, { expires: 1 });
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
