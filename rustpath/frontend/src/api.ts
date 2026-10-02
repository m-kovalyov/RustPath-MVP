export class ApiError extends Error {
  constructor(public status: number, message: string) { super(message) }
}
export async function api<T>(path: string, options: RequestInit = {}): Promise<T> {
  let response: Response
  try {
    response = await fetch(`/api${path}`, { ...options, credentials: 'same-origin', headers: { 'Content-Type': 'application/json', ...options.headers } })
  } catch {
    throw new Error('Не удалось связаться с сервером. Проверьте соединение и попробуйте ещё раз.')
  }
  if (!response.ok) {
    const data = await response.json().catch(() => ({}))
    throw new ApiError(response.status, data.error ?? `Ошибка сервера (${response.status}). Повторите позже.`)
  }
  return response.status === 204 ? undefined as T : response.json()
}
