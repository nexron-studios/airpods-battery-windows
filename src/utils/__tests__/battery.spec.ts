import { describe, expect, it } from 'vitest'
import { airPodsNoticeOf, airPodsTilesOf, deviceTilesOf, toneOf } from '../battery'

const NOW_MS = 1_800_000_000_000
const MINUTE_MS = 60_000
const NO_LEVELS = { left: null, right: null, case: null }
const SEEN_NOW = { left: NOW_MS, right: NOW_MS, case: NOW_MS }

describe('toneOf', () => {
  it('should mark a level at the low threshold as low', () => {
    expect(toneOf({ percent: 20, isCharging: false })).toBe('low')
  })

  it('should not mark a low level as low while it charges', () => {
    expect(toneOf({ percent: 5, isCharging: true })).toBe('ok')
  })

  it('should mark a missing level as unknown', () => {
    expect(toneOf(null)).toBe('unknown')
  })
})

describe('airPodsTilesOf', () => {
  it('should return left, right and case with their levels when recently seen', () => {
    const tiles = airPodsTilesOf(
      {
        status: 'seen',
        battery: { left: { percent: 38, isCharging: false }, right: { percent: 45, isCharging: true }, case: null },
        partsSeenMs: { left: NOW_MS - MINUTE_MS, right: NOW_MS - MINUTE_MS, case: null },
        lastSeenMs: NOW_MS - MINUTE_MS,
      },
      NOW_MS,
    )

    expect(tiles).toEqual([
      { id: 'airpod_left', kind: 'airpod_left', name: null, level: { percent: 38, isCharging: false }, isStale: false },
      { id: 'airpod_right', kind: 'airpod_right', name: null, level: { percent: 45, isCharging: true }, isStale: false },
      { id: 'airpods_case', kind: 'airpods_case', name: null, level: null, isStale: true },
    ])
  })

  it('should dim only the pod that has gone quiet in the closed case', () => {
    const tiles = airPodsTilesOf(
      {
        status: 'seen',
        battery: { left: { percent: 0, isCharging: false }, right: { percent: 24, isCharging: false }, case: null },
        partsSeenMs: { left: NOW_MS - 25 * MINUTE_MS, right: NOW_MS, case: null },
        lastSeenMs: NOW_MS,
      },
      NOW_MS,
    )

    expect(tiles.map(({ id, isStale }) => ({ id, isStale }))).toEqual([
      { id: 'airpod_left', isStale: true },
      { id: 'airpod_right', isStale: false },
      { id: 'airpods_case', isStale: true },
    ])
  })

  it('should return three empty stale tiles while waiting for the first advert', () => {
    const tiles = airPodsTilesOf({ status: 'waiting' }, NOW_MS)

    expect(tiles.map(({ level, isStale }) => ({ level, isStale }))).toEqual([
      { level: null, isStale: true },
      { level: null, isStale: true },
      { level: null, isStale: true },
    ])
  })
})

describe('deviceTilesOf', () => {
  it('should show an iPhone as an iPhone tile with its Windows name', () => {
    expect(deviceTilesOf([{ name: 'iPhone', percent: 87 }])).toEqual([
      { id: 'device:iPhone', kind: 'iphone', name: 'iPhone', level: { percent: 87, isCharging: false }, isStale: false },
    ])
  })

  it('should show any other device as a generic tile', () => {
    expect(deviceTilesOf([{ name: 'MX KEYS S', percent: 65 }])[0].kind).toBe('other')
  })
})

describe('airPodsNoticeOf', () => {
  it('should say nothing while the values are fresh', () => {
    const notice = airPodsNoticeOf({ status: 'seen', battery: NO_LEVELS, partsSeenMs: SEEN_NOW, lastSeenMs: NOW_MS - MINUTE_MS }, NOW_MS)

    expect(notice).toEqual({ kind: 'none' })
  })

  it('should report how many whole minutes ago stale values were seen', () => {
    const notice = airPodsNoticeOf(
      { status: 'seen', battery: NO_LEVELS, partsSeenMs: SEEN_NOW, lastSeenMs: NOW_MS - 14.5 * MINUTE_MS },
      NOW_MS,
    )

    expect(notice).toEqual({ kind: 'stale', minutes: 14 })
  })

  it('should point to the case when only some parts have gone quiet', () => {
    const notice = airPodsNoticeOf(
      { status: 'seen', battery: NO_LEVELS, partsSeenMs: { ...SEEN_NOW, left: null }, lastSeenMs: NOW_MS },
      NOW_MS,
    )

    expect(notice).toEqual({ kind: 'parts_quiet' })
  })

  it('should not ask to open the case when only the case is quiet', () => {
    const notice = airPodsNoticeOf(
      { status: 'seen', battery: NO_LEVELS, partsSeenMs: { ...SEEN_NOW, case: null }, lastSeenMs: NOW_MS },
      NOW_MS,
    )

    expect(notice).toEqual({ kind: 'none' })
  })

  it('should pass the missing keys state through', () => {
    expect(airPodsNoticeOf({ status: 'missing_keys' }, NOW_MS)).toEqual({ kind: 'missing_keys' })
  })

  it('should pass the invalid config state through', () => {
    expect(airPodsNoticeOf({ status: 'invalid_config' }, NOW_MS)).toEqual({ kind: 'invalid_config' })
  })
})
