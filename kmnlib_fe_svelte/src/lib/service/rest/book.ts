import type { BookService } from "$lib/service/api/book";
import { writable } from 'svelte/store';
import type { Book } from "$lib/entity/book";
import axios from "axios";
import type { MutexReadable } from '$lib/service';

export class RestBookService implements BookService {
	getBooksCache?: MutexReadable<Promise<Book[]>>;
	getBooks(): MutexReadable<Promise<Book[]>> {
		if (this.getBooksCache) {
			return this.getBooksCache;
		}
		const get = () => axios.get("/api/book").then((res) => {
			const books = res.data;
			return books as Book[];
		});

		const { subscribe, set } =  writable(get(), (set) => {
			const interval = setInterval(() => {
				set(get());
			}, 1000);
			return () => {
				clearInterval(interval);
				delete this.getBooksCache
			}
		});
		const result = {
			subscribe,
			update: () => set(get())
		}
		this.getBooksCache = result;
		return result;
	}

	getBookCache: Map<string, MutexReadable<Promise<Book>>> = new Map();

	getBook(id: string): MutexReadable<Promise<Book>> {
		const cache = this.getBookCache.get(id);
		if (cache) {
			return cache;
		}
		const get = () =>
			axios.get(`/api/book/${id}`).then((res) => {
				const book = res.data;
				return book as Book;
			});

		const { subscribe, set } = writable(get(), (set) => {
			const interval = setInterval(() => {
				set(get());
			}, 1000);
			return () => {
				clearInterval(interval);
				this.getBookCache.delete(id);
			}
		});
		const result = {
			subscribe,
			update: () => set(get())
		}
		this.getBookCache.set(id, result);
		return result;
	}
}
