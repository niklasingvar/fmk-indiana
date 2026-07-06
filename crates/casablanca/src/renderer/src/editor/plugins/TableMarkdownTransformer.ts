import {
  $convertFromMarkdownString,
  $convertToMarkdownString,
  type ElementTransformer,
  type Transformer
} from '@lexical/markdown'
import {
  $createTableCellNode,
  $createTableNode,
  $createTableRowNode,
  $isTableCellNode,
  $isTableNode,
  $isTableRowNode,
  TableCellHeaderStates,
  TableCellNode,
  TableNode,
  TableRowNode
} from '@lexical/table'
import { $isParagraphNode, $isTextNode, type LexicalNode } from 'lexical'

const TABLE_ROW_REG_EXP = /^(?:\|)(.+)(?:\|)\s?$/
const TABLE_ROW_DIVIDER_REG_EXP = /^(\| ?:?-*:? ?)+\|\s?$/

function getTableColumnsSize(table: TableNode): number {
  const row = table.getFirstChild()
  return $isTableRowNode(row) ? row.getChildrenSize() : 0
}

/** Port of Lexical playground TABLE transformer (v0.22). */
export function createTableTransformer(getTransformers: () => Transformer[]): ElementTransformer {
  const $createTableCell = (textContent: string): TableCellNode => {
    textContent = textContent.replace(/\\n/g, '\n')
    const cell = $createTableCellNode(TableCellHeaderStates.NO_STATUS)
    $convertFromMarkdownString(textContent, getTransformers(), cell)
    return cell
  }

  const mapToTableCells = (line: string): Array<TableCellNode> | null => {
    const match = line.match(TABLE_ROW_REG_EXP)
    if (!match?.[1]) return null
    return match[1].split('|').map((text) => $createTableCell(text.trim()))
  }

  return {
    dependencies: [TableNode, TableRowNode, TableCellNode],
    export: (node: LexicalNode) => {
      if (!$isTableNode(node)) return null

      const output: string[] = []
      for (const row of node.getChildren()) {
        if (!$isTableRowNode(row)) continue

        const rowOutput: string[] = []
        let isHeaderRow = false
        for (const cell of row.getChildren()) {
          if (!$isTableCellNode(cell)) continue
          rowOutput.push(
            $convertToMarkdownString(getTransformers(), cell)
              .replace(/\n/g, '\\n')
              .trim()
          )
          if (cell.__headerState === TableCellHeaderStates.ROW) {
            isHeaderRow = true
          }
        }

        output.push(`| ${rowOutput.join(' | ')} |`)
        if (isHeaderRow) {
          output.push(`| ${rowOutput.map(() => '---').join(' | ')} |`)
        }
      }

      return output.join('\n')
    },
    regExp: TABLE_ROW_REG_EXP,
    replace: (parentNode, _1, match) => {
      if (TABLE_ROW_DIVIDER_REG_EXP.test(match[0])) {
        const table = parentNode.getPreviousSibling()
        if (!table || !$isTableNode(table)) return

        const rows = table.getChildren()
        const lastRow = rows[rows.length - 1]
        if (!lastRow || !$isTableRowNode(lastRow)) return

        lastRow.getChildren().forEach((cell) => {
          if (!$isTableCellNode(cell)) return
          cell.setHeaderStyles(TableCellHeaderStates.ROW, TableCellHeaderStates.ROW)
        })

        parentNode.remove()
        return
      }

      const matchCells = mapToTableCells(match[0])
      if (matchCells == null) return

      const rows = [matchCells]
      let sibling = parentNode.getPreviousSibling()
      let maxCells = matchCells.length

      while (sibling) {
        if (!$isParagraphNode(sibling) || sibling.getChildrenSize() !== 1) break

        const firstChild = sibling.getFirstChild()
        if (!$isTextNode(firstChild)) break

        const cells = mapToTableCells(firstChild.getTextContent())
        if (cells == null) break

        maxCells = Math.max(maxCells, cells.length)
        rows.unshift(cells)
        const previousSibling = sibling.getPreviousSibling()
        sibling.remove()
        sibling = previousSibling
      }

      const table = $createTableNode()
      for (const cells of rows) {
        const tableRow = $createTableRowNode()
        table.append(tableRow)
        for (let i = 0; i < maxCells; i++) {
          tableRow.append(i < cells.length ? cells[i] : $createTableCell(''))
        }
      }

      const previousSibling = parentNode.getPreviousSibling()
      if ($isTableNode(previousSibling) && getTableColumnsSize(previousSibling) === maxCells) {
        previousSibling.append(...table.getChildren())
        parentNode.remove()
      } else {
        parentNode.replace(table)
      }

      table.selectEnd()
    },
    type: 'element'
  }
}
