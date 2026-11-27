declare module "react" {
  export function useState<T>(initial: T): [T, (next: T | ((previous: T) => T)) => void];
}
