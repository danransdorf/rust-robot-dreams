<script lang="ts">
	import * as Tabs from '$lib/components/ui/tabs';
	import type { ProcessedMessage } from '$lib/utils/types';
	import { Download, Send } from 'lucide-svelte';
	import { createEventDispatcher } from 'svelte';
	import Button from './ui/button/button.svelte';
	import * as Card from './ui/card';
	import Input from './ui/input/input.svelte';

	export let messages: ProcessedMessage[] = [];
	export let user_id: number;
	export let username: string;

	let value = '';
	let files: FileList | null = null;
	let images: FileList | null = null;
	let imagesInput: HTMLInputElement;
	let filesInput: HTMLInputElement;

	let chat: HTMLDivElement;

	$: messages,
		{
			if (chat) {
				chat.scrollTop = chat.scrollHeight;
			}
		};

	const dispatch = createEventDispatcher();

	const sendMessage = () => {
		if (value) {
			dispatch('message', { Text: value });
			value = '';
		}
	};

	const sendFile = () => {
		if (files) {
			const reader = new FileReader();
			reader.onload = () => {
				dispatch('file', { File: [(files as FileList)[0].name, reader.result] });
				files = null;
				filesInput.value = '';
			};
			reader.readAsDataURL(files[0]);
		}
	};

	const sendImage = () => {
		if (images) {
			const reader = new FileReader();
			reader.onload = () => {
				dispatch('image', { Image: reader.result });
				images = null;
				imagesInput.value = '';
			};
			reader.readAsDataURL(images[0]);
		}
	};
</script>

<Card.Root>
	<Card.Content class="space-y-6">
		<div class="flex flex-col gap-3 max-h-[50vh] overflow-y-auto" bind:this={chat}>
			{#each messages as message}
				<article class={`flex flex-col ${message.user_id === user_id && 'items-end'}`}>
					<p class="font-bold text-xl">
						{#if message.user_id === user_id}
							You:
						{:else}
							{message.username}:
						{/if}
					</p>

					{#if message.content.Image !== undefined}
						<img src={`data:image/png;base64,${message.content.Image}`} alt="" />
					{:else if message.content.Text !== undefined}
						{#if message.content.Text === ''}
							<p><i>**empty message**</i></p>
						{:else}
							<p>{message.content.Text}</p>
						{/if}
					{:else if message.content.File !== undefined}
						<a
							class="underline flex gap-2 items-center"
							href={`data:application/octet-stream;base64, ${message.content.File[1]}`}
							download={message.content.File[0]}
						>
							<Download />{message.content.File[0]}
						</a>
					{:else}
						<p>Unknown message type</p>
					{/if}
				</article>
			{/each}
		</div>
		<Tabs.Root>
			<Tabs.Content value="message">
				<div class="flex gap-2">
					<Input type="text" placeholder="Type your message..." bind:value class="pr-12" />
					<Button on:click={sendMessage}>
						<Send />
					</Button>
				</div>
			</Tabs.Content>
			<Tabs.Content value="image">
				<div class="flex gap-2">
					<input
						bind:this={imagesInput}
						type="file"
						bind:files={images}
						accept="image/png"
						class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
					/>
					<Button on:click={sendImage}>
						<Send />
					</Button>
				</div>
			</Tabs.Content>
			<Tabs.Content value="file">
				<div class="flex gap-2">
					<input
						bind:this={filesInput}
						type="file"
						bind:files
						class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
					/>
					<Button on:click={sendFile}>
						<Send />
					</Button>
				</div>
			</Tabs.Content>

			<Tabs.List class="w-full [&>*]:w-full">
				<Tabs.Trigger value="message">Send message</Tabs.Trigger>
				<Tabs.Trigger value="file">Send file</Tabs.Trigger>
				<Tabs.Trigger value="image">Send PNG</Tabs.Trigger>
			</Tabs.List>
		</Tabs.Root>

		Signed in as {username}
	</Card.Content>
</Card.Root>
