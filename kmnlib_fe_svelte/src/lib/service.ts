import type { Readable } from 'svelte/store';

export type MutexReadable<T> = Readable<T> & {
			update: () => void;
};
