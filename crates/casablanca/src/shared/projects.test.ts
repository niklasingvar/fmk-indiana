import { describe, it, expect } from 'vitest'
import {
  PROJECT_PALETTE,
  pickColor,
  projectName,
  normalizeConfig,
  addProjectToConfig,
  switchActive,
  setColor,
  removeProject,
  activeRecord,
  toProjectList
} from './projects'

describe('pickColor', () => {
  it('returns the first unused palette entry', () => {
    expect(pickColor([])).toBe(PROJECT_PALETTE[0])
    expect(pickColor([PROJECT_PALETTE[0]])).toBe(PROJECT_PALETTE[1])
  })

  it('wraps once every color is taken', () => {
    const all = [...PROJECT_PALETTE]
    expect(pickColor(all)).toBe(PROJECT_PALETTE[all.length % PROJECT_PALETTE.length])
  })
})

describe('projectName', () => {
  it('takes the last path segment', () => {
    expect(projectName('/Users/x/workspace/indiana')).toBe('indiana')
  })
  it('tolerates trailing slashes and backslashes', () => {
    expect(projectName('/Users/x/indiana/')).toBe('indiana')
    expect(projectName('C:\\repos\\site\\')).toBe('site')
  })
})

describe('normalizeConfig', () => {
  it('migrates a legacy single vault into a one-entry project list', () => {
    const cfg = normalizeConfig({ vaultRootPath: '/repo/a' })
    expect(cfg.projects).toEqual([{ rootPath: '/repo/a', color: PROJECT_PALETTE[0] }])
    expect(cfg.activePath).toBe('/repo/a')
  })

  it('returns an empty registry for null/empty input', () => {
    expect(normalizeConfig(null)).toEqual({ projects: [], activePath: null })
    expect(normalizeConfig({})).toEqual({ projects: [], activePath: null })
  })

  it('dedupes by path and assigns colors to entries missing one', () => {
    const cfg = normalizeConfig({
      projects: [
        { rootPath: '/repo/a', color: '1 2 3' },
        { rootPath: '/repo/a', color: '9 9 9' },
        { rootPath: '/repo/b' } as never
      ],
      activePath: '/repo/b'
    })
    expect(cfg.projects.map((p) => p.rootPath)).toEqual(['/repo/a', '/repo/b'])
    expect(cfg.projects[0].color).toBe('1 2 3')
    expect(cfg.projects[1].color).toBeTruthy()
    expect(cfg.activePath).toBe('/repo/b')
  })

  it('falls back to the first project when activePath is stale', () => {
    const cfg = normalizeConfig({
      projects: [{ rootPath: '/repo/a', color: '1 2 3' }],
      activePath: '/repo/gone'
    })
    expect(cfg.activePath).toBe('/repo/a')
  })
})

describe('addProjectToConfig', () => {
  const base = normalizeConfig({ projects: [{ rootPath: '/repo/a', color: PROJECT_PALETTE[0] }], activePath: '/repo/a' })

  it('appends a new project with a distinct color and activates it', () => {
    const next = addProjectToConfig(base, '/repo/b')
    expect(next.projects).toHaveLength(2)
    expect(next.projects[1].color).not.toBe(next.projects[0].color)
    expect(next.activePath).toBe('/repo/b')
  })

  it('re-adding an existing path only re-activates it (no duplicate)', () => {
    const next = addProjectToConfig(base, '/repo/a')
    expect(next.projects).toHaveLength(1)
    expect(next.activePath).toBe('/repo/a')
  })
})

describe('switchActive / setColor / removeProject', () => {
  const base = normalizeConfig({
    projects: [
      { rootPath: '/repo/a', color: PROJECT_PALETTE[0] },
      { rootPath: '/repo/b', color: PROJECT_PALETTE[1] }
    ],
    activePath: '/repo/a'
  })

  it('switches to a known project and ignores unknown paths', () => {
    expect(switchActive(base, '/repo/b').activePath).toBe('/repo/b')
    expect(switchActive(base, '/repo/gone').activePath).toBe('/repo/a')
  })

  it('recolors one project only', () => {
    const next = setColor(base, '/repo/b', '5 5 5')
    expect(activeRecord(next)?.color).toBe(PROJECT_PALETTE[0])
    expect(next.projects.find((p) => p.rootPath === '/repo/b')?.color).toBe('5 5 5')
  })

  it('removing the active project falls back to the first remaining', () => {
    const next = removeProject(base, '/repo/a')
    expect(next.projects.map((p) => p.rootPath)).toEqual(['/repo/b'])
    expect(next.activePath).toBe('/repo/b')
  })
})

describe('toProjectList', () => {
  it('projects name + active flag for the renderer', () => {
    const cfg = normalizeConfig({
      projects: [
        { rootPath: '/repo/a', color: PROJECT_PALETTE[0] },
        { rootPath: '/repo/b', color: PROJECT_PALETTE[1] }
      ],
      activePath: '/repo/b'
    })
    expect(toProjectList(cfg)).toEqual([
      { rootPath: '/repo/a', name: 'a', color: PROJECT_PALETTE[0], active: false },
      { rootPath: '/repo/b', name: 'b', color: PROJECT_PALETTE[1], active: true }
    ])
  })
})
