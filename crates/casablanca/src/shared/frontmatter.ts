import { isMap, isScalar, parseDocument } from 'yaml'

export type FrontmatterScalar = string | number | boolean | null

export interface FrontmatterProperty {
  key: string
  value: FrontmatterScalar
}

export type FrontmatterProjection =
  | { kind: 'properties'; properties: FrontmatterProperty[] }
  | { kind: 'raw'; reason: string }

const OPEN = '---\n'
const CLOSE = '---\n'
const COMMENT_PREFIX = '# frontmatter.'

/** The YAML source between the two fences. */
export function frontmatterSource(block: string): string {
  if (!block.startsWith(OPEN) || !block.endsWith(CLOSE)) return block
  return block.slice(OPEN.length, -CLOSE.length)
}

/** Wrap editable YAML source in a byte-stable frontmatter fence. */
export function wrapFrontmatter(source: string): string {
  const yaml = source === '' || source.endsWith('\n') ? source : `${source}\n`
  return `${OPEN}${yaml}${CLOSE}`
}

/** Project simple top-level YAML into property rows; complex YAML stays raw. */
export function projectFrontmatter(block: string): FrontmatterProjection {
  const doc = parseDocument(frontmatterSource(block))
  if (doc.errors.length > 0) return { kind: 'raw', reason: doc.errors[0].message }
  if (doc.contents === null) return { kind: 'properties', properties: [] }
  if (!isMap(doc.contents)) return { kind: 'raw', reason: 'Frontmatter must be a key-value map.' }

  const properties: FrontmatterProperty[] = []
  for (const pair of doc.contents.items) {
    if (!isScalar(pair.key) || typeof pair.key.value !== 'string') {
      return { kind: 'raw', reason: 'Complex property keys are edited as raw YAML.' }
    }
    if (pair.value !== null && !isScalar(pair.value)) {
      return { kind: 'raw', reason: 'Nested properties are edited as raw YAML.' }
    }
    const value = pair.value?.value ?? null
    if (
      value !== null &&
      typeof value !== 'string' &&
      typeof value !== 'number' &&
      typeof value !== 'boolean'
    ) {
      return { kind: 'raw', reason: 'Complex property values are edited as raw YAML.' }
    }
    properties.push({ key: pair.key.value, value })
  }
  return { kind: 'properties', properties }
}

function editableDocument(block: string) {
  const doc = parseDocument(frontmatterSource(block))
  if (doc.errors.length > 0) throw new Error(doc.errors[0].message)
  if (doc.contents !== null && !isMap(doc.contents)) throw new Error('Frontmatter must be a key-value map.')
  return doc
}

export function setFrontmatterProperty(
  block: string,
  key: string,
  value: FrontmatterScalar
): string {
  const cleanKey = key.trim()
  if (cleanKey === '') throw new Error('Property name is required.')
  const doc = editableDocument(block)
  doc.set(cleanKey, value)
  return wrapFrontmatter(doc.toString())
}

export function removeFrontmatterProperty(block: string, key: string): string {
  const commentPrefix = `${COMMENT_PREFIX}${encodedKey(key)} `
  const source = frontmatterSource(block)
    .split('\n')
    .filter((line) => !line.startsWith(commentPrefix))
    .join('\n')
  const doc = editableDocument(wrapFrontmatter(source))
  doc.delete(key)
  return wrapFrontmatter(doc.toString())
}

export function normalizeCommandText(text: string): string {
  const command = text.trim().replace(/\s+/g, ' ')
  if (!/^::(?:[A-Za-z]+|\?)(?:\s|$)/.test(command)) {
    throw new Error('Start with an Indiana command, for example ::fix.')
  }
  if (command.slice(2).includes('::')) throw new Error('Use one Indiana command per comment.')
  if (command.includes('`')) throw new Error('Backticks would hide the command from Indiana.')
  return command
}

function encodedKey(key: string): string {
  return encodeURIComponent(key)
}

/** Insert an explicit Indiana-readable YAML comment directly after its property. */
export function addFrontmatterAnnotation(block: string, key: string, commandText: string): string {
  const source = frontmatterSource(block)
  const doc = editableDocument(block)
  if (!isMap(doc.contents)) throw new Error('Property not found.')
  const pair = doc.contents.items.find((item) => isScalar(item.key) && item.key.value === key)
  if (!pair) throw new Error(`Property not found: ${key}`)

  const range = pair.value?.range ?? pair.key?.range
  if (!range) throw new Error(`Cannot locate property: ${key}`)
  const lineEnd = source.indexOf('\n', range[1])
  const insertAt = lineEnd === -1 ? source.length : lineEnd + 1
  const before = source.slice(0, insertAt)
  const separator = before === '' || before.endsWith('\n') ? '' : '\n'
  const line = `${COMMENT_PREFIX}${encodedKey(key)} ${normalizeCommandText(commandText)}\n`
  return wrapFrontmatter(`${before}${separator}${line}${source.slice(insertAt)}`)
}

export function frontmatterAnnotations(block: string, key: string): string[] {
  const prefix = `${COMMENT_PREFIX}${encodedKey(key)} `
  return frontmatterSource(block)
    .split('\n')
    .filter((line) => line.startsWith(prefix))
    .map((line) => line.slice(prefix.length))
}
