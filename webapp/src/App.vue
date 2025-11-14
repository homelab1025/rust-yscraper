<template>
  <div class="page">
    <header class="hero">
      <i class="pi pi-comments hero-icon" aria-hidden="true"></i>
      <h1>YScraper Dashboard</h1>
    </header>

    <main class="content">
      <section class="table-wrap">
        <h2>Not Filtered Comments</h2>
        <div v-if="loading" class="loading">Loading...</div>
        <div v-else ref="scroller" class="table-scroller">
          <table class="comments-table" aria-label="Comments table (use j/k to move, a to mark blue, d to mark red)">
            <thead>
            <tr>
              <th>Text</th>
              <th>User</th>
              <th>URL ID</th>
              <th>Date</th>
            </tr>
            </thead>
            <tbody>
            <tr
              v-for="(c, idx) in items"
              :key="c.id"
              :class="rowClass(c, idx)"
              :aria-selected="selectedIndex === idx"
              @click="selectRow(idx)"
              :ref="setRowRef(idx)"
            >
              <td class="text">{{ c.text }}</td>
              <td>{{ c.user }}</td>
              <td>{{ c.url_id }}</td>
              <td>{{ c.date }}</td>
            </tr>
            <tr v-if="items.length === 0">
              <td colspan="4" class="empty">No comments</td>
            </tr>
            </tbody>
          </table>
        </div>

        <div class="legend" aria-hidden="true">Keys: j = down, k = up, a = mark blue, d = mark red</div>

        <div class="pagination" role="navigation" aria-label="Pagination">
          <button :disabled="page===1" @click="prevPage">Prev</button>
          <button
            v-for="p in pagesToShow"
            :key="p"
            :class="['page-btn', {active: p===page}]"
            @click="goPage(p)"
          >{{ p }}</button>
          <button :disabled="page===totalPages || totalPages===0" @click="nextPage">Next</button>
          <span class="total">Total: {{ total }}</span>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup>
import {ref, computed, onMounted, onUnmounted, watch, nextTick} from 'vue';

