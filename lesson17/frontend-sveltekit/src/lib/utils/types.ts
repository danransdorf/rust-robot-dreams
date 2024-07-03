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
export function isTextVariant(obj: MessageContent): obj is { Text: string } {
	return (obj as { Text: string }).Text !== undefined;
}
export type MessageContent =
	| ImageVariant
	| FileVariant
	| { Text: string };

export interface User {
	//** Option<i32> */
	id: Option<number>;
	username: string;
	password: string;
	//** Vec<u8> */
	salt: Vec<number>;
}

export interface MessageResponse {
	id: number;
	user: User;
	content: MessageContent;
}

export type AuthServerResponse = { AuthToken: string };
export type MessageServerResponse = { Message: MessageResponse };
export function isMessageServerResponse(obj: ServerResponse): obj is MessageServerResponse {
	return (obj as MessageServerResponse).Message !== undefined;
}

export type ServerResponse = AuthServerResponse | MessageServerResponse;

export interface MessageRequest {
	jwt: string;
	message: MessageContent;
}
export type AuthRequestKind = 'Login' | 'Register';
export interface AuthRequest {
	kind: AuthRequestKind;
	username: string;
	password: string;
}
export interface ReadRequest {
	jwt: string;
	amount: number;
}
export type StreamRequest =
	| { MessageRequest: MessageRequest }
	| { AuthRequest: AuthRequest }
	| { ReadRequest: ReadRequest };
