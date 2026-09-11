import createClient from 'openapi-fetch'

import type { paths } from './api-types'

type ProcessRequest = paths['/api/process']['post']['requestBody']['content']['application/json']
type ProcessResponse = paths['/api/process']['post']['responses'][200]['content']['application/json']

const client = createClient<paths>({
  baseUrl:
    typeof window !== 'undefined' && window.location?.origin
      ? window.location.origin
      : 'http://localhost',
})

export async function processSqlRequest(input: ProcessRequest): Promise<ProcessResponse> {
  let result
  try {
    result = await client.POST('/api/process', {
      body: input,
      fetch: globalThis.fetch,
    })
  } catch (error) {
    if (error instanceof SyntaxError) {
      throw new Error('服务返回了空响应，请检查后端日志。')
    }
    throw error
  }

  const { data, error, response } = result

  if (!response.ok || error) {
    const message = error && typeof error === 'object' && 'error' in error ? error.error : undefined
    throw new Error(typeof message === 'string' ? message : '处理失败，请稍后重试。')
  }

  if (!data) {
    throw new Error('服务返回了空响应，请检查后端日志。')
  }

  return data
}
