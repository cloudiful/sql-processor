import assert from 'node:assert/strict'
import test from 'node:test'

import {
  buildValidationConfig,
  loadSelectedValidationProfileId,
  loadValidationProfiles,
  saveSelectedValidationProfileId,
  saveValidationProfiles,
} from './validation-profiles.js'

function installStorage(raw = new Map()) {
  global.window = {
    localStorage: {
      getItem(key) {
        return raw.has(key) ? raw.get(key) : null
      },
      setItem(key, value) {
        raw.set(key, String(value))
      },
      removeItem(key) {
        raw.delete(key)
      },
    },
  }

  return raw
}

test('loadValidationProfiles falls back to a default profile on invalid storage data', () => {
  const storage = installStorage(new Map([
    ['sql-processor.validationProfiles', '{bad json'],
  ]))

  const profiles = loadValidationProfiles()
  assert.equal(profiles.length, 1)
  assert.equal(typeof profiles[0].id, 'string')
  assert.equal(storage.get('sql-processor.validationProfiles'), '{bad json')
})

test('save/load selected validation profile id round-trips against existing profiles', () => {
  installStorage()
  const profiles = loadValidationProfiles()
  profiles.push({
    ...profiles[0],
    id: 'profile-2',
    profileName: '测试库 2',
  })

  saveValidationProfiles(profiles)
  saveSelectedValidationProfileId('profile-2')

  const loadedProfiles = loadValidationProfiles()
  const selectedId = loadSelectedValidationProfileId(loadedProfiles)
  assert.equal(selectedId, 'profile-2')
})

test('buildValidationConfig normalizes string inputs for transport', () => {
  const config = buildValidationConfig({
    enabled: true,
    profileName: '本地库',
    host: ' db.internal ',
    port: '1521',
    service: ' XE ',
    username: ' scott ',
    password: 'tiger',
    timeoutSeconds: '5',
  })

  assert.deepEqual(config, {
    enabled: true,
    profileName: '本地库',
    host: 'db.internal',
    port: 1521,
    service: 'XE',
    username: 'scott',
    password: 'tiger',
    timeoutSeconds: 5,
  })
})

test('saveValidationProfiles never persists Oracle passwords', () => {
  const storage = installStorage()
  saveValidationProfiles([{
    id: 'profile-secret',
    profileName: '受保护配置',
    enabled: true,
    host: 'db.internal',
    port: '1521',
    service: 'XE',
    username: 'scott',
    password: 'secret',
    timeoutSeconds: '5',
  }])

  assert.equal(JSON.parse(storage.get('sql-processor.validationProfiles'))[0].password, '')
  assert.equal(loadValidationProfiles()[0].password, '')
})
