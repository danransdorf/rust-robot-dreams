<script lang="ts">
	import type { ProcessedMessage } from '$lib/utils/types';
	import { Send } from 'lucide-svelte';

	export let messages: ProcessedMessage[] = [];

	let value = '';
	let files: FileList | null = null;
</script>

<div class="space-y-6">
	<details>
		<fieldset role="group">
			<summary>Send a message</summary>
			<input type="text" bind:value class="pr-12" />

			<button>
				<Send />
			</button>
		</fieldset>
	</details>

	<hr />

	<details>
		<fieldset role="group">
			<summary>Send a file</summary>
			<label for="file">
				<input type="file" name="file" bind:files class="!hidden" />
			</label>
			<button>
				<Send />
			</button>
		</fieldset>
	</details>

	<hr />

	<details>
		<fieldset role="group">
			<summary>Send an image</summary>
			<input type="text" bind:value class="pr-12" />

			<button>
				<Send />
			</button>
		</fieldset>
	</details>
	<div class="space-y-3 max-h-[70vh] overflow-scroll">
		{#each messages as message}
			<article class="space-y-1">
				<p class="font-bold text-xl">{message.username}:</p>

				{#if message.content.Image}
					<img src={`data:image/png;base64, ${message.content.Image}`} alt="" />
				{:else if message.content.Text}
					<p>{message.content.Text}</p>
				{:else if message.content.File}
					<a
						href={`data:application/octet-stream;base64, ${message.content.File[1]}`}
						download={message.content.File[0]}>Download file</a
					>
				{:else}
					<p>Unknown message type</p>
				{/if}
			</article>
		{/each}
	</div>
</div>
