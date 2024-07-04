import type { processMessage } from './index';

type Vec<T> = Array<T>;
type Option<T> = T | null;

type ImageVariant = { Image: Vec<number> };
type FileVariant = { File: [string, Vec<number>] };
type TextVariant = { Text: string };
export function isImageVariant(obj: MessageContent): obj is ImageVariant {
	return (obj as ImageVariant).Image !== undefined;
}
export function isFileVariant(obj: MessageContent): obj is FileVariant {
	return (obj as FileVariant).File !== undefined;
}
export function isTextVariant(obj: MessageContent): obj is TextVariant {
	return (obj as TextVariant).Text !== undefined;
}
export type MessageContent = ImageVariant | FileVariant | TextVariant;

export type User = {
	//** Option<i32> */
	id: Option<number>;
	username: string;
	password: string;
	//** Vec<u8> */
	salt: Vec<number>;
};

export type MessageResponse = {
	id: number;
	username: string;
	user_id: number;
	content: MessageContent;
};
export type Auth = {
	token: string;
	username: string;
	user_id: number;
};

export type AuthServerResponse = { Auth: Auth };
export type MessageServerResponse = { Message: MessageResponse };
export function isMessageServerResponse(obj: ServerResponse): obj is MessageServerResponse {
	return (obj as MessageServerResponse).Message !== undefined;
}

export type ServerResponse = AuthServerResponse | MessageServerResponse;

export type MessageRequest = {
	jwt: string;
	message: MessageContent;
};
export type AuthRequestKind = 'Login' | 'Register';
export type AuthRequest = {
	kind: AuthRequestKind;
	username: string;
	password: string;
};
export type ReadRequest = {
	jwt: string;
	amount: number;
  offset: number;
};
export type StreamRequest =
	| { MessageRequest: MessageRequest }
	| { AuthRequest: AuthRequest }
	| { ReadRequest: ReadRequest };

export type ProcessedMessage = ReturnType<typeof processMessage>;
