/** Rejection handler for fire-and-forget Tauri calls: the UI stays usable, the cause lands in the console. */
export const logFailure = (action: string) => (error: unknown) => console.error(`${action} failed`, error)
