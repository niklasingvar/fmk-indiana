/**
 * Injected into every HTML document served over vault:// — sensing only.
 * Hover outlines the element under the cursor; click posts the element's
 * selector + excerpt to the host, which renders the annotation bubble.
 * The iframe has no preload and no window.api; postMessage is the only
 * channel out.
 */
;(function () {
  'use strict'
  if (window.__casablancaAnnotator) return
  window.__casablancaAnnotator = true

  var mode = true
  var hovered = null
  var savedOutline = ''
  var savedOffset = ''

  function docRelPath() {
    var p = location.pathname.replace(/^\/+/, '')
    try {
      return decodeURIComponent(p)
    } catch (e) {
      return p
    }
  }

  function post(msg) {
    try {
      window.parent.postMessage(msg, '*')
    } catch (e) {
      /* host gone; nothing to do */
    }
  }

  function uniqueId(el) {
    return el.id && document.querySelectorAll('#' + CSS.escape(el.id)).length === 1
  }

  function cssPath(el) {
    if (uniqueId(el)) return '#' + el.id
    var parts = []
    var node = el
    while (node && node.nodeType === 1 && node !== document.documentElement) {
      if (uniqueId(node)) {
        parts.unshift('#' + node.id)
        break
      }
      var part = node.tagName.toLowerCase()
      var parent = node.parentElement
      if (parent) {
        var same = []
        for (var i = 0; i < parent.children.length; i++) {
          if (parent.children[i].tagName === node.tagName) same.push(parent.children[i])
        }
        if (same.length > 1) part += ':nth-of-type(' + (same.indexOf(node) + 1) + ')'
      }
      parts.unshift(part)
      var sel = parts.join(' > ')
      try {
        if (document.querySelectorAll(sel).length === 1) return sel
      } catch (e) {
        /* keep walking */
      }
      node = parent
    }
    return parts.join(' > ')
  }

  function excerptOf(el) {
    var text =
      el.innerText ||
      el.getAttribute('alt') ||
      el.getAttribute('aria-label') ||
      el.tagName.toLowerCase()
    return String(text).replace(/\s+/g, ' ').trim().slice(0, 120)
  }

  function restore() {
    if (!hovered) return
    hovered.style.outline = savedOutline
    hovered.style.outlineOffset = savedOffset
    hovered = null
  }

  function annotatable(t) {
    return t && t.nodeType === 1 && t !== document.documentElement && t !== document.body
  }

  document.addEventListener(
    'mouseover',
    function (e) {
      if (!mode || !annotatable(e.target)) return
      if (hovered && hovered !== e.target) restore()
      hovered = e.target
      savedOutline = hovered.style.outline
      savedOffset = hovered.style.outlineOffset
      hovered.style.outline = '2px solid rgb(96, 132, 250)'
      hovered.style.outlineOffset = '1px'
    },
    true
  )

  document.addEventListener(
    'mouseout',
    function (e) {
      if (e.target === hovered) restore()
    },
    true
  )

  document.addEventListener(
    'click',
    function (e) {
      if (!mode || !annotatable(e.target)) return
      e.preventDefault()
      e.stopPropagation()
      var r = e.target.getBoundingClientRect()
      post({
        type: 'casablanca:select',
        docRelPath: docRelPath(),
        selector: cssPath(e.target),
        excerpt: excerptOf(e.target),
        rect: { x: r.left, y: r.top, w: r.width, h: r.height }
      })
    },
    true
  )

  window.addEventListener(
    'scroll',
    function () {
      post({ type: 'casablanca:invalidate' })
    },
    true
  )
  window.addEventListener('resize', function () {
    post({ type: 'casablanca:invalidate' })
  })

  window.addEventListener('message', function (e) {
    var d = e.data
    if (d && d.type === 'casablanca:set-mode') {
      mode = !!d.on
      if (!mode) restore()
    }
  })
})()
