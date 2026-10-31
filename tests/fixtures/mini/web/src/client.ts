/** Base HTTP client. */
export class HttpClient {
  get(path: string): Promise<string> {
    return fetch(path).then((r) => r.text());
  }
}

export interface Handler {
  handle(): void;
}
