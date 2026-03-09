"use strict";

const COLLAPSED_CLASS = "bot-collapser--collapsed";
const EXPANDED_CLASS = "bot-collapser--expanded";
const PROCESSED_ATTR = "data-bot-collapser-processed";
const GROUP_ATTR = "data-bot-collapser-group";
const GROUP_HIDDEN_CLASS = "bot-collapser--group-hidden";
const GROUP_MEMBER_CLASS = "bot-collapser--group-member";
const GROUP_HEAD_CLASS = "bot-collapser--group-head";
const GROUP_ACTIVE_CLASS = "bot-collapser--group-active";

const CHEVRON_SVG = `<svg class="bot-collapser--chevron" viewBox="0 0 16 16" fill="currentColor">
  <path d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z"/>
</svg>`;

const DURATION_EXPAND = 200;
const DURATION_COLLAPSE = 150;

let groupCounter = 0;

// ── Utility ──────────────────────────────────────

function isBotComment(el) {
  const label = el.querySelector(".timeline-comment-header .Label");
  return label !== null && label.textContent.trim().toLowerCase() === "bot";
}

function getBotAuthorName(el) {
  return el.querySelector(".timeline-comment-header .author")?.textContent.trim() ?? "bot";
}

function getRelativeTime(el) {
  return el.querySelector(".timeline-comment-header relative-time")?.textContent.trim() ?? "";
}

function getCommentPreview(el, maxLength = 100) {
  const body = el.querySelector(".comment-body");
  if (!body) return "";
  const heading = body.querySelector("h1, h2, h3");
  const headingText = heading?.textContent.trim() ?? "";
  const fullText = body.textContent.trim().replace(/\s+/g, " ");
  let preview = headingText
    ? headingText + " — " + fullText.replace(headingText, "").trim()
    : fullText;
  return preview.length > maxLength ? preview.substring(0, maxLength) + "…" : preview;
}

// ── Slide Animation ──────────────────────────────

/**
 * .timeline-comment の max-height をアニメーションしてスライド開閉する。
 * collapsed 時はヘッダーの高さに制限 + overflow:hidden。
 */
function collapseComment(comment) {
  const header = comment.querySelector(".timeline-comment-header");
  const headerHeight = header?.offsetHeight ?? 40;
  comment.style.overflow = "hidden";
  comment.style.maxHeight = headerHeight + "px";
}

function slideOpen(comment, onDone) {
  // 現在の高さ（collapsed = ヘッダーのみ）
  const startHeight = comment.offsetHeight;
  // フルの高さを計測するため一時的に制限解除
  comment.style.maxHeight = "none";
  const endHeight = comment.scrollHeight;

  // 高さが変わらない場合は transitionend が発火しないので即完了
  if (startHeight === endHeight) {
    comment.style.maxHeight = "";
    comment.style.overflow = "";
    onDone?.();
    return;
  }

  // アニメーション開始位置に戻す
  comment.style.maxHeight = startHeight + "px";
  comment.offsetHeight; // reflow
  comment.style.transition = `max-height ${DURATION_EXPAND}ms ease-out`;
  comment.style.maxHeight = endHeight + "px";

  comment.addEventListener("transitionend", function handler(e) {
    if (e.target !== comment || e.propertyName !== "max-height") return;
    comment.removeEventListener("transitionend", handler);
    comment.style.maxHeight = "";
    comment.style.overflow = "";
    comment.style.transition = "";
    onDone?.();
  });
}

function slideClose(comment, onDone) {
  const startHeight = comment.scrollHeight;
  const header = comment.querySelector(".timeline-comment-header");
  const endHeight = header?.offsetHeight ?? 40;

  // 高さが変わらない場合は即完了
  if (startHeight <= endHeight) {
    comment.style.overflow = "hidden";
    comment.style.maxHeight = endHeight + "px";
    onDone?.();
    return;
  }

  comment.style.overflow = "hidden";
  comment.style.maxHeight = startHeight + "px";
  comment.offsetHeight; // reflow
  comment.style.transition = `max-height ${DURATION_COLLAPSE}ms ease-in`;
  comment.style.maxHeight = endHeight + "px";

  comment.addEventListener("transitionend", function handler(e) {
    if (e.target !== comment || e.propertyName !== "max-height") return;
    comment.removeEventListener("transitionend", handler);
    comment.style.transition = "";
    // max-height と overflow は閉じた状態のまま維持
    onDone?.();
  });
}

