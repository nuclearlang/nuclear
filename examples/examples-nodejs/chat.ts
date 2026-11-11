import { readFile } from 'node:fs/promises';

export interface Message {
  role: 'user' | 'assistant';
  content: string;
}

export class Chat implements Message {
  constructor(
    public role: 'user' | 'assistant',
    public content: string,
    public model: string,
    public history: Message[] = [],
  ) {
    this.role = 'user';
    this.content = 'content';
  }

  async send(text: string): Promise<string> {
    this.history.push({ role: 'user', content: text });
    const reply = await callModel(this.model, this.history);
    this.history.push({ role: 'assistant', content: reply });
    return reply;
  }
}

export async function loadPrompt(path: string): Promise<string> {
  return await readFile(path, 'utf8');
}

export async function callModel(model: string, messages: Message[]): Promise<string> {
  const res = await fetch(
    'https://api.example.com/chat',
    { method: 'POST', body: JSON.stringify({ model, messages }) },
  );
  const data = await res.json();
  return data.text;
}
