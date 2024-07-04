<script lang="ts">
	import Chat from '$lib/components/Chat.svelte';
	import Connect from '$lib/components/Connect.svelte';
	import Login from '$lib/components/Login.svelte';
	import { processMessage } from '$lib/utils/index';
	import { connectWebsocket } from '$lib/utils/socket';
	import {
		isMessageServerResponse,
		type ProcessedMessage,
		type ServerResponse,
		type StreamRequest
	} from '$lib/utils/types';

	let connected = false;

	let authToken = '';
	let user_id = -1;
	let username = '';

	let messages: ProcessedMessage[] = [];

	let ws: WebSocket | null = null;

	const connect = (address: string) => {
		ws = connectWebsocket(
			address,
			() => {
				connected = true;

				if (authToken) {
					ws?.send(JSON.stringify({ ReadRequest: { jwt: authToken, amount: 10, offset: 0 } }));
				}
			},
			(event) => {
				const serverResponse: ServerResponse = JSON.parse(event.data);
				console.log(serverResponse);

				if (isMessageServerResponse(serverResponse)) {
					messages = [...messages, processMessage(serverResponse.Message)];
					console.log(messages);
				} else if (serverResponse.Auth) {
					authToken = serverResponse.Auth.token;
					username = serverResponse.Auth.username;
					user_id = serverResponse.Auth.user_id;

					ws?.send(JSON.stringify({ ReadRequest: { jwt: authToken, amount: 20, offset: 0 } }));
				} else {
					//@ts-expect-error
					console.log('Error: ' + serverResponse.Error);
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

	function base64ToArrayBuffer(base64: string) {
		console.log(base64);
		var binaryString = atob(base64);
		var bytes = new Uint8Array(binaryString.length);
		for (var i = 0; i < binaryString.length; i++) {
			bytes[i] = binaryString.charCodeAt(i);
		}
		return Array.from(bytes);
	}

	const sendMessage = (data: { Text: string }) => {
		const streamRequest: StreamRequest = {
			MessageRequest: {
				jwt: authToken,
				message: {
					Text: data.Text
				}
			}
		};

		ws?.send(JSON.stringify(streamRequest));
	};
	const sendFile = (data: { File: [string, string] }) => {
		const streamRequest: StreamRequest = {
			MessageRequest: {
				jwt: authToken,
				message: {
					File: [data.File[0], base64ToArrayBuffer(data.File[1].split(",")[1])]
				}
			}
		};

		ws?.send(JSON.stringify(streamRequest));
	};
	const sendImage = (data: { Image: string }) => {
		const streamRequest: StreamRequest = {
			MessageRequest: {
				jwt: authToken,
				message: {
					Image: base64ToArrayBuffer(data.Image.split(',')[1])
				}
			}
		};

		ws?.send(JSON.stringify(streamRequest));
	};
</script>

<main>
	{#if connected}
		{#if authToken}
			<Chat
				{user_id}
				{username}
				{messages}
				on:message={(e) => sendMessage(e.detail)}
				on:image={(e) => sendImage(e.detail)}
				on:file={(e) => sendFile(e.detail)}
			/>
		{:else}
			<Login on:login={(e) => getAuth(e.detail)} />
		{/if}
	{:else}
		<Connect on:connect={(e) => connect(e.detail.address)} />
	{/if}
</main>