// ── Element Creation ─────────────────────────────

function createToggleSpan() {
  const span = document.createElement("span");
  span.className = "bot-collapser--toggle";
  span.setAttribute("role", "button");
  span.setAttribute("tabindex", "0");
  span.insertAdjacentHTML("beforeend", CHEVRON_SVG);
  return span;
}

function createSummary(authorName, preview) {
  const el = document.createElement("div");
  el.className = "bot-collapser--summary";

  el.appendChild(createToggleSpan());

  const authorEl = document.createElement("span");
  authorEl.className = "bot-collapser--author";
  authorEl.textContent = authorName;
  el.appendChild(authorEl);

  const badge = document.createElement("span");
  badge.className = "bot-collapser--badge";
  badge.textContent = "bot";
  el.appendChild(badge);

  if (preview) {
    const sep = document.createElement("span");
    sep.className = "bot-collapser--sep";
    sep.textContent = "—";
    el.appendChild(sep);

    const previewEl = document.createElement("span");
    previewEl.className = "bot-collapser--preview-text";
    previewEl.textContent = preview;
    el.appendChild(previewEl);
  }

  return el;
}

function createGroupSummary(authorName, count, timeRange) {
  const el = document.createElement("div");
  el.className = "bot-collapser--summary";

  el.appendChild(createToggleSpan());

  const authorEl = document.createElement("span");
  authorEl.className = "bot-collapser--author";
  authorEl.textContent = authorName;
  el.appendChild(authorEl);

  const badge = document.createElement("span");
  badge.className = "bot-collapser--badge";
  badge.textContent = "bot";
  el.appendChild(badge);

  const countEl = document.createElement("span");
  countEl.className = "bot-collapser--group-count";
  countEl.textContent = `${count} comments`;
  el.appendChild(countEl);

  if (timeRange) {
    const timeEl = document.createElement("span");
    timeEl.className = "bot-collapser--time";
    timeEl.textContent = timeRange;
    el.appendChild(timeEl);
  }

  return el;
}

// ── Toggle UI（共通 DOM 操作）────────────────────

/**
 * ヘッダーにトグルボタンと summary を挿入し、クリックで handler を呼ぶ。
 * 初期状態を collapsed にセットする。
 */
function installToggleUI(item, summary, handler) {
  const header = item.querySelector(".timeline-comment-header");
  const comment = item.querySelector(".timeline-comment");

  const h3InnerDiv = header?.querySelector("h3 > .d-flex");
  if (h3InnerDiv) {
    const h3Toggle = createToggleSpan();
    h3InnerDiv.insertBefore(h3Toggle, h3InnerDiv.firstChild);
    h3Toggle.addEventListener("click", (e) => { e.stopPropagation(); handler(); });
  }

  if (header) header.appendChild(summary);

  summary.addEventListener("click", (e) => { e.stopPropagation(); handler(); });

  // 初期状態: 折り畳み
  item.classList.add(COLLAPSED_CLASS);
  if (comment) collapseComment(comment);
}

/**
 * アニメーション付きで単一アイテムの開閉を切り替える。
 * animating ガードを内包し、重複呼び出しを防ぐ。
 */
function createSingleToggle(item, comment) {
  let animating = false;

  return function switchState() {
    if (animating) return;
    const expanding = item.classList.contains(COLLAPSED_CLASS);

    if (expanding) {
      item.classList.remove(COLLAPSED_CLASS);
      item.classList.add(EXPANDED_CLASS);
      if (comment) {
        animating = true;
        slideOpen(comment, () => { animating = false; });
      }
    } else if (comment) {
      animating = true;
      slideClose(comment, () => {
        item.classList.remove(EXPANDED_CLASS);
        item.classList.add(COLLAPSED_CLASS);
        animating = false;
      });
    } else {
      item.classList.remove(EXPANDED_CLASS);
      item.classList.add(COLLAPSED_CLASS);
    }
  };
}

// ── Single Item Processing ───────────────────────

function processItem(item) {
  if (item.hasAttribute(PROCESSED_ATTR)) return;
  item.setAttribute(PROCESSED_ATTR, "true");

  const summary = createSummary(getBotAuthorName(item), getCommentPreview(item));
  const comment = item.querySelector(".timeline-comment");
  const switchState = createSingleToggle(item, comment);

  installToggleUI(item, summary, switchState);
}

// ── Group Processing ─────────────────────────────