const items = ref([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(10);
const loading = ref(false);

// selection and marks
const selectedIndex = ref(null);
const marks = ref({}); // { [id: number]: 'blue' | 'red' }

// scrolling support
const scroller = ref(null);
const rowRefs = ref([]); // HTMLElement[] parallel to items
function setRowRef(idx) {
  return (el) => {
    if (el) {
      rowRefs.value[idx] = el;
    }
  };
}

function ensureRowInView(idx, buffer = 2) {
  if (idx == null) return;
  const container = scroller.value;
  if (!container) return;
  const topIdx = Math.max(0, idx - buffer);
  const botIdx = Math.min(items.value.length - 1, idx + buffer);
  const topEl = rowRefs.value[topIdx];
  const botEl = rowRefs.value[botIdx] || rowRefs.value[idx];
  if (!topEl || !botEl) return;

  const desiredTop = topEl.offsetTop;
  const desiredBottom = botEl.offsetTop + botEl.offsetHeight;

  const viewTop = container.scrollTop;
  const viewBottom = viewTop + container.clientHeight;

  let newTop = null;
  if (desiredTop < viewTop) {
    newTop = desiredTop;
  } else if (desiredBottom > viewBottom) {
    newTop = desiredBottom - container.clientHeight;
  }
  if (newTop != null) {
    container.scrollTo({ top: Math.max(0, newTop), behavior: 'smooth' });
  }
}

const totalPages = computed(() => Math.ceil(total.value / pageSize.value));
const pagesToShow = computed(() => {
  const tp = totalPages.value;
  if (tp <= 7) return Array.from({length: tp}, (_, i) => i + 1);
  // Show a sliding window around the current page
  const start = Math.max(1, page.value - 2);
  const end = Math.min(tp, start + 4);
  return Array.from({length: end - start + 1}, (_, i) => start + i);
});

async function load() {
  loading.value = true;
  try {
    const offset = (page.value - 1) * pageSize.value;
    const base = import.meta.env.VITE_API || 'http://localhost:3000';
    const resp = await fetch(`${base}/comments?offset=${offset}&count=${pageSize.value}`);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();

    // reset row refs and set new items
    rowRefs.value = [];
    items.value = data.items ?? [];
    total.value = data.total ?? 0;

    // adjust selection after data loads
    if (items.value.length > 0) {
      if (selectedIndex.value == null) {
        selectedIndex.value = 0;
      } else if (selectedIndex.value >= items.value.length) {
        selectedIndex.value = items.value.length - 1;
      }
    } else {
      selectedIndex.value = null;
    }

    // after DOM updates, ensure selected row is visible (with buffer)
    await nextTick();
    if (scroller.value) scroller.value.scrollTop = 0; // reset view to top for new page
    ensureRowInView(selectedIndex.value);
  } catch (e) {
    // eslint-disable-next-line no-console
    console.error('Failed to load comments', e);
    items.value = [];
    total.value = 0;
    selectedIndex.value = null;
  } finally {
    loading.value = false;
  }
}

function nextPage() { if (page.value < totalPages.value) { page.value += 1; } }
function prevPage() { if (page.value > 1) { page.value -= 1; } }
function goPage(p) { if (p >= 1 && p <= totalPages.value) { page.value = p; } }

function selectRow(idx) { selectedIndex.value = idx; ensureRowInView(idx); }
function rowClass(c, idx) {
  const cls = [];
  if (selectedIndex.value === idx) cls.push('selected');
  const m = marks.value[c.id];
  if (m === 'blue') cls.push('mark-blue');
  if (m === 'red') cls.push('mark-red');
  return cls;
}

function handleKeydown(e) {
  const activeTag = document.activeElement?.tagName;
  if (activeTag && ['INPUT','TEXTAREA','SELECT'].includes(activeTag)) return;
  if (loading.value) return;

  // j/k navigation
  if (e.key === 'j') {
    if (!items.value.length) return;
    if (selectedIndex.value == null) selectedIndex.value = 0; else selectedIndex.value = Math.min(items.value.length - 1, selectedIndex.value + 1);
    e.preventDefault();
    nextTick().then(() => ensureRowInView(selectedIndex.value));
  } else if (e.key === 'k') {
    if (!items.value.length) return;
    if (selectedIndex.value == null) selectedIndex.value = 0; else selectedIndex.value = Math.max(0, selectedIndex.value - 1);
    e.preventDefault();
    nextTick().then(() => ensureRowInView(selectedIndex.value));
  } else if (e.key === 'a') {
    // mark blue
    if (selectedIndex.value != null && items.value[selectedIndex.value]) {
      const id = items.value[selectedIndex.value].id;
      marks.value = { ...marks.value, [id]: 'blue' };
      e.preventDefault();
    }
  } else if (e.key === 'd') {
    // mark red
    if (selectedIndex.value != null && items.value[selectedIndex.value]) {
      const id = items.value[selectedIndex.value].id;
      marks.value = { ...marks.value, [id]: 'red' };
      e.preventDefault();
    }
  }
}

watch(page, load);
watch(selectedIndex, async (idx) => {
  await nextTick();
  ensureRowInView(idx);
});
onMounted(() => {
  load();
  window.addEventListener('keydown', handleKeydown);
});
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>

<style>
:root {
  --page-padding: clamp(16px, 4vw, 32px);
}

html, body, #app { height: 100%; }
body { margin: 0; }

.page {
  min-height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.hero { text-align: center; padding: var(--page-padding) var(--page-padding) 0; }
.hero-icon { font-size: 2rem; }

.content { width: min(1100px, 95%); padding: var(--page-padding); }
.table-wrap { background: #ffffff; border-radius: 8px; padding: 1rem; box-shadow: 0 2px 8px rgba(0,0,0,0.06); }

.table-scroller { max-height: 60vh; overflow: auto; border: 1px solid rgba(0,0,0,0.06); border-radius: 6px; }

.comments-table { width: 100%; border-collapse: collapse; }
.comments-table th, .comments-table td { padding: 0.5rem 0.75rem; border-bottom: 1px solid rgba(0,0,0,0.08); vertical-align: top; }
.comments-table tbody tr { cursor: pointer; transition: background-color .12s ease; }
/* Zebra theme */
.comments-table tbody tr:nth-child(even) { background: rgba(0,0,0,0.03); }
.comments-table tbody tr:hover { background: rgba(0,0,0,0.06); }
/* Selection and marks override zebra */
.comments-table tbody tr.selected { outline: 2px solid #3b82f6; outline-offset: -2px; background: rgba(59,130,246,0.12); }
.comments-table tbody tr.mark-blue { background: rgba(59,130,246,0.18); }
.comments-table tbody tr.mark-red { background: rgba(239,68,68,0.18); }
.comments-table .text { max-width: 600px; overflow-wrap: anywhere; }
.empty { text-align: center; color: #777; }

.legend { margin-top: 0.5rem; color: #666; font-size: 0.9rem; }

.pagination { display: flex; gap: 0.5rem; align-items: center; justify-content: flex-end; padding-top: 0.75rem; }
.page-btn { padding: 0.25rem 0.5rem; }
.page-btn.active { font-weight: bold; text-decoration: underline; }
.total { margin-left: 0.5rem; color: #555; }

.loading { padding: 1rem; }
</style>
