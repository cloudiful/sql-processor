import assert from 'node:assert/strict'
import test from 'node:test'

import { nextTick } from 'vue'

import { useValidationProfiles } from './useValidationProfiles.js'

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

test('useValidationProfiles creates a replacement profile when deleting the last one', async () => {
  installStorage()
  const profiles = useValidationProfiles()

  const originalId = profiles.selectedValidationProfileId.value
  const replacement = profiles.deleteProfile()
  await nextTick()

  assert.ok(replacement)
  assert.equal(profiles.validationProfiles.value.length, 1)
  assert.notEqual(profiles.selectedValidationProfileId.value, originalId)
  assert.equal(profiles.selectedValidationProfileId.value, replacement.id)
  assert.equal(profiles.isValidationConfigExpanded.value, true)
})

test('useValidationProfiles saves profile changes back to the active list', () => {
  installStorage()
  const profiles = useValidationProfiles()

  profiles.validationProfile.value.profileName = '回归测试库'
  const saved = profiles.saveProfile()

  assert.equal(saved.profileName, '回归测试库')
  assert.equal(profiles.validationProfiles.value[0].profileName, '回归测试库')
  assert.equal(profiles.selectedValidationProfileId.value, saved.id)
})
