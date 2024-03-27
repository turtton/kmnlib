import type { Book } from "$lib/entity/book";
import type { MutexReadable } from '$lib/service';

export interface CreateBook {
	name: string;
	amount: number;
}

export interface BookService {
	getBooks(): MutexReadable<Promise<Book[]>>;
	createBook(book: CreateBook): MutexReadable<Promise<Book>>;
	getBook(id: string): MutexReadable<Promise<Book>>;
	updateBook(id: string, book: CreateBook): MutexReadable<Promise<Book>>;
	deleteBook(id: string): MutexReadable<Promise<void>>;
}
