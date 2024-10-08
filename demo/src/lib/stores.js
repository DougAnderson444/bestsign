// Svelte stores.
import { writable } from 'svelte/store';

// Keep the Plog as a store, so we can call peerpiper.command(Put) to update it automatically
// each time plog changes

// logStore is Uint8Array
/** @type {Writable<Uint8Array>} */
export const logStore = writable(null);

// The vlad is a Uint8Array of bytes
/** @type {Writable<Uint8Array>} */
export const vladStore = writable(null);

export const piperStore = writable(null);