function detectBotRuns(allItems) {
  const runs = [];
  let currentRun = null;

  for (const item of allItems) {
    if (item.hasAttribute(GROUP_ATTR)) continue;

    if (isBotComment(item)) {
      const author = getBotAuthorName(item);

      if (currentRun && currentRun.author === author) {
        currentRun.items.push(item);
      } else {
        if (currentRun) runs.push(currentRun);
        currentRun = { author, items: [item] };
      }
    } else {
      if (currentRun) {
        runs.push(currentRun);
        currentRun = null;
      }
    }
  }
  if (currentRun) runs.push(currentRun);
  return runs;
}

/**
 * グループの tail アイテムの表示/非表示と GROUP_ACTIVE_CLASS を一括切り替え。
 */
function setGroupTailVisibility(items, tailItems, visible) {
  for (const item of tailItems) {
    item.classList.toggle(GROUP_HIDDEN_CLASS, !visible);
  }
  for (const item of items) {
    item.classList.toggle(GROUP_ACTIVE_CLASS, visible);
  }
}

/**
 * グループ用トグルを生成。ヘッドアイテムの開閉 + tail の表示切り替え。
 */
function createGroupToggle(headItem, headComment, items, tailItems) {
  let isExpanded = false;
  let animating = false;

  return function toggleGroup() {
    if (animating) return;
    isExpanded = !isExpanded;

    if (isExpanded) {
      headItem.classList.remove(COLLAPSED_CLASS);
      headItem.classList.add(EXPANDED_CLASS);
      setGroupTailVisibility(items, tailItems, true);
      if (headComment) {
        animating = true;
        slideOpen(headComment, () => { animating = false; });
      }
    } else {
      animating = true;
      const onClosed = () => {
        headItem.classList.remove(EXPANDED_CLASS);
        headItem.classList.add(COLLAPSED_CLASS);
        setGroupTailVisibility(items, tailItems, false);
        animating = false;
      };
      if (headComment) {
        slideClose(headComment, onClosed);
      } else {
        onClosed();
      }
    }
  };
}

function wrapGroup(run) {
  const { author, items } = run;
  const groupId = `group-${groupCounter++}`;
  const headItem = items[0];
  const tailItems = items.slice(1);

  // 時間範囲の算出
  const firstTime = getRelativeTime(items[0]);
  const lastTime = getRelativeTime(items[items.length - 1]);
  const timeRange = firstTime && lastTime && firstTime !== lastTime
    ? `${lastTime} – ${firstTime}`
    : firstTime;

  // グループ属性の設定
  headItem.setAttribute(GROUP_ATTR, groupId);
  headItem.setAttribute(PROCESSED_ATTR, "true");
  headItem.classList.add(GROUP_HEAD_CLASS);
  for (const item of tailItems) item.setAttribute(GROUP_ATTR, groupId);
  for (const item of items) item.classList.add(GROUP_MEMBER_CLASS);

  // UI の構築とトグル接続
  const headComment = headItem.querySelector(".timeline-comment");
  const groupSummary = createGroupSummary(author, items.length, timeRange);
  const toggleGroup = createGroupToggle(headItem, headComment, items, tailItems);

  installToggleUI(headItem, groupSummary, toggleGroup);

  // tail の初期非表示（installToggleUI で head は collapsed 済み）
  for (const item of tailItems) item.classList.add(GROUP_HIDDEN_CLASS);
}

// ── Main ─────────────────────────────────────────

function processAll() {
  const allItems = Array.from(document.querySelectorAll(
    ".js-timeline-item:not([" + GROUP_ATTR + "])"
  ));
  if (allItems.length === 0) return;

  const runs = detectBotRuns(allItems);

  for (const run of runs) {
    if (run.items.some((i) => i.hasAttribute(PROCESSED_ATTR))) continue;

    if (run.items.length >= 2) {
      wrapGroup(run);
    } else {
      processItem(run.items[0]);
    }
  }
}

processAll();

const observer = new MutationObserver((mutations) => {
  let shouldProcess = false;
  for (const mutation of mutations) {
    for (const node of mutation.addedNodes) {
      if (node.nodeType === Node.ELEMENT_NODE) {
        if (
          node.matches?.(".js-timeline-item") ||
          node.querySelector?.(".js-timeline-item")
        ) {
          shouldProcess = true;
          break;
        }
      }
    }
    if (shouldProcess) break;
  }
  if (shouldProcess) processAll();
});

observer.observe(document.body, { childList: true, subtree: true });
