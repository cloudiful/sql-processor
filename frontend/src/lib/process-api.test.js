import assert from 'node:assert/strict'
import test from 'node:test'

import { processSqlRequest } from './process-api.js'

test('processSqlRequest throws JSON error payload on non-2xx response', async () => {
  global.fetch = async () =>
    new Response(JSON.stringify({ error: '后端炸了' }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' },
    })

  await assert.rejects(
    () => processSqlRequest({ sql: 'SELECT 1 FROM DUAL' }),
    /后端炸了/,
  )
})

test('processSqlRequest throws fallback error when non-2xx response is not JSON', async () => {
  global.fetch = async () =>
    new Response('gateway timeout', {
      status: 504,
      headers: { 'Content-Type': 'text/plain' },
    })

  await assert.rejects(
    () => processSqlRequest({ sql: 'SELECT 1 FROM DUAL' }),
    /处理失败，请稍后重试。/,
  )
})

test('processSqlRequest throws when 2xx response body is empty', async () => {
  global.fetch = async () =>
    new Response('', {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    })

  await assert.rejects(
    () => processSqlRequest({ sql: 'SELECT 1 FROM DUAL' }),
    /服务返回了空响应，请检查后端日志。/,
  )
})

test('processSqlRequest returns parsed payload on success', async () => {
  global.fetch = async (request) => {
    assert.equal(new URL(request.url).pathname, '/api/process')
    assert.equal(request.method, 'POST')
    assert.equal(request.headers.get('Content-Type'), 'application/json')

    const body = JSON.parse(await request.text())
    assert.equal(body.sql, 'SELECT 1 FROM DUAL')
    assert.equal(body.schema, 'EBANK')

    return new Response(JSON.stringify({
      processedSql: 'SELECT 1 FROM DUAL',
      messages: ['No processing changes required'],
    }), {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    })
  }

  const payload = await processSqlRequest({
    sql: 'SELECT 1 FROM DUAL',
    schema: 'EBANK',
  })

  assert.deepEqual(payload, {
    processedSql: 'SELECT 1 FROM DUAL',
    messages: ['No processing changes required'],
  })
})
