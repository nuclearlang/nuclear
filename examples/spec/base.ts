// Result<E, T>: an operation that fails with an error E (Err) or succeeds with a value T (Ok)
export class Result<E, T> {
  private constructor(
    readonly isOk: boolean,
    private readonly okValue: T | undefined,
    private readonly errValue: E | undefined,
  ) {}

  static ok<T>(value: T): Result<never, T> {
    return new Result<never, T>(true, value, undefined);
  }

  static err<E>(error: E): Result<E, never> {
    return new Result<E, never>(false, undefined, error);
  }

  get isErr(): boolean {
    return !this.isOk;
  }

  get value(): T {
    if (!this.isOk) throw new Error('Err has no value');
    return this.okValue as T;
  }

  get error(): E {
    if (this.isOk) throw new Error('Ok has no error');
    return this.errValue as E;
  }

  andThen<F, U>(f: (value: T) => Result<F, U>): Result<E | F, U> {
    return this.isOk ? f(this.okValue as T) : Result.err(this.errValue as E);
  }

  orElse<F, U>(f: (error: E) => Result<F, U>): Result<F, T | U> {
    return this.isOk ? Result.ok(this.okValue as T) : f(this.errValue as E);
  }

  map<U>(f: (value: T) => U): Result<E, U> {
    return this.isOk ? Result.ok(f(this.okValue as T)) : Result.err(this.errValue as E);
  }

  getOrElse(f: (error: E) => T): T {
    return this.isOk ? (this.okValue as T) : f(this.errValue as E);
  }

  inspect(f: (value: T) => unknown): Result<E, T> {
    if (this.isOk) f(this.okValue as T);
    return this;
  }

  inspectErr(f: (error: E) => unknown): Result<E, T> {
    if (!this.isOk) f(this.errValue as E);
    return this;
  }
}

export function Ok<T = void>(value?: T): Result<never, T> {
  return Result.ok(value as T);
}

export function Err<E>(error: E): Result<E, never> {
  return Result.err(error);
}

// Examples of spec/nuclear-base.md: the same file is transpiled with typescript and with dart.

export enum Role {
  user = 'user',
  admin = 'administrator'
}

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  name?: string;
}

export class User {
  token?: string;

  constructor(
    public name: string,
    public email: string,
    public role: Role = Role.user,
  ) {}

  isAdmin(): boolean {
    return this.role === Role.admin;
  }

  rename(newName: string): void {
    this.name = newName;
  }
}

export class Users {
  get(name: string): Record<string, string> {
    return { 'name': name };
  }
}

export class Sales {
  getSales(name: string, lastName: string): Record<string, string> {
    console.log(`sale by: ${name}`);
    return new Users().get(lastName);
  }
}

export function addDoubles(a: number, b: number): number {
  return a + b;
}

export function greet(name: string): string {
  return `Hello ${name}`;
}

export function classify(n: number): string {
  const label = n === 0 ? 'zero' : n === 1 || n === 2 ? 'small' : 'big';
  return label;
}

export function sumUntil(limit: number): number {
  let total = 0;
  for (let i = 0; i < limit; i++) {
    if (i % 2 === 0) {
      total += i;
    } else if (i > 100) {
      break;
    }
  }
  return total;
}

/** if and match are values: the last line of each branch */
export function describe(user: User | null): string {
  let result;
  if (user == null) {
    const prefix = 'no';
    result = prefix + 'body';
  } else {
    result = user.name;
  }
  return result;
}

export async function loadUsers(ids: string[]): Promise<User[]> {
  const users = [];
  for (const id of ids) {
    try {
      users.push(await fetchUser(id));
    } catch (error) {
      console.log(error);
    }
  }
  return users;
}

export function fetchUser(id: string): User {
  return new User(id, `${id}@example.com`);
}

export function run() {
  const doubles = [1, 2, 3].map((n) => n * 2);
  const adults = Array.from({ length: 10 - 1 }, (_, i) => 1 + i).map((i: number) => i * 2);
  const log = (message: string) => console.log(message);
  log(`done ${doubles} ${adults}`);
  const config = { 'debug': true, 'retries': 3 };
  for (const [key, value] of Object.entries(config)) {
    console.log(key);
  }
  const ready = !(doubles.length === 0) && ['x'].includes('x');
  return ready;
}

/** destructuring: lists and maps */
export function bounds(xs: number[]): Record<string, number> {
  return { 'min': xs[0], 'max': xs[1] };
}

export function unpack(xs: number[]): number {
  const [head, ...tail] = xs;
  const { min, max } = bounds(xs);
  return head + tail.length + min + max;
}

/** generators: yield */
export function* countTo(limit: number): Generator<number> {
  for (let i = 0; i < limit; i++) {
    yield i;
  }
}

/** generics with a bound */
export interface Comparable {
  compareTo(other: Comparable): number;
}

export function larger<T extends Comparable>(a: T, b: T): T {
  return a.compareTo(b) > 0 ? a : b;
}

/** if inside a list */
export function labels(loading: boolean): string[] {
  return ['start', ...(loading ? ['loading'] : []), 'end'];
}

/** results: errors propagate on their own */
export function parseAge(text: string): Result<string, number> {
  if (text === '') {
    return Err('empty');
  }
  return Ok(text.length);
}

export function describeAge(text: string): Result<string, string> {
  const ageResult = parseAge(text);
  if (ageResult.isErr) return Err(ageResult.error);
  const age = ageResult.value;
  const safeResult = parseAge('0').orElse((e) => Ok(0));
  if (safeResult.isErr) return Err(safeResult.error);
  const safe = safeResult.value;
  return Ok(`${age + safe}`);
}

export function ageLabel(text: string): string {
  const describeAgeResult = describeAge(text);
  if (describeAgeResult.isOk) {
    return describeAgeResult.value;
  } else {
    return `error: ${describeAgeResult.error}`;
  }
}

/** alias: ageText is the same function as Ages.text */
export class Ages {
  text(age: number): string {
    return `${age} years`;
  }
}

export class Profile {
  ageText(age: number): string {
    return new Ages().text(age);
  }
}
