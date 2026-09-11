const glyphs = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ#$%&*+-=<>?/\\|[]{}'
const basePalette = {
  background: '#050805',
  glow: '202, 255, 216',
  stream: '106, 248, 140',
  trail: '82, 205, 110',
}

export function createMatrixRainEngine({ getCanvas, getEnabled, getQuality }) {
  let animationFrameId = 0
  let mediaQuery = null
  let columns = []

  const state = {
    ctx: null,
    width: 0,
    height: 0,
    visible: true,
    reducedMotion: false,
    lastTick: 0,
    lastLogicTick: 0,
    targetFrameMs: 1000 / 34,
    dpr: 1,
    fontSize: 18,
    rowHeight: 22,
    cellWidth: 22,
    fadeAlpha: 0.24,
    overlayAlpha: 0.15,
    headGlowBlur: 10,
    speedMin: 100,
    speedMax: 175,
    trailMin: 10,
    trailMax: 18,
    logicTickMs: 84,
    hotCells: 2,
    headMutationMin: 48,
    headMutationMax: 96,
    nearMutationMin: 90,
    nearMutationMax: 168,
    tailMutationMin: 220,
    tailMutationMax: 420,
    pauseChance: 0.18,
    pauseDelayMin: 110,
    pauseDelayMax: 260,
    accentChance: 0.1,
  }

  function randomFromSeed(seed) {
    return Math.abs(Math.sin(seed * 9999.13) * 10000) % 1
  }

  function clamp(value, min, max) {
    return Math.max(min, Math.min(max, value))
  }

  function randomBetween(min, max) {
    return min + Math.random() * (max - min)
  }

  function randomGlyph() {
    return glyphs[Math.floor(Math.random() * glyphs.length)]
  }

  function resolvePreset(width, height) {
    const pixelDensity = window.devicePixelRatio || 1
    const pixelLoad = width * height * pixelDensity * pixelDensity
    const quality = getQuality()
    const normalizedQuality = quality === 'auto' ? 'normal' : quality
    const preset = {
      targetFps: 34,
      dprCap: 1.3,
      fontSize: 18,
      rowHeight: 22,
      cellWidth: 22,
      fadeAlpha: 0.24,
      overlayAlpha: 0.15,
      headGlowBlur: 10,
      speedMin: 100,
      speedMax: 175,
      trailMin: 10,
      trailMax: 18,
      logicTickMs: 84,
      hotCells: 2,
      headMutationMin: 48,
      headMutationMax: 96,
      nearMutationMin: 90,
      nearMutationMax: 168,
      tailMutationMin: 220,
      tailMutationMax: 420,
      pauseChance: 0.18,
      pauseDelayMin: 110,
      pauseDelayMax: 260,
      accentChance: 0.1,
    }

    if (normalizedQuality === 'low') {
      preset.targetFps = 30
      preset.dprCap = 1.05
      preset.fontSize = 17
      preset.rowHeight = 20
      preset.cellWidth = 25
      preset.fadeAlpha = 0.3
      preset.overlayAlpha = 0.18
      preset.headGlowBlur = 6
      preset.speedMin = 90
      preset.speedMax = 145
      preset.trailMin = 8
      preset.trailMax = 13
      preset.logicTickMs = 120
      preset.hotCells = 1
      preset.headMutationMin = 80
      preset.headMutationMax = 140
      preset.nearMutationMin = 140
      preset.nearMutationMax = 220
      preset.tailMutationMin = 340
      preset.tailMutationMax = 560
      preset.pauseChance = 0.26
      preset.pauseDelayMin = 140
      preset.pauseDelayMax = 320
      preset.accentChance = 0.04
    }

    if (quality === 'auto') {
      if (width >= 1600 || pixelDensity >= 1.75 || pixelLoad >= 4_200_000) {
        preset.targetFps = Math.min(preset.targetFps, 32)
        preset.dprCap = Math.min(preset.dprCap, 1.16)
        preset.cellWidth += 2
        preset.fadeAlpha += 0.05
        preset.overlayAlpha += 0.04
        preset.headGlowBlur = Math.max(5, preset.headGlowBlur - 2)
        preset.speedMax = Math.max(preset.speedMin + 16, preset.speedMax - 14)
        preset.trailMin = Math.max(7, preset.trailMin - 2)
        preset.trailMax = Math.max(preset.trailMin + 2, preset.trailMax - 4)
        preset.logicTickMs += 24
        preset.headMutationMin += 12
        preset.headMutationMax += 18
        preset.nearMutationMin += 20
        preset.nearMutationMax += 36
        preset.tailMutationMin += 50
        preset.tailMutationMax += 70
        preset.accentChance = Math.min(preset.accentChance, 0.06)
      }

      if (width <= 960) {
        preset.cellWidth += 1
        preset.fontSize = Math.max(16, preset.fontSize - 1)
        preset.rowHeight = Math.max(19, preset.rowHeight - 1)
        preset.trailMax = Math.max(preset.trailMin + 2, preset.trailMax - 1)
      }
    }

    if (state.reducedMotion) {
      preset.targetFps = Math.min(preset.targetFps, 24)
      preset.dprCap = Math.min(preset.dprCap, 1)
      preset.fadeAlpha = Math.max(preset.fadeAlpha, 0.34)
      preset.overlayAlpha = Math.max(preset.overlayAlpha, 0.22)
      preset.headGlowBlur = Math.max(3, preset.headGlowBlur - 4)
      preset.speedMin *= 0.68
      preset.speedMax *= 0.74
      preset.trailMin = Math.max(6, preset.trailMin - 2)
      preset.trailMax = Math.max(preset.trailMin + 2, preset.trailMax - 5)
      preset.logicTickMs += 40
      preset.hotCells = 1
      preset.headMutationMin += 40
      preset.headMutationMax += 55
      preset.nearMutationMin += 60
      preset.nearMutationMax += 80
      preset.tailMutationMin += 100
      preset.tailMutationMax += 140
      preset.accentChance = Math.min(preset.accentChance, 0.02)
    }

    return preset
  }

  function resolveMutationDelay(trailIndex, isAccent = false) {
    let minDelay = state.tailMutationMin
    let maxDelay = state.tailMutationMax

    if (trailIndex === 0) {
      minDelay = state.headMutationMin
      maxDelay = state.headMutationMax
    } else if (trailIndex <= state.hotCells) {
      minDelay = state.nearMutationMin
      maxDelay = state.nearMutationMax
    }

    let delay = randomBetween(minDelay, maxDelay)

    if (!isAccent && trailIndex > state.hotCells && Math.random() < state.pauseChance) {
      delay += randomBetween(state.pauseDelayMin, state.pauseDelayMax)
    }

    return delay
  }

  function resolveCellBrightness(trailIndex, accentBoost = 0) {
    let baseBrightness = 0.82

    if (trailIndex === 0) {
      baseBrightness = 1.08
    } else if (trailIndex <= state.hotCells) {
      baseBrightness = 0.96
    }

    return clamp(baseBrightness + (Math.random() - 0.5) * 0.18 + accentBoost, 0.56, 1.35)
  }

  function createTrailCell(trailIndex, timestamp, accentBoost = 0) {
    const isAccent = accentBoost > 0

    return {
      glyph: randomGlyph(),
      nextChangeAt: timestamp + resolveMutationDelay(trailIndex, isAccent),
      brightness: resolveCellBrightness(trailIndex, accentBoost),
    }
  }

  function createTrailCells(trailLength, timestamp) {
    return Array.from({ length: trailLength }, (_, trailIndex) =>
      createTrailCell(trailIndex, timestamp, trailIndex === 0 ? 0.16 : 0),
    )
  }

  function createColumns(timestamp) {
    const columnCount = Math.ceil(state.width / state.cellWidth)

    return Array.from({ length: columnCount }, (_, index) => {
      const seed = index + 11
      const opacity = 0.52 + randomFromSeed(seed + 1) * 0.36
      const speed = state.speedMin + randomFromSeed(seed + 2) * (state.speedMax - state.speedMin)
      const trailLength =
        state.trailMin + Math.floor(randomFromSeed(seed + 3) * (state.trailMax - state.trailMin + 1))
      const x = index * state.cellWidth + randomFromSeed(seed + 4) * Math.max(2, state.cellWidth * 0.2)

      return createColumnState(seed, x, opacity, speed, trailLength, timestamp)
    })
  }

  function createColumnState(seed, x, opacity, speed, trailLength, timestamp) {
    const resetOffset = randomFromSeed(seed + 5) * state.height
    const headY = -(trailLength * state.rowHeight) - resetOffset

    return {
      x,
      speed,
      opacity,
      trailLength,
      headY,
      lastHeadStep: Math.floor(headY / state.rowHeight),
      cells: createTrailCells(trailLength, timestamp + randomFromSeed(seed + 6) * state.logicTickMs),
    }
  }

  function recycleColumn(column, index, timestamp) {
    const seed = index * 29 + Math.floor(column.headY) + 17
    const opacity = 0.52 + randomFromSeed(seed + 1) * 0.36
    const speed = state.speedMin + randomFromSeed(seed + 2) * (state.speedMax - state.speedMin)
    const trailLength =
      state.trailMin + Math.floor(randomFromSeed(seed + 3) * (state.trailMax - state.trailMin + 1))

    column.opacity = opacity
    column.speed = speed
    column.trailLength = trailLength
    column.headY = -(trailLength * state.rowHeight) - randomFromSeed(seed + 5) * state.height * 0.9
    column.lastHeadStep = Math.floor(column.headY / state.rowHeight)
    column.cells = createTrailCells(trailLength, timestamp + randomFromSeed(seed + 6) * state.logicTickMs)
  }

  function shiftTrailCells(column, steps, timestamp) {
    if (steps >= column.trailLength) {
      column.cells = createTrailCells(column.trailLength, timestamp)
      return
    }

    for (let step = 0; step < steps; step += 1) {
      column.cells.pop()
      column.cells.unshift(createTrailCell(0, timestamp, Math.random() < state.accentChance ? 0.18 : 0))
    }

    if (column.cells[1] && Math.random() < state.accentChance * 0.8) {
      column.cells[1].brightness = resolveCellBrightness(1, 0.12)
      column.cells[1].nextChangeAt = timestamp + randomBetween(state.headMutationMin, state.nearMutationMax)
    }
  }

  function mutateTrailCells(column, timestamp) {
    for (let trailIndex = 0; trailIndex < column.cells.length; trailIndex += 1) {
      const cell = column.cells[trailIndex]
      if (timestamp < cell.nextChangeAt) {
        continue
      }

      const accentBoost =
        trailIndex === 0
          ? (Math.random() < 0.32 ? 0.16 : 0)
          : trailIndex <= state.hotCells && Math.random() < state.accentChance
            ? 0.1
            : 0

      cell.glyph = randomGlyph()
      cell.brightness = resolveCellBrightness(trailIndex, accentBoost)
      cell.nextChangeAt = timestamp + resolveMutationDelay(trailIndex, accentBoost > 0)
    }
  }

  function advanceColumnState(column, index, timestamp) {
    if (column.headY - column.trailLength * state.rowHeight > state.height + state.rowHeight * 2) {
      recycleColumn(column, index, timestamp)
      return
    }

    const headStep = Math.floor(column.headY / state.rowHeight)
    const stepsAdvanced = headStep - column.lastHeadStep

    if (stepsAdvanced > 0) {
      shiftTrailCells(column, Math.min(stepsAdvanced, column.trailLength), timestamp)
      column.lastHeadStep = headStep
    } else if (headStep !== column.lastHeadStep) {
      column.lastHeadStep = headStep
    }

    mutateTrailCells(column, timestamp)
  }

  function drawColumn(ctx, column) {
    for (let trailIndex = column.cells.length - 1; trailIndex >= 0; trailIndex -= 1) {
      const cell = column.cells[trailIndex]
      const y = column.headY - trailIndex * state.rowHeight

      if (y < -state.rowHeight || y > state.height + state.rowHeight) {
        continue
      }

      const fade = 1 - trailIndex / column.trailLength

      if (trailIndex === 0) {
        const alpha = clamp(0.78 * column.opacity * cell.brightness, 0.18, 1)
        ctx.shadowColor = `rgba(${basePalette.glow}, ${0.46 * column.opacity})`
        ctx.shadowBlur = state.headGlowBlur
        ctx.fillStyle = `rgba(243, 255, 246, ${alpha})`
      } else {
        const isHotCell = trailIndex <= state.hotCells
        const alpha = clamp(
          fade * fade * column.opacity * cell.brightness * (isHotCell ? 0.92 : 0.82),
          0.04,
          0.82,
        )

        ctx.shadowColor = isHotCell ? `rgba(${basePalette.glow}, ${0.12 * column.opacity})` : 'transparent'
        ctx.shadowBlur = isHotCell ? Math.max(0, state.headGlowBlur - trailIndex * 3.5) : 0
        ctx.fillStyle = `rgba(${isHotCell ? basePalette.stream : basePalette.trail}, ${alpha})`
      }

      ctx.fillText(cell.glyph, column.x, y)
    }
  }

  function drawFrame(deltaMs, timestamp) {
    if (!state.ctx) {
      return
    }

    const deltaSeconds = deltaMs / 1000
    const ctx = state.ctx
    const shouldRunLogic = state.lastLogicTick === 0 || timestamp - state.lastLogicTick >= state.logicTickMs

    ctx.shadowBlur = 0
    ctx.fillStyle = `rgba(5, 8, 5, ${state.fadeAlpha})`
    ctx.fillRect(0, 0, state.width, state.height)

    for (let index = 0; index < columns.length; index += 1) {
      const column = columns[index]

      if (deltaSeconds > 0) {
        column.headY += column.speed * deltaSeconds
      }

      if (shouldRunLogic) {
        advanceColumnState(column, index, timestamp)
      }

      drawColumn(ctx, column)
    }

    if (shouldRunLogic) {
      state.lastLogicTick = timestamp
    }

    ctx.shadowBlur = 0
    ctx.fillStyle = `rgba(5, 8, 5, ${state.overlayAlpha})`
    ctx.fillRect(0, 0, state.width, state.height)
  }

  function tick(timestamp) {
    if (!getEnabled() || !state.visible || !state.ctx) {
      animationFrameId = 0
      return
    }

    if (state.lastTick === 0) {
      state.lastTick = timestamp
    }

    const elapsed = timestamp - state.lastTick
    if (elapsed >= state.targetFrameMs) {
      drawFrame(elapsed, timestamp)
      state.lastTick = timestamp - (elapsed % state.targetFrameMs)
    }

    animationFrameId = window.requestAnimationFrame(tick)
  }

  function startLoop() {
    if (animationFrameId || !getEnabled() || !state.visible || !state.ctx) {
      return
    }

    state.lastTick = 0
    state.lastLogicTick = 0
    animationFrameId = window.requestAnimationFrame(tick)
  }

  function stopLoop() {
    if (!animationFrameId) {
      return
    }

    window.cancelAnimationFrame(animationFrameId)
    animationFrameId = 0
  }

  function syncReducedMotion() {
    state.reducedMotion = Boolean(mediaQuery?.matches)
  }

  function applyCanvasMetrics() {
    const canvas = getCanvas()
    if (!canvas) {
      return
    }

    const width = Math.max(window.innerWidth, 1)
    const height = Math.max(window.innerHeight, 1)
    const preset = resolvePreset(width, height)
    const dpr = clamp(window.devicePixelRatio || 1, 1, preset.dprCap)
    const context = canvas.getContext('2d', { alpha: true, desynchronized: true })
    const now = performance.now()
    if (!context) {
      return
    }

    state.ctx = context
    state.width = width
    state.height = height
    state.dpr = dpr
    state.fontSize = preset.fontSize
    state.rowHeight = preset.rowHeight
    state.cellWidth = preset.cellWidth
    state.fadeAlpha = preset.fadeAlpha
    state.overlayAlpha = preset.overlayAlpha
    state.headGlowBlur = preset.headGlowBlur
    state.speedMin = preset.speedMin
    state.speedMax = preset.speedMax
    state.trailMin = preset.trailMin
    state.trailMax = preset.trailMax
    state.logicTickMs = preset.logicTickMs
    state.hotCells = preset.hotCells
    state.headMutationMin = preset.headMutationMin
    state.headMutationMax = preset.headMutationMax
    state.nearMutationMin = preset.nearMutationMin
    state.nearMutationMax = preset.nearMutationMax
    state.tailMutationMin = preset.tailMutationMin
    state.tailMutationMax = preset.tailMutationMax
    state.pauseChance = preset.pauseChance
    state.pauseDelayMin = preset.pauseDelayMin
    state.pauseDelayMax = preset.pauseDelayMax
    state.accentChance = preset.accentChance
    state.targetFrameMs = 1000 / preset.targetFps
    state.lastLogicTick = 0

    canvas.width = Math.round(width * dpr)
    canvas.height = Math.round(height * dpr)
    canvas.style.width = `${width}px`
    canvas.style.height = `${height}px`

    context.setTransform(dpr, 0, 0, dpr, 0, 0)
    context.textBaseline = 'top'
    context.textAlign = 'left'
    context.font = `${state.fontSize}px var(--terminal-font-stack, monospace)`
    context.fillStyle = basePalette.background
    context.fillRect(0, 0, width, height)

    columns = createColumns(now)
    drawFrame(0, now)
    state.lastTick = 0
  }

  function handleResize() {
    applyCanvasMetrics()
    if (getEnabled() && state.visible) {
      startLoop()
    }
  }

  function handleVisibilityChange() {
    state.visible = document.visibilityState !== 'hidden'
    if (state.visible) {
      startLoop()
      return
    }
    stopLoop()
  }

  function handleReducedMotionChange() {
    syncReducedMotion()
    applyCanvasMetrics()
    if (getEnabled() && state.visible) {
      startLoop()
    }
  }

  function mount() {
    mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
    syncReducedMotion()
    state.visible = document.visibilityState !== 'hidden'
    applyCanvasMetrics()

    document.addEventListener('visibilitychange', handleVisibilityChange)
    window.addEventListener('resize', handleResize, { passive: true })

    if (typeof mediaQuery.addEventListener === 'function') {
      mediaQuery.addEventListener('change', handleReducedMotionChange)
    } else {
      mediaQuery.addListener(handleReducedMotionChange)
    }

    if (getEnabled() && state.visible) {
      startLoop()
    }
  }

  function unmount() {
    stopLoop()
    document.removeEventListener('visibilitychange', handleVisibilityChange)
    window.removeEventListener('resize', handleResize)

    if (!mediaQuery) {
      return
    }

    if (typeof mediaQuery.removeEventListener === 'function') {
      mediaQuery.removeEventListener('change', handleReducedMotionChange)
    } else {
      mediaQuery.removeListener(handleReducedMotionChange)
    }
  }

  function syncOptions() {
    if (!getCanvas()) {
      return
    }

    applyCanvasMetrics()
    if (getEnabled() && state.visible) {
      startLoop()
      return
    }

    stopLoop()
  }

  return {
    mount,
    unmount,
    syncOptions,
  }
}
