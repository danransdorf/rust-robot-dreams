import { type MessageResponse, isFileVariant, isImageVariant, isTextVariant } from './types';

export const processMessage = (message: MessageResponse) => {
	if (isImageVariant(message.content)) {
    
		const base64 = btoa(
			new Uint8Array(message.content.Image).reduce(
				(data, byte) => data + String.fromCharCode(byte),
				''
			)
		);

		return {
			id: message.id,
			username: message.username,
			user_id: message.user_id,
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
			username: message.username,
			user_id: message.user_id,
			content: {
				kind: 'File',
				File: [message.content.File[0], base64] as const
			}
		};
	} else if (isTextVariant(message.content)) {
		return {
			id: message.id,
			username: message.username,
			user_id: message.user_id,
			content: {
				kind: 'Text',
				Text: message.content.Text
			}
		};
	} else {
		console.log('unknown message type', message);
		return {
			id: message.id,
			username: message.username,
			user_id: message.user_id,
			content: {
				kind: 'Text',
				Text: 'Unknown message type'
			}
		};
	}
};
