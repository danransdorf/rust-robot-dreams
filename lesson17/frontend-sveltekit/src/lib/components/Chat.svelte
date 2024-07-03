<script lang="ts">
	import type { ProcessedMessage } from '$lib/utils/types';

	export let messages: ProcessedMessage[] = [];
</script>

<div class="space-y-3">
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
